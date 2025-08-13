use std::{process::Stdio, time::Duration};

use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
    sync::mpsc::Sender,
};
use uuid::Uuid;

use crate::tools::{
    code_executor::{
        dockerfiles::{get_dockerfile, get_dockerfile_info},
        CodeLanguage,
    },
    core::{ToolMessageChunk, ToolResult},
    ToolError,
};

const TIMEOUT_SECONDS: u32 = 30;
const GRACE_PERIOD_SECONDS: u32 = 5;
const MEMORY_LIMIT_MB: u32 = 512;
const CPU_LIMIT: f32 = 0.5;

pub struct DockerExecutor {
    pub lang: CodeLanguage,
    pub timeout_seconds: u32,
    pub memory_limit_mb: u32,
    pub cpu_limit: f32,
    pub network: bool,
}

#[derive(Debug, Default)]
pub struct DockerExecutorOptions {
    pub timeout_seconds: Option<u32>,
    pub memory_limit_mb: Option<u32>,
    pub cpu_limit: Option<f32>,
    pub network: Option<bool>,
}

impl DockerExecutor {
    pub fn new(lang: CodeLanguage, options: DockerExecutorOptions) -> Self {
        DockerExecutor {
            lang,
            timeout_seconds: options.timeout_seconds.unwrap_or(TIMEOUT_SECONDS),
            memory_limit_mb: options.memory_limit_mb.unwrap_or(MEMORY_LIMIT_MB),
            cpu_limit: options.cpu_limit.unwrap_or(CPU_LIMIT),
            network: options.network.unwrap_or(false),
        }
    }

    pub async fn execute(
        &self,
        code: &str,
        dependencies: &[String],
        tx: Sender<ToolMessageChunk>,
    ) -> ToolResult<String> {
        let image_tag = format!("code-exec-{}", Uuid::new_v4());
        let container_id = format!("code-exec-{}", Uuid::new_v4());

        // Run the code in a Docker container, returning early if the client disconnects
        let result = tokio::select! {
            result = self.run(&image_tag, &container_id, code, dependencies, &tx) => result,
            _ = tx.closed() => {
                Err(ToolError::Cancelled("client disconnected".to_string()))
            }
        };

        // Cleanup container and image
        let rm_container_command = Command::new("docker")
            .args(&["rm", "-f", &container_id])
            .output();
        let rm_image_command = Command::new("docker")
            .args(&["rmi", "-f", &image_tag])
            .output();
        if !tx.is_closed() {
            let _ = tx
                .send(ToolMessageChunk::Log("Cleaning up...".to_owned()))
                .await;
            let _ = tokio::join!(rm_container_command, rm_image_command);
        } else {
            tokio::spawn(async move {
                let _ = tokio::join!(rm_container_command, rm_image_command);
            });
        }

        result
    }

    async fn run(
        &self,
        image_tag: &str,
        container_id: &str,
        code: &str,
        dependencies: &[String],
        tx: &Sender<ToolMessageChunk>,
    ) -> ToolResult<String> {
        let (base_image, file_name, cmd) = get_dockerfile_info(&self.lang);

        // Check if base image exists locally, pull if needed
        let _ = tx
            .send(ToolMessageChunk::Log(format!(
                "Checking base image '{base_image}'..."
            )))
            .await;
        let image_check_output = Command::new("docker")
            .args(&["image", "inspect", base_image, "--format='{{.Id}}'"])
            .kill_on_drop(true)
            .output()
            .await?;
        if !image_check_output.status.success() {
            let _ = tx
                .send(ToolMessageChunk::Log(format!(
                    "Pulling base image '{base_image}'..."
                )))
                .await;
            let pull_output = Command::new("docker")
                .args(&["pull", base_image, "-q"])
                .kill_on_drop(true)
                .output()
                .await?;
            if !pull_output.status.success() {
                let stderr = String::from_utf8_lossy(&pull_output.stderr);
                let _ = tx
                    .send(ToolMessageChunk::Error(format!(
                        "Failed to pull Docker image '{base_image}': {stderr}",
                    )))
                    .await;
                return Err(ToolError::ToolExecutionError(format!(
                    "Failed to pull Docker image '{base_image}':\n\n{stderr}"
                )));
            }
            let _ = tx
                .send(ToolMessageChunk::Log(format!(
                    "Pulled image '{base_image}' successfully"
                )))
                .await;
        }

        // Write code and Dockerfile to temporary folder
        let temp_dir = tempfile::tempdir()?;
        let temp_dir_str = temp_dir.path().to_string_lossy();

        let code_file_path = temp_dir.path().join(file_name);
        tokio::fs::write(&code_file_path, code).await?;

        let dockerfile_path = temp_dir.path().join("Dockerfile");
        tokio::fs::write(&dockerfile_path, get_dockerfile(&self.lang)).await?;

        // Build Docker image
        let _ = tx
            .send(ToolMessageChunk::Log(format!(
                "Building Docker image '{image_tag}'..."
            )))
            .await;
        let deps_arg = format!(
            "DEPENDENCIES={}",
            self.build_dependency_string(dependencies)
        );
        let mut build_process = Command::new("docker")
            .args(&[
                "build",
                "--build-arg",
                &deps_arg,
                "-t",
                &image_tag,
                &temp_dir_str,
            ])
            .stderr(Stdio::piped())
            .spawn()?;
        let mut build_stderr = String::new();
        if let Some(stderr) = build_process.stderr.take() {
            let mut reader = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                build_stderr.push_str(&format!("{line}\n"));
                let _ = tx.send(ToolMessageChunk::Debug(line)).await;
            }
        }

