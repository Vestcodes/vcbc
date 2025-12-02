//! Blockchain chain management for VCBC.
//!
//! Provides the main Blockchain struct and chain operations including
//! block addition, validation, data retrieval, and network compatibility.

use crate::blockchain::block::{Block, NetworkConfig};
use crate::error::Result;
use crate::mpt::MerklePatriciaTrie;
use serde::{Deserialize, Serialize};

/// Main blockchain structure
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Blockchain {
    /// Chain of blocks
    pub chain: Vec<Block>,
    /// Mining difficulty
    pub difficulty: usize,
    /// Network configuration for identity and isolation
    pub network_config: NetworkConfig,
}

impl Blockchain {
    /// Create a new blockchain with given difficulty and network configuration
    #[must_use]
    pub fn new(difficulty: usize, mut network_config: NetworkConfig) -> Self {
        let genesis_block = Block::genesis_with_config(network_config.clone());
        network_config.genesis_hash = genesis_block.hash.clone();

        Self {
            chain: vec![genesis_block],
            difficulty,
            network_config,
        }
    }

    /// Create a new blockchain with default network configuration
    #[must_use]
    pub fn new_default(difficulty: usize) -> Self {
        let network_config = NetworkConfig {
            network_id: "vc-mainnet".to_string(),
            chain_id: 1,
            genesis_hash: "".to_string(), // Will be set by genesis creation
            protocol_version: "1.0.0".to_string(),
        };
        Self::new(difficulty, network_config)
    }

    /// Create a blockchain with genesis block using network configuration
    #[must_use]
    pub fn genesis(network_config: NetworkConfig) -> Self {
        Self::new(0, network_config)
    }

    /// Add a new block with MPT data
    pub fn add_block(&mut self, data: MerklePatriciaTrie) {
        let previous_hash = self.get_latest_block().hash.clone();
        let index = self.chain.len() as u64;
        let mut new_block = Block::new(index, data, previous_hash);
        new_block.mine_block(self.difficulty);
        self.chain.push(new_block);
    }

    /// Add a block with key-value data
    ///
    /// # Panics
    ///
    /// Panics if MPT insertion fails (should not happen with valid data).
    pub fn add_block_with_key_value(&mut self, key: &str, value: Vec<u8>) {
        let mut mpt = MerklePatriciaTrie::new();
        mpt.insert(key, value)
            .expect("MPT insertion should not fail for valid data");
        self.add_block(mpt);
    }

    /// Get a block by index
    #[must_use]
    pub fn get_block(&self, index: usize) -> Option<&Block> {
        self.chain.get(index)
    }

    /// Get the latest block
    ///
    /// # Panics
    ///
    /// Panics if the blockchain has no blocks (should never happen in normal operation).
    #[must_use]
    pub fn get_latest_block(&self) -> &Block {
        self.chain
            .last()
            .expect("Blockchain should always have at least genesis block")
    }

    /// Validate the entire blockchain
    #[must_use]
    pub fn is_chain_valid(&self) -> bool {
        for (i, block) in self.chain.iter().enumerate() {
            // Validate block structure
            if block.hash != block.calculate_hash() {
                return false;
            }

            // Validate proof-of-work (skip for genesis block)
            if i > 0 && !block.hash.starts_with(&"0".repeat(self.difficulty)) {
                return false;
            }

            // Validate chain linkage (skip genesis block)
            if i > 0 {
                let previous_block = &self.chain[i - 1];
                if block.previous_hash != previous_block.hash {
                    return false;
                }
            }
        }
        true
    }

