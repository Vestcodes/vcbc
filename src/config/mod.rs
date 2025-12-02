//! Configuration module for VCBC blockchain nodes.
//!
//! Provides centralized configuration management for blockchain networks,
//! node setup, certificate authority management, and secure bootnode certificates.

pub mod authority;
pub mod certificate;
pub mod node;

// Re-export main configuration types
pub use authority::NetworkAuthority;
pub use certificate::BootnodeCertificate;
pub use node::{ensure_data_directory, NodeConfig, NodeType};
