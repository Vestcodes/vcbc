//! Merkle-Patricia Trie module for VCBC blockchain.
//!
//! This module provides an efficient Merkle-Patricia Trie implementation
//! for storing and verifying blockchain data with cryptographic proofs.

pub mod hash;
pub mod node;
pub mod trie;

// Re-export main types for convenience
pub use hash::KeyUtils;
pub use node::{CompactEncoding, MPTNode};
pub use trie::MerklePatriciaTrie;
