//! Network module for VCBC blockchain.
//!
//! Provides P2P networking capabilities and HTTP API interfaces
//! for blockchain node communication and data access.

pub mod http;
pub mod p2p;

// Re-export main networking types
pub use http::HttpApi;
pub use p2p::P2PNetwork;
