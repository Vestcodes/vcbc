//! Proof module for VCBC blockchain.
//!
//! Provides cryptographic proof generation and verification
//! for data integrity and Merkle tree proofs.

pub mod core;

// Re-export main proof types
pub use core::{MerkleProof, ProofVerifier};
