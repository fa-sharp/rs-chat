use rocket::{
    http::Status,
    outcome::{try_outcome, IntoOutcome},
    request::Outcome,
};
use uuid::Uuid;

use crate::{
    auth::ChatRsUserId,
    db::{services::ApiKeyDbService, DbConnection},
    utils::encryption::Encryptor,
};

const API_KEY_PREFIX: &str = "rs-chat-key";
const API_KEY_HEADER_PREFIX: &str = "Bearer rs-chat-key|";

/// Build an API key string from the given ciphertext and nonce
pub fn build_api_key_string(ciphertext: &[u8], nonce: &[u8]) -> String {
    format!(
        "{}|{}|{}",
        API_KEY_PREFIX,
        hex::encode(nonce),
        hex::encode(ciphertext)
    )
}

/// Handle login/authentication via API key
pub async fn get_api_key_auth_outcome<'r>(
    auth_header: &str,
    encryptor: &Encryptor,
    db: &mut DbConnection,
) -> Outcome<ChatRsUserId, &'r str> {
    let (nonce, ciphertext) = try_outcome!(auth_header
        .strip_prefix(API_KEY_HEADER_PREFIX)
        .and_then(|s| s.split_once('|'))
        .and_then(|(nonce_hex, cipher_hex)| (hex::decode(nonce_hex)
            .ok()
            .zip(hex::decode(cipher_hex).ok())))
        .or_error((Status::Unauthorized, "Invalid API key format")));

    let key_id = try_outcome!(encryptor
        .decrypt_bytes(&ciphertext, &nonce)
        .map_err(|_| "Couldn't decrypt API key")
        .and_then(|key_bytes| Uuid::from_slice(&key_bytes).map_err(|_| "Couldn't parse UUID"))
        .or_error(Status::Unauthorized));

    match ApiKeyDbService::new(db).find_by_id(&key_id).await {
        Ok(Some(api_key)) => Outcome::Success(ChatRsUserId(api_key.user_id)),
        Ok(None) => Outcome::Error((Status::Unauthorized, "API key not found")),
        Err(_) => Outcome::Error((Status::InternalServerError, "Database error")),
    }
}