        let build_status = build_process.wait().await?;
        if !build_status.success() {
            let _ = tx
                .send(ToolMessageChunk::Error(format!(
                    "Failed to build Docker image '{image_tag}'"
                )))
                .await;
            return Err(ToolError::ToolExecutionError(format!(
                "Failed to build Docker image '{image_tag}':\n\n{build_stderr}"
            )));
        }
        let _ = tx
            .send(ToolMessageChunk::Log(format!(
                "Built image '{image_tag}' successfully"
            )))
            .await;

        // Create docker arguments
        let memory_limit = format!("{}m", self.memory_limit_mb);
        let cpu_limit = self.cpu_limit.to_string();
        let timeout_str = format!("{}s", self.timeout_seconds + GRACE_PERIOD_SECONDS);

        #[rustfmt::skip]
        let docker_args = vec![
            "run", "--rm", "--name", &container_id,
            "--network", if self.network { "bridge" } else { "none" },
            "--read-only",
            "--tmpfs", "/tmp:rw,noexec,nosuid,size=100m",
            "--memory", &memory_limit,
            "--cpus", &cpu_limit,
            "--pids-limit", "50",       // Limit number of processes
            "--ulimit", "nproc=50:50",  // Limit processes
            "--cap-drop", "ALL",        // Drop all capabilities
            "--security-opt", "no-new-privileges", // Prevent privilege escalation
            "-e", "HOME=/tmp/home",     // Set writable home directory
            &image_tag,
            "timeout", &timeout_str,
            "sh", "-c", &cmd
        ];

        // Run code
        let _ = tx
            .send(ToolMessageChunk::Log(format!(
                "Running command: docker {}",
                docker_args.join(" ")
            )))
            .await;
        let mut run_process = Command::new("docker")
            .args(&docker_args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let mut run_stdout = String::new();
        let run_output = tokio::select! {
            output = async {
                if let Some(stdout) = run_process.stdout.take() {
                    let mut reader = BufReader::new(stdout).lines();
                    while let Ok(Some(line)) = reader.next_line().await {
                        run_stdout.push_str(&format!("{line}\n"));
                        let _ = tx.send(ToolMessageChunk::Result(line)).await;
                    }
                }
                run_process.wait_with_output().await
            } => output?,
            _ = tokio::time::sleep(Duration::from_secs(self.timeout_seconds.into())) => {
                let _ = tx.send(ToolMessageChunk::Error("Code execution timed out".to_string())).await;
                return Err(ToolError::ToolExecutionError("Code execution timed out".to_string()));
            }
        };

        // Return result
        if run_output.status.success() {
            Ok(format!(
                "✅ Code executed successfully:\n\n**Output:**\n\n{run_stdout}\n"
            ))
        } else {
            Ok(format!(
                "❌ Code execution failed:\n\n**Error:**\n\n{}\n\n\n**Output:**\n\n{run_stdout}\n",
                String::from_utf8_lossy(&run_output.stderr)
            ))
        }
    }

    fn build_dependency_string(&self, dependencies: &[String]) -> String {
        dependencies
            .iter()
            .filter_map(|d| {
                let sanitized = self.sanitize_package_name(d);
                if sanitized.trim().is_empty() {
                    None
                } else {
                    Some(sanitized)
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn sanitize_package_name(&self, package: &str) -> String {
        let sanitized = package
            .chars()
            .filter(|c| {
                c.is_alphanumeric()
                    || *c == '-'
                    || *c == '_'
                    || *c == '.'
                    || *c == '='
                    || *c == '"'
                    || *c == ':'
                    || *c == '/'
                    || *c == '@' // For npm scoped packages like @types/node
            })
            .collect::<String>();

        sanitized
    }
}
