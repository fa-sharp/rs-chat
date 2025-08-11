use std::time::Duration;

use tokio::process::Command;
use uuid::Uuid;

use crate::tools::{code_executor::CodeLanguage, core::ToolResult, ToolError};

const TIMEOUT_SECONDS: u32 = 10;
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

    pub async fn execute(&self, code: &str) -> ToolResult<String> {
        let (image, file_name, cmd) = match self.lang {
            CodeLanguage::Python => ("python:3.13-alpine", "main.py", "python /workspace/main.py"),
            CodeLanguage::JavaScript => ("node:20-alpine", "main.js", "node /workspace/main.js"),
            CodeLanguage::TypeScript => ("node:20-alpine", "main.ts", "npx tsx /workspace/main.ts"),
            CodeLanguage::Rust => (
                "rust:1.85-alpine",
                "main.rs",
                "cd /workspace && rustc main.rs && ./main",
            ),
            CodeLanguage::Bash => ("bash:latest", "script.sh", "bash /workspace/script.sh"),
        };

        // Write code to temporary file
        let temp_dir = tempfile::tempdir().map_err(|e| {
            ToolError::ToolExecutionError(format!("Failed to create temporary directory: {}", e))
        })?;
        let code_file_path = temp_dir.path().join(file_name);
        tokio::fs::write(&code_file_path, code).await.map_err(|e| {
            ToolError::ToolExecutionError(format!("Failed to write code file: {}", e))
        })?;

        // Check if image exists locally, pull if needed
        let image_check = Command::new("docker")
            .args(&["image", "inspect", image, "--format='{{.Id}}'"])
            .output()
            .await;
        if !image_check.is_ok_and(|output| output.status.success()) {
            println!("Image '{}' not found locally, pulling...", image);
            let pull_output = Command::new("docker")
                .args(&["pull", image, "-q"])
                .output()
                .await
                .map_err(|e| {
                    ToolError::ToolExecutionError(format!("Failed to pull Docker image: {}", e))
                })?;
            if !pull_output.status.success() {
                let stderr = String::from_utf8_lossy(&pull_output.stderr);
                return Err(ToolError::ToolExecutionError(format!(
                    "Failed to pull Docker image {}: {}",
                    image, stderr
                )));
            }
            println!("Pulled image '{}' successfully", image);
        }

        // Create docker arguments
        let container_id = format!("code-exec-{}", Uuid::new_v4());
        let memory_limit = format!("{}m", self.memory_limit_mb);
        let cpu_limit = self.cpu_limit.to_string();
        let timeout_str = format!("{}s", self.timeout_seconds);
        let volume_mount = format!("{}:/workspace:ro", temp_dir.path().to_string_lossy());

        #[rustfmt::skip]
        let docker_args = vec![
            "run", "--rm", "--name", &container_id,
            "--network", "none", // No network access
            "--memory", &memory_limit,
            "--cpus", &cpu_limit,
            "--user", "1000:1000", // Non-root user
            "-v", &volume_mount, // Mount temp directory as read-only
            image,
            "timeout", &timeout_str,
            "sh", "-c", &cmd
        ];

        // Run code and clean up container if needed
        println!("Running command: docker {}", docker_args.join(" "));
        let result = tokio::time::timeout(
            Duration::from_secs((self.timeout_seconds + GRACE_PERIOD_SECONDS).into()),
            Command::new("docker").args(&docker_args).output(),
        )
        .await;
        let _ = Command::new("docker")
            .args(&["rm", "-f", &container_id])
            .output()
            .await;

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
}
