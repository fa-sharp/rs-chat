mod docker;
mod dockerfiles;

use docker::{DockerExecutor, DockerExecutorOptions};

use rocket::async_trait;
use schemars::{gen::SchemaSettings, schema_for, JsonSchema};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    provider::{LlmTool, LlmToolType},
    tools::{
        core::{ToolLog, ToolParameters, ToolResult},
        system::{SystemTool, SystemToolConfig},
        ToolError,
    },
    utils::sender_with_logging::SenderWithLogging,
};

const CODE_RUNNER_NAME: &str = "code_runner";
const CODE_RUNNER_DESCRIPTION: &str = "Run code snippet in a sandboxed environment. \
    Any output files should be written to the `/var/output` directory.";

#[derive(Debug)]
pub struct CodeRunner<'a> {
    config: &'a CodeRunnerConfig,
}
impl<'a> CodeRunner<'a> {
    pub fn new(config: &'a CodeRunnerConfig) -> Self {
        CodeRunner { config }
    }
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct CodeRunnerInput {
    /// The code to execute.
    code: String,
    /// The language of the code.
    language: CodeLanguage,
    /// The packages/dependencies required for the code to execute. For example, for Python: `["numpy", "pandas"]`.
    /// For Rust, features can be added at the end of the list as supported by `cargo add`, e.g., `["package1", "package2", "--features", "package2/feature1"]`.
    dependencies: Vec<String>,
    /// Whether to enable network access. Set to `true` only if the program needs to access the internet at runtime.
    /// Network access is not needed for downloading dependencies.
    network: bool,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
enum CodeLanguage {
    Python,
    JavaScript,
    TypeScript,
    Rust,
    Go,
    Bash,
}

fn get_input_schema() -> serde_json::Value {
    let settings = SchemaSettings::draft07().with(|s| {
        s.inline_subschemas = true; // Enable inline subschemas for compatibility with LLM providers
    });
    let schema = settings
        .into_generator()
        .into_root_schema_for::<CodeRunnerInput>();
    serde_json::to_value(schema).expect("Should be valid JSON Schema")
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CodeRunnerConfig {
    pub timeout_seconds: Option<u32>,
    pub memory_limit_mb: Option<u32>,
    pub cpu_limit: Option<f32>,
}

impl SystemToolConfig for CodeRunnerConfig {
    type DynamicConfig = ();

    fn validate(&self) -> ToolResult<()> {
        if let Some(timeout) = self.timeout_seconds {
            if timeout > 120 {
                return Err(ToolError::InvalidConfiguration(
                    "Timeout cannot exceed 2 minutes".to_string(),
                ));
            }
        }
        if let Some(memory) = self.memory_limit_mb {
            if memory > 1024 {
                return Err(ToolError::InvalidConfiguration(
                    "Memory limit cannot exceed 1024 MB".to_string(),
                ));
            }
        }
        if let Some(cpu) = self.cpu_limit {
            if cpu <= 0.0 || cpu > 2.0 {
                return Err(ToolError::InvalidConfiguration(
                    "CPU limit must be between 0 and 2".to_string(),
                ));
            }
        }
        Ok(())
    }

    fn get_llm_tools(&self, tool_id: Uuid, _input_config: Option<()>) -> Vec<LlmTool> {
        let input_schema = get_input_schema();
        vec![LlmTool {
            name: CODE_RUNNER_NAME.into(),
            description: CODE_RUNNER_DESCRIPTION.into(),
            input_schema,
            tool_id,
            tool_type: LlmToolType::Internal,
        }]
    }
}

#[async_trait]
impl SystemTool for CodeRunner<'_> {
    fn input_schema(&self, _tool_name: &str) -> serde_json::Value {
        let schema = schema_for!(CodeRunnerInput);
        serde_json::to_value(schema).expect("Should be valid JSON Schema")
    }

    async fn execute(
        &self,
        _tool_name: &str,
        params: &ToolParameters,
        sender: &SenderWithLogging<ToolLog>,
    ) -> ToolResult<String> {
        let input = serde_json::from_value::<CodeRunnerInput>(
            serde_json::to_value(params).expect("Should be valid JSON"),
        )
        .map_err(|e| ToolError::InvalidParameters(e.to_string()))?;
        let executor = DockerExecutor::new(
            input.language,
            DockerExecutorOptions {
                timeout_seconds: self.config.timeout_seconds,
                memory_limit_mb: self.config.memory_limit_mb,
                cpu_limit: self.config.cpu_limit,
                network: Some(input.network),
            },
        );

        executor
            .execute(&input.code, &input.dependencies, sender)
            .await
    }
}
