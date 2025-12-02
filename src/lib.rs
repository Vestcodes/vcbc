//! # VCBC - Blockchain Implementation
//!
//! A modular, clean-code blockchain implementation with Merkle-Patricia Tries,
//! multi-node networking, and cryptographic proofs.
//!
//! ## Architecture
//!
//! ### Core Modules
//! - `blockchain/` - Block and chain management with network isolation
//! - `mpt/` - Merkle-Patricia Trie for efficient key-value storage
//! - `proof/` - Cryptographic proof generation and verification
//! - `network/` - P2P networking and HTTP API
//! - `storage/` - Persistence layer for blockchain data
//! - `config/` - Configuration management and validation
//!
//! ### Interface Modules
//! - `cli/` - Command-line interface with configuration commands
//!
//! ## Clean Code Principles Applied
//!
//! - Modular architecture with single responsibility per module
//! - Small functions that do one thing
//! - Proper error handling with custom error types
//! - Dependency injection and abstraction
//! - Comprehensive test coverage
//! - Clear naming and documentation
//! - Separation of concerns

pub mod blockchain;
pub mod cli;
pub mod config;
pub mod error;
pub mod mpt;
pub mod network;
pub mod proof;
pub mod storage;

// Re-export main types for convenience
pub use blockchain::{Block, Blockchain, NetworkConfig};
pub use error::BlockchainError;
pub use mpt::MerklePatriciaTrie;
pub use proof::MerkleProof;
pub use storage::db::Storage;

// Re-export commonly used types
pub use config::{BootnodeCertificate, NetworkAuthority, NodeConfig, NodeType};
pub type Result<T> = std::result::Result<T, BlockchainError>;
