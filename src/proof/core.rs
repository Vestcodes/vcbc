//! Provides cryptographic proof generation and verification for MPT data.
//! Clean separation of concerns with focused, single-responsibility functions.

use crate::error::{BlockchainError, Result};
use chrono::Utc;
use serde::{Deserialize, Serialize};

/// Merkle proof for MPT data verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleProof {
    /// The key being proven
    pub key: String,
    /// The value associated with the key
    pub value: Vec<u8>,
    /// Block index containing the MPT
    pub block_index: u64,
    /// MPT root hash for verification
    pub mpt_root_hash: Vec<u8>,
    /// Block hash for additional verification
    pub block_hash: String,
    /// Timestamp when proof was generated
    pub timestamp: String,
}

impl MerkleProof {
    /// Create a new Merkle proof
    pub fn new(
        key: String,
        value: Vec<u8>,
        block_index: u64,
        mpt_root_hash: Vec<u8>,
        block_hash: String,
    ) -> Self {
        let timestamp = Utc::now().to_rfc3339();
        MerkleProof {
            key,
            value,
            block_index,
            mpt_root_hash,
            block_hash,
            timestamp,
        }
    }

    /// Verify the proof against a blockchain
    pub fn verify(&self, blockchain: &crate::Blockchain) -> bool {
        self.verify_block_exists(blockchain)
            && self.verify_mpt_root(blockchain)
            && self.verify_key_value(blockchain)
    }

    /// Verify against a known MPT root hash
    pub fn verify_against_root(&self, expected_root: &[u8]) -> bool {
        self.mpt_root_hash == expected_root
    }

    /// Get the proof data as a formatted string
    pub fn format(&self) -> String {
        format!(
            "Merkle Proof:\n\
             Key: {}\n\
             Value: {} ({} bytes)\n\
             Block Index: {}\n\
             Block Hash: {}\n\
             MPT Root: {}\n\
             Timestamp: {}",
            self.key,
            String::from_utf8_lossy(&self.value),
            self.value.len(),
            self.block_index,
            self.block_hash,
            hex::encode(&self.mpt_root_hash),
            self.timestamp
        )
    }

    // Private verification methods

    fn verify_block_exists(&self, blockchain: &crate::Blockchain) -> bool {
        blockchain.get_block(self.block_index as usize).is_some()
    }

    fn verify_mpt_root(&self, blockchain: &crate::Blockchain) -> bool {
        if let Some(block) = blockchain.get_block(self.block_index as usize) {
            if block.hash == self.block_hash {
                let expected_root = block.data.root_hash();
                expected_root == self.mpt_root_hash
            } else {
                false
            }
        } else {
            false
        }
    }

    fn verify_key_value(&self, blockchain: &crate::Blockchain) -> bool {
        if let Some(block) = blockchain.get_block(self.block_index as usize) {
            if let Some(stored_value) = block.data.get(&self.key) {
                stored_value == self.value
            } else {
                false
            }
        } else {
            false
        }
    }
}

#[test]
fn test_proof_creation() {
    let mut blockchain = crate::blockchain::Blockchain::new_default(1);
    let mut mpt = crate::mpt::MerklePatriciaTrie::new();
    mpt.insert("user", b"alice".to_vec()).unwrap();
    blockchain.add_block(mpt);
    let block = blockchain.get_block(1).unwrap();

    let proof = MerkleProof::new(
        "user".to_string(),
        b"alice".to_vec(),
        1,
        block.data.root_hash(),
        block.hash.clone(),
    );

    assert_eq!(proof.key, "user");
    assert_eq!(proof.value, b"alice");
    assert_eq!(proof.block_index, 1);
    assert_eq!(proof.mpt_root_hash, block.data.root_hash());
    assert_eq!(proof.block_hash, block.hash);
    assert!(!proof.timestamp.is_empty());
}

