//! Block implementation for VCBC blockchain.
//!
//! Provides block creation, validation, mining, and network configuration
//! structures for blockchain identity and isolation.

use crate::mpt::MerklePatriciaTrie;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Network configuration for blockchain identity and isolation
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct NetworkConfig {
    /// Unique network identifier (e.g., "vc-mainnet", "vc-testnet", "my-project-v1")
    pub network_id: String,
    /// Numeric chain identifier for additional isolation
    pub chain_id: u64,
    /// Genesis block hash for network validation
    pub genesis_hash: String,
    /// Protocol version for compatibility checking
    pub protocol_version: String,
}

/// Represents a block in the blockchain
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Block {
    /// Block index in the chain
    pub index: u64,
    /// Block creation timestamp
    pub timestamp: DateTime<Utc>,
    /// MPT-based data storage
    pub data: MerklePatriciaTrie,
    /// Hash of the previous block
    pub previous_hash: String,
    /// Hash of this block
    pub hash: String,
    /// Proof-of-work nonce
    pub nonce: u64,
}

impl Block {
    /// Create a new block with MPT data
    #[must_use]
    pub fn new(index: u64, data: MerklePatriciaTrie, previous_hash: String) -> Self {
        let timestamp = Utc::now();
        let mut block = Self {
            index,
            timestamp,
            data,
            previous_hash,
            hash: String::new(),
            nonce: 0,
        };
        block.hash = block.calculate_hash();
        block
    }

    /// Create the genesis block with network configuration
    ///
    /// # Panics
    ///
    /// Panics if inserting genesis data into the MPT fails (should never happen in normal operation).
    #[must_use]
    pub fn genesis_with_config(mut network_config: NetworkConfig) -> Self {
        let mut genesis_data = MerklePatriciaTrie::new();
        genesis_data
            .insert("message", b"Genesis Block".to_vec())
            .expect("Genesis block creation should not fail");
        genesis_data
            .insert("version", b"1.0".to_vec())
            .expect("Genesis block creation should not fail");
        genesis_data
            .insert("network_id", network_config.network_id.as_bytes().to_vec())
            .expect("Genesis block creation should not fail");
        genesis_data
            .insert(
                "chain_id",
                network_config.chain_id.to_string().as_bytes().to_vec(),
            )
            .expect("Genesis block creation should not fail");
        genesis_data
            .insert(
                "protocol_version",
                network_config.protocol_version.as_bytes().to_vec(),
            )
            .expect("Genesis block creation should not fail");

        // Create genesis block first to get its hash
        let genesis_block = Self::new(0, genesis_data, "0".to_string());
        network_config.genesis_hash = genesis_block.hash.clone();

        // Create final genesis block with correct network config
        let mut final_genesis_data = MerklePatriciaTrie::new();
        final_genesis_data
            .insert("message", b"Genesis Block".to_vec())
            .expect("Genesis block creation should not fail");
        final_genesis_data
            .insert("version", b"1.0".to_vec())
            .expect("Genesis block creation should not fail");
        final_genesis_data
            .insert("network_id", network_config.network_id.as_bytes().to_vec())
            .expect("Genesis block creation should not fail");
        final_genesis_data
            .insert(
                "chain_id",
                network_config.chain_id.to_string().as_bytes().to_vec(),
            )
            .expect("Genesis block creation should not fail");
        final_genesis_data
            .insert(
                "protocol_version",
                network_config.protocol_version.as_bytes().to_vec(),
            )
            .expect("Genesis block creation should not fail");
        final_genesis_data
            .insert(
                "genesis_hash",
                network_config.genesis_hash.as_bytes().to_vec(),
            )
            .expect("Genesis block creation should not fail");

        // Create final genesis block
        Self::new(0, final_genesis_data, "0".to_string())
    }

    /// Create the legacy genesis block (for backward compatibility)
    #[must_use]
    pub fn genesis() -> Self {
        let network_config = NetworkConfig {
            network_id: "vc-mainnet".to_string(),
            chain_id: 1,
            genesis_hash: "".to_string(),
            protocol_version: "1.0.0".to_string(),
        };
        Self::genesis_with_config(network_config)
    }

