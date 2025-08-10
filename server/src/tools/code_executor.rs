mod docker;

use rocket::async_trait;
use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::tools::{
    code_executor::docker::{DockerExecutor, DockerExecutorOptions},
    core::{Tool, ToolParameters, ToolResult},
    ToolError,
};

#[derive(Debug)]
pub struct CodeExecutorTool<'a> {
    name: &'a str,
    config: &'a CodeExecutorToolConfig,
}
impl<'a> CodeExecutorTool<'a> {
    pub fn new(name: &'a str, config: &'a CodeExecutorToolConfig) -> Self {
        CodeExecutorTool { name, config }
    }
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CodeExecutorToolConfig {
    pub timeout_seconds: Option<u32>,
    pub memory_limit_mb: Option<u32>,
    pub cpu_limit: Option<f32>,
}
impl CodeExecutorToolConfig {
    pub fn validate(&self) -> ToolResult<()> {
        if let Some(timeout) = self.timeout_seconds {
            if timeout >= 60 {
                return Err(ToolError::InvalidConfiguration(
                    "timeout_seconds must be less than 60".to_string(),
                ));
            }
        }
        if let Some(memory) = self.memory_limit_mb {
            if memory >= 1024 {
                return Err(ToolError::InvalidConfiguration(
                    "memory_limit_mb must be less than 1024".to_string(),
                ));
            }
        }
        if let Some(cpu) = self.cpu_limit {
            if cpu <= 0.0 || cpu >= 1.0 {
                return Err(ToolError::InvalidConfiguration(
                    "cpu_limit must be between 0 and 1".to_string(),
                ));
            }
        }
        Ok(())
    }

    pub fn get_input_schema(&self) -> serde_json::Value {
        let schema = schema_for!(CodeExecutorInput);
        serde_json::to_value(schema).expect("Should be valid JSON Schema")
    }
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct CodeExecutorInput {
    code: String,
    language: CodeLanguage,
    files: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
enum CodeLanguage {
    Python,
    JavaScript,
    TypeScript,
    Rust,
    Bash,
}

#[async_trait]
impl Tool for CodeExecutorTool<'_> {
    fn name(&self) -> &str {
        &self.name
    }

    fn input_schema(&self) -> serde_json::Value {
        let schema = schema_for!(CodeExecutorInput);
        serde_json::to_value(schema).expect("Should be valid JSON Schema")
    }

    async fn execute(&self, params: &ToolParameters) -> ToolResult<String> {
        let input: CodeExecutorInput =
            serde_json::from_value(serde_json::to_value(params).expect("Should be valid JSON"))
                .map_err(|e| ToolError::InvalidParameters(e.to_string()))?;
        let options = DockerExecutorOptions {
            timeout_seconds: self.config.timeout_seconds.unwrap_or_default(),
            memory_limit_mb: self.config.memory_limit_mb.unwrap_or_default(),
            cpu_limit: self.config.cpu_limit.unwrap_or_default(),
        };
        let executor = DockerExecutor::new(input.language, options);

        executor.execute(&input.code).await
    }
}
