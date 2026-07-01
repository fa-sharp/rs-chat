use std::sync::Arc;

use uuid::Uuid;

use crate::{
    db::{DbPool, DbService, models::UpdateChatRsSession},
    llm::{
        interface::LlmProvider,
        types::{LlmChatOptions, LlmPrompt},
    },
    services::chat::error::ChatError,
};

/// Spawn a task to generate a title for the chat session
pub fn generate_title(
    user_id: Uuid,
    session_id: Uuid,
    first_message: &str,
    provider: &Arc<dyn LlmProvider>,
    model: &str,
    pool: &DbPool,
) {
    let user_message = first_message.to_owned();
    let provider = Arc::clone(provider);
    let model = model.to_owned();
    let pool = pool.to_owned();

    tokio::spawn(async move {
        if let Err(err) = generate(user_id, session_id, user_message, provider, model, pool).await {
            tracing::warn!("Failed to generate title: {}", err);
        }
    });
}

const TITLE_PROMPT: &str = "This is the first message sent by a human in a chat session with an AI chatbot. \
    Please generate a short title for the chat session (3-7 words) in plain text, with no quotes or prefixes";
const TITLE_PROMPT_TEMPERATURE: f32 = 0.7;
const TITLE_PROMPT_MAX_TOKENS: u32 = 20;

async fn generate(
    user_id: Uuid,
    session_id: Uuid,
    user_message: String,
    provider: Arc<dyn LlmProvider>,
    model: String,
    db_pool: DbPool,
) -> Result<(), ChatError> {
    let message = format!("{TITLE_PROMPT}: \"{user_message}\"");
    let title = provider
        .prompt(LlmPrompt {
            text: &message,
            options: &LlmChatOptions {
                model,
                temperature: Some(TITLE_PROMPT_TEMPERATURE),
                max_tokens: Some(TITLE_PROMPT_MAX_TOKENS),
                ..Default::default()
            },
        })
        .await?;

    let mut db = DbService::from_pool(&db_pool).await?;
    db.chats()
        .update_session(
            &user_id,
            &session_id,
            UpdateChatRsSession {
                title: Some(title.trim()),
                ..Default::default()
            },
        )
        .await?;

    Ok(())
}
