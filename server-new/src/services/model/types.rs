use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A model supported by the LLM provider
#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LlmModel {
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
    // pub modified_at: Option<String>,
    // pub format: Option<String>,
    // pub family: Option<String>,
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
