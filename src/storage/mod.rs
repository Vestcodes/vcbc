//! Storage module for VCBC blockchain persistence.
//!
//! Provides persistent storage capabilities for blockchain data,
//! supporting both file-based and database storage backends.

pub mod db;
pub mod persistence;

// Re-export main storage types
pub use db::StorageManager;
pub use persistence::PersistenceLayer;
