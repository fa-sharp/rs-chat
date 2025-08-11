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

    pub async fn execute(&self, code: &str, dependencies: &[String]) -> ToolResult<String> {
        let (image, file_name, cmd) = match self.lang {
            CodeLanguage::Python => ("python:3.13-alpine", "main.py", "PYTHONPATH=/tmp/python/lib/python3.13/site-packages:$PYTHONPATH python /workspace/main.py"),
            CodeLanguage::JavaScript => ("node:20-alpine", "main.js", "PATH=/tmp/npm/bin:$PATH node /workspace/main.js"),
            CodeLanguage::TypeScript => ("node:20-alpine", "main.ts", "PATH=/tmp/npm/bin:$PATH /tmp/npm/bin/tsx /workspace/main.ts"),
            CodeLanguage::Rust => ("rust:1.85-alpine", "main.rs", "cd /workspace && cargo run --quiet"),
            CodeLanguage::Bash => ("alpine:latest", "script.sh", "sh /workspace/script.sh"),
        };

        // Write code to temporary file
        let temp_dir = tempfile::tempdir().map_err(|e| {
            ToolError::ToolExecutionError(format!("Failed to create temporary directory: {}", e))
        })?;

        // For Rust, create proper project structure
        if matches!(self.lang, CodeLanguage::Rust) {
            let src_dir = temp_dir.path().join("src");
            tokio::fs::create_dir_all(&src_dir).await.map_err(|e| {
                ToolError::ToolExecutionError(format!("Failed to create src directory: {}", e))
            })?;
            let code_file_path = src_dir.join("main.rs");
            tokio::fs::write(&code_file_path, code).await.map_err(|e| {
                ToolError::ToolExecutionError(format!("Failed to write code file: {}", e))
            })?;
        } else {
            let code_file_path = temp_dir.path().join(file_name);
            tokio::fs::write(&code_file_path, code).await.map_err(|e| {
                ToolError::ToolExecutionError(format!("Failed to write code file: {}", e))
            })?;
        }

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
        let command = format!("{} && {}", self.build_install_command(dependencies), cmd);

        #[rustfmt::skip]
        let docker_args = vec![
            "run", "--rm", "--name", &container_id,
            // "--network", "none", // No network access during dependency installation
            "--memory", &memory_limit,
            "--cpus", &cpu_limit,
            "--user", "1000:1000", // Non-root user
            "-e", "HOME=/tmp/home", // Set writable home directory
            "-e", "PYTHONUSERBASE=/tmp/python", // Python user install directory
            "-e", "npm_config_prefix=/tmp/npm", // NPM prefix for user installs
            "-v", &volume_mount, // Mount temp directory as read-only
            image,
            "timeout", &timeout_str,
            "sh", "-c", &command
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

    fn build_install_command(&self, dependencies: &[String]) -> String {
        let packages = dependencies
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
            .join(" ");
        match self.lang {
            CodeLanguage::Python => {
                if packages.is_empty() {
                    return "mkdir -p /tmp/home /tmp/python".to_string();
                }
                format!("mkdir -p /tmp/home /tmp/python && pip install --user --no-cache-dir --quiet {}", packages)
            }
            CodeLanguage::JavaScript => {
                if packages.is_empty() {
                    return "mkdir -p /tmp/home".to_string();
                }
                format!(
                    "mkdir -p /tmp/home /tmp/npm && npm install --silent {}",
                    packages
                )
            }
            CodeLanguage::TypeScript => {
                if packages.is_empty() {
                    "mkdir -p /tmp/home /tmp/npm && npm install --silent tsx".to_string()
                } else {
                    format!(
                        "mkdir -p /tmp/home /tmp/npm && npm install --silent tsx {}",
                        packages
                    )
                }
            }
            CodeLanguage::Rust => {
                if packages.is_empty() {
                    "mkdir -p /tmp/home && cd /workspace && cargo init --name temp --quiet"
                        .to_string()
                } else {
                    format!(
                        "mkdir -p /tmp/home && cd /workspace && cargo init --name temp --quiet && cargo add {} --quiet",
                        packages
                    )
                }
            }
            CodeLanguage::Bash => {
                if packages.is_empty() {
                    "mkdir -p /tmp/home".to_string()
                } else {
                    format!(
                        "mkdir -p /tmp/home && apk add --no-cache --quiet {}",
                        packages
                    )
                }
            }
        }
    }

    fn sanitize_package_name(&self, package: &str) -> String {
        // Basic sanitization to prevent command injection
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

        // Additional validation: ensure it doesn't start with dangerous characters
        if sanitized.starts_with('-') || sanitized.starts_with('.') {
            return String::new(); // Return empty string for invalid packages
        }

        sanitized
    }
}
