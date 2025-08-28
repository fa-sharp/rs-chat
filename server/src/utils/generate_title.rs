use std::ops::Deref;

use uuid::Uuid;

use crate::{
    db::{models::UpdateChatRsSession, services::ChatDbService, DbConnection, DbPool},
    errors::ApiError,
    provider::{LlmApiProvider, LlmProviderOptions, DEFAULT_TEMPERATURE},
};

const TITLE_TOKENS: u32 = 20;
const TITLE_PROMPT: &str =
    "This is the first message sent by a human in a chat session with an AI chatbot. \
    Please generate a short title for the session (3-7 words) in plain text \
    (no quotes or prefixes)";

/// Spawn a task to generate a title for the chat session
pub fn generate_title(
    user_id: &Uuid,
    session_id: &Uuid,
    user_message: &str,
    provider: &Box<dyn LlmApiProvider>,
    model: &str,
    pool: &DbPool,
) {
    let user_id = user_id.to_owned();
    let session_id = session_id.to_owned();
    let user_message = user_message.to_owned();
    let provider = dyn_clone::clone_box(provider.deref());
    let model = model.to_owned();
    let pool = pool.clone();

    tokio::spawn(async move {
        if let Err(err) = generate(user_id, session_id, user_message, provider, model, pool).await {
            rocket::warn!("Failed to generate title: {}", err);
        }
    });
}

async fn generate(
    user_id: Uuid,
    session_id: Uuid,
    user_message: String,
    provider: Box<dyn LlmApiProvider>,
    model: String,
    pool: DbPool,
) -> Result<(), ApiError> {
    let provider_options = LlmProviderOptions {
        model,
        temperature: Some(DEFAULT_TEMPERATURE),
        max_tokens: Some(TITLE_TOKENS),
    };
    let message = format!("{}: \"{}\"", TITLE_PROMPT, user_message);
    let title = provider.prompt(&message, &provider_options).await?;

    let mut db = DbConnection(pool.get().await?);
    ChatDbService::new(&mut db)
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
