use std::sync::Arc;

use uuid::Uuid;

use crate::{
    db::{
        DbPool, DbService,
        models::{ChatRsLogKind, ChatRsLogStatus, UpdateChatRsSession},
    },
    llm::{
        interface::{LlmProvider, LlmResponse},
        types::{LlmChatOptions, LlmPrompt},
    },
    services::chat::error::ChatError,
};

/// Spawn a task to generate a title for the chat session
pub fn generate_title(
    user_id: Uuid,
    sess_id: Uuid,
    provider_id: i32,
    provider: &Arc<dyn LlmProvider>,
    first_message: &str,
    model: &str,
    pool: &DbPool,
) {
    let msg = first_message.to_owned();
    let provider = Arc::clone(provider);
    let model = model.to_owned();
    let pool = pool.to_owned();

    tokio::spawn(async move {
        if let Err(err) = generate(user_id, sess_id, provider_id, provider, msg, model, pool).await
        {
            tracing::warn!("Error while generating session title: {err}");
        }
    });
}

const TITLE_PROMPT: &str = "This is the first message sent by a human in a chat session with an AI chatbot. \
    Please generate a short title for the chat session (3-7 words) in plain text, with no quotes or prefixes.";
const TITLE_PROMPT_TEMPERATURE: f32 = 0.7;
const TITLE_PROMPT_MAX_TOKENS: u32 = 20;

async fn generate(
    user_id: Uuid,
    session_id: Uuid,
    provider_id: i32,
    provider: Arc<dyn LlmProvider>,
    user_message: String,
    model: String,
    db_pool: DbPool,
) -> Result<(), ChatError> {
    let mut db = DbService::from_pool(&db_pool).await?;
    let log_id = db
        .logs()
        .create()
        .user_id(&user_id)
        .session_id(&session_id)
        .provider_id(provider_id)
        .kind(ChatRsLogKind::Title)
        .model(&model)
        .build()
        .await?;

    let prompt = LlmPrompt {
        text: &format!("{TITLE_PROMPT}\n\n\"{user_message}\""),
        options: &LlmChatOptions {
            model,
            temperature: Some(TITLE_PROMPT_TEMPERATURE),
            max_tokens: Some(TITLE_PROMPT_MAX_TOKENS),
            ..Default::default()
        },
    };
    match provider.prompt(prompt).await {
        Ok(LlmResponse { text, usage, meta }) => {
            db.logs()
                .complete()
                .id(log_id)
                .status(ChatRsLogStatus::Completed)
                .usage(&usage)
                .maybe_request_id(meta.request_id.as_deref())
                .build()
                .await?;
            db.chats()
                .update_session(
                    &user_id,
                    &session_id,
                    UpdateChatRsSession {
                        title: Some(text.trim()),
                        ..Default::default()
                    },
                )
                .await?;
        }
        Err(err) => {
            db.logs()
                .complete()
                .id(log_id)
                .status(ChatRsLogStatus::Failed)
                .error(&err.to_string())
                .build()
                .await?;
        }
    }

    Ok(())
}
