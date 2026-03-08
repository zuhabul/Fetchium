//! AES-256-GCM cache encryption (PRD §36.3).
//!
//! Key derivation: Argon2id with a fixed salt (uniqueness via per-message nonces).
//! Nonce generation: UUID v4 bytes (CSPRNG-backed via the `uuid` crate).

use crate::error::FetchiumError;
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce,
};

const NONCE_LEN: usize = 12;

/// Fixed 16-byte salt for deterministic key derivation.
/// Uniqueness is ensured by per-message random nonces, not the salt.
const KDF_SALT: &[u8] = b"fetchiumcache__";

/// AES-256-GCM encryption engine, keyed from a passphrase via Argon2id.
pub struct CacheEncryption {
    cipher: Aes256Gcm,
}

impl CacheEncryption {
    /// Derive a 256-bit key from `passphrase` using Argon2id.
    pub fn new(passphrase: &str) -> Result<Self, FetchiumError> {
        let mut key_bytes = [0u8; 32];
        argon2::Argon2::default()
            .hash_password_into(passphrase.as_bytes(), KDF_SALT, &mut key_bytes)
            .map_err(|e| FetchiumError::Internal(format!("argon2 key derivation error: {e}")))?;

        let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
        let cipher = Aes256Gcm::new(key);
        Ok(Self { cipher })
    }

    /// Encrypt `data` with a random nonce. Returns `nonce || ciphertext`.
    pub fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>, FetchiumError> {
        // Generate a 12-byte nonce from a UUID v4 (CSPRNG-backed).
        let uuid_bytes = uuid::Uuid::new_v4().into_bytes();
        let nonce_bytes: [u8; NONCE_LEN] = uuid_bytes[..NONCE_LEN]
            .try_into()
            .expect("uuid is 16 bytes, nonce is 12");

        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = self
            .cipher
            .encrypt(nonce, data)
            .map_err(|e| FetchiumError::Internal(format!("encrypt error: {e}")))?;

        let mut out = nonce_bytes.to_vec();
        out.extend(ciphertext);
        Ok(out)
    }

    /// Decrypt `data` (format: `nonce || ciphertext`).
    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>, FetchiumError> {
        if data.len() <= NONCE_LEN {
            return Err(FetchiumError::Internal("Data too short to decrypt".into()));
        }
        let (nonce_bytes, ciphertext) = data.split_at(NONCE_LEN);
        let nonce = Nonce::from_slice(nonce_bytes);
        self.cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| FetchiumError::Internal(format!("decrypt error: {e}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_encrypt_decrypt() {
        let enc = CacheEncryption::new("test-passphrase").unwrap();
        let plaintext = b"hello, world!";
        let encrypted = enc.encrypt(plaintext).unwrap();
        let decrypted = enc.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn different_passphrases_fail_to_decrypt() {
        let enc1 = CacheEncryption::new("passphrase-A").unwrap();
        let enc2 = CacheEncryption::new("passphrase-B").unwrap();
        let encrypted = enc1.encrypt(b"secret").unwrap();
        assert!(enc2.decrypt(&encrypted).is_err());
    }
}
