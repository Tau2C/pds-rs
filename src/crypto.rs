use crate::id::KeyId;
use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use rand::RngExt;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum CryptoError {
    #[error("Encryption failure: {0}")]
    EncryptionFailed(String),
    #[error("Decryption failure: {0}")]
    DecryptionFailed(String),
    #[error("Key not found: {0}")]
    KeyNotFound(KeyId),
    #[error("Invalid key length: expected 32 bytes")]
    InvalidKeyLength,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncryptedData {
    pub key_id: KeyId,
    pub nonce: [u8; 12],
    pub ciphertext: Vec<u8>,
}

#[derive(Debug, Clone, Default)]
pub struct KeyStore {
    keys: BTreeMap<KeyId, [u8; 32]>,
}

impl KeyStore {
    pub fn new() -> Self {
        Self {
            keys: BTreeMap::new(),
        }
    }

    pub fn insert_key(&mut self, key_id: KeyId, key_bytes: [u8; 32]) {
        self.keys.insert(key_id, key_bytes);
    }

    pub fn generate_key(&mut self) -> (KeyId, [u8; 32]) {
        let key_id = KeyId::new();
        let mut key_bytes = [0u8; 32];
        rand::rng().fill(&mut key_bytes);
        self.keys.insert(key_id, key_bytes);
        (key_id, key_bytes)
    }

    pub fn get_key(&self, key_id: &KeyId) -> Option<&[u8; 32]> {
        self.keys.get(key_id)
    }

    pub fn encrypt(&self, key_id: &KeyId, plaintext: &[u8]) -> Result<EncryptedData, CryptoError> {
        let key_bytes = self.get_key(key_id).ok_or(CryptoError::KeyNotFound(*key_id))?;
        let cipher = ChaCha20Poly1305::new(&Key::from(*key_bytes));

        let mut nonce_bytes = [0u8; 12];
        rand::rng().fill(&mut nonce_bytes);
        let nonce = Nonce::from(nonce_bytes);

        let ciphertext = cipher
            .encrypt(&nonce, plaintext)
            .map_err(|e| CryptoError::EncryptionFailed(e.to_string()))?;

        Ok(EncryptedData {
            key_id: *key_id,
            nonce: nonce_bytes,
            ciphertext,
        })
    }

    pub fn decrypt(&self, encrypted: &EncryptedData) -> Result<Vec<u8>, CryptoError> {
        let key_bytes = self
            .get_key(&encrypted.key_id)
            .ok_or(CryptoError::KeyNotFound(encrypted.key_id))?;
        let cipher = ChaCha20Poly1305::new(&Key::from(*key_bytes));
        let nonce = Nonce::from(encrypted.nonce);

        cipher
            .decrypt(&nonce, encrypted.ciphertext.as_ref())
            .map_err(|e| CryptoError::DecryptionFailed(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keystore_encrypt_decrypt_roundtrip() {
        let mut keystore = KeyStore::new();
        let (key_id, _) = keystore.generate_key();

        let secret_payload = b"Top secret personal healthcare records";
        let encrypted = keystore.encrypt(&key_id, secret_payload).unwrap();
        assert_ne!(encrypted.ciphertext, secret_payload);

        let decrypted = keystore.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, secret_payload);
    }

    #[test]
    fn test_decrypt_with_tampered_ciphertext() {
        let mut keystore = KeyStore::new();
        let (key_id, _) = keystore.generate_key();

        let mut encrypted = keystore.encrypt(&key_id, b"sensitive data").unwrap();
        encrypted.ciphertext[0] ^= 0xFF; // Tamper

        assert!(keystore.decrypt(&encrypted).is_err());
    }
}
