use rocket::futures::StreamExt;
use tinirun_client::{
    models::{CodeRunnerChunk, CodeRunnerError, CodeRunnerLanguage},
    TinirunClient,
};

use crate::{
    tools::{
        core::{ToolLog, ToolResult},
        system::code_runner::CodeLanguage,
        ToolError,
    },
    utils::SenderWithLogging,
};

pub struct TinirunExecutor<'a> {
    client: &'a TinirunClient,
    lang: CodeLanguage,
    timeout_seconds: u32,
    memory_limit_mb: u32,
    cpu_limit: f32,
}

#[derive(Debug, Default)]
pub struct TinirunExecutorOptions {
    pub timeout_seconds: u32,
    pub memory_limit_mb: u32,
    pub cpu_limit: f32,
}

impl<'a> TinirunExecutor<'a> {
    pub fn new(
        client: &'a TinirunClient,
        lang: CodeLanguage,
        options: TinirunExecutorOptions,
    ) -> Self {
        TinirunExecutor {
            client,
            lang,
            timeout_seconds: options.timeout_seconds,
            memory_limit_mb: options.memory_limit_mb,
            cpu_limit: options.cpu_limit,
        }
    }

    pub async fn execute(
        &self,
        code: &str,
        dependencies: &[String],
        tx: &SenderWithLogging<ToolLog>,
    ) -> ToolResult<String> {
        let input = tinirun_client::models::CodeRunnerInput {
            code: code.to_owned(),
            lang: self.lang.into(),
            dependencies: Some(dependencies.to_vec()),
            files: None,
            timeout: self.timeout_seconds,
            mem_limit_mb: self.memory_limit_mb,
            cpu_limit: self.cpu_limit,
        };
        let mut stream = match self.client.run_code(&input).await {
            Ok(stream) => stream,
            Err(err) => {
                return Err(ToolError::ToolExecutionError(err.to_string()));
            }
        };
        while let Some(item) = stream.next().await {
            match item {
                Ok(event) => match event {
                    CodeRunnerChunk::Info(log) => tx.send(ToolLog::Log(log)).await.ok(),
                    CodeRunnerChunk::Debug(log) => tx.send(ToolLog::Debug(log)).await.ok(),
                    CodeRunnerChunk::Stdout(stdout) => tx.send(ToolLog::Result(stdout)).await.ok(),
                    CodeRunnerChunk::Stderr(stderr) => tx.send(ToolLog::Result(stderr)).await.ok(),
                    CodeRunnerChunk::Error(err) => {
                        tx.send(ToolLog::Error(err.to_string())).await.ok();
                        let error_message = match err {
                            CodeRunnerError::BuildFailed { message, logs } => {
                                format!("{}\n## Build logs\n{}", message, logs)
                            }
                            _ => err.to_string(),
                        };
                        return Err(ToolError::ToolExecutionError(error_message));
                    }
                    CodeRunnerChunk::Result {
                        stdout,
                        stderr,
                        exit_code,
                        timeout,
                    } => {
                        let mut markdown = String::new();
                        if let Some(code) = exit_code {
                            if code != 0 {
                                markdown += &format!("⚠️ Program exited with code {code}\n\n");
                            }
                        } else if timeout {
                            markdown +=
                                &format!("⚠️ Timed out after {} seconds\n\n", self.timeout_seconds);
                        }

                        if !stdout.is_empty() {
                            markdown += &format!("Output:\n{stdout}\n\n");
                        }
                        if !stderr.is_empty() {
                            markdown += &format!("Stderr:\n{stderr}\n");
                        }

                        return Ok(markdown);
                    }
                },
                Err(err) => tx.send(ToolLog::Error(err.to_string())).await.ok(),
            };
        }

        Err(ToolError::ToolExecutionError("No output".to_string()))
    }
}

impl From<CodeLanguage> for CodeRunnerLanguage {
    fn from(language: CodeLanguage) -> Self {
        match language {
            CodeLanguage::Python => CodeRunnerLanguage::Python,
            CodeLanguage::JavaScript => CodeRunnerLanguage::JavaScript,
            CodeLanguage::TypeScript => CodeRunnerLanguage::TypeScript,
            CodeLanguage::Rust => CodeRunnerLanguage::Rust,
            CodeLanguage::Go => CodeRunnerLanguage::Go,
            CodeLanguage::Bash => CodeRunnerLanguage::Bash,
        }
    }
}