    /// Calculate the block hash
    #[must_use]
    pub fn calculate_hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.index.to_string().as_bytes());
        hasher.update(self.timestamp.timestamp().to_string().as_bytes());
        hasher.update(self.data.root_hash());
        hasher.update(self.previous_hash.as_bytes());
        hasher.update(self.nonce.to_string().as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Mine the block with proof-of-work
    pub fn mine_block(&mut self, difficulty: usize) {
        let target = "0".repeat(difficulty);

        while !self.hash.starts_with(&target) {
            self.nonce += 1;
            self.hash = self.calculate_hash();
        }

        println!("✅ Block mined: {}", self.hash);
    }

    /// Validate block structure and proof-of-work
    #[must_use]
    pub fn is_valid(&self, difficulty: usize) -> bool {
        // Check hash calculation
        if self.hash != self.calculate_hash() {
            return false;
        }

        // Skip proof-of-work validation for genesis block
        if self.index == 0 {
            return true;
        }

        // Check proof-of-work for non-genesis blocks
        if !self.hash.starts_with(&"0".repeat(difficulty)) {
            return false;
        }

        true
    }

    /// Get formatted block information
    #[must_use]
    pub fn format_info(&self) -> String {
        let data_entries = self.data.get_all();
        format!(
            "Block #{}:\n\
             Hash: {}\n\
             Previous: {}\n\
             MPT Root: {}\n\
             Timestamp: {}\n\
             Nonce: {}\n\
             Data Entries: {}\n{}",
            self.index,
            self.hash,
            self.previous_hash,
            self.data.hex_root_hash(),
            self.timestamp,
            self.nonce,
            data_entries.len(),
            if data_entries.is_empty() {
                "  (empty)".to_string()
            } else {
                data_entries
                    .iter()
                    .take(3)
                    .map(|(k, v)| format!("  {} -> {}", k, String::from_utf8_lossy(v)))
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mpt::MerklePatriciaTrie;

    #[test]
    fn test_block_creation() {
        let mut data = MerklePatriciaTrie::new();
        data.insert("test", b"value".to_vec()).unwrap();
        let block = Block::new(1, data, "previous_hash".to_string());

        assert_eq!(block.index, 1);
        assert_eq!(block.previous_hash, "previous_hash");
        assert_eq!(block.nonce, 0);
        assert!(!block.hash.is_empty());
    }

    #[test]
    fn test_genesis_block() {
        let genesis = Block::genesis();

        assert_eq!(genesis.index, 0);
        assert_eq!(genesis.previous_hash, "0");
        assert_eq!(genesis.nonce, 0);
        assert!(!genesis.hash.is_empty());
    }

    #[test]
    fn test_block_hash_calculation() {
        let mut data = MerklePatriciaTrie::new();
        data.insert("key", b"value".to_vec()).unwrap();
        let block = Block::new(1, data, "prev".to_string());
        let hash1 = block.calculate_hash();
        let hash2 = block.calculate_hash();

        assert_eq!(hash1, hash2);
        assert_eq!(block.hash, hash1);
    }

    #[test]
    fn test_block_mining() {
        let mut data = MerklePatriciaTrie::new();
        data.insert("test", b"mining".to_vec()).unwrap();
        let mut block = Block::new(1, data, "prev".to_string());

        block.mine_block(2); // Difficulty 2

        assert!(block.hash.starts_with("00"));
        assert!(block.nonce > 0);
    }

    #[test]
    fn test_block_format_info() {
        let mut data = MerklePatriciaTrie::new();
        data.insert("test", b"value".to_vec()).unwrap();
        let block = Block::new(1, data, "prev".to_string());
        let info = block.format_info();

        assert!(info.contains("Block #1"));
        assert!(info.contains(&block.hash));
        assert!(info.contains("test -> value"));
    }

    #[test]
    fn test_block_is_valid() {
        let mut data = MerklePatriciaTrie::new();
        data.insert("test", b"value".to_vec()).unwrap();
        let mut block = Block::new(1, data, "prev".to_string());

        // Mine block to make it valid
        block.mine_block(2);

        assert!(block.is_valid(2));
        assert!(!block.is_valid(3)); // Higher difficulty should fail
    }

    #[test]
    fn test_genesis_block_with_config() {
        let config = NetworkConfig {
            network_id: "test-net".to_string(),
            chain_id: 42,
            genesis_hash: "".to_string(),
            protocol_version: "2.0.0".to_string(),
        };

        let genesis = Block::genesis_with_config(config);

        assert_eq!(genesis.index, 0);
        assert_eq!(genesis.previous_hash, "0");

        // Check that config data was stored
        assert_eq!(genesis.data.get("network_id").unwrap(), b"test-net");
        assert_eq!(genesis.data.get("chain_id").unwrap(), b"42");
        assert_eq!(genesis.data.get("protocol_version").unwrap(), b"2.0.0");
    }

    #[test]
    fn test_network_config_serialization() {
        let config = NetworkConfig {
            network_id: "test".to_string(),
            chain_id: 1,
            genesis_hash: "hash".to_string(),
            protocol_version: "1.0".to_string(),
        };

        let serialized = serde_json::to_string(&config).unwrap();
        let deserialized: NetworkConfig = serde_json::from_str(&serialized).unwrap();

        assert_eq!(config, deserialized);
    }
}
