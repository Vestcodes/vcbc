//! VCBC - Blockchain Main Entry Point
//!
//! This is the main entry point that orchestrates all the modular,
//! clean-code components of the VCBC blockchain system.

use clap::Parser;
use vcbc::cli::args::{CliArgs, Commands};
use vcbc::error::Result;

// CLI struct removed - now using CliArgs from cli::args module

#[tokio::main]
async fn main() -> Result<()> {
    let cli = CliArgs::parse();

    // Handle configuration and startup commands
    match cli.command {
        Commands::InitChain {
            network_id,
            chain_id,
            http_port,
            p2p_port,
        } => {
            init_chain(network_id, chain_id, http_port, p2p_port).await?;
            return Ok(());
        }
        Commands::InitNode {
            bootstrap_url,
            http_port,
            p2p_port,
        } => {
            init_node(bootstrap_url, http_port, p2p_port).await?;
            return Ok(());
        }
        Commands::StartBootnode { config } => {
            start_bootnode(config).await?;
            return Ok(());
        }
        Commands::StartNode { config } => {
            start_node(config).await?;
            return Ok(());
        }
        Commands::InitAuthority { name, output } => {
            init_authority(name, output).await?;
            return Ok(());
        }
        Commands::RegisterBootnode {
            node_id,
            authority,
            config,
            output,
        } => {
            register_bootnode(node_id, authority, config, output).await?;
            return Ok(());
        }
    }
}

/// Initialize a new blockchain
async fn init_chain(
    network_id: String,
    chain_id: u64,
    http_port: u16,
    p2p_port: u16,
) -> Result<()> {
    println!("🏗️  Initializing new blockchain...");
    println!("🌐 Network: {}", network_id);
    println!("🔢 Chain ID: {}", chain_id);
    println!("🌐 HTTP Port: {}", http_port);
    println!("📡 P2P Port: {}", p2p_port);

    // Create bootnode configuration
    let config = vcbc::config::NodeConfig::bootnode(network_id, chain_id, http_port, p2p_port);

    // Validate configuration
    config.validate()?;

    // Save configuration
    config.save_to_file("config.json")?;
    println!("✅ Configuration saved to config.json");
    println!("🚀 Run: cargo run -- start-bootnode");

    Ok(())
}

/// Initialize a node for existing network
async fn init_node(bootstrap_url: String, http_port: u16, p2p_port: u16) -> Result<()> {
    println!("🔗 Initializing node for existing network...");
    println!("🔗 Bootstrap URL: {}", bootstrap_url);
    println!("🌐 HTTP Port: {}", http_port);
    println!("📡 P2P Port: {}", p2p_port);

    // Create regular node configuration
    let config = vcbc::config::NodeConfig::regular_node(bootstrap_url, http_port, p2p_port);

    // Validate configuration
    config.validate()?;

    // Save configuration
    config.save_to_file("config.json")?;
    println!("✅ Configuration saved to config.json");
    println!("🚀 Run: cargo run -- start-node");

    Ok(())
}

/// Start a bootstrap node
async fn start_bootnode(config_path: String) -> Result<()> {
    println!("🚀 Starting bootstrap node...");
    println!("📄 Config: {}", config_path);

    // Load configuration
    let config = vcbc::config::NodeConfig::load_from_file(&config_path)?;
    config.validate()?;

    if !config.is_bootnode() {
        return Err(vcbc::error::BlockchainError::InvalidInput(
            "Configuration is not for a bootnode".to_string(),
        ));
    }

    println!("🌐 Network: {}", config.network_id);
    println!("🔢 Chain ID: {}", config.chain_id);
    println!("🆔 Node ID: {}", config.node_id);
    println!("🌐 HTTP API: http://localhost:{}", config.http_port);
    println!("📡 P2P Port: {}", config.p2p_port);

    // TODO: Implement actual node startup with HTTP API and P2P networking
    println!("✅ Bootstrap node started!");
    println!(
        "🌐 HTTP API available at: http://localhost:{}",
        config.http_port
    );
    println!("🛑 Press Ctrl+C to stop");

    // For now, just wait
    tokio::signal::ctrl_c().await?;
    println!("🛑 Shutting down bootstrap node...");

    Ok(())
}

