use std::{sync::LazyLock, time::Duration};

use bollard::{
    body_try_stream,
    container::{AttachContainerResults, LogOutput},
    models::{BuildInfoAux, ContainerCreateBody, HostConfig, ResourcesUlimits},
    query_parameters::{
        AttachContainerOptionsBuilder, BuildImageOptionsBuilder, BuilderVersion,
        CreateContainerOptionsBuilder, CreateImageOptionsBuilder, RemoveContainerOptionsBuilder,
        RemoveImageOptionsBuilder, StartContainerOptions, StopContainerOptions,
        WaitContainerOptions,
    },
    Docker,
};
use rocket::futures::StreamExt;
use tokio_util::io::ReaderStream;
use uuid::Uuid;

use crate::{
    tools::{
        code_executor::{
            dockerfiles::{get_dockerfile, get_dockerfile_info},
            CodeLanguage,
        },
        core::{ToolLog, ToolResult},
        ToolError,
    },
    utils::sender_with_logging::SenderWithLogging,
};

static DOCKER: LazyLock<Result<Docker, bollard::errors::Error>> =
    LazyLock::new(|| Docker::connect_with_defaults());

const TIMEOUT_SECONDS: u32 = 30;
const GRACE_PERIOD_SECONDS: u32 = 5;
const MEMORY_LIMIT_MB: u32 = 512;
const CPU_LIMIT: f32 = 0.5;

