use serde::{Deserialize, Serialize};

/// Generic LLM prompt
pub struct LlmPrompt<'r> {
    pub text: &'r str,
    pub options: &'r LlmChatOptions,
}

/// Generic LLM chat request
pub struct LlmChatRequest<'r> {
    pub messages: &'r [LlmMessage],
    // tools: Option<Vec<LlmTool>>,
    pub options: &'r LlmChatOptions,
}

/// Generic message type to send to LLM providers
pub enum LlmMessage {
    User(LlmUserMessage),
    Assistant(LlmAssistantMessage),
    System(String),
    // Tool(LlmToolResult),
}

/// Generic chat options for all LLM providers
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct LlmChatOptions {
    pub model: String,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    // /// Only supported for OpenRouter
    // #[serde(skip_serializing_if = "Option::is_none")]
    // pub modalities: Option<Vec<ModalityType>>,
}

#[derive(Default)]
pub struct LlmUserMessage {
    pub text: String,
    pub files: Option<Vec<LlmFileInput>>,
}

pub struct LlmFileInput {
    pub name: String,
    pub file_type: LlmFileType,
    pub content_type: String,
    pub content: String,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum LlmFileType {
    Text,
    Image,
    Pdf,
}

pub struct LlmAssistantMessage {
    pub text: String,
    // pub tool_calls: Option<Vec<LlmToolCall>>,
}

/// Usage stats from the LLM provider
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct LlmUsage {
    pub input_tokens: Option<u32>,
    pub output_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost: Option<f32>,
}

// pub struct LlmToolCall {
//     pub id: String,
//     pub tool_id: Uuid,
//     pub name: String,
//     pub tool_type: LlmToolType,
//     pub arguments: serde_json::Value,
// }

// /// Generic tool that can be passed to LLM providers
// #[derive(Debug)]
// pub struct LlmTool {
//     pub name: String,
//     pub description: String,
//     pub input_schema: serde_json::Value,
//     /// ID of the RsChat tool that this is derived from
//     pub tool_id: Uuid,
//     /// The type of tool this is derived from (internal, external API, etc.)
//     pub tool_type: LlmToolType,
// }

// #[derive(Default, Debug, Clone, Copy, Serialize, Deserialize)]
// #[serde(rename_all = "snake_case")]
// pub enum LlmToolType {
//     #[default]
//     System,
//     ExternalApi,
// }

// pub struct LlmToolResult {
//     pub tool_call_id: String,
//     pub tool_name: String,
//     pub content: String,
// }
