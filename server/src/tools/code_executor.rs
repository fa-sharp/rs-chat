mod docker;

use rocket::async_trait;
use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};

use crate::tools::{
    code_executor::docker::{DockerExecutor, DockerExecutorOptions},
    core::{Tool, ToolParameters, ToolResult},
    ToolError,
};

#[derive(Debug)]
pub struct CodeExecutorTool<'a> {
    config: &'a CodeExecutorToolConfig,
}
impl<'a> CodeExecutorTool<'a> {
    pub fn new(config: &'a CodeExecutorToolConfig) -> Self {
        CodeExecutorTool { config }
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
                    "Timeout must be less than 60 seconds".to_string(),
                ));
            }
        }
        if let Some(memory) = self.memory_limit_mb {
            if memory >= 1024 {
                return Err(ToolError::InvalidConfiguration(
                    "Memory limit must be less than 1024 MB".to_string(),
                ));
            }
        }
        if let Some(cpu) = self.cpu_limit {
            if cpu <= 0.0 || cpu > 1.0 {
                return Err(ToolError::InvalidConfiguration(
                    "CPU limit must be between 0 and 1".to_string(),
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
#[serde(deny_unknown_fields)]
struct CodeExecutorInput {
    /// The code to execute.
    code: String,
    /// The language of the code.
    language: CodeLanguage,
    /// The packages/dependencies required for the code to execute. For example, for Python: `["numpy", "pandas"]`.
    dependencies: Vec<String>,
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
        "Code Executor"
    }

    fn input_schema(&self) -> serde_json::Value {
        let schema = schema_for!(CodeExecutorInput);
        serde_json::to_value(schema).expect("Should be valid JSON Schema")
    }

    async fn execute(&self, params: &ToolParameters) -> ToolResult<String> {
        let input = serde_json::from_value::<CodeExecutorInput>(
            serde_json::to_value(params).expect("Should be valid JSON"),
        )
        .map_err(|e| ToolError::InvalidParameters(e.to_string()))?;
        let executor = DockerExecutor::new(
            input.language,
            DockerExecutorOptions {
                timeout_seconds: self.config.timeout_seconds,
                memory_limit_mb: self.config.memory_limit_mb,
                cpu_limit: self.config.cpu_limit,
            },
        );

        executor.execute(&input.code, &input.dependencies).await
    }
}
