use serde::Serialize;

use crate::llm::{
    providers::utils,
    types::{LlmFileType, LlmMessage},
};

pub fn build_openai_messages<'a>(messages: &'a [LlmMessage]) -> Vec<OpenAIMessage<'a>> {
    messages
        .iter()
        .map(|message| match message {
            LlmMessage::User(user_message) => {
                let mut content = Vec::new();
                if !user_message.text.is_empty() {
                    content.push(OpenAIContent::Text {
                        text: &user_message.text,
                    });
                }
                if let Some(ref files) = user_message.files {
                    content.extend(files.iter().map(|file| match file.file_type {
                        LlmFileType::Text => OpenAIContent::Text {
                            text: &file.content,
                        },
                        LlmFileType::Image => OpenAIContent::ImageUrl {
                            image_url: OpenAIImageUrl {
                                url: utils::create_data_uri(&file.content_type, &file.content),
                            },
                        },
                        LlmFileType::Pdf => OpenAIContent::File {
                            file: OpenAIFile {
                                file_data: utils::create_data_uri(
                                    &file.content_type,
                                    &file.content,
                                ),
                                filename: &file.name,
                            },
                        },
                    }));
                }
                OpenAIMessage {
                    role: "user",
                    content: Some(content),
                    ..Default::default()
                }
            }
            LlmMessage::Assistant(assistant_message) => {
                // let tool_calls = assistant_message.tool_calls.as_ref().map(|tc| {
                //     tc.iter()
                //         .map(|tc| OpenAIToolCall {
                //             id: &tc.id,
                //             tool_type: "function",
                //             function: OpenAIToolCallFunction {
                //                 name: &tc.tool_name,
                //                 arguments: serde_json::to_string(&tc.parameters)
                //                     .unwrap_or_default(),
                //             },
                //         })
                //         .collect()
                // });
                OpenAIMessage {
                    role: "assistant",
                    content: (!assistant_message.text.is_empty()).then(|| {
                        vec![OpenAIContent::Text {
                            text: &assistant_message.text,
                        }]
                    }),
                    // tool_calls,
                    ..Default::default()
                }
            }
            LlmMessage::System(text) => OpenAIMessage {
                role: "system",
                content: Some(vec![OpenAIContent::Text { text }]),
                ..Default::default()
            },
            // LlmMessage::Tool(tool_result) => OpenAIMessage {
            //     role: "tool",
            //     content: Some(vec![OpenAIContent::Text {
            //         text: &tool_result.content,
            //     }]),
            //     tool_call_id: Some(&tool_result.tool_call_id),
            //     ..Default::default()
            // },
        })
        .collect()
}

// pub fn build_openai_tools<'a>(tools: &'a [LlmTool]) -> Vec<OpenAITool<'a>> {
//     tools
//         .iter()
//         .map(|tool| OpenAITool {
//             tool_type: "function",
//             function: OpenAIToolFunction {
//                 name: &tool.name,
//                 description: &tool.description,
//                 parameters: &tool.input_schema,
//                 strict: true,
//             },
//         })
//         .collect()
// }

/// OpenAI API request body
#[derive(Debug, Default, Serialize)]
pub struct OpenAIRequest<'a> {
    pub model: &'a str,
    pub messages: Vec<OpenAIMessage<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_completion_tokens: Option<u32>,
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub store: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_options: Option<OpenAIStreamOptions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<OpenAITool<'a>>>,
    // #[serde(skip_serializing_if = "Option::is_none")]
    // pub modalities: Option<&'a Vec<ModalityType>>,
}

/// OpenAI API request stream options
#[derive(Debug, Serialize)]
pub struct OpenAIStreamOptions {
    pub include_usage: bool,
}

/// OpenAI API request message
#[derive(Debug, Default, Serialize)]
pub struct OpenAIMessage<'a> {
    pub role: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Vec<OpenAIContent<'a>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<OpenAIToolCall<'a>>>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OpenAIContent<'a> {
    Text { text: &'a str },
    ImageUrl { image_url: OpenAIImageUrl },
    File { file: OpenAIFile<'a> },
}

#[derive(Debug, Serialize)]
pub struct OpenAIImageUrl {
    url: String,
}

#[derive(Debug, Serialize)]
pub struct OpenAIFile<'a> {
    file_data: String,
    filename: &'a str,
}

/// OpenAI tool definition
#[derive(Debug, Serialize)]
pub struct OpenAITool<'a> {
    #[serde(rename = "type")]
    tool_type: &'a str,
    function: OpenAIToolFunction<'a>,
}

/// OpenAI tool function definition
#[derive(Debug, Serialize)]
pub struct OpenAIToolFunction<'a> {
    name: &'a str,
    description: &'a str,
    strict: bool,
    parameters: &'a serde_json::Value,
}

/// OpenAI tool call in messages
#[derive(Debug, Serialize)]
pub struct OpenAIToolCall<'a> {
    id: &'a str,
    #[serde(rename = "type")]
    tool_type: &'a str,
    function: OpenAIToolCallFunction<'a>,
}

/// OpenAI tool call function in messages
#[derive(Debug, Serialize)]
pub struct OpenAIToolCallFunction<'a> {
    name: &'a str,
    arguments: String,
}
