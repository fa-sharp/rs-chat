use uuid::Uuid;

use crate::{
    db::{
        models::{ChatRsProviderType, UpdateChatRsSession},
        services::ChatDbService,
        DbConnection, DbPool,
    },
    provider::{build_llm_provider_api, LlmApiProviderSharedOptions, DEFAULT_TEMPERATURE},
};

const TITLE_PROMPT: &str = "This is the first message sent by a human in a session with an AI chatbot. Please generate a short title for the session (max 6 words) in plain text";

/// Spawns a task to generate a title for the chat session
pub fn generate_title(
    user_id: &Uuid,
    session_id: &Uuid,
    user_message: &str,
    provider_type: ChatRsProviderType,
    model: &str,
    base_url: Option<&str>,
    api_key: Option<String>,
    http_client: &reqwest::Client,
    redis: &fred::prelude::Client,
    pool: &DbPool,
) {
    let user_id = user_id.to_owned();
    let session_id = session_id.to_owned();
    let user_message = user_message.to_owned();
    let model = model.to_owned();
    let base_url = base_url.map(|url| url.to_owned());
    let http_client = http_client.clone();
    let redis = redis.clone();
    let pool = pool.clone();

    tokio::spawn(async move {
        let Ok(conn) = pool.get().await else {
            rocket::error!("Couldn't get database connection");
            return;
        };
        let mut db = DbConnection(conn);
        let Ok(provider) = build_llm_provider_api(
            &provider_type,
            base_url.as_deref(),
            api_key.as_deref(),
            &http_client,
            &redis,
        ) else {
            rocket::warn!("Error creating provider for chat {}", session_id);
            return;
        };

        let provider_options = LlmApiProviderSharedOptions {
            model,
            temperature: Some(DEFAULT_TEMPERATURE),
            max_tokens: Some(20),
        };
        let provider_response = provider
            .prompt(
                &format!("{}: \"{}\"", TITLE_PROMPT, user_message),
                &provider_options,
            )
            .await;

        match provider_response {
            Ok(title) => {
                rocket::info!("Generated title for chat {}", session_id);
                if let Err(e) = ChatDbService::new(&mut db)
                    .update_session(
                        &user_id,
                        &session_id,
                        UpdateChatRsSession {
                            title: title.trim(),
                        },
                    )
                    .await
                {
                    rocket::warn!("Error saving title for chat {}: {}", session_id, e);
                };
            }
            Err(e) => {
                rocket::warn!("Error generating title for chat {}: {}", session_id, e);
            }
        }
    });
}