#[test]
fn test_valid_proof_verification() {
    let mut blockchain = crate::blockchain::Blockchain::new_default(1);
    let mut mpt = crate::mpt::MerklePatriciaTrie::new();
    mpt.insert("user", b"alice".to_vec()).unwrap();
    blockchain.add_block(mpt);
    let block = blockchain.get_block(1).unwrap();

    // Create valid proof
    let proof = MerkleProof::new(
        "user".to_string(),
        b"alice".to_vec(),
        1,
        block.data.root_hash(),
        block.hash.clone(),
    );

    let is_valid = proof.verify(&blockchain);
    assert!(is_valid);
}

#[test]
fn test_invalid_proof_wrong_value() {
    let mut blockchain = crate::blockchain::Blockchain::new_default(1);
    let mut mpt = crate::mpt::MerklePatriciaTrie::new();
    mpt.insert("user", b"alice".to_vec()).unwrap();
    blockchain.add_block(mpt);
    let block = blockchain.get_block(1).unwrap();

    // Create proof with wrong value
    let proof = MerkleProof::new(
        "user".to_string(),
        b"bob".to_vec(), // Wrong value
        1,
        block.data.root_hash(),
        block.hash.clone(),
    );

    let is_valid = proof.verify(&blockchain);
    assert!(!is_valid);
}

#[test]
fn test_invalid_proof_wrong_key() {
    let mut blockchain = crate::blockchain::Blockchain::new_default(1);
    let mut mpt = crate::mpt::MerklePatriciaTrie::new();
    mpt.insert("user", b"alice".to_vec()).unwrap();
    blockchain.add_block(mpt);
    let block = blockchain.get_block(1).unwrap();

    // Create proof with non-existent key
    let proof = MerkleProof::new(
        "nonexistent".to_string(),
        b"some_value".to_vec(),
        1,
        block.data.root_hash(),
        block.hash.clone(),
    );

    let is_valid = proof.verify(&blockchain);
    assert!(!is_valid);
}

#[test]
fn test_invalid_proof_wrong_block_index() {
    let mut blockchain = crate::blockchain::Blockchain::new_default(1);
    let mut mpt = crate::mpt::MerklePatriciaTrie::new();
    mpt.insert("user", b"alice".to_vec()).unwrap();
    blockchain.add_block(mpt);
    let block = blockchain.get_block(1).unwrap();

    // Create proof with wrong block index
    let proof = MerkleProof::new(
        "user".to_string(),
        b"alice".to_vec(),
        999, // Non-existent block
        block.data.root_hash(),
        block.hash.clone(),
    );

    let is_valid = proof.verify(&blockchain);
    assert!(!is_valid);
}

#[test]
fn test_invalid_proof_wrong_block_hash() {
    let mut blockchain = crate::blockchain::Blockchain::new_default(1);
    let mut mpt = crate::mpt::MerklePatriciaTrie::new();
    mpt.insert("user", b"alice".to_vec()).unwrap();
    blockchain.add_block(mpt);
    let block = blockchain.get_block(1).unwrap();

    // Create proof with wrong block hash
    let proof = MerkleProof::new(
        "user".to_string(),
        b"alice".to_vec(),
        1,
        block.data.root_hash(),
        "wrong_hash".to_string(),
    );

    let is_valid = proof.verify(&blockchain);
    assert!(!is_valid);
}

#[test]
fn test_invalid_proof_wrong_mpt_root() {
    let mut blockchain = crate::blockchain::Blockchain::new_default(1);
    let mut mpt = crate::mpt::MerklePatriciaTrie::new();
    mpt.insert("user", b"alice".to_vec()).unwrap();
    blockchain.add_block(mpt);
    let block = blockchain.get_block(1).unwrap();

    // Create proof with wrong MPT root hash
    let proof = MerkleProof::new(
        "user".to_string(),
        b"alice".to_vec(),
        1,
        vec![0u8; 32], // Wrong root hash
        block.hash.clone(),
    );

    let is_valid = proof.verify(&blockchain);
    assert!(!is_valid);
}

