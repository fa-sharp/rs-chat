use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    AeadCore, Aes256Gcm, Nonce,
};

use crate::provider::ChatRsError;

pub fn encrypt_string(plaintext: &str, key: &str) -> Result<(Vec<u8>, Vec<u8>), ChatRsError> {
    let cipher = Aes256Gcm::new_from_slice(&key.as_bytes()[..32])
        .map_err(|_| ChatRsError::EncryptionError)?;
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher
        .encrypt(&nonce, plaintext.as_bytes())
        .map_err(|_| ChatRsError::EncryptionError)?;

    Ok((ciphertext, nonce.to_vec()))
}

pub fn decrypt_string(ciphertext: &[u8], nonce: &[u8], key: &str) -> Result<String, ChatRsError> {
    let cipher = Aes256Gcm::new_from_slice(&key.as_bytes()[..32])
        .map_err(|_| ChatRsError::DecryptionError)?;
    let nonce = Nonce::from_slice(nonce);
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| ChatRsError::DecryptionError)?;

    Ok(String::from_utf8(plaintext).map_err(|_| ChatRsError::DecryptionError)?)
}
