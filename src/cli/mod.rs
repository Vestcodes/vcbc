//! CLI module for VCBC blockchain.
//!
//! Provides command-line interface for node configuration,
//! chain initialization, and bootnode management.

pub mod args;
pub mod commands;

// Re-export main CLI types
pub use args::{CliArgs, Commands};
pub use commands::VcbcApp;