#[test]
fn test_verify_against_root() {
    let mut mpt = crate::mpt::MerklePatriciaTrie::new();
    mpt.insert("key", b"value".to_vec()).unwrap();
    let root_hash = mpt.root_hash();

    let proof = MerkleProof::new(
        "key".to_string(),
        b"value".to_vec(),
        0,
        root_hash.clone(),
        "block_hash".to_string(),
    );

    // Should verify against correct root
    assert!(proof.verify_against_root(&root_hash));

    // Should not verify against wrong root
    let wrong_root = vec![1u8; 32];
    assert!(!proof.verify_against_root(&wrong_root));
}

#[test]
fn test_proof_with_empty_value() {
    let mut blockchain = crate::blockchain::Blockchain::new_default(1);
    let mut mpt = crate::mpt::MerklePatriciaTrie::new();
    mpt.insert("user", b"alice".to_vec()).unwrap();
    blockchain.add_block(mpt);
    let block = blockchain.get_block(1).unwrap();

    let proof = MerkleProof::new(
        "user".to_string(),
        Vec::new(), // Empty value
        1,
        block.data.root_hash(),
        block.hash.clone(),
    );

    // This should fail since "user" has value "alice", not empty
    let is_valid = proof.verify(&blockchain);
    assert!(!is_valid);
}

#[test]
fn test_proof_with_large_value() {
    let mut blockchain = crate::blockchain::Blockchain::new_default(1);
    let large_value = vec![0u8; 10000]; // 10KB value
    let mut mpt = crate::mpt::MerklePatriciaTrie::new();
    mpt.insert("large_key", large_value.clone()).unwrap();
    blockchain.add_block(mpt);

    let block = blockchain.get_block(1).unwrap();
    let proof = MerkleProof::new(
        "large_key".to_string(),
        large_value,
        1,
        block.data.root_hash(),
        block.hash.clone(),
    );

    let is_valid = proof.verify(&blockchain);
    assert!(is_valid);
}

#[test]
fn test_proof_with_unicode_key_value() {
    let mut blockchain = crate::blockchain::Blockchain::new_default(1);
    let unicode_key = "🚀测试";
    let unicode_value = "区块链数据".as_bytes().to_vec();
    let mut mpt = crate::mpt::MerklePatriciaTrie::new();
    mpt.insert(unicode_key, unicode_value.clone()).unwrap();
    blockchain.add_block(mpt);

    let block = blockchain.get_block(1).unwrap();
    let proof = MerkleProof::new(
        unicode_key.to_string(),
        unicode_value,
        1,
        block.data.root_hash(),
        block.hash.clone(),
    );

    let is_valid = proof.verify(&blockchain);
    assert!(is_valid);
}

#[test]
fn test_proof_timestamp() {
    let mut blockchain = crate::blockchain::Blockchain::new_default(1);
    let mut mpt = crate::mpt::MerklePatriciaTrie::new();
    mpt.insert("user", b"alice".to_vec()).unwrap();
    blockchain.add_block(mpt);
    let block = blockchain.get_block(1).unwrap();

    let before_creation = chrono::Utc::now();
    let proof = MerkleProof::new(
        "user".to_string(),
        b"alice".to_vec(),
        1,
        block.data.root_hash(),
        block.hash.clone(),
    );
    let after_creation = chrono::Utc::now();

    // Timestamp should be reasonable (within a few seconds)
    let proof_time = chrono::DateTime::parse_from_rfc3339(&proof.timestamp).unwrap();
    assert!(proof_time > before_creation - chrono::Duration::seconds(5));
    assert!(proof_time < after_creation + chrono::Duration::seconds(5));
}

