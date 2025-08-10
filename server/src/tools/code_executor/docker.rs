use std::time::Duration;

use tokio::process::Command;
use uuid::Uuid;

use crate::tools::{code_executor::CodeLanguage, core::ToolResult, ToolError};

pub struct DockerExecutor {
    pub lang: CodeLanguage,
    pub options: DockerExecutorOptions,
}

pub struct DockerExecutorOptions {
    pub timeout_seconds: u32,
    pub memory_limit_mb: u32,
    pub cpu_limit: f32,
}
impl Default for DockerExecutorOptions {
    fn default() -> Self {
        DockerExecutorOptions {
            timeout_seconds: 10,
            memory_limit_mb: 256,
            cpu_limit: 0.5,
        }
    }
}

impl DockerExecutor {
    pub fn new(lang: CodeLanguage, options: DockerExecutorOptions) -> Self {
        DockerExecutor { lang, options }
    }

    pub async fn execute(&self, code: &str) -> ToolResult<String> {
        let rust_command = format!("echo '{}' > main.rs && rustc main.rs && ./main", code);
        let (image, cmd) = match self.lang {
            CodeLanguage::Python => ("python:3.13-alpine", vec!["python", "-c", code]),
            CodeLanguage::JavaScript => ("node:20-alpine", vec!["node", "-e", code]),
            CodeLanguage::TypeScript => ("node:20-alpine", vec!["npx", "tsx", "-e", code]),
            CodeLanguage::Rust => ("rust:1.85-alpine", vec!["sh", "-c", &rust_command]),
            CodeLanguage::Bash => ("alpine:latest", vec!["sh", "-c", code]),
        };

        let container_id = format!("code-exec-{}", Uuid::new_v4());
        let memory_limit = format!("{}m", self.options.memory_limit_mb);
        let cpu_limit = self.options.cpu_limit.to_string();
        let timeout_str = self.options.timeout_seconds.to_string();
        let mut docker_args = vec![
            "run",
            "--rm",
            "--name",
            &container_id,
            "--network",
            "none", // No network access
            "--memory",
            &memory_limit,
            "--cpus",
            &cpu_limit,
            "--timeout",
            &timeout_str,
            "--user",
            "1000:1000", // Non-root user
        ];

        docker_args.push(image);
        docker_args.extend_from_slice(&cmd);

        let output = tokio::time::timeout(
            Duration::from_secs(self.options.timeout_seconds.into()),
            Command::new("docker").args(&docker_args).output(),
        )
        .await
        .map_err(|_| ToolError::ToolExecutionError("Code execution timed out".to_string()))?
        .map_err(|e| ToolError::ToolExecutionError(format!("Docker execution failed: {}", e)))?;
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        if output.status.success() {
            Ok(format!(
                "✅ Code executed successfully:\n\n**Output:**\n```\n{}\n```",
                stdout
            ))
        } else {
            Ok(format!("❌ Code execution failed:\n\n**Error:**\n```\n{}\n```\n\n**Output:**\n```\n{}\n```", stderr, stdout))
        }
    }
}
