use super::{
    error::ModelError,
    types::{LlmModel, OllamaModelsResponse},
};

pub(super) async fn ollama_models(
    client: &reqwest::Client,
    base_url: Option<&str>,
) -> Result<Vec<LlmModel>, ModelError> {
    const DEFAULT_BASE_URL: &str = "http://localhost:11434";
    const MODELS_API_PATH: &str = "/api/tags";

    let models_url = format!("{}{MODELS_API_PATH}", base_url.unwrap_or(DEFAULT_BASE_URL));
    let response: OllamaModelsResponse = client
        .get(models_url)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    let models = response
        .models
        .into_iter()
        .map(|model| LlmModel {
            id: model.name.clone(),
            name: model.name,
            temperature: Some(true),
            modified_at: Some(model.modified_at),
            format: Some(model.details.format),
            family: Some(model.details.family),
            tool_call: Some(model.capabilities.iter().any(|c| c.is_tools())),
            reasoning: Some(model.capabilities.iter().any(|c| c.is_thinking())),
            ..Default::default()
        })
        .collect();

    Ok(models)
}
