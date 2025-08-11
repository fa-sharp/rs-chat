use std::time::Duration;

use tokio::process::Command;
use uuid::Uuid;

use crate::tools::{
    code_executor::{
        dockerfiles::{get_dockerfile, get_dockerfile_info},
        CodeLanguage,
    },
    core::ToolResult,
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
}

#[derive(Debug, Default)]
pub struct DockerExecutorOptions {
    pub timeout_seconds: Option<u32>,
    pub memory_limit_mb: Option<u32>,
    pub cpu_limit: Option<f32>,
}

impl DockerExecutor {
    pub fn new(lang: CodeLanguage, options: DockerExecutorOptions) -> Self {
        DockerExecutor {
            lang,
            timeout_seconds: options.timeout_seconds.unwrap_or(TIMEOUT_SECONDS),
            memory_limit_mb: options.memory_limit_mb.unwrap_or(MEMORY_LIMIT_MB),
            cpu_limit: options.cpu_limit.unwrap_or(CPU_LIMIT),
        }
    }

    pub async fn execute(&self, code: &str, dependencies: &[String]) -> ToolResult<String> {
        let (base_image, file_name, cmd) = get_dockerfile_info(&self.lang);

        // Check if base image exists locally, pull if needed
        let image_check = Command::new("docker")
            .args(&["image", "inspect", base_image, "--format='{{.Id}}'"])
            .output()
            .await;
        if !image_check.is_ok_and(|output| output.status.success()) {
            println!("Image '{}' not found locally, pulling...", base_image);
            let pull_output = Command::new("docker")
                .args(&["pull", base_image, "-q"])
                .output()
                .await?;
            if !pull_output.status.success() {
                let stderr = String::from_utf8_lossy(&pull_output.stderr);
                return Err(ToolError::ToolExecutionError(format!(
                    "Failed to pull Docker image {}: {}",
                    base_image, stderr
                )));
            }
            println!("Pulled image '{}' successfully", base_image);
        }

        // Write code and Dockerfile to temporary folder
        let temp_dir = tempfile::tempdir()?;
        let temp_dir_str = temp_dir.path().to_string_lossy();

        let code_file_path = temp_dir.path().join(file_name);
        tokio::fs::write(&code_file_path, code).await?;

        let dockerfile_path = temp_dir.path().join("Dockerfile");
        tokio::fs::write(&dockerfile_path, get_dockerfile(&self.lang)).await?;

        // Build Docker image
        let image_tag = format!("code-exec-{}", Uuid::new_v4());
        println!("Building Docker image '{}'...", image_tag);
        let deps_arg = format!(
            "DEPENDENCIES={}",
            self.build_dependency_string(dependencies)
        );
        let build_output = Command::new("docker")
            .args(&[
                "build",
                "--build-arg",
                &deps_arg,
                "-t",
                &image_tag,
                &temp_dir_str,
            ])
            .output()
            .await?;
        if !build_output.status.success() {
            let stderr = String::from_utf8_lossy(&build_output.stderr);
            return Err(ToolError::ToolExecutionError(format!(
                "Failed to build Docker image {}: {}",
                image_tag, stderr
            )));
        }
        println!("Built image '{}' successfully", image_tag);

        // Create docker arguments
        let container_id = format!("code-exec-{}", Uuid::new_v4());
        let memory_limit = format!("{}m", self.memory_limit_mb);
        let cpu_limit = self.cpu_limit.to_string();
        let timeout_str = format!("{}s", self.timeout_seconds + GRACE_PERIOD_SECONDS);

        #[rustfmt::skip]
        let docker_args = vec![
            "run", "--rm", "--name", &container_id,
            "--network", if true { "none" } else { "bridge" },
            "--memory", &memory_limit,
            "--cpus", &cpu_limit,
            "-e", "HOME=/tmp/home", // Set writable home directory
            &image_tag,
            "timeout", &timeout_str,
            "sh", "-c", &cmd
        ];

        // Run code
        println!("Running command: docker {}", docker_args.join(" "));
        let result = tokio::time::timeout(
            Duration::from_secs(self.timeout_seconds.into()),
            Command::new("docker").args(&docker_args).output(),
        )
        .await;

        // Cleanup container and image
        let _ = Command::new("docker")
            .args(&["rm", "-f", &container_id])
            .output()
            .await;
        let _ = Command::new("docker")
            .args(&["rmi", "-f", &image_tag])
            .output()
            .await;

        // Process output
        let output = result
            .map_err(|_| ToolError::ToolExecutionError("Code execution timed out".to_string()))?
            .map_err(|e| {
                ToolError::ToolExecutionError(format!("Docker execution failed: {}", e))
            })?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if output.status.success() {
            Ok(format!(
                "✅ Code executed successfully:\n\n**Output:**\n\n{}\n",
                stdout
            ))
        } else {
            Ok(format!(
                "❌ Code execution failed:\n\n**Error:**\n\n{}\n\n\n**Output:**\n\n{}\n",
                stderr, stdout
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
