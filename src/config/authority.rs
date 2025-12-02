//! Network authority implementation for VCBC certificate management.
//!
//! Provides cryptographic key generation and management for network
//! authorities that sign bootnode certificates.

use crate::error::BlockchainError;
use ed25519_dalek::SigningKey;
use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};
use std::fs;
use zerocopy::FromBytes;

/// Network authority that can sign bootnode certificates
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NetworkAuthority {
    /// Authority name (e.g., "vc-mainnet-authority")
    pub name: String,
    /// Authority public key for verification (base64 encoded)
    pub public_key: String,
    /// Optional contact information
    pub contact: Option<String>,
}

impl NetworkAuthority {
    /// Create a new network authority
    pub fn new(name: String) -> (Self, SigningKey) {
        // Generate a random secret key
        let mut secret_bytes = [0u8; 32];
        OsRng.fill_bytes(&mut secret_bytes);
        let secret_key = ed25519_dalek::SecretKey::mut_from_bytes(&mut secret_bytes)
            .expect("Failed to create secret key from random bytes");
        let signing_key = SigningKey::from_bytes(secret_key);
        let verifying_key = signing_key.verifying_key();

        use base64::{engine::general_purpose, Engine as _};
        let public_key = general_purpose::STANDARD.encode(verifying_key.to_bytes());

        let authority = Self {
            name,
            public_key,
            contact: None,
        };

        (authority, signing_key)
    }

    /// Load authority from file
    pub fn load_from_file(path: &str) -> Result<Self, BlockchainError> {
        let contents = fs::read_to_string(path).map_err(|e| {
            BlockchainError::StorageError(format!(
                "Failed to read authority file '{}': {}",
                path, e
            ))
        })?;
        let authority: Self = serde_json::from_str(&contents).map_err(|e| {
            BlockchainError::StorageError(format!("Failed to parse authority: {}", e))
        })?;
        Ok(authority)
    }

    /// Save authority to file
    pub fn save_to_file(&self, path: &str) -> Result<(), BlockchainError> {
        let json = serde_json::to_string_pretty(self).map_err(|e| {
            BlockchainError::StorageError(format!("Failed to serialize authority: {}", e))
        })?;
        fs::write(path, json).map_err(|e| {
            BlockchainError::StorageError(format!(
                "Failed to write authority file '{}': {}",
                path, e
            ))
        })?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::ed25519::signature::SignerMut;
    use tempfile::tempdir;

    #[test]
    fn test_authority_creation() {
        let (authority, mut signing_key) =
            NetworkAuthority::new("vc-mainnet-authority".to_string());

        assert_eq!(authority.name, "vc-mainnet-authority");
        assert!(!authority.public_key.is_empty());
        assert!(authority.contact.is_none());

        // Verify the signing key works
        let message = b"test message";
        let signature = signing_key
            .try_sign(message)
            .expect("Failed to sign message");
        assert!(!signature.to_bytes().is_empty());
    }

    #[test]
    fn test_authority_save_load() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("authority.json");

        let (original_authority, _) = NetworkAuthority::new("test-authority".to_string());

        // Save authority
        original_authority
            .save_to_file(file_path.to_str().unwrap())
            .unwrap();

        // Load authority
        let loaded_authority =
            NetworkAuthority::load_from_file(file_path.to_str().unwrap()).unwrap();

        assert_eq!(original_authority.name, loaded_authority.name);
        assert_eq!(original_authority.public_key, loaded_authority.public_key);
        assert_eq!(original_authority.contact, loaded_authority.contact);
    }

    #[test]
    fn test_authority_unique_keys() {
        let (authority1, _) = NetworkAuthority::new("authority1".to_string());
        let (authority2, _) = NetworkAuthority::new("authority2".to_string());

        // Different authorities should have different public keys
        assert_ne!(authority1.public_key, authority2.public_key);
    }
}
