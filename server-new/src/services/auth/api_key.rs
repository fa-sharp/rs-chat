use uuid::Uuid;

use crate::{
    db::{DbService, models::NewChatRsApiKey},
    services::auth::{
        encryption::Encryptor,
        error::{AuthError, AuthResult},
    },
};

const API_KEY_PREFIX: &str = "rs-chat-key";
const API_KEY_HEADER_PREFIX: &str = "Bearer rs-chat-key|";

pub struct ApiKeyService<'r> {
    encryptor: &'r Encryptor,
}

impl<'r> ApiKeyService<'r> {
    pub fn new(encryptor: &'r Encryptor) -> Self {
        Self { encryptor }
    }

    /// Build an API key string from the given ciphertext and nonce
    fn build_api_key(ciphertext: &[u8], nonce: &[u8]) -> String {
        format!(
            "{API_KEY_PREFIX}|{}|{}",
            hex::encode(nonce),
            hex::encode(ciphertext)
        )
    }

    /// Create an API key, and return its ID and the encrypted key
    pub async fn create_api_key(
        &self,
        db: &mut DbService,
        user_id: &Uuid,
        name: &str,
    ) -> AuthResult<(Uuid, String)> {
        let key_id = db
            .api_keys()
            .create(NewChatRsApiKey {
                user_id: &user_id,
                name: &name,
            })
            .await?;
        let (ciphertext, nonce) = self.encryptor.encrypt_bytes(key_id.as_bytes())?;

        Ok((key_id, Self::build_api_key(&ciphertext, &nonce)))
    }

    /// Validate the API key and get the user ID
    pub async fn validate_api_key(
        &self,
        db: &mut DbService,
        auth_header: &str,
    ) -> AuthResult<Uuid> {
        let (nonce, ciphertext) = auth_header
            .strip_prefix(API_KEY_HEADER_PREFIX)
            .and_then(|s| s.split_once('|'))
            .and_then(|(nonce_hex, cipher_hex)| {
                hex::decode(nonce_hex)
                    .ok()
                    .zip(hex::decode(cipher_hex).ok())
            })
            .ok_or(AuthError::Unauthorized("invalid API key format"))?;
        let api_key_id = self
            .encryptor
            .decrypt_bytes(&ciphertext, &nonce)
            .map_err(|_| AuthError::Unauthorized("failed to decrypt API key"))
            .and_then(|key_bytes| {
                Uuid::from_slice(&key_bytes)
                    .map_err(|_| AuthError::Unauthorized("couldn't parse API key id"))
            })?;

        match db.api_keys().find_by_id(&api_key_id).await? {
            Some(api_key) => Ok(api_key.user_id),
            None => Err(AuthError::Unauthorized("API key not found")),
        }
    }
}
