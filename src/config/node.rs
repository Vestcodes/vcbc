//! Node configuration for VCBC blockchain network.
//!
//! Provides configuration structures and utilities for both bootnodes
//! and regular nodes in the VCBC network.

use crate::config::authority::NetworkAuthority;
use crate::config::certificate::BootnodeCertificate;
use crate::error::BlockchainError;
use serde::{Deserialize, Serialize};
use std::fs;
use uuid::Uuid;

/// Node configuration for VCBC blockchain network
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeConfig {
    /// Unique node identifier (auto-generated)
    pub node_id: String,
    /// Network identifier (e.g., "vc-mainnet", "vc-testnet")
    pub network_id: String,
    /// Chain identifier for network isolation
    pub chain_id: u64,
    /// Genesis block hash for network validation
    pub genesis_hash: String,
    /// Bootstrap node URLs (HTTP endpoints for peer discovery)
    pub bootstrap_nodes: Vec<String>,
    /// Local data storage path
    pub storage_path: String,
    /// HTTP API server port
    pub http_port: u16,
    /// P2P network port
    pub p2p_port: u16,
    /// Mining difficulty level
    pub difficulty: usize,
    /// Node type (bootnode or regular)
    pub node_type: NodeType,
    /// Bootnode certificate (only for certified bootnodes)
    pub certificate: Option<BootnodeCertificate>,
    /// Trusted network authorities for certificate verification
    pub trusted_authorities: Vec<NetworkAuthority>,
}

/// Type of node in the network
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum NodeType {
    /// Bootstrap node that helps other nodes discover peers
    Bootnode,
    /// Regular node that participates in the network
    Regular,
}

impl NodeConfig {
    /// Create a new bootnode configuration
    pub fn bootnode(network_id: String, chain_id: u64, http_port: u16, p2p_port: u16) -> Self {
        Self {
            node_id: generate_node_id(),
            network_id,
            chain_id,
            genesis_hash: String::new(), // Will be set when blockchain is created
            bootstrap_nodes: vec![],     // Bootnode doesn't need bootstrap nodes
            storage_path: "./data".to_string(),
            http_port,
            p2p_port,
            difficulty: 2,
            node_type: NodeType::Bootnode,
            certificate: None,
            trusted_authorities: vec![],
        }
    }

    /// Create a new regular node configuration
    pub fn regular_node(bootstrap_url: String, http_port: u16, p2p_port: u16) -> Self {
        Self {
            node_id: generate_node_id(),
            network_id: String::new(), // Will be discovered from bootstrap
            chain_id: 0,               // Will be discovered from bootstrap
            genesis_hash: String::new(), // Will be discovered from bootstrap
            bootstrap_nodes: vec![bootstrap_url],
            storage_path: "./data".to_string(),
            http_port,
            p2p_port,
            difficulty: 2,
            node_type: NodeType::Regular,
            certificate: None,
            trusted_authorities: vec![],
        }
    }

    /// Save configuration to JSON file
    pub fn save_to_file(&self, path: &str) -> Result<(), BlockchainError> {
        let json = serde_json::to_string_pretty(self).map_err(|e| {
            BlockchainError::StorageError(format!("Failed to serialize config: {}", e))
        })?;
        fs::write(path, json).map_err(|e| {
            BlockchainError::StorageError(format!("Failed to write config file '{}': {}", path, e))
        })?;
        Ok(())
    }

