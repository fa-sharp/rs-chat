use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, Generate, KeyInit},
};

/// Service for encrypting and decrypting secrets
pub struct Encryptor {
    cipher: Aes256Gcm,
}

type EncryptorResult<T> = Result<T, EncryptorError>;

/// Errors that can occur during encryption / decryption
#[derive(Debug, thiserror::Error)]
pub enum EncryptorError {
    #[error("encryption error")]
    Encryption,
    #[error("decryption error")]
    Decryption,
    #[error("invalid key")]
    InvalidKey,
    #[error("invalid nonce")]
    InvalidNonce,
}

impl Encryptor {
    pub fn new(key_bytes: &[u8]) -> EncryptorResult<Self> {
        let cipher =
            Aes256Gcm::new_from_slice(key_bytes).map_err(|_| EncryptorError::InvalidKey)?;
        Ok(Self { cipher })
    }

    /// Encrypts a string using AES-256-GCM and returns the ciphertext and nonce.
    pub fn encrypt_string(&self, plaintext: &str) -> EncryptorResult<(Vec<u8>, Vec<u8>)> {
        let nonce = Nonce::generate();
        let ciphertext = self
            .cipher
            .encrypt(&nonce, plaintext.as_bytes())
            .map_err(|_| EncryptorError::Encryption)?;

        Ok((ciphertext, nonce.to_vec()))
    }

    /// Encrypts a byte slice using AES-256-GCM and returns the ciphertext and nonce.
    pub fn encrypt_bytes(&self, bytes: &[u8]) -> EncryptorResult<(Vec<u8>, Vec<u8>)> {
        let nonce = Nonce::generate();
        let ciphertext = self
            .cipher
            .encrypt(&nonce, bytes)
            .map_err(|_| EncryptorError::Encryption)?;

        Ok((ciphertext, nonce.to_vec()))
    }

    /// Decrypts a string using AES-256-GCM.
    pub fn decrypt_string(&self, ciphertext: &[u8], nonce: &[u8]) -> EncryptorResult<String> {
        let nonce = Nonce::try_from(nonce).map_err(|_| EncryptorError::InvalidNonce)?;
        let plaintext = self
            .cipher
            .decrypt(&nonce, ciphertext)
            .map_err(|_| EncryptorError::Decryption)?;

        Ok(String::from_utf8(plaintext).map_err(|_| EncryptorError::Decryption)?)
    }

    /// Decrypts a byte slice using AES-256-GCM.
    pub fn decrypt_bytes(&self, ciphertext: &[u8], nonce: &[u8]) -> EncryptorResult<Vec<u8>> {
        let nonce = Nonce::try_from(nonce).map_err(|_| EncryptorError::InvalidNonce)?;
        let bytes = self
            .cipher
            .decrypt(&nonce, ciphertext)
            .map_err(|_| EncryptorError::Decryption)?;

        Ok(bytes)
    }
}
