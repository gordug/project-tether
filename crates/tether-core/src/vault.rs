use argon2::Argon2;
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Nonce,
};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Error, Debug)]
pub enum VaultError {
    #[error("Failed to derive master encryption key: {0}")]
    KeyDerivation(String),
    #[error("Decryption failed: invalid master password or corrupted vault payload")]
    DecryptionFailed,
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize, Zeroize, ZeroizeOnDrop)]
pub struct SecretString(pub String);

impl SecretString {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    pub fn expose(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct VaultStore {
    /// Maps host_id to secret (password, passphrase, or private key)
    pub secrets: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedVault {
    pub version: u32,
    pub salt: Vec<u8>,
    pub nonce: Vec<u8>,
    pub ciphertext: Vec<u8>,
}

pub struct VaultEngine;

impl VaultEngine {
    const KEY_LEN: usize = 32;
    const NONCE_LEN: usize = 12;
    const SALT_LEN: usize = 16;

    fn derive_key(master_password: &str, salt: &[u8]) -> Result<[u8; Self::KEY_LEN], VaultError> {
        let mut key = [0u8; Self::KEY_LEN];
        let argon = Argon2::default();
        argon
            .hash_password_into(master_password.as_bytes(), salt, &mut key)
            .map_err(|e| VaultError::KeyDerivation(e.to_string()))?;
        Ok(key)
    }

    pub fn encrypt(master_password: &str, store: &VaultStore) -> Result<EncryptedVault, VaultError> {
        let mut salt = vec![0u8; Self::SALT_LEN];
        let mut nonce_bytes = vec![0u8; Self::NONCE_LEN];
        rand::thread_rng().fill_bytes(&mut salt);
        rand::thread_rng().fill_bytes(&mut nonce_bytes);

        let mut key = Self::derive_key(master_password, &salt)?;
        let cipher = ChaCha20Poly1305::new_from_slice(&key)
            .map_err(|e| VaultError::EncryptionFailed(e.to_string()))?;
        key.zeroize();

        let json_plain = serde_json::to_vec(store)?;
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = cipher
            .encrypt(nonce, json_plain.as_ref())
            .map_err(|e| VaultError::EncryptionFailed(e.to_string()))?;

        Ok(EncryptedVault {
            version: 1,
            salt,
            nonce: nonce_bytes,
            ciphertext,
        })
    }

    pub fn decrypt(
        master_password: &str,
        vault: &EncryptedVault,
    ) -> Result<VaultStore, VaultError> {
        if vault.nonce.len() != Self::NONCE_LEN {
            return Err(VaultError::DecryptionFailed);
        }

        let mut key = Self::derive_key(master_password, &vault.salt)?;
        let cipher = ChaCha20Poly1305::new_from_slice(&key)
            .map_err(|_| VaultError::DecryptionFailed)?;
        key.zeroize();

        let nonce = Nonce::from_slice(&vault.nonce);
        let decrypted_bytes = cipher
            .decrypt(nonce, vault.ciphertext.as_ref())
            .map_err(|_| VaultError::DecryptionFailed)?;

        let store: VaultStore = serde_json::from_slice(&decrypted_bytes)?;
        Ok(store)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vault_roundtrip() {
        let mut store = VaultStore::default();
        store.secrets.insert("host-1".into(), "s3cret_p@ssw0rd".into());
        store.secrets.insert("host-2".into(), "-----BEGIN RSA PRIVATE KEY-----...".into());

        let master = "StrongMasterPassw0rd!";
        let encrypted = VaultEngine::encrypt(master, &store).expect("encryption succeeds");

        assert_eq!(encrypted.version, 1);
        assert_eq!(encrypted.salt.len(), 16);
        assert_eq!(encrypted.nonce.len(), 12);
        assert!(!encrypted.ciphertext.is_empty());

        let decrypted = VaultEngine::decrypt(master, &encrypted).expect("decryption succeeds");
        assert_eq!(store, decrypted);

        // Wrong master password must fail
        let wrong_result = VaultEngine::decrypt("WrongPassword", &encrypted);
        assert!(wrong_result.is_err());
    }
}
