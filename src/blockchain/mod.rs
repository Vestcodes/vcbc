//! Blockchain module providing core blockchain functionality.
//!
//! This module contains the fundamental blockchain structures and operations
//! including blocks, chains, network configuration, and transaction support.

pub mod block;
pub mod chain;

// Re-export main types for convenience
pub use block::{Block, NetworkConfig};
pub use chain::{Blockchain, BlockchainStats};

// Transaction support will be added later
// pub mod transaction;
// pub use transaction::Transaction;
