mod docker;
mod dockerfiles;
use docker::{DockerExecutor, DockerExecutorOptions};

use std::sync::LazyLock;

use rocket::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    provider::{LlmTool, LlmToolType},
    tools::{
        core::{ToolLog, ToolParameters, ToolResult},
        system::{SystemTool, SystemToolConfig},
        utils::get_json_schema,
        ToolError,
    },
    utils::SenderWithLogging,
};

const CODE_RUNNER_NAME: &str = "code_runner";
const CODE_RUNNER_DESCRIPTION: &str = "Run code snippet in a sandboxed environment. \
    Any output files should be written to the `/var/output` directory.";
const DEFAULT_TIMEOUT_SECONDS: u32 = 30;
const DEFAULT_MEMORY_LIMIT_MB: u32 = 512;
const DEFAULT_CPU_LIMIT: f32 = 0.5;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct CodeRunnerInput {
    /// The code to execute.
    code: String,
    /// The language of the code.
    language: CodeLanguage,
    /// The packages/dependencies required for the code to execute. Version constraints can optionally be added
    /// as supported by the language's package manager CLI, e.g. for Python, `["numpy==1.23.4", "pandas>=1.0.0"]`
    /// or for JavaScript: `["axios@0.27.2", "lodash@4.17.21"]`.
    /// For Rust, features can be added at the end of the list as supported by `cargo add`, e.g., `["package1", "package2", "--features", "package2/feature1"]`.
    dependencies: Vec<String>,
    /// Whether to enable network access. Set to `true` only if the program needs to access the internet at runtime.
    /// Network access is not needed for downloading dependencies.
    network: bool,
}

static CODE_RUNNER_INPUT_SCHEMA: LazyLock<serde_json::Value> =
    LazyLock::new(|| get_json_schema::<CodeRunnerInput>());

/// Tool to run code snippets in a sandboxed environment.
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
#[serde(rename_all = "lowercase")]
enum CodeLanguage {
    Python,
    JavaScript,
    TypeScript,
    Rust,
    Go,
    Bash,
}

/// Configuration for the code runner tool.
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CodeRunnerConfig {
    /// Timeout in seconds for the code execution.
    #[serde(default = "default_timeout")]
    #[validate(range(min = 5, max = 60))]
    pub timeout_seconds: u32,
    /// Memory limit in MB for the code execution.
    #[serde(default = "default_memory_limit")]
    #[validate(range(min = 100, max = 1024))]
    pub memory_limit_mb: u32,
    /// CPU limit for the code execution
    #[serde(default = "default_cpu_limit")]
    #[validate(range(min = 0.1, max = 1.2))]
    pub cpu_limit: f32,
}
fn default_timeout() -> u32 {
    DEFAULT_TIMEOUT_SECONDS
}
fn default_memory_limit() -> u32 {
    DEFAULT_MEMORY_LIMIT_MB
}
fn default_cpu_limit() -> f32 {
    DEFAULT_CPU_LIMIT
}

impl SystemToolConfig for CodeRunnerConfig {
    type DynamicConfig = ();

    fn validate(&self) -> ToolResult<()> {
        let config_schema = serde_json::to_value(schemars::schema_for!(Self))?;
        jsonschema::validate(&config_schema, &serde_json::to_value(self)?)
            .map_err(|e| ToolError::InvalidConfiguration(e.to_string()))
    }

    fn get_llm_tools(&self, tool_id: Uuid, _input_config: Option<()>) -> Vec<LlmTool> {
        vec![LlmTool {
            name: CODE_RUNNER_NAME.into(),
            description: CODE_RUNNER_DESCRIPTION.into(),
            input_schema: CODE_RUNNER_INPUT_SCHEMA.to_owned(),
            tool_id,
            tool_type: LlmToolType::System,
        }]
    }
}

#[async_trait]
impl SystemTool for CodeRunner<'_> {
    fn input_schema(&self, _tool_name: &str) -> &serde_json::Value {
        &CODE_RUNNER_INPUT_SCHEMA
    }

    async fn execute(
        &self,
        _tool_name: &str,
        params: &ToolParameters,
        sender: &SenderWithLogging<ToolLog>,
    ) -> ToolResult<String> {
        let input = serde_json::from_value::<CodeRunnerInput>(serde_json::to_value(params)?)
            .map_err(|e| ToolError::InvalidParameters(e.to_string()))?;
        let executor = DockerExecutor::new(
            input.language,
            DockerExecutorOptions {
                timeout_seconds: self.config.timeout_seconds,
                memory_limit_mb: self.config.memory_limit_mb,
                cpu_limit: self.config.cpu_limit,
                network: input.network,
            },
        );

        executor
            .execute(&input.code, &input.dependencies, sender)
            .await
    }
}
