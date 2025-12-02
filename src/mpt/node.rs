//! MPT Node implementation for VCBC blockchain.
//!
//! Provides the core MPT node types and compact encoding utilities
//! for efficient trie operations and path compression.

use crate::error::Result;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Merkle-Patricia Trie node types
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MPTNode {
    /// Branch node with up to 16 children
    Branch {
        children: [Option<Box<MPTNode>>; 16],
        value: Option<Vec<u8>>,
    },
    /// Extension node for path compression
    Extension {
        path: Vec<u8>, // Compact encoded path
        child: Box<MPTNode>,
    },
    /// Leaf node containing a value
    Leaf {
        path: Vec<u8>, // Compact encoded path
        value: Vec<u8>,
    },
    /// Empty node (root state)
    Empty,
}

impl MPTNode {
    /// Create a new branch node
    pub fn new_branch() -> Self {
        MPTNode::Branch {
            children: Default::default(),
            value: None,
        }
    }

    /// Calculate the hash of this node
    pub fn calculate_hash(&self) -> Vec<u8> {
        let serialized =
            serde_json::to_vec(self).unwrap_or_else(|_| b"serialization_error".to_vec());
        let mut hasher = Sha256::new();
        hasher.update(&serialized);
        hasher.finalize().to_vec()
    }

    /// Check if this node is empty
    pub fn is_empty(&self) -> bool {
        matches!(self, MPTNode::Empty)
    }

    /// Get the value if this is a branch with a value
    pub fn get_branch_value(&self) -> Option<&Vec<u8>> {
        match self {
            MPTNode::Branch { value, .. } => value.as_ref(),
            _ => None,
        }
    }
}

/// Compact encoding utilities for MPT paths
pub struct CompactEncoding;

impl CompactEncoding {
    /// Encode a path with terminator flag
    pub fn encode(path: &[u8], terminator: bool) -> Vec<u8> {
        let mut encoded = Vec::new();
        let mut flag = 0u8;

        if terminator {
            flag |= 0x20; // Set terminator bit
        }

        if path.len() % 2 == 1 {
            flag |= 0x10; // Set odd length bit
            encoded.push(flag);
            encoded.push(path[0]);
            encoded.extend_from_slice(&path[1..]);
        } else {
            encoded.push(flag);
            encoded.extend_from_slice(path);
        }

        encoded
    }

    /// Decode compact encoding to path and terminator flag
    pub fn decode(encoded: &[u8]) -> Result<(Vec<u8>, bool)> {
        if encoded.is_empty() {
            return Ok((Vec::new(), false));
        }

        let flag = encoded[0];
        let terminator = (flag & 0x20) != 0;
        let odd_length = (flag & 0x10) != 0;

        let mut path = Vec::new();

        if odd_length {
            if encoded.len() > 1 {
                path.push(encoded[1]);
                path.extend_from_slice(&encoded[2..]);
            }
        } else {
            path.extend_from_slice(&encoded[1..]);
        }

        Ok((path, terminator))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mpt_node_new_branch() {
        let branch = MPTNode::new_branch();
        match branch {
            MPTNode::Branch { children, value } => {
                assert!(children.iter().all(Option::is_none));
                assert!(value.is_none());
            }
            _ => panic!("Expected branch node"),
        }
    }

    #[test]
    fn test_mpt_node_calculate_hash() {
        let node = MPTNode::Empty;
        let hash = node.calculate_hash();
        assert_eq!(hash.len(), 32); // SHA256 hash length
    }

    #[test]
    fn test_mpt_node_is_empty() {
        assert!(MPTNode::Empty.is_empty());
        assert!(!MPTNode::new_branch().is_empty());
    }

    #[test]
    fn test_mpt_node_get_branch_value() {
        let branch = MPTNode::new_branch();
        assert_eq!(branch.get_branch_value(), None);

        let leaf = MPTNode::Leaf {
            path: vec![1, 2, 3],
            value: vec![4, 5, 6],
        };
        assert_eq!(leaf.get_branch_value(), None);
    }

    #[test]
    fn test_compact_encode_decode() {
        // Test even length path
        let path = vec![1, 2, 3, 4];
        let encoded = CompactEncoding::encode(&path, false);
        let (decoded_path, terminator) = CompactEncoding::decode(&encoded).unwrap();
        assert_eq!(decoded_path, path);
        assert!(!terminator);

        // Test odd length path
        let path = vec![1, 2, 3];
        let encoded = CompactEncoding::encode(&path, true);
        let (decoded_path, terminator) = CompactEncoding::decode(&encoded).unwrap();
        assert_eq!(decoded_path, path);
        assert!(terminator);

        // Test empty path
        let path = vec![];
        let encoded = CompactEncoding::encode(&path, false);
        let (decoded_path, terminator) = CompactEncoding::decode(&encoded).unwrap();
        assert_eq!(decoded_path, path);
        assert!(!terminator);
    }

    #[test]
    fn test_compact_encoding() {
        // Test encoding/decoding roundtrip
        let test_cases = vec![
            (vec![], false),
            (vec![1], true),
            (vec![1, 2, 3, 4], false),
            (vec![1, 2, 3, 4, 5], true),
        ];

        for (path, terminator) in test_cases {
            let encoded = CompactEncoding::encode(&path, terminator);
            let (decoded_path, decoded_terminator) = CompactEncoding::decode(&encoded).unwrap();
            assert_eq!(decoded_path, path);
            assert_eq!(decoded_terminator, terminator);
        }
    }
}