pub struct DockerExecutor {
    lang: CodeLanguage,
    timeout_seconds: u32,
    memory_limit_mb: u32,
    cpu_limit: f32,
    network: bool,
    image_tag: String,
    container_name: String,
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
            image_tag: format!("code-runner-{}", Uuid::new_v4()),
            container_name: format!("code-runner-{}", Uuid::new_v4()),
        }
    }

    pub async fn execute(
        &self,
        code: &str,
        dependencies: &[String],
        tx: &SenderWithLogging<ToolLog>,
    ) -> ToolResult<String> {
        let docker = DOCKER
            .as_ref()
            .inspect_err(|e| rocket::error!("Failed to initialize Docker client: {}", e))
            .map_err(|_| ToolError::ToolExecutionError("Failed to initialize Docker".into()))?;
        docker
            .ping()
            .await
            .inspect_err(|e| rocket::warn!("Failed to ping Docker daemon: {}", e))
            .map_err(|_| ToolError::ToolExecutionError("Couldn't connect to Docker".into()))?;

        // Run the code in a Docker container, returning early if the client disconnects
        let result = tokio::select! {
            result = self.run(docker, code, dependencies, &tx) => result,
            _ = tx.closed() => Err(ToolError::Cancelled("client disconnected".to_string()))
        };

        // Cleanup container and image
        if !tx.is_closed() {
            send_log(tx, "Cleaning up...".into()).await;
            docker_cleanup(docker, &self.container_name, &self.image_tag).await;
        } else {
            let container_name = self.container_name.clone();
            let image_tag = self.image_tag.clone();
            tokio::spawn(async move {
                docker_cleanup(docker, &container_name, &image_tag).await;
            });
        }

        result
    }

    async fn run(
        &self,
        docker: &Docker,
        code: &str,
        dependencies: &[String],
        tx: &SenderWithLogging<ToolLog>,
    ) -> ToolResult<String> {
        let (base_image, file_name, cmd) = get_dockerfile_info(&self.lang);

        // Check if base image exists locally, pull if needed
        send_log(tx, format!("Checking base image '{base_image}'...")).await;
        if docker.inspect_image(base_image).await.is_err() {
            send_log(tx, format!("Pulling base image '{base_image}'...")).await;
            let image_options = CreateImageOptionsBuilder::new()
                .from_image(base_image)
                .build();
            let mut pull_image_stream = docker.create_image(Some(image_options), None, None);
            while let Some(result) = pull_image_stream.next().await {
                match result {
                    Ok(mut response) => {
                        let status = response.status.unwrap_or_default();
                        let progress_detail = response.progress_detail.take().unwrap_or_default();
                        if let Some(progress) = response.progress {
                            send_debug(tx, format!("Pulling image: {status} {progress}")).await;
                        } else if let Some((current, total)) =
                            progress_detail.current.zip(progress_detail.total)
                        {
                            send_debug(tx, format!("Pulling image: {status} {current}/{total}"))
                                .await;
                        }
                        if let Some(error_detail) = response.error_detail {
                            send_error(tx, format!("Error pulling image: {:?}", error_detail))
                                .await;
                        }
                    }
                    Err(err) => {
                        let message = format!("Error pulling image: {err}");
                        send_error(tx, message.clone()).await;
                        return Err(ToolError::ToolExecutionError(message));
                    }
                }
            }
        }

        // Create tar archive with build context (Dockerfile and code files)
        let (tar_writer, tar_reader) = tokio::io::duplex(8192); // 8KB buffer
        let dockerfile = get_dockerfile(&self.lang);
        let code = code.to_owned();
        send_log(tx, "Creating build context with 2 files...".into()).await;

        let tar_creation_task = tokio::spawn(async move {
            let mut tar = tokio_tar::Builder::new(tar_writer);
            for (path, content) in [("Dockerfile", dockerfile), (file_name, &code)] {
                let mut header = tokio_tar::Header::new_gnu();
                header.set_size(content.len() as u64);
                header.set_mode(0o644);
                let _ = tar.append_data(&mut header, path, content.as_bytes()).await;
            }
            tar.finish().await
        });

        // Build Docker image (streaming the build context tar file)
        send_log(tx, format!("Building image '{}'...", self.image_tag)).await;
        let build_options = BuildImageOptionsBuilder::new()
            .buildargs(&[("DEPENDENCIES", self.build_dependency_string(dependencies))].into())
            .t(&self.image_tag)
            .version(BuilderVersion::BuilderBuildKit)
            .session(&Uuid::new_v4().to_string())
            .build();
        let mut build_stream = docker.build_image(
            build_options,
            None,
            Some(body_try_stream(ReaderStream::new(tar_reader))),
        );

        let mut build_logs = String::new();
        while let Some(build_info_result) = build_stream.next().await {
            match build_info_result {
                Ok(info) => {
                    if let Some(aux) = info.aux {
                        if let BuildInfoAux::BuildKit(status) = aux {
                            for vertex in status.vertexes {
                                let cached = if vertex.cached { "CACHED " } else { "" };
                                let message = format!("{}{}\n", cached, vertex.name);
                                build_logs.push_str(&message);
                                send_debug(tx, message.to_string()).await;
                            }
                            for log in status.logs {
                                let message = String::from_utf8_lossy(&log.msg);
                                build_logs.push_str(&format!("{}\n", message));
                                send_debug(tx, message.to_string()).await;
                            }
                        }
                    }
                    if let Some(stream) = info.stream {
                        build_logs.push_str(&format!("{stream}\n"));
                        send_debug(tx, stream).await;
                    }
                    if let Some(err) = info.error_detail.and_then(|e| e.message) {
                        build_logs.push_str(&format!("{err}\n"));
                        send_error(tx, format!("Failed to build image: {err}")).await;
                        return Err(ToolError::ToolExecutionError(format!(
                            "Failed to build image '{}': {err}. Build logs:\n\n{build_logs}",
                            self.image_tag
                        )));
                    }
                }
                Err(err) => {
                    build_logs.push_str(&format!("{err}\n"));
                    send_error(tx, format!("Failed to build image: {err}")).await;
                    return Err(ToolError::ToolExecutionError(format!(
                        "Failed to build image '{}': {err}. Build logs:\n\n{build_logs}",
                        self.image_tag
                    )));
                }
            }
        }
        if let Ok(Err(err)) = tar_creation_task.await {
            let message = format!("Error creating build context: {err}");
            build_logs.push_str(&format!("{message}\n"));
            send_error(tx, message).await;
        }

        // Create container with run command
        let timeout_str = format!("{}s", self.timeout_seconds + GRACE_PERIOD_SECONDS);
        let run_command = ["timeout", &timeout_str, "sh", "-c", &cmd];
        let container_body = ContainerCreateBody {
            image: Some(self.image_tag.clone()),
            cmd: Some(run_command.iter().map(|s| s.to_string()).collect()),
            env: Some(vec!["HOME=/tmp/home".into()]),
            network_disabled: Some(!self.network),
            host_config: Some(HostConfig {
                readonly_rootfs: Some(true),
                tmpfs: Some([("/tmp".into(), "rw,noexec,nosuid,size=100m".into())].into()),
                memory: Some((self.memory_limit_mb * 1024 * 1024).into()),
                nano_cpus: Some((self.cpu_limit * 1000.0).round() as i64 * 1_000_000),
                pids_limit: Some(50),
                ulimits: Some(vec![ResourcesUlimits {
                    name: Some("nproc".into()),
                    soft: Some(50),
                    hard: Some(50),
                }]),
                cap_drop: Some(vec!["ALL".into()]),
                security_opt: Some(vec!["no-new-privileges".into()]),
                ..Default::default()
            }),
            ..Default::default()
        };
        let container_options = CreateContainerOptionsBuilder::new()
            .name(&self.container_name)
            .build();
        match docker
            .create_container(Some(container_options), container_body)
            .await
        {
            Ok(res) => {
                if !res.warnings.is_empty() {
                    let message = format!(
                        "⚠️ Warning while creating container '{}': {}",
                        self.container_name,
                        res.warnings.join(", ")
                    );
                    send_log(tx, message).await;
                }
                let message = format!(
                    "Created container '{}' with ID {}",
                    self.container_name, res.id
                );
                send_log(tx, message).await;
            }
            Err(err) => {
                let message = format!(
                    "Failed to create container '{}': {err}",
                    self.container_name
                );
                send_error(tx, message.clone()).await;
                return Err(ToolError::ToolExecutionError(message));
            }
        };

        // Spawn task to attach to container and capture logs/output
        let attach_options = AttachContainerOptionsBuilder::new()
            .stream(true)
            .stdout(true)
            .stderr(true)
            .logs(true)
            .build();
        let attached_container = match docker
            .attach_container(&self.container_name, Some(attach_options))
            .await
        {
            Ok(container) => container,
            Err(e) => {
                let message = format!(
                    "Failed to attach to container '{}': {e}",
                    self.container_name
                );
                send_error(tx, message.clone()).await;
                return Err(ToolError::ToolExecutionError(message));
            }
        };
        let output_tx = tx.clone();
        let output_timeout_secs = self.timeout_seconds + GRACE_PERIOD_SECONDS;
        let container_output_task = tokio::spawn(async move {
            let mut stdout = String::new();
            let mut stderr = String::new();
            let _ = tokio::time::timeout(
                Duration::from_secs(output_timeout_secs.into()),
                capture_container_output(attached_container, &mut stdout, &mut stderr, &output_tx),
            )
            .await;
            (stdout, stderr)
        });

        // Start container
        send_log(tx, format!("Running command {run_command:?}...")).await;
        if let Err(e) = docker
            .start_container(&self.container_name, None::<StartContainerOptions>)
            .await
        {
            let message = format!("Failed to start container '{}': {e}", self.container_name);
            send_error(tx, message.clone()).await;
            return Err(ToolError::ToolExecutionError(message));
        }

        // Wait for container to exit and get exit status
        let container_exit_result = tokio::time::timeout(
            Duration::from_secs(self.timeout_seconds.into()),
            docker
                .wait_container(&self.container_name, None::<WaitContainerOptions>)
                .next(),
        )
        .await;

        // Process output and exit status
        let (stdout, stderr) = container_output_task.await.unwrap_or_default();
        let formatted_output =
            format!("**Output (stdout):**\n\n{stdout}\n\n**Logs (stderr):**\n\n{stderr}\n");
        match container_exit_result {
            Err(_) => {
                send_error(tx, "Code execution timed out".into()).await;
                Err(ToolError::ToolExecutionError(format!(
                    "❌ Code execution timed out.\n\n{formatted_output}"
                )))
            }
            Ok(Some(wait_result)) => match wait_result {
                Ok(_) => Ok(format!(
                    "✅ Code executed successfully!\n\n{formatted_output}"
                )),
                Err(err) => {
                    if let bollard::errors::Error::DockerContainerWaitError { code, .. } = err {
                        let message = format!("Code execution failed with exit status {code}");
                        send_error(tx, message.clone()).await;
                        Err(ToolError::ToolExecutionError(format!(
                            "❌ {message}.\n\n{formatted_output}"
                        )))
                    } else {
                        send_error(tx, "Code execution failed".into()).await;
                        Err(ToolError::ToolExecutionError(format!(
                            "❌ Code execution failed.\n\n{formatted_output}"
                        )))
                    }
                }
            },
            Ok(None) => Ok(format!(
                "Code executed with unknown exit status.\n\n{formatted_output}"
            )),
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

/// Capture stdout and stderr from the attached container
async fn capture_container_output(
    mut attached_container: AttachContainerResults,
    stdout: &mut String,
    stderr: &mut String,
    tx: &SenderWithLogging<ToolLog>,
) {
    while let Some(output_result) = attached_container.output.next().await {
        match output_result {
            Ok(output) => match output {
                LogOutput::StdOut { message } => {
                    let message_str = String::from_utf8_lossy(&message).to_string();
                    stdout.push_str(&format!("{message_str}\n"));
                    let _ = tx.send(ToolLog::Result(message_str)).await;
                }
                LogOutput::StdErr { message } => {
                    let message_str = String::from_utf8_lossy(&message).to_string();
                    stderr.push_str(&format!("{message_str}\n"));
                    let _ = tx.send(ToolLog::Result(message_str)).await;
                }
                _ => {}
            },
            Err(e) => {
                let _ = tx.send(ToolLog::Error(e.to_string())).await;
            }
        }
    }
}

async fn docker_cleanup(docker: &Docker, container_name: &str, image_tag: &str) {
    let _ = docker
        .stop_container(container_name, None::<StopContainerOptions>)
        .await;
    let _ = tokio::join!(
        docker.remove_container(
            container_name,
            Some(RemoveContainerOptionsBuilder::new().force(true).build()),
        ),
        docker.remove_image(
            image_tag,
            Some(RemoveImageOptionsBuilder::new().force(true).build()),
            None,
        )
    );
}

async fn send_log(tx: &SenderWithLogging<ToolLog>, message: String) {
    let _ = tx.send(ToolLog::Log(message)).await;
}
async fn send_debug(tx: &SenderWithLogging<ToolLog>, message: String) {
    let _ = tx.send(ToolLog::Debug(message)).await;
}
async fn send_error(tx: &SenderWithLogging<ToolLog>, message: String) {
    let _ = tx.send(ToolLog::Error(message)).await;
}
