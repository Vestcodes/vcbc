//! Bootnode certificate implementation for VCBC network security.
//!
//! Provides cryptographic certificate generation, signing, and verification
//! for secure bootnode authentication and network trust establishment.

use crate::error::BlockchainError;
use chrono::{DateTime, Utc};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};

/// Bootnode certificate signed by a network authority
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BootnodeCertificate {
    /// Bootnode's node ID
    pub node_id: String,
    /// Network ID this bootnode serves
    pub network_id: String,
    /// Chain ID this bootnode serves
    pub chain_id: u64,
    /// Bootnode's HTTP endpoint
    pub http_url: String,
    /// Bootnode's P2P multiaddr
    pub p2p_multiaddr: String,
    /// Certificate expiry timestamp (Unix timestamp)
    pub expires_at: i64,
    /// Authority signature (base64 encoded)
    pub signature: String,
}

impl BootnodeCertificate {
    /// Create a new certificate for signing
    pub fn new(
        node_id: String,
        network_id: String,
        chain_id: u64,
        http_url: String,
        p2p_multiaddr: String,
        expires_at: i64,
    ) -> Self {
        Self {
            node_id,
            network_id,
            chain_id,
            http_url,
            p2p_multiaddr,
            expires_at,
            signature: String::new(),
        }
    }

    /// Generate the message to be signed (all fields except signature)
    pub fn signing_message(&self) -> String {
        format!(
            "{},{},{},{},{},{}",
            self.node_id,
            self.network_id,
            self.chain_id,
            self.http_url,
            self.p2p_multiaddr,
            self.expires_at
        )
    }

    /// Sign the certificate with an authority's private key
    pub fn sign(&mut self, signing_key: &SigningKey) -> Result<(), BlockchainError> {
        use base64::{engine::general_purpose, Engine as _};

        let message = self.signing_message();
        let signature = signing_key
            .try_sign(message.as_bytes())
            .map_err(|_| BlockchainError::InvalidInput("Failed to sign certificate".to_string()))?;
        self.signature = general_purpose::STANDARD.encode(signature.to_bytes());
        Ok(())
    }

    /// Verify the certificate signature against trusted authorities
    pub fn verify(
        &self,
        trusted_authorities: &[super::authority::NetworkAuthority],
    ) -> Result<(), BlockchainError> {
        use base64::{engine::general_purpose, Engine as _};

        if self.is_expired() {
            return Err(BlockchainError::InvalidInput(
                "Certificate has expired".to_string(),
            ));
        }

        // Find the authority that matches this network
        let authority = trusted_authorities
            .iter()
            .find(|auth| auth.name == format!("{}-authority", self.network_id))
            .ok_or_else(|| {
                BlockchainError::InvalidInput(format!(
                    "No trusted authority found for network: {}",
                    self.network_id
                ))
            })?;

        // Decode the public key
        let public_key_bytes = general_purpose::STANDARD
            .decode(&authority.public_key)
            .map_err(|e| BlockchainError::InvalidInput(format!("Invalid public key: {}", e)))?;

        let public_key_array: [u8; 32] = public_key_bytes
            .try_into()
            .map_err(|_| BlockchainError::InvalidInput("Invalid public key length".to_string()))?;

        let verifying_key = VerifyingKey::from_bytes(&public_key_array)
            .map_err(|e| BlockchainError::InvalidInput(format!("Invalid verifying key: {}", e)))?;

        // Decode the signature
        let signature_bytes = general_purpose::STANDARD
            .decode(&self.signature)
            .map_err(|e| BlockchainError::InvalidInput(format!("Invalid signature: {}", e)))?;

        let signature_array: [u8; 64] = signature_bytes
            .try_into()
            .map_err(|_| BlockchainError::InvalidInput("Invalid signature length".to_string()))?;

        let signature = Signature::from_bytes(&signature_array);

        // Verify the signature
        let message = self.signing_message();
        verifying_key
            .verify(message.as_bytes(), &signature)
            .map_err(|_| {
                BlockchainError::InvalidInput(
                    "Certificate signature verification failed".to_string(),
                )
            })?;

        Ok(())
    }

    /// Check if the certificate has expired
    pub fn is_expired(&self) -> bool {
        let now = Utc::now().timestamp();
        self.expires_at <= now
    }

    /// Get expiry as DateTime
    pub fn expiry_datetime(&self) -> DateTime<Utc> {
        DateTime::from_timestamp(self.expires_at, 0).unwrap_or_else(Utc::now)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::authority::NetworkAuthority;

    #[test]
    fn test_certificate_creation() {
        let cert = BootnodeCertificate::new(
            "node1".to_string(),
            "vc-mainnet".to_string(),
            1,
            "http://localhost:8080".to_string(),
            "/ip4/127.0.0.1/tcp/9090".to_string(),
            Utc::now().timestamp() + 3600,
        );

        assert_eq!(cert.node_id, "node1");
        assert_eq!(cert.network_id, "vc-mainnet");
        assert_eq!(cert.chain_id, 1);
        assert!(cert.signature.is_empty());
    }

    #[test]
    fn test_signing_message() {
        let cert = BootnodeCertificate::new(
            "node1".to_string(),
            "vc-mainnet".to_string(),
            1,
            "http://localhost:8080".to_string(),
            "/ip4/127.0.0.1/tcp/9090".to_string(),
            1234567890,
        );

        let message = cert.signing_message();
        assert!(message.contains("node1"));
        assert!(message.contains("vc-mainnet"));
        assert!(message.contains("1234567890"));
    }

    #[test]
    fn test_certificate_expiry() {
        let expired_cert = BootnodeCertificate::new(
            "node1".to_string(),
            "vc-mainnet".to_string(),
            1,
            "http://localhost:8080".to_string(),
            "/ip4/127.0.0.1/tcp/9090".to_string(),
            Utc::now().timestamp() - 3600, // 1 hour ago
        );

        let valid_cert = BootnodeCertificate::new(
            "node1".to_string(),
            "vc-mainnet".to_string(),
            1,
            "http://localhost:8080".to_string(),
            "/ip4/127.0.0.1/tcp/9090".to_string(),
            Utc::now().timestamp() + 3600, // 1 hour from now
        );

        assert!(expired_cert.is_expired());
        assert!(!valid_cert.is_expired());
    }

    #[test]
    fn test_certificate_sign_and_verify() {
        // Create authority
        let (authority, signing_key) = NetworkAuthority::new("vc-mainnet-authority".to_string());

        // Create certificate
        let mut cert = BootnodeCertificate::new(
            "node1".to_string(),
            "vc-mainnet".to_string(),
            1,
            "http://localhost:8080".to_string(),
            "/ip4/127.0.0.1/tcp/9090".to_string(),
            Utc::now().timestamp() + 3600,
        );

        // Sign certificate
        cert.sign(&signing_key).unwrap();
        assert!(!cert.signature.is_empty());

        // Verify certificate
        let trusted_authorities = vec![authority];
        assert!(cert.verify(&trusted_authorities).is_ok());
    }

    #[test]
    fn test_certificate_verify_expired() {
        let (authority, signing_key) = NetworkAuthority::new("vc-mainnet-authority".to_string());

        let mut cert = BootnodeCertificate::new(
            "node1".to_string(),
            "vc-mainnet".to_string(),
            1,
            "http://localhost:8080".to_string(),
            "/ip4/127.0.0.1/tcp/9090".to_string(),
            Utc::now().timestamp() - 3600, // Already expired
        );

        cert.sign(&signing_key).unwrap();

        let trusted_authorities = vec![authority];
        assert!(cert.verify(&trusted_authorities).is_err());
    }
}
