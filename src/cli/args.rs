//! CLI argument definitions for VCBC.
//!
//! Defines command-line argument structures and parsing
//! for all VCBC node management commands.

use clap::{Parser, Subcommand};

/// Main CLI structure
#[derive(Parser)]
#[command(name = "vcbc")]
#[command(about = "VCBC - Blockchain with MPT storage and P2P networking")]
#[command(version = "1.0")]
pub struct CliArgs {
    #[command(subcommand)]
    pub command: Commands,
}

/// Available CLI commands for VCBC node management
#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new blockchain from scratch
    InitChain {
        /// Network ID (e.g., vc-mainnet, vc-testnet)
        #[arg(short, long, default_value = "vc-mainnet")]
        network_id: String,
        /// Chain ID for network isolation
        #[arg(short, long, default_value = "1")]
        chain_id: u64,
        /// HTTP API port
        #[arg(long, default_value = "8080")]
        http_port: u16,
        /// P2P network port
        #[arg(short, long, default_value = "9090")]
        p2p_port: u16,
    },

    /// Initialize a node for an existing blockchain network
    InitNode {
        /// Bootstrap node URL (HTTP endpoint)
        #[arg(short, long)]
        bootstrap_url: String,
        /// HTTP API port for this node
        #[arg(long, default_value = "8081")]
        http_port: u16,
        /// P2P network port for this node
        #[arg(short, long, default_value = "9091")]
        p2p_port: u16,
    },

    /// Start a bootnode for the blockchain network
    StartBootnode {
        /// Path to configuration file
        #[arg(short, long, default_value = "config.json")]
        config: String,
    },

    /// Start a regular node in the blockchain network
    StartNode {
        /// Path to configuration file
        #[arg(short, long, default_value = "config.json")]
        config: String,
    },

    /// Initialize a network authority for bootnode certificates
    InitAuthority {
        /// Authority name/identifier
        #[arg(short, long)]
        name: String,
        /// Output path for authority keys
        #[arg(short, long, default_value = "authority.json")]
        output: String,
    },

    /// Register and certify a bootnode
    RegisterBootnode {
        /// Bootnode identifier
        #[arg(short, long)]
        node_id: String,
        /// Authority key file
        #[arg(short, long)]
        authority: String,
        /// Bootnode configuration file
        #[arg(short, long, default_value = "config.json")]
        config: String,
        /// Certificate output path
        #[arg(short, long, default_value = "bootnode_cert.json")]
        output: String,
    },
}