#[test]
fn test_proof_serialization() {
    let mut blockchain = crate::blockchain::Blockchain::new_default(1);
    let mut mpt = crate::mpt::MerklePatriciaTrie::new();
    mpt.insert("user", b"alice".to_vec()).unwrap();
    blockchain.add_block(mpt);
    let block = blockchain.get_block(1).unwrap();

    let original_proof = MerkleProof::new(
        "user".to_string(),
        b"alice".to_vec(),
        1,
        block.data.root_hash(),
        block.hash.clone(),
    );

    // Test JSON serialization
    let json = serde_json::to_string(&original_proof).unwrap();
    let deserialized_proof: MerkleProof = serde_json::from_str(&json).unwrap();

    assert_eq!(original_proof.key, deserialized_proof.key);
    assert_eq!(original_proof.value, deserialized_proof.value);
    assert_eq!(original_proof.block_index, deserialized_proof.block_index);
    assert_eq!(
        original_proof.mpt_root_hash,
        deserialized_proof.mpt_root_hash
    );
    assert_eq!(original_proof.block_hash, deserialized_proof.block_hash);
    assert_eq!(original_proof.timestamp, deserialized_proof.timestamp);

    // Deserialized proof should still verify
    let is_valid = deserialized_proof.verify(&blockchain);
    assert!(is_valid);
}

/// Proof verification utilities
pub struct ProofVerifier;

impl ProofVerifier {
    /// Verify a proof against a blockchain
    pub fn verify_proof(proof: &MerkleProof, blockchain: &crate::Blockchain) -> Result<bool> {
        if !proof.verify_block_exists(blockchain) {
            return Err(BlockchainError::ProofError(format!(
                "Block {} does not exist in blockchain",
                proof.block_index
            )));
        }

        if !proof.verify_mpt_root(blockchain) {
            return Err(BlockchainError::ProofError(format!(
                "MPT root hash mismatch for block {}",
                proof.block_index
            )));
        }

        if !proof.verify_key_value(blockchain) {
            return Err(BlockchainError::ProofError(format!(
                "Key-value mismatch for key '{}' in block {}",
                proof.key, proof.block_index
            )));
        }

        Ok(true)
    }

    /// Verify a proof against a root hash only
    pub fn verify_against_root(proof: &MerkleProof, expected_root: &[u8]) -> bool {
        proof.verify_against_root(expected_root)
    }

    /// Batch verify multiple proofs
    pub fn verify_batch(
        proofs: &[MerkleProof],
        blockchain: &crate::Blockchain,
    ) -> Result<Vec<bool>> {
        let mut results = Vec::with_capacity(proofs.len());

        for proof in proofs {
            let result = Self::verify_proof(proof, blockchain)?;
            results.push(result);
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_blockchain() -> crate::blockchain::Blockchain {
        let mut blockchain = crate::blockchain::Blockchain::new_default(2);
        let mut mpt = crate::mpt::MerklePatriciaTrie::new();
        mpt.insert("test_key", b"test_value".to_vec()).unwrap();
        blockchain.add_block(mpt);
        blockchain
    }

    #[test]
    fn test_proof_creation_and_verification() {
        let blockchain = create_test_blockchain();
        let block = blockchain.get_block(1).unwrap();

        let proof = block
            .data
            .generate_proof("test_key", 1, block.hash.clone())
            .unwrap();

        assert!(proof.verify(&blockchain));
        assert!(ProofVerifier::verify_proof(&proof, &blockchain).unwrap());
    }

    #[test]
    fn test_invalid_proof_fails() {
        let blockchain = create_test_blockchain();
        let block = blockchain.get_block(1).unwrap();

        // Create a proof with wrong value
        let proof = MerkleProof::new(
            "test_key".to_string(),
            b"wrong_value".to_vec(),
            1,
            block.data.root_hash(),
            block.hash.clone(),
        );

        assert!(!proof.verify(&blockchain));
        assert!(ProofVerifier::verify_proof(&proof, &blockchain).is_err());
    }

    #[test]
    fn test_proof_formatting() {
        let blockchain = create_test_blockchain();
        let block = blockchain.get_block(1).unwrap();

        let proof = block
            .data
            .generate_proof("test_key", 1, block.hash.clone())
            .unwrap();
        let formatted = proof.format();

        assert!(formatted.contains("test_key"));
        assert!(formatted.contains("test_value"));
        assert!(formatted.contains(&block.hash));
    }
}
