use uuid::Uuid;

use crate::{
    db::{
        models::UpdateChatRsSession,
        services::{ChatDbService, ProviderKeyDbService},
        DbConnection, DbPool,
    },
    utils::encryption::Encryptor,
};

use super::create_provider::{create_provider, ProviderConfigInput};

/// Spawns a task to generate a title for the chat session
pub fn generate_title(
    user_id: &Uuid,
    session_id: &Uuid,
    user_message: &str,
    provider_config: &ProviderConfigInput,
    encryptor: &Encryptor,
    http_client: &reqwest::Client,
    pool: &DbPool,
) {
    let user_id = user_id.to_owned();
    let session_id = session_id.to_owned();
    let message = user_message.to_string();
    let config = provider_config.clone();
    let encryptor = encryptor.clone();
    let http_client = http_client.clone();
    let pool = pool.clone();

    tokio::spawn(async move {
        let Ok(conn) = pool.get().await else {
            rocket::error!("Couldn't get database connection");
            return;
        };
        let mut db = DbConnection(conn);
        let Ok(provider) = create_provider(
            &user_id,
            &config,
            &mut ProviderKeyDbService::new(&mut db),
            &encryptor,
            &http_client,
        )
        .await
        else {
            rocket::warn!("Error creating provider for chat {}", session_id);
            return;
        };

        let title_prompt = "This is the first message sent by a human in a session with an AI chatbot. Please generate a short title for the session (max 6 words) in plain text";
        let provider_response = provider
            .prompt(&format!("{}: \"{}\"", title_prompt, message))
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