    /// Verify data integrity for a specific block and key
    #[must_use]
    pub fn verify_data_integrity(
        &self,
        block_index: usize,
        key: &str,
        expected_value: &[u8],
    ) -> bool {
        if let Some(block) = self.get_block(block_index) {
            if let Some(stored_value) = block.data.get(key) {
                stored_value == expected_value
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Get data from a specific block
    #[must_use]
    pub fn get_data_from_block(&self, block_index: usize, key: &str) -> Option<Vec<u8>> {
        self.get_block(block_index)
            .and_then(|block| block.data.get(key))
    }

    /// Get blockchain statistics
    #[must_use]
    pub fn get_stats(&self) -> BlockchainStats {
        BlockchainStats {
            total_blocks: self.chain.len() as u64,
            latest_index: self.chain.len().saturating_sub(1) as u64,
            difficulty: self.difficulty,
            is_valid: self.is_chain_valid(),
        }
    }

    /// Get all block hashes
    #[must_use]
    pub fn get_all_block_hashes(&self) -> Vec<String> {
        self.chain.iter().map(|block| block.hash.clone()).collect()
    }

    /// Find blocks containing specific key
    pub fn find_blocks_with_key(&self, key: &str) -> Vec<usize> {
        self.chain
            .iter()
            .enumerate()
            .filter_map(|(index, block)| block.data.get(key).map(|_| index))
            .collect()
    }

    /// Save blockchain to file
    pub fn save_to_file(&self, filename: &str) -> Result<()> {
        let json = serde_json::to_string_pretty(self).map_err(|e| {
            crate::error::BlockchainError::StorageError(format!(
                "Failed to serialize blockchain: {}",
                e
            ))
        })?;

        std::fs::write(filename, json).map_err(|e| {
            crate::error::BlockchainError::StorageError(format!(
                "Failed to write blockchain file '{}': {}",
                filename, e
            ))
        })?;

        Ok(())
    }

    /// Load blockchain from file
    pub fn load_from_file(filename: &str) -> Result<Self> {
        let contents = std::fs::read_to_string(filename).map_err(|e| {
            crate::error::BlockchainError::StorageError(format!(
                "Failed to read blockchain file '{}': {}",
                filename, e
            ))
        })?;

        let blockchain: Blockchain = serde_json::from_str(&contents).map_err(|e| {
            crate::error::BlockchainError::StorageError(format!(
                "Failed to deserialize blockchain: {}",
                e
            ))
        })?;

        Ok(blockchain)
    }

    /// Validate network compatibility with another blockchain
    pub fn validate_network_compatibility(&self, other_config: &NetworkConfig) -> Result<()> {
        if self.network_config.network_id != other_config.network_id {
            return Err(crate::error::BlockchainError::NetworkMismatch {
                expected: self.network_config.network_id.clone(),
                received: other_config.network_id.clone(),
            });
        }
        if self.network_config.genesis_hash != other_config.genesis_hash {
            return Err(crate::error::BlockchainError::GenesisMismatch {
                expected: self.network_config.genesis_hash.clone(),
                received: other_config.genesis_hash.clone(),
            });
        }
        if self.network_config.protocol_version != other_config.protocol_version {
            return Err(crate::error::BlockchainError::ProtocolMismatch {
                expected: self.network_config.protocol_version.clone(),
                received: other_config.protocol_version.clone(),
            });
        }
        Ok(())
    }

    /// Check if two blockchains are on the same network
    #[must_use]
    pub fn is_same_network(&self, other: &Blockchain) -> bool {
        self.network_config.network_id == other.network_config.network_id
            && self.network_config.genesis_hash == other.network_config.genesis_hash
    }
}

/// Blockchain statistics
#[derive(Debug, Clone)]
pub struct BlockchainStats {
    /// Total number of blocks
    pub total_blocks: u64,
    /// Latest block index
    pub latest_index: u64,
    /// Current mining difficulty
    pub difficulty: usize,
    /// Whether the chain is valid
    pub is_valid: bool,
}

impl BlockchainStats {
    /// Format statistics as a string
    pub fn format(&self) -> String {
        format!(
            "Blockchain Statistics:\n\
             Total Blocks: {}\n\
             Latest Index: {}\n\
             Difficulty: {}\n\
             Chain Valid: {}",
            self.total_blocks,
            self.latest_index,
            self.difficulty,
            if self.is_valid { "✅ Yes" } else { "❌ No" }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_block_to_chain() {
        let mut blockchain = Blockchain::new_default(1);
        let mut mpt = MerklePatriciaTrie::new();
        mpt.insert("test", b"data".to_vec()).unwrap();

        blockchain.add_block(mpt);

        assert_eq!(blockchain.chain.len(), 2);
        assert_eq!(blockchain.get_latest_block().index, 1);
        assert_eq!(
            blockchain.get_latest_block().previous_hash,
            blockchain.chain[0].hash
        );
        assert!(blockchain.is_chain_valid());
    }

    #[test]
    fn test_get_block_by_index() {
        let mut blockchain = Blockchain::new_default(1);
        let mut mpt = MerklePatriciaTrie::new();
        mpt.insert("test", b"data".to_vec()).unwrap();

        blockchain.add_block(mpt);

        let block = blockchain.get_block(0).unwrap();
        assert_eq!(block.index, 0);

        let block = blockchain.get_block(1).unwrap();
        assert_eq!(block.index, 1);

        assert!(blockchain.get_block(999).is_none());
    }

    #[test]
    fn test_get_latest_block() {
        let mut blockchain = Blockchain::new_default(1);
        let mut mpt = MerklePatriciaTrie::new();
        mpt.insert("test", b"data".to_vec()).unwrap();

        blockchain.add_block(mpt);

        let latest = blockchain.get_latest_block();
        assert_eq!(latest.index, 1);
    }

    #[test]
    fn test_chain_validation_valid_chain() {
        let mut blockchain = Blockchain::new_default(1);
        let mut mpt = MerklePatriciaTrie::new();
        mpt.insert("test", b"data".to_vec()).unwrap();

        blockchain.add_block(mpt);
        assert!(blockchain.is_chain_valid());
    }

    #[test]
    fn test_chain_validation_invalid_hash() {
        let mut blockchain = Blockchain::new_default(1);
        let mut mpt = MerklePatriciaTrie::new();
        mpt.insert("test", b"data".to_vec()).unwrap();

        blockchain.add_block(mpt);

        // Tamper with a block hash
        blockchain.chain[1].hash = "tampered_hash".to_string();

        assert!(!blockchain.is_chain_valid());
    }

    #[test]
    fn test_chain_validation_invalid_previous_hash() {
        let mut blockchain = Blockchain::new_default(1);
        let mut mpt = MerklePatriciaTrie::new();
        mpt.insert("test", b"data".to_vec()).unwrap();

        blockchain.add_block(mpt);

        // Tamper with previous hash link
        blockchain.chain[1].previous_hash = "tampered_previous".to_string();

        assert!(!blockchain.is_chain_valid());
    }

    #[test]
    fn test_chain_validation_invalid_pow() {
        let mut blockchain = Blockchain::new_default(2);
        let mut mpt = MerklePatriciaTrie::new();
        mpt.insert("test", b"data".to_vec()).unwrap();

        blockchain.add_block(mpt);

        // Tamper with nonce to break PoW
        blockchain.chain[1].nonce = 0;

        assert!(!blockchain.is_chain_valid());
    }

    #[test]
    fn test_empty_blockchain_is_valid() {
        let blockchain = Blockchain::new_default(1);
        assert!(blockchain.is_chain_valid());
    }

    #[test]
    fn test_mpt_data_preservation() {
        let mut blockchain = Blockchain::new_default(1);
        let mut mpt = MerklePatriciaTrie::new();
        mpt.insert("user", b"alice".to_vec()).unwrap();
        mpt.insert("balance", b"100".to_vec()).unwrap();

        blockchain.add_block(mpt);

        let stored_block = blockchain.get_block(1).unwrap();
        assert_eq!(stored_block.data.get("user").unwrap(), b"alice");
        assert_eq!(stored_block.data.get("balance").unwrap(), b"100");
        assert!(stored_block.data.get("nonexistent").is_none());
    }

    #[test]
    fn test_genesis_block_properties() {
        let blockchain = Blockchain::new_default(1);
        let genesis = &blockchain.chain[0];

        assert_eq!(genesis.index, 0);
        assert_eq!(genesis.previous_hash, "0");
        assert_eq!(genesis.nonce, 0);
        // Genesis block now contains metadata
        assert!(!genesis.data.is_empty());
        assert_eq!(genesis.data.get("message"), Some(b"Genesis Block".to_vec()));
        assert_eq!(genesis.data.get("version"), Some(b"1.0".to_vec()));
        // Check network config data
        assert_eq!(genesis.data.get("network_id"), Some(b"vc-mainnet".to_vec()));
        assert_eq!(genesis.data.get("chain_id"), Some(b"1".to_vec()));
        assert_eq!(
            genesis.data.get("protocol_version"),
            Some(b"1.0.0".to_vec())
        );
    }

    #[test]
    fn test_blockchain_creation() {
        let blockchain = Blockchain::new_default(2);
        assert_eq!(blockchain.chain.len(), 1);
        assert_eq!(blockchain.difficulty, 2);
        assert_eq!(blockchain.network_config.network_id, "vc-mainnet");
        assert_eq!(blockchain.network_config.chain_id, 1);
    }

    #[test]
    fn test_block_creation_and_validation() {
        let mut blockchain = Blockchain::new_default(2);

        // Add a block
        blockchain.add_block_with_key_value("test", b"value".to_vec());

        // Check blockchain state
        assert_eq!(blockchain.chain.len(), 2);
        assert!(blockchain.is_chain_valid());

        // Check data retrieval
        let value = blockchain.get_data_from_block(1, "test");
        assert_eq!(value, Some(b"value".to_vec()));
    }

    #[test]
    fn test_data_integrity_verification() {
        let mut blockchain = Blockchain::new_default(2);
        blockchain.add_block_with_key_value("key1", b"value1".to_vec());

        // Verify correct data
        assert!(blockchain.verify_data_integrity(1, "key1", b"value1"));

        // Verify incorrect data fails
        assert!(!blockchain.verify_data_integrity(1, "key1", b"wrong"));

        // Verify non-existent key fails
        assert!(!blockchain.verify_data_integrity(1, "nonexistent", b"value1"));
    }

    #[test]
    fn test_genesis_block() {
        let blockchain = Blockchain::new_default(2);
        let genesis = blockchain.get_block(0).unwrap();

        assert_eq!(genesis.index, 0);
        assert_eq!(genesis.previous_hash, "0");
        assert!(genesis.is_valid(2));

        // Check genesis data
        let message = genesis.data.get("message");
        assert_eq!(message, Some(b"Genesis Block".to_vec()));
    }

    #[test]
    fn test_blockchain_stats() {
        let mut blockchain = Blockchain::new_default(3);
        blockchain.add_block_with_key_value("test", b"data".to_vec());

        let stats = blockchain.get_stats();
        assert_eq!(stats.total_blocks, 2);
        assert_eq!(stats.latest_index, 1);
        assert_eq!(stats.difficulty, 3);
        assert!(stats.is_valid);
    }

    #[test]
    fn test_network_config_creation() {
        let config = NetworkConfig {
            network_id: "test-net".to_string(),
            chain_id: 42,
            genesis_hash: "test-hash".to_string(),
            protocol_version: "2.0.0".to_string(),
        };

        let blockchain = Blockchain::new(2, config.clone());
        assert_eq!(blockchain.network_config.network_id, "test-net");
        assert_eq!(blockchain.network_config.chain_id, 42);
        assert_eq!(blockchain.network_config.protocol_version, "2.0.0");
    }

    #[test]
    fn test_network_compatibility_same_network() {
        let config1 = NetworkConfig {
            network_id: "test-net".to_string(),
            chain_id: 1,
            genesis_hash: "genesis-hash".to_string(),
            protocol_version: "1.0.0".to_string(),
        };

        let config2 = config1.clone();

        let blockchain1 = Blockchain::new(2, config1);
        let blockchain2 = Blockchain::new(2, config2);

        assert!(blockchain1
            .validate_network_compatibility(&blockchain2.network_config)
            .is_ok());
        assert!(blockchain1.is_same_network(&blockchain2));
    }

    #[test]
    fn test_network_compatibility_different_network() {
        let config1 = NetworkConfig {
            network_id: "test-net-1".to_string(),
            chain_id: 1,
            genesis_hash: "genesis-hash-1".to_string(),
            protocol_version: "1.0.0".to_string(),
        };

        let config2 = NetworkConfig {
            network_id: "test-net-2".to_string(),
            chain_id: 1,
            genesis_hash: "genesis-hash-1".to_string(),
            protocol_version: "1.0.0".to_string(),
        };

        let blockchain1 = Blockchain::new(2, config1);
        let blockchain2 = Blockchain::new(2, config2);

        assert!(blockchain1
            .validate_network_compatibility(&blockchain2.network_config)
            .is_err());
        assert!(!blockchain1.is_same_network(&blockchain2));
    }

    #[test]
    fn test_network_compatibility_different_genesis() {
        let config1 = NetworkConfig {
            network_id: "test-net-1".to_string(),
            chain_id: 1,
            genesis_hash: "".to_string(), // Will be set by genesis creation
            protocol_version: "1.0.0".to_string(),
        };

        let config2 = NetworkConfig {
            network_id: "test-net-2".to_string(),
            chain_id: 1,
            genesis_hash: "".to_string(), // Will be set by genesis creation
            protocol_version: "1.0.0".to_string(),
        };

        let blockchain1 = Blockchain::new(2, config1);
        let blockchain2 = Blockchain::new(2, config2);

        // Different network IDs should fail validation
        assert!(blockchain1
            .validate_network_compatibility(&blockchain2.network_config)
            .is_err());
        assert!(!blockchain1.is_same_network(&blockchain2));

        // Same network with different genesis hashes would also fail, but here we test different network IDs
        assert_ne!(
            blockchain1.network_config.genesis_hash,
            blockchain2.network_config.genesis_hash
        );
    }

    #[test]
    fn test_add_block_with_key_value() {
        let mut blockchain = Blockchain::new_default(1);
        blockchain.add_block_with_key_value("test-key", b"test-value".to_vec());

        assert_eq!(blockchain.chain.len(), 2);
        let latest_block = blockchain.get_latest_block();
        assert_eq!(latest_block.index, 1);
        assert_eq!(
            blockchain.get_data_from_block(1, "test-key"),
            Some(b"test-value".to_vec())
        );
    }

    #[test]
    fn test_get_data_from_block() {
        let mut blockchain = Blockchain::new_default(1);
        blockchain.add_block_with_key_value("key1", b"value1".to_vec());
        blockchain.add_block_with_key_value("key2", b"value2".to_vec());

        assert_eq!(
            blockchain.get_data_from_block(1, "key1"),
            Some(b"value1".to_vec())
        );
        assert_eq!(
            blockchain.get_data_from_block(2, "key2"),
            Some(b"value2".to_vec())
        );
        assert_eq!(blockchain.get_data_from_block(1, "nonexistent"), None);
        assert_eq!(blockchain.get_data_from_block(99, "key1"), None);
    }

    #[test]
    fn test_get_all_block_hashes() {
        let mut blockchain = Blockchain::new_default(1);
        blockchain.add_block_with_key_value("key", b"value".to_vec());
        blockchain.add_block_with_key_value("key2", b"value2".to_vec());

        let hashes = blockchain.get_all_block_hashes();
        assert_eq!(hashes.len(), 3); // genesis + 2 blocks
        assert!(!hashes[0].is_empty());
        assert!(!hashes[1].is_empty());
        assert!(!hashes[2].is_empty());
    }

    #[test]
    fn test_find_blocks_with_key() {
        let mut blockchain = Blockchain::new_default(1);
        blockchain.add_block_with_key_value("shared-key", b"value1".to_vec());
        blockchain.add_block_with_key_value("unique-key", b"value2".to_vec());
        blockchain.add_block_with_key_value("shared-key", b"value3".to_vec());

        let shared_blocks = blockchain.find_blocks_with_key("shared-key");
        assert_eq!(shared_blocks, vec![1, 3]);

        let unique_blocks = blockchain.find_blocks_with_key("unique-key");
        assert_eq!(unique_blocks, vec![2]);

        let nonexistent_blocks = blockchain.find_blocks_with_key("nonexistent");
        assert_eq!(nonexistent_blocks, Vec::<usize>::new());
    }

    #[test]
    fn test_blockchain_save_and_load() {
        let mut blockchain = Blockchain::new_default(1);
        blockchain.add_block_with_key_value("test-key", b"test-value".to_vec());

        let temp_file = "test_blockchain_save_load.json";
        blockchain.save_to_file(temp_file).unwrap();

        let loaded_blockchain = Blockchain::load_from_file(temp_file).unwrap();
        assert_eq!(loaded_blockchain.chain.len(), blockchain.chain.len());
        assert_eq!(loaded_blockchain.difficulty, blockchain.difficulty);
        assert_eq!(
            loaded_blockchain.network_config.network_id,
            blockchain.network_config.network_id
        );

        // Cleanup
        std::fs::remove_file(temp_file).unwrap();
    }

    #[test]
    fn test_validate_network_compatibility() {
        let config1 = NetworkConfig {
            network_id: "test-net".to_string(),
            chain_id: 1,
            genesis_hash: "hash1".to_string(),
            protocol_version: "1.0.0".to_string(),
        };
        let _config2 = NetworkConfig {
            network_id: "test-net".to_string(),
            chain_id: 1,
            genesis_hash: "hash1".to_string(),
            protocol_version: "1.0.0".to_string(),
        };
        let config3 = NetworkConfig {
            network_id: "different-net".to_string(),
            chain_id: 1,
            genesis_hash: "hash1".to_string(),
            protocol_version: "1.0.0".to_string(),
        };

        let blockchain = Blockchain::new(1, config1);
        assert!(blockchain
            .validate_network_compatibility(&blockchain.network_config)
            .is_ok());
        assert!(blockchain.validate_network_compatibility(&config3).is_err());
    }

    #[test]
    fn test_is_same_network() {
        let config1 = NetworkConfig {
            network_id: "test-net".to_string(),
            chain_id: 1,
            genesis_hash: "hash1".to_string(),
            protocol_version: "1.0.0".to_string(),
        };
        let config2 = NetworkConfig {
            network_id: "test-net".to_string(),
            chain_id: 1,
            genesis_hash: "hash1".to_string(),
            protocol_version: "1.0.0".to_string(),
        };
        let config3 = NetworkConfig {
            network_id: "different-net".to_string(),
            chain_id: 1,
            genesis_hash: "hash1".to_string(),
            protocol_version: "1.0.0".to_string(),
        };

        let blockchain1 = Blockchain::new(1, config1);
        let blockchain2 = Blockchain::new(1, config2);
        let blockchain3 = Blockchain::new(1, config3);

        assert!(blockchain1.is_same_network(&blockchain2));
        assert!(!blockchain1.is_same_network(&blockchain3));
    }

    #[test]
    fn test_genesis_with_network_config() {
        let config = NetworkConfig {
            network_id: "custom-net".to_string(),
            chain_id: 99,
            genesis_hash: "".to_string(), // Will be set by genesis
            protocol_version: "1.5.0".to_string(),
        };

        let blockchain = Blockchain::genesis(config);
        let genesis = &blockchain.chain[0];

        // Check that genesis contains network data
        assert_eq!(genesis.data.get("network_id"), Some(b"custom-net".to_vec()));
        assert_eq!(genesis.data.get("chain_id"), Some(b"99".to_vec()));
        assert_eq!(
            genesis.data.get("protocol_version"),
            Some(b"1.5.0".to_vec())
        );

        // Check that network config has correct genesis hash
        assert!(!blockchain.network_config.genesis_hash.is_empty());
        assert_eq!(genesis.hash, blockchain.network_config.genesis_hash);
    }
}
