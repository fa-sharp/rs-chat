use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    AeadCore, Aes256Gcm, Nonce,
};
use rocket::fairing::AdHoc;

use crate::{config::get_app_config, provider::LlmError};

/// Encryption service for encrypting and decrypting API keys
#[derive(Clone)]
pub struct Encryptor {
    cipher: Aes256Gcm,
}

impl Encryptor {
    pub fn new(key: &str) -> Result<Self, LlmError> {
        let key_bytes = hex::decode(key).or(Err(LlmError::EncryptionError))?;
        let cipher =
            Aes256Gcm::new_from_slice(&key_bytes).map_err(|_| LlmError::EncryptionError)?;
        Ok(Self { cipher })
    }

    /// Encrypts a string using AES-256-GCM and returns the ciphertext and nonce.
    pub fn encrypt_string(&self, plaintext: &str) -> Result<(Vec<u8>, Vec<u8>), LlmError> {
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ciphertext = self
            .cipher
            .encrypt(&nonce, plaintext.as_bytes())
            .map_err(|_| LlmError::EncryptionError)?;

        Ok((ciphertext, nonce.to_vec()))
    }

    /// Encrypts a byte slice using AES-256-GCM and returns the ciphertext and nonce.
    pub fn encrypt_bytes(&self, bytes: &[u8]) -> Result<(Vec<u8>, Vec<u8>), LlmError> {
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ciphertext = self
            .cipher
            .encrypt(&nonce, bytes)
            .map_err(|_| LlmError::EncryptionError)?;

        Ok((ciphertext, nonce.to_vec()))
    }

    /// Decrypts a string using AES-256-GCM.
    pub fn decrypt_string(&self, ciphertext: &[u8], nonce: &[u8]) -> Result<String, LlmError> {
        let nonce = Nonce::from_slice(nonce);
        let plaintext = self
            .cipher
            .decrypt(nonce, ciphertext)
            .map_err(|_| LlmError::DecryptionError)?;

        Ok(String::from_utf8(plaintext).map_err(|_| LlmError::DecryptionError)?)
    }

    /// Decrypts a byte slice using AES-256-GCM.
    pub fn decrypt_bytes(&self, ciphertext: &[u8], nonce: &[u8]) -> Result<Vec<u8>, LlmError> {
        let nonce = Nonce::from_slice(nonce);
        let bytes = self
            .cipher
            .decrypt(nonce, ciphertext)
            .map_err(|_| LlmError::DecryptionError)?;

        Ok(bytes)
    }
}

/// Fairing that sets up an encryption service
pub fn setup_encryption() -> AdHoc {
    AdHoc::on_ignite("Encryption setup", |rocket| async {
        let app_config = get_app_config(&rocket);
        let encryptor = Encryptor::new(&app_config.secret_key)
            .expect("Invalid secret key: must be 64-character hexadecimal string");

        rocket.manage(encryptor)
    })
}