    /// Load configuration from JSON file
    pub fn load_from_file(path: &str) -> Result<Self, BlockchainError> {
        let contents = fs::read_to_string(path).map_err(|e| {
            BlockchainError::StorageError(format!("Failed to read config file '{}': {}", path, e))
        })?;
        let config: Self = serde_json::from_str(&contents)
            .map_err(|e| BlockchainError::StorageError(format!("Failed to parse config: {}", e)))?;
        Ok(config)
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), BlockchainError> {
        if self.node_id.is_empty() {
            return Err(BlockchainError::InvalidInput(
                "Node ID cannot be empty".to_string(),
            ));
        }
        if self.network_id.is_empty() && matches!(self.node_type, NodeType::Bootnode) {
            return Err(BlockchainError::InvalidInput(
                "Network ID required for bootnode".to_string(),
            ));
        }
        if self.http_port == 0 {
            return Err(BlockchainError::InvalidInput(
                "HTTP port must be greater than 0".to_string(),
            ));
        }
        if self.p2p_port == 0 {
            return Err(BlockchainError::InvalidInput(
                "P2P port must be greater than 0".to_string(),
            ));
        }
        if matches!(self.node_type, NodeType::Regular) && self.bootstrap_nodes.is_empty() {
            return Err(BlockchainError::InvalidInput(
                "Regular node must have at least one bootstrap node".to_string(),
            ));
        }

        // Validate certificate if present
        if let Some(ref cert) = self.certificate {
            if cert.is_expired() {
                return Err(BlockchainError::InvalidInput(
                    "Bootnode certificate has expired".to_string(),
                ));
            }
            if cert.node_id != self.node_id {
                return Err(BlockchainError::InvalidInput(
                    "Certificate node ID mismatch".to_string(),
                ));
            }
            if cert.network_id != self.network_id {
                return Err(BlockchainError::InvalidInput(
                    "Certificate network ID mismatch".to_string(),
                ));
            }
            if cert.chain_id != self.chain_id {
                return Err(BlockchainError::InvalidInput(
                    "Certificate chain ID mismatch".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Get bootstrap URLs as HTTP endpoints
    pub fn bootstrap_urls(&self) -> Vec<String> {
        self.bootstrap_nodes.clone()
    }

    /// Check if this is a bootnode
    pub fn is_bootnode(&self) -> bool {
        matches!(self.node_type, NodeType::Bootnode)
    }
}

/// Generate a unique node ID
fn generate_node_id() -> String {
    format!("node-{}", Uuid::new_v4().simple())
}

/// Create directory if it doesn't exist
pub fn ensure_data_directory(path: &str) -> Result<(), BlockchainError> {
    use std::path::Path;
    let path = Path::new(path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            BlockchainError::StorageError(format!("Failed to create directory: {}", e))
        })?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bootnode_config_creation() {
        let config = NodeConfig::bootnode("vc-mainnet".to_string(), 1, 8080, 9090);

        assert!(!config.node_id.is_empty());
        assert_eq!(config.network_id, "vc-mainnet");
        assert_eq!(config.chain_id, 1);
        assert_eq!(config.http_port, 8080);
        assert_eq!(config.p2p_port, 9090);
        assert!(matches!(config.node_type, NodeType::Bootnode));
        assert!(config.bootstrap_nodes.is_empty());
    }

    #[test]
    fn test_regular_node_config_creation() {
        let config = NodeConfig::regular_node("http://localhost:8080".to_string(), 8081, 9091);

        assert!(!config.node_id.is_empty());
        assert_eq!(config.bootstrap_nodes, vec!["http://localhost:8080"]);
        assert_eq!(config.http_port, 8081);
        assert_eq!(config.p2p_port, 9091);
        assert!(matches!(config.node_type, NodeType::Regular));
    }

    #[test]
    fn test_config_validation() {
        let mut config = NodeConfig::bootnode("vc-mainnet".to_string(), 1, 8080, 9090);

        // Should pass validation
        assert!(config.validate().is_ok());

        // Test invalid cases
        config.node_id = "".to_string();
        assert!(config.validate().is_err());

        config.node_id = "test-node".to_string();
        config.http_port = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_is_bootnode() {
        let bootnode_config = NodeConfig::bootnode("vc-mainnet".to_string(), 1, 8080, 9090);
        let regular_config =
            NodeConfig::regular_node("http://localhost:8080".to_string(), 8081, 9091);

        assert!(bootnode_config.is_bootnode());
        assert!(!regular_config.is_bootnode());
    }

    #[test]
    fn test_generate_node_id() {
        let id1 = generate_node_id();
        let id2 = generate_node_id();

        assert!(!id1.is_empty());
        assert!(!id2.is_empty());
        assert_ne!(id1, id2);
        assert!(id1.starts_with("node-"));
        assert!(id2.starts_with("node-"));
    }
}
