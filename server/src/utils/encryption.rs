use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    AeadCore, Aes256Gcm, Nonce,
};

use crate::provider::ChatRsError;

/// Encryption service for encrypting and decrypting API keys
#[derive(Clone)]
pub struct Encryptor {
    cipher: Aes256Gcm,
}

impl Encryptor {
    pub fn new(key: &str) -> Result<Self, ChatRsError> {
        let key_bytes = hex::decode(key).or(Err(ChatRsError::EncryptionError))?;
        let cipher =
            Aes256Gcm::new_from_slice(&key_bytes).map_err(|_| ChatRsError::EncryptionError)?;
        Ok(Self { cipher })
    }

    pub fn encrypt_string(&self, plaintext: &str) -> Result<(Vec<u8>, Vec<u8>), ChatRsError> {
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ciphertext = self
            .cipher
            .encrypt(&nonce, plaintext.as_bytes())
            .map_err(|_| ChatRsError::EncryptionError)?;

        Ok((ciphertext, nonce.to_vec()))
    }

    pub fn decrypt_string(&self, ciphertext: &[u8], nonce: &[u8]) -> Result<String, ChatRsError> {
        let nonce = Nonce::from_slice(nonce);
        let plaintext = self
            .cipher
            .decrypt(nonce, ciphertext)
            .map_err(|_| ChatRsError::DecryptionError)?;

        Ok(String::from_utf8(plaintext).map_err(|_| ChatRsError::DecryptionError)?)
    }
}
