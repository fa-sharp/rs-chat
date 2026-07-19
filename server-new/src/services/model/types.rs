use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// A model supported by the LLM provider
#[skip_serializing_none]
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LlmModel {
    /// The model ID to use in a chat / prompt request
    pub id: String,
    pub name: String,
    pub attachment: Option<bool>,
    pub reasoning: Option<bool>,
    pub temperature: Option<bool>,
    pub tool_call: Option<bool>,
    pub release_date: Option<String>,
    pub knowledge: Option<String>,
    pub modalities: Option<Modalities>,
    // // Ollama fields
    pub modified_at: Option<String>,
    pub format: Option<String>,
    pub family: Option<String>,
}

#[derive(Debug, Clone, JsonSchema, Serialize, Deserialize)]
pub struct Modalities {
    input: Vec<ModalityType>,
    output: Vec<ModalityType>,
}

#[derive(Debug, Clone, JsonSchema, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModalityType {
    Text,
    Image,
    Audio,
    Video,
    Pdf,
}

/// Ollama models list response
#[derive(Debug, Deserialize)]
pub struct OllamaModelsResponse {
    pub models: Vec<OllamaModelInfo>,
}

/// Ollama model information
#[derive(Debug, Deserialize)]
pub struct OllamaModelInfo {
    pub name: String,
    pub model: String,
    pub modified_at: String,
    // pub size: u64,
    // pub digest: String,
    pub details: OllamaModelDetails,
    #[serde(default)]
    pub capabilities: Vec<OllamaCapabilities>,
}

/// Ollama model details
#[derive(Debug, Deserialize)]
pub struct OllamaModelDetails {
    // #[serde(default)]
    // pub parent_model: String,
    pub format: String,
    pub family: String,
    // #[serde(default)]
    // pub families: Vec<String>,
    // pub parameter_size: String,
    // #[serde(default)]
    // pub quantization_level: Option<String>,
}

#[derive(Debug, Deserialize, strum::EnumIs)]
#[serde(rename_all = "lowercase")]
pub enum OllamaCapabilities {
    Completion,
    Tools,
    Vision,
    Thinking,
    #[serde(other)]
    Unknown,
}
