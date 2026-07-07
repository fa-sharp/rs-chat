use schemars::JsonSchema;
use serde::Deserialize;

use crate::db::models::{ChatRsProviderType, OpenAISubtype};

#[derive(Deserialize, JsonSchema)]
pub struct ProviderCreateInput {
    pub(super) name: String,
    pub(super) r#type: ChatRsProviderType,
    pub(super) openai_type: Option<OpenAISubtype>,
    pub(super) base_url: Option<String>,
    pub(super) default_model: String,
    pub(super) api_key: Option<String>,
}

#[derive(Deserialize, JsonSchema)]
pub struct ProviderUpdateInput {
    pub(super) name: Option<String>,
    pub(super) base_url: Option<String>,
    pub(super) default_model: Option<String>,
    pub(super) api_key: Option<String>,
}