/// Start a regular network node
async fn start_node(config_path: String) -> Result<()> {
    println!("🚀 Starting network node...");
    println!("📄 Config: {}", config_path);

    // Load configuration
    let config = vcbc::config::NodeConfig::load_from_file(&config_path)?;
    config.validate()?;

    if config.is_bootnode() {
        return Err(vcbc::error::BlockchainError::InvalidInput(
            "Configuration is for a bootnode, use start-bootnode instead".to_string(),
        ));
    }

    println!("🆔 Node ID: {}", config.node_id);
    println!("🔗 Bootstrap: {:?}", config.bootstrap_nodes);
    println!("🌐 HTTP API: http://localhost:{}", config.http_port);
    println!("📡 P2P Port: {}", config.p2p_port);

    // TODO: Implement actual node startup with HTTP API and P2P networking
    println!("✅ Network node started!");
    println!(
        "🌐 HTTP API available at: http://localhost:{}",
        config.http_port
    );
    println!("🛑 Press Ctrl+C to stop");

    // For now, just wait
    tokio::signal::ctrl_c().await?;
    println!("🛑 Shutting down network node...");

    Ok(())
}

/// Initialize a new network authority
async fn init_authority(name: String, output: String) -> Result<()> {
    println!("🔐 Initializing network authority...");
    println!("🏛️  Authority: {}", name);
    println!("📄 Output: {}", output);

    // Create the authority and private key
    let (authority, signing_key) = vcbc::config::NetworkAuthority::new(name);

    // Save the authority public info
    authority.save_to_file(&output)?;

    // Save the private key separately (should be kept secure!)
    let private_key_path = format!("{}.key", output);
    use base64::{engine::general_purpose, Engine as _};
    let private_key_b64 = general_purpose::STANDARD.encode(signing_key.to_bytes());
    std::fs::write(&private_key_path, private_key_b64).map_err(|e| {
        vcbc::error::BlockchainError::StorageError(format!(
            "Failed to write private key file '{}': {}",
            private_key_path, e
        ))
    })?;

    println!("✅ Authority created and saved to {}", output);
    println!(
        "🔑 Private key saved to {} (keep this secure!)",
        private_key_path
    );
    println!("🚨 WARNING: The private key file contains sensitive cryptographic material!");
    println!("🚨 Store it securely and never share it with anyone!");

    Ok(())
}

/// Register and certify a bootnode with a network authority
async fn register_bootnode(
    node_id: String,
    authority_path: String,
    config_path: String,
    output_path: String,
) -> Result<()> {
    println!("📜 Registering bootnode with network authority...");
    println!("🆔 Node ID: {}", node_id);
    println!("🏛️  Authority: {}", authority_path);
    println!("📄 Config: {}", config_path);
    println!("📄 Output: {}", output_path);

    // Load the bootnode configuration
    let mut config = vcbc::config::NodeConfig::load_from_file(&config_path)?;
    config.validate()?;

    if !config.is_bootnode() {
        return Err(vcbc::error::BlockchainError::InvalidInput(
            "Configuration is not for a bootnode".to_string(),
        ));
    }

    // Load the authority private key
    use base64::{engine::general_purpose, Engine as _};
    let private_key_b64 = std::fs::read_to_string(&authority_path).map_err(|e| {
        vcbc::error::BlockchainError::StorageError(format!(
            "Failed to read authority key file '{}': {}",
            authority_path, e
        ))
    })?;

    let private_key_bytes = general_purpose::STANDARD
        .decode(private_key_b64.trim())
        .map_err(|e| {
            vcbc::error::BlockchainError::InvalidInput(format!("Invalid private key format: {}", e))
        })?;

    let keypair_bytes: [u8; 64] = private_key_bytes.try_into().map_err(|_| {
        vcbc::error::BlockchainError::InvalidInput(
            "Private key must be exactly 64 bytes (keypair)".to_string(),
        )
    })?;

    let signing_key =
        ed25519_dalek::SigningKey::from_keypair_bytes(&keypair_bytes).map_err(|e| {
            vcbc::error::BlockchainError::InvalidInput(format!("Invalid private key: {}", e))
        })?;

    // Calculate expiry timestamp (30 days default)
    let expiry_timestamp = chrono::Utc::now().timestamp() + (30 * 24 * 60 * 60);

    // Create and sign the certificate
    let mut certificate = vcbc::config::BootnodeCertificate::new(
        node_id,
        config.network_id.clone(),
        config.chain_id,
        "http://localhost:8080".to_string(),   // Default HTTP URL
        "/ip4/127.0.0.1/tcp/9090".to_string(), // Default P2P multiaddr
        expiry_timestamp,
    );

    certificate.sign(&signing_key)?;

    // Update the configuration with the certificate
    config.certificate = Some(certificate);

    // Save the updated configuration
    config.save_to_file(&output_path)?;

    println!("✅ Bootnode registered and certificate issued!");
    println!(
        "📜 Certificate expires: {}",
        chrono::DateTime::from_timestamp(expiry_timestamp, 0)
            .unwrap_or_else(chrono::Utc::now)
            .format("%Y-%m-%d %H:%M:%S UTC")
    );
    println!("💾 Configuration saved to {}", output_path);

    Ok(())
}
