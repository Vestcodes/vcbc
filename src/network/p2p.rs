//! P2P Network Implementation
//!
//! Provides decentralized peer-to-peer networking framework,
//! enabling automatic peer discovery, network isolation, and secure communication.

use crate::blockchain::{Block, NetworkConfig};
use crate::error::{BlockchainError, Result};
use libp2p::{Multiaddr, PeerId};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Protocol identifiers for blockchain communication
pub const SYNC_PROTOCOL: &str = "/blockchain/sync/1.0.0";
pub const BLOCKS_PROTOCOL: &str = "/blockchain/blocks/1.0.0";
pub const PROOF_PROTOCOL: &str = "/blockchain/proof/1.0.0";

/// Handshake message exchanged between peers for network validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakeMessage {
    pub network_id: String,
    pub chain_id: u64,
    pub genesis_hash: String,
    pub protocol_version: String,
    pub supported_protocols: Vec<String>,
    pub node_version: String,
}

/// Protocol message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProtocolMessage {
    /// Handshake message for peer introduction
    Handshake(HandshakeMessage),
    /// Block propagation message
    BlockBroadcast(Block),
    /// Chain synchronization request
    ChainSync { from_block: u64 },
    /// Chain synchronization response
    ChainData { blocks: Vec<Block> },
    /// Proof request
    ProofRequest { block_index: u64, key: String },
    /// Proof response
    ProofResponse { proof: Option<crate::MerkleProof> },
}

/// Events emitted by the P2P network
#[derive(Debug)]
pub enum P2PEvent {
    /// New peer discovered and connected
    PeerConnected {
        peer_id: PeerId,
        handshake: HandshakeMessage,
    },
    /// Peer disconnected
    PeerDisconnected { peer_id: PeerId },
    /// New block received from peer
    BlockReceived { peer_id: PeerId, block: Block },
    /// Chain synchronization request
    ChainSyncRequested { peer_id: PeerId, from_block: u64 },
    /// Chain data received from peer
    ChainDataReceived { peer_id: PeerId, blocks: Vec<Block> },
    /// Proof request received
    ProofRequested {
        peer_id: PeerId,
        block_index: u64,
        key: String,
    },
    /// Network compatibility validation failed
    IncompatiblePeer { peer_id: PeerId, reason: String },
}

/// Main P2P network manager
pub struct P2PNetwork {
    network_config: NetworkConfig,
    connected_peers: HashSet<PeerId>,
    local_peer_id: PeerId,
}

impl P2PNetwork {
    /// Create a new P2P network instance
    pub fn new(network_config: NetworkConfig) -> Self {
        let local_key = libp2p::identity::Keypair::generate_ed25519();
        let local_peer_id = PeerId::from(local_key.public());

        Self {
            network_config,
            connected_peers: HashSet::new(),
            local_peer_id,
        }
    }

    /// Validate peer compatibility based on handshake
    pub fn validate_peer_handshake(&self, handshake: &HandshakeMessage) -> Result<()> {
        if handshake.network_id != self.network_config.network_id {
            return Err(BlockchainError::NetworkMismatch {
                expected: self.network_config.network_id.clone(),
                received: handshake.network_id.clone(),
            });
        }
        if handshake.genesis_hash != self.network_config.genesis_hash {
            return Err(BlockchainError::GenesisMismatch {
                expected: self.network_config.genesis_hash.clone(),
                received: handshake.genesis_hash.clone(),
            });
        }
        if handshake.protocol_version != self.network_config.protocol_version {
            return Err(BlockchainError::ProtocolMismatch {
                expected: self.network_config.protocol_version.clone(),
                received: handshake.protocol_version.clone(),
            });
        }
        Ok(())
    }

    /// Get the local peer ID
    pub fn local_peer_id(&self) -> &PeerId {
        &self.local_peer_id
    }

    /// Get connected peers count
    pub fn connected_peers_count(&self) -> usize {
        self.connected_peers.len()
    }

    /// Add a peer to the connected set (for testing/simulation)
    pub fn add_peer(&mut self, peer_id: PeerId) {
        self.connected_peers.insert(peer_id);
    }

    /// Remove a peer from the connected set
    pub fn remove_peer(&mut self, peer_id: &PeerId) {
        self.connected_peers.remove(peer_id);
    }

    /// Simulate receiving a block from a peer (for testing)
    pub fn simulate_block_received(&mut self, peer_id: PeerId, block: Block) {
        println!(
            "📦 Simulated receiving block {} from peer {}",
            block.index, peer_id
        );
        // In a real implementation, this would trigger the event system
    }

    /// Simulate peer handshake (for testing)
    pub fn simulate_handshake(&mut self, peer_id: PeerId, handshake: HandshakeMessage) {
        match self.validate_peer_handshake(&handshake) {
            Ok(()) => {
                println!("✅ Peer {} handshake validated", peer_id);
                self.connected_peers.insert(peer_id);
            }
            Err(e) => {
                println!("❌ Peer {} handshake failed: {}", peer_id, e);
            }
        }
    }

    /// Send a protocol message (simulated for now)
    pub fn send_message(&self, peer_id: &PeerId, message: ProtocolMessage) {
        println!("📤 Sending message to {}: {:?}", peer_id, message);
        // In a real implementation, this would serialize and send via libp2p
    }

    /// Broadcast a block to all connected peers
    pub fn broadcast_block(&self, block: &Block) {
        let message = ProtocolMessage::BlockBroadcast(block.clone());
        for peer_id in &self.connected_peers {
            self.send_message(peer_id, message.clone());
        }
        println!(
            "📡 Broadcasted block {} to {} peers",
            block.index,
            self.connected_peers.len()
        );
    }

    /// Request chain synchronization from a peer
    pub fn request_chain_sync(&self, peer_id: &PeerId, from_block: u64) {
        let message = ProtocolMessage::ChainSync { from_block };
        self.send_message(peer_id, message);
        println!(
            "🔄 Requested chain sync from {} starting at block {}",
            peer_id, from_block
        );
    }

    /// Send chain data in response to a sync request
    pub fn send_chain_data(&self, peer_id: &PeerId, blocks: Vec<Block>) {
        let block_count = blocks.len();
        let message = ProtocolMessage::ChainData { blocks };
        self.send_message(peer_id, message);
        println!(
            "📤 Sent {} blocks to {} for chain sync",
            block_count, peer_id
        );
    }

    /// Request a Merkle proof from a peer
    pub fn request_proof(&self, peer_id: &PeerId, block_index: u64, key: &str) {
        let message = ProtocolMessage::ProofRequest {
            block_index,
            key: key.to_string(),
        };
        self.send_message(peer_id, message);
        println!(
            "🔍 Requested proof for key '{}' in block {} from {}",
            key, block_index, peer_id
        );
    }

    /// Send a Merkle proof in response to a request
    pub fn send_proof(&self, peer_id: &PeerId, proof: Option<crate::MerkleProof>) {
        let has_proof = proof.is_some();
        let key_name = proof
            .as_ref()
            .map(|p| p.key.clone())
            .unwrap_or_else(|| "none".to_string());
        let message = ProtocolMessage::ProofResponse { proof };
        self.send_message(peer_id, message);
        if has_proof {
            println!("📤 Sent proof for key '{}' to {}", key_name, peer_id);
        } else {
            println!("📤 Sent proof (not found) to {}", peer_id);
        }
    }

    /// Connect to a peer (placeholder implementation)
    pub async fn connect_to_peer(&mut self, addr: &Multiaddr) -> Result<()> {
        // TODO: Implement actual P2P connection
        // For now, this is a placeholder that simulates connection
        println!("🔗 Connecting to peer at: {}", addr);
        // In a real implementation, this would:
        // 1. Parse the multiaddr
        // 2. Establish connection via libp2p swarm
        // 3. Add peer to connected_peers set
        Ok(())
    }

    /// Handle incoming protocol messages (simulated)
    pub fn handle_message(
        &mut self,
        peer_id: PeerId,
        message: ProtocolMessage,
    ) -> Option<P2PEvent> {
        match message {
            ProtocolMessage::Handshake(handshake) => {
                match self.validate_peer_handshake(&handshake) {
                    Ok(()) => {
                        self.connected_peers.insert(peer_id);
                        Some(P2PEvent::PeerConnected { peer_id, handshake })
                    }
                    Err(e) => Some(P2PEvent::IncompatiblePeer {
                        peer_id,
                        reason: format!("{}", e),
                    }),
                }
            }
            ProtocolMessage::BlockBroadcast(block) => {
                Some(P2PEvent::BlockReceived { peer_id, block })
            }
            ProtocolMessage::ChainSync { from_block } => Some(P2PEvent::ChainSyncRequested {
                peer_id,
                from_block,
            }),
            ProtocolMessage::ChainData { blocks } => {
                Some(P2PEvent::ChainDataReceived { peer_id, blocks })
            }
            ProtocolMessage::ProofRequest { block_index, key } => Some(P2PEvent::ProofRequested {
                peer_id,
                block_index,
                key,
            }),
            ProtocolMessage::ProofResponse { proof: _ } => {
                // Proof responses would be handled by the requesting party
                println!("📥 Received proof response from {}", peer_id);
                None
            }
        }
    }

    /// Discover and validate bootnodes securely
    ///
    /// This method queries certified bootnodes and validates their certificates
    /// against the provided trusted authorities before adding them to the network.
    pub async fn discover_secure_bootnodes(
        &mut self,
        bootnode_urls: &[String],
        trusted_authorities: &[crate::config::NetworkAuthority],
    ) -> crate::error::Result<Vec<crate::config::BootnodeCertificate>> {
        let mut valid_bootnodes = Vec::new();

        for url in bootnode_urls {
            match self.query_bootnode_certificate(url).await {
                Ok(Some(cert)) => {
                    // Validate the certificate
                    if let Err(e) = cert.verify(trusted_authorities) {
                        eprintln!(
                            "⚠️  Bootnode certificate verification failed for {}: {}",
                            url, e
                        );
                        continue;
                    }

                    // Check if this matches our network
                    if cert.network_id == self.network_config.network_id
                        && cert.chain_id == self.network_config.chain_id
                    {
                        println!(
                            "✅ Valid bootnode discovered: {} ({})",
                            cert.http_url, cert.node_id
                        );
                        valid_bootnodes.push(cert);
                    } else {
                        eprintln!(
                            "⚠️  Bootnode network mismatch: expected {}/{}, got {}/{}",
                            self.network_config.network_id,
                            self.network_config.chain_id,
                            cert.network_id,
                            cert.chain_id
                        );
                    }
                }
                Ok(None) => {
                    eprintln!("⚠️  Bootnode {} does not provide certificate", url);
                }
                Err(e) => {
                    eprintln!("⚠️  Failed to query bootnode {}: {}", url, e);
                }
            }
        }

        Ok(valid_bootnodes)
    }

    /// Query a bootnode for its certificate
    async fn query_bootnode_certificate(
        &self,
        bootnode_url: &str,
    ) -> crate::error::Result<Option<crate::config::BootnodeCertificate>> {
        // For now, this is a placeholder - in a real implementation,
        // this would make an HTTP request to the bootnode's /certificate endpoint
        // to retrieve the certificate information.

        // TODO: Implement actual HTTP request to bootnode
        // The bootnode should expose a /certificate endpoint that returns
        // the BootnodeCertificate as JSON

        eprintln!("🔍 Querying bootnode certificate from: {}", bootnode_url);
        eprintln!("⚠️  Certificate querying not yet implemented - placeholder");

        // Placeholder return - in real implementation, this would parse
        // the response from the bootnode's certificate endpoint
        Ok(None)
    }

    /// Connect to validated bootnodes
    pub async fn connect_to_validated_bootnodes(
        &mut self,
        certificates: &[crate::config::BootnodeCertificate],
    ) -> crate::error::Result<()> {
        for cert in certificates {
            // Parse the P2P multiaddr and attempt connection
            match cert.p2p_multiaddr.parse::<Multiaddr>() {
                Ok(addr) => {
                    println!("🔗 Connecting to bootnode: {}", cert.p2p_multiaddr);
                    if let Err(e) = self.connect_to_peer(&addr).await {
                        eprintln!(
                            "⚠️  Failed to connect to bootnode {}: {}",
                            cert.p2p_multiaddr, e
                        );
                        continue;
                    }
                }
                Err(e) => {
                    eprintln!("⚠️  Invalid P2P multiaddr in certificate: {}", e);
                    continue;
                }
            }
        }

        Ok(())
    }

    /// Initialize secure network with certificate validation
    ///
    /// This is the main entry point for secure P2P network initialization.
    /// It discovers validated bootnodes, connects to them, and establishes
    /// the secure network foundation.
    pub async fn initialize_secure_network(
        &mut self,
        bootstrap_urls: &[String],
        trusted_authorities: &[crate::config::NetworkAuthority],
    ) -> crate::error::Result<()> {
        println!("🔐 Initializing secure P2P network...");

        // Discover and validate bootnodes
        let valid_bootnodes = self
            .discover_secure_bootnodes(bootstrap_urls, trusted_authorities)
            .await?;

        if valid_bootnodes.is_empty() {
            return Err(crate::error::BlockchainError::NetworkError(
                "No valid bootnodes found for this network".to_string(),
            ));
        }

        println!("📋 Found {} valid bootnode(s)", valid_bootnodes.len());

        // Connect to validated bootnodes
        self.connect_to_validated_bootnodes(&valid_bootnodes)
            .await?;

        println!("✅ Secure P2P network initialized");
        Ok(())
    }
}

/// Helper function to create default network configuration
pub fn default_network_config() -> NetworkConfig {
    NetworkConfig {
        network_id: "vc-mainnet".to_string(),
        chain_id: 1,
        genesis_hash: "".to_string(),
        protocol_version: "1.0.0".to_string(),
    }
}

/// Helper function to parse multiaddr from string
pub fn parse_multiaddr(addr_str: &str) -> Result<Multiaddr> {
    addr_str.parse().map_err(|e| {
        BlockchainError::InvalidInput(format!("Invalid multiaddr '{}': {}", addr_str, e))
    })
}

/// Helper function to format peer info for display
pub fn format_peer_info(peer_id: &PeerId, handshake: &Option<HandshakeMessage>) -> String {
    match handshake {
        Some(hs) => format!(
            "Peer {}: Network '{}', Chain ID {}, Version {}",
            peer_id, hs.network_id, hs.chain_id, hs.protocol_version
        ),
        None => format!("Peer {}: Handshake pending", peer_id),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use libp2p::PeerId;

    #[test]
    fn test_p2p_network_creation() {
        let config = NetworkConfig {
            network_id: "test-net".to_string(),
            chain_id: 1,
            genesis_hash: "test-hash".to_string(),
            protocol_version: "1.0.0".to_string(),
        };

        let network = P2PNetwork::new(config);
        assert_eq!(network.network_config.network_id, "test-net");
        assert_eq!(network.connected_peers_count(), 0);
    }

    #[test]
    fn test_peer_handshake_validation_success() {
        let config = NetworkConfig {
            network_id: "test-net".to_string(),
            chain_id: 1,
            genesis_hash: "test-genesis".to_string(),
            protocol_version: "1.0.0".to_string(),
        };

        let network = P2PNetwork::new(config);
        let handshake = HandshakeMessage {
            network_id: "test-net".to_string(),
            chain_id: 1,
            genesis_hash: "test-genesis".to_string(),
            protocol_version: "1.0.0".to_string(),
            supported_protocols: vec!["sync".to_string(), "blocks".to_string()],
            node_version: "1.0.0".to_string(),
        };

        assert!(network.validate_peer_handshake(&handshake).is_ok());
    }

    #[test]
    fn test_peer_handshake_validation_network_mismatch() {
        let config = NetworkConfig {
            network_id: "test-net".to_string(),
            chain_id: 1,
            genesis_hash: "test-genesis".to_string(),
            protocol_version: "1.0.0".to_string(),
        };

        let network = P2PNetwork::new(config);
        let handshake = HandshakeMessage {
            network_id: "different-net".to_string(), // Different network
            chain_id: 1,
            genesis_hash: "test-genesis".to_string(),
            protocol_version: "1.0.0".to_string(),
            supported_protocols: vec!["sync".to_string(), "blocks".to_string()],
            node_version: "1.0.0".to_string(),
        };

        assert!(network.validate_peer_handshake(&handshake).is_err());
    }

    #[test]
    fn test_peer_handshake_validation_genesis_mismatch() {
        let config = NetworkConfig {
            network_id: "test-net".to_string(),
            chain_id: 1,
            genesis_hash: "test-genesis".to_string(),
            protocol_version: "1.0.0".to_string(),
        };

        let network = P2PNetwork::new(config);
        let handshake = HandshakeMessage {
            network_id: "test-net".to_string(),
            chain_id: 1,
            genesis_hash: "different-genesis".to_string(), // Different genesis
            protocol_version: "1.0.0".to_string(),
            supported_protocols: vec!["sync".to_string(), "blocks".to_string()],
            node_version: "1.0.0".to_string(),
        };

        assert!(network.validate_peer_handshake(&handshake).is_err());
    }

    #[test]
    fn test_peer_management() {
        let config = NetworkConfig {
            network_id: "test-net".to_string(),
            chain_id: 1,
            genesis_hash: "test-genesis".to_string(),
            protocol_version: "1.0.0".to_string(),
        };

        let mut network = P2PNetwork::new(config);
        let peer_id = PeerId::random();

        // Initially no peers
        assert_eq!(network.connected_peers_count(), 0);

        // Add peer
        network.add_peer(peer_id);
        assert_eq!(network.connected_peers_count(), 1);

        // Remove peer
        network.remove_peer(&peer_id);
        assert_eq!(network.connected_peers_count(), 0);
    }

    #[test]
    fn test_parse_multiaddr() {
        let addr_str = "/ip4/127.0.0.1/tcp/8080";
        let result = parse_multiaddr(addr_str);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_multiaddr_invalid() {
        let addr_str = "invalid-address";
        let result = parse_multiaddr(addr_str);
        assert!(result.is_err());
    }

    #[test]
    fn test_protocol_messages() {
        let config = NetworkConfig {
            network_id: "test-net".to_string(),
            chain_id: 1,
            genesis_hash: "test-genesis".to_string(),
            protocol_version: "1.0.0".to_string(),
        };

        let mut network = P2PNetwork::new(config);
        let peer_id = PeerId::random();

        // Test handshake message
        let handshake = HandshakeMessage {
            network_id: "test-net".to_string(),
            chain_id: 1,
            genesis_hash: "test-genesis".to_string(),
            protocol_version: "1.0.0".to_string(),
            supported_protocols: vec!["sync".to_string()],
            node_version: "1.0.0".to_string(),
        };

        let handshake_msg = ProtocolMessage::Handshake(handshake.clone());
        let event = network.handle_message(peer_id, handshake_msg);

        assert!(event.is_some());
        match event.unwrap() {
            P2PEvent::PeerConnected {
                peer_id: event_peer_id,
                handshake: event_handshake,
            } => {
                assert_eq!(event_peer_id, peer_id);
                assert_eq!(event_handshake.network_id, handshake.network_id);
            }
            _ => panic!("Expected PeerConnected event"),
        }

        // Test block broadcast message
        let block = crate::Block::genesis();
        let block_msg = ProtocolMessage::BlockBroadcast(block.clone());
        let event = network.handle_message(peer_id, block_msg);

        assert!(event.is_some());
        match event.unwrap() {
            P2PEvent::BlockReceived {
                peer_id: event_peer_id,
                block: event_block,
            } => {
                assert_eq!(event_peer_id, peer_id);
                assert_eq!(event_block.index, block.index);
            }
            _ => panic!("Expected BlockReceived event"),
        }
    }

    #[test]
    fn test_chain_sync_messages() {
        let config = NetworkConfig {
            network_id: "test-net".to_string(),
            chain_id: 1,
            genesis_hash: "test-genesis".to_string(),
            protocol_version: "1.0.0".to_string(),
        };

        let mut network = P2PNetwork::new(config);
        let peer_id = PeerId::random();

        // Test chain sync request
        let sync_msg = ProtocolMessage::ChainSync { from_block: 5 };
        let event = network.handle_message(peer_id, sync_msg);

        assert!(event.is_some());
        match event.unwrap() {
            P2PEvent::ChainSyncRequested {
                peer_id: event_peer_id,
                from_block,
            } => {
                assert_eq!(event_peer_id, peer_id);
                assert_eq!(from_block, 5);
            }
            _ => panic!("Expected ChainSyncRequested event"),
        }

        // Test chain data response
        let blocks = vec![crate::Block::genesis()];
        let chain_msg = ProtocolMessage::ChainData {
            blocks: blocks.clone(),
        };
        let event = network.handle_message(peer_id, chain_msg);

        assert!(event.is_some());
        match event.unwrap() {
            P2PEvent::ChainDataReceived {
                peer_id: event_peer_id,
                blocks: event_blocks,
            } => {
                assert_eq!(event_peer_id, peer_id);
                assert_eq!(event_blocks.len(), blocks.len());
            }
            _ => panic!("Expected ChainDataReceived event"),
        }
    }

    #[test]
    fn test_proof_messages() {
        let config = NetworkConfig {
            network_id: "test-net".to_string(),
            chain_id: 1,
            genesis_hash: "test-genesis".to_string(),
            protocol_version: "1.0.0".to_string(),
        };

        let mut network = P2PNetwork::new(config);
        let peer_id = PeerId::random();

        // Test proof request
        let proof_request_msg = ProtocolMessage::ProofRequest {
            block_index: 1,
            key: "test_key".to_string(),
        };
        let event = network.handle_message(peer_id, proof_request_msg);

        assert!(event.is_some());
        match event.unwrap() {
            P2PEvent::ProofRequested {
                peer_id: event_peer_id,
                block_index,
                key,
            } => {
                assert_eq!(event_peer_id, peer_id);
                assert_eq!(block_index, 1);
                assert_eq!(key, "test_key");
            }
            _ => panic!("Expected ProofRequested event"),
        }
    }

    #[test]
    fn test_secure_bootnode_discovery() {
        use crate::config::{BootnodeCertificate, NetworkAuthority};
        use chrono::Utc;

        let network_config = NetworkConfig {
            network_id: "vc-mainnet".to_string(),
            chain_id: 1,
            genesis_hash: "test-genesis".to_string(),
            protocol_version: "1.0.0".to_string(),
        };

        let _network = P2PNetwork::new(network_config);

        // Create a mock authority and certificate
        let (authority, signing_key) = NetworkAuthority::new("vc-mainnet-authority".to_string());

        let mut cert = BootnodeCertificate::new(
            "node-123".to_string(),
            "vc-mainnet".to_string(),
            1,
            "http://bootnode.example.com:8080".to_string(),
            "/ip4/127.0.0.1/tcp/9090".to_string(),
            Utc::now().timestamp() + 86400,
        );
        cert.sign(&signing_key).expect("Failed to sign certificate");

        // For this test, we can't easily mock the HTTP request,
        // so we test the validation logic separately
        // The query_bootnode_certificate method returns None as placeholder
        // In real usage, this would be implemented with actual HTTP calls

        // Test certificate validation
        let result = cert.verify(&[authority]);
        assert!(
            result.is_ok(),
            "Valid certificate should verify successfully"
        );
    }

    #[test]
    fn test_secure_discovery_wrong_network() {
        use crate::config::{BootnodeCertificate, NetworkAuthority};
        use chrono::Utc;

        let network_config = NetworkConfig {
            network_id: "vc-testnet".to_string(), // Different network
            chain_id: 1,
            genesis_hash: "test-genesis".to_string(),
            protocol_version: "1.0.0".to_string(),
        };

        let _network = P2PNetwork::new(network_config);

        // Create certificate for different network
        let (authority, signing_key) = NetworkAuthority::new("vc-mainnet-authority".to_string());

        let mut cert = BootnodeCertificate::new(
            "node-123".to_string(),
            "vc-mainnet".to_string(), // Different network than P2P network
            1,
            "http://bootnode.example.com:8080".to_string(),
            "/ip4/127.0.0.1/tcp/9090".to_string(),
            Utc::now().timestamp() + 86400,
        );
        cert.sign(&signing_key).expect("Failed to sign certificate");

        // Certificate should verify against authority
        let result = cert.verify(&[authority]);
        assert!(
            result.is_ok(),
            "Certificate should verify against its authority"
        );

        // But network initialization should reject mismatched network
        // (This would be tested in the discover_secure_bootnodes method)
    }

    #[test]
    fn test_local_peer_id() {
        let config = NetworkConfig {
            network_id: "test-net".to_string(),
            chain_id: 1,
            genesis_hash: "test-hash".to_string(),
            protocol_version: "1.0.0".to_string(),
        };
        let network = P2PNetwork::new(config);
        let peer_id = network.local_peer_id();
        assert!(!peer_id.to_string().is_empty());
    }

    #[test]
    fn test_connected_peers_count() {
        let config = NetworkConfig {
            network_id: "test-net".to_string(),
            chain_id: 1,
            genesis_hash: "test-hash".to_string(),
            protocol_version: "1.0.0".to_string(),
        };
        let mut network = P2PNetwork::new(config);
        assert_eq!(network.connected_peers_count(), 0);

        let peer_id = PeerId::random();
        network.add_peer(peer_id);
        assert_eq!(network.connected_peers_count(), 1);
    }

    #[test]
    fn test_simulate_block_received() {
        let config = NetworkConfig {
            network_id: "test-net".to_string(),
            chain_id: 1,
            genesis_hash: "test-hash".to_string(),
            protocol_version: "1.0.0".to_string(),
        };
        let mut network = P2PNetwork::new(config.clone());
        let peer_id = PeerId::random();
        let block = crate::blockchain::Block::genesis_with_config(config.clone());

        // This should not panic
        network.simulate_block_received(peer_id, block);
    }

    #[test]
    fn test_simulate_handshake() {
        let config = NetworkConfig {
            network_id: "test-net".to_string(),
            chain_id: 1,
            genesis_hash: "test-hash".to_string(),
            protocol_version: "1.0.0".to_string(),
        };
        let mut network = P2PNetwork::new(config);
        let peer_id = PeerId::random();
        let handshake = HandshakeMessage {
            network_id: "test-net".to_string(),
            chain_id: 1,
            genesis_hash: "test-hash".to_string(),
            protocol_version: "1.0.0".to_string(),
            supported_protocols: vec!["sync".to_string()],
            node_version: "1.0.0".to_string(),
        };

        network.simulate_handshake(peer_id, handshake);
        assert_eq!(network.connected_peers_count(), 1);
        assert!(network.connected_peers.contains(&peer_id));
    }

    #[test]
    fn test_send_message() {
        let config = NetworkConfig {
            network_id: "test-net".to_string(),
            chain_id: 1,
            genesis_hash: "test-hash".to_string(),
            protocol_version: "1.0.0".to_string(),
        };
        let network = P2PNetwork::new(config.clone());
        let peer_id = PeerId::random();
        let message = ProtocolMessage::ChainSync { from_block: 0 };

        // This should not panic - in real implementation would send over network
        network.send_message(&peer_id, message);
    }

    #[test]
    fn test_broadcast_block() {
        let config = NetworkConfig {
            network_id: "test-net".to_string(),
            chain_id: 1,
            genesis_hash: "test-hash".to_string(),
            protocol_version: "1.0.0".to_string(),
        };
        let network = P2PNetwork::new(config.clone());
        let block = crate::blockchain::Block::genesis_with_config(config.clone());

        // This should not panic
        network.broadcast_block(&block);
    }

    #[test]
    fn test_request_chain_sync() {
        let config = NetworkConfig {
            network_id: "test-net".to_string(),
            chain_id: 1,
            genesis_hash: "test-hash".to_string(),
            protocol_version: "1.0.0".to_string(),
        };
        let network = P2PNetwork::new(config);
        let peer_id = PeerId::random();

        // This should not panic
        network.request_chain_sync(&peer_id, 0);
    }

    #[test]
    fn test_send_chain_data() {
        let config = NetworkConfig {
            network_id: "test-net".to_string(),
            chain_id: 1,
            genesis_hash: "test-hash".to_string(),
            protocol_version: "1.0.0".to_string(),
        };
        let network = P2PNetwork::new(config.clone());
        let peer_id = PeerId::random();
        let blocks = vec![crate::blockchain::Block::genesis_with_config(config)];

        // This should not panic
        network.send_chain_data(&peer_id, blocks);
    }

    #[test]
    fn test_request_proof() {
        let config = NetworkConfig {
            network_id: "test-net".to_string(),
            chain_id: 1,
            genesis_hash: "test-hash".to_string(),
            protocol_version: "1.0.0".to_string(),
        };
        let network = P2PNetwork::new(config);
        let peer_id = PeerId::random();

        // This should not panic
        network.request_proof(&peer_id, 0, "test-key");
    }

    #[test]
    fn test_send_proof_with_data() {
        let config = NetworkConfig {
            network_id: "test-net".to_string(),
            chain_id: 1,
            genesis_hash: "test-hash".to_string(),
            protocol_version: "1.0.0".to_string(),
        };
        let network = P2PNetwork::new(config.clone());
        let peer_id = PeerId::random();

        // Create a mock proof
        let proof = crate::MerkleProof {
            key: "test-key".to_string(),
            value: b"test-value".to_vec(),
            block_index: 1,
            mpt_root_hash: b"test-root".to_vec(),
            block_hash: "test-block-hash".to_string(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
        };

        // This should not panic
        network.send_proof(&peer_id, Some(proof));
    }

    #[test]
    fn test_send_proof_without_data() {
        let config = NetworkConfig {
            network_id: "test-net".to_string(),
            chain_id: 1,
            genesis_hash: "test-hash".to_string(),
            protocol_version: "1.0.0".to_string(),
        };
        let network = P2PNetwork::new(config);
        let peer_id = PeerId::random();

        // This should not panic
        network.send_proof(&peer_id, None);
    }

    #[test]
    fn test_handle_message_handshake() {
        let config = NetworkConfig {
            network_id: "test-net".to_string(),
            chain_id: 1,
            genesis_hash: "test-hash".to_string(),
            protocol_version: "1.0.0".to_string(),
        };
        let mut network = P2PNetwork::new(config);
        let peer_id = PeerId::random();
        let handshake = HandshakeMessage {
            network_id: "test-net".to_string(),
            chain_id: 1,
            genesis_hash: "test-hash".to_string(),
            protocol_version: "1.0.0".to_string(),
            supported_protocols: vec!["sync".to_string()],
            node_version: "1.0.0".to_string(),
        };
        let message = ProtocolMessage::Handshake(handshake.clone());

        let event = network.handle_message(peer_id, message);
        assert!(matches!(event, Some(P2PEvent::PeerConnected { .. })));
        assert!(network.connected_peers.contains(&peer_id));
    }

    #[test]
    fn test_handle_message_block_broadcast() {
        let config = NetworkConfig {
            network_id: "test-net".to_string(),
            chain_id: 1,
            genesis_hash: "test-hash".to_string(),
            protocol_version: "1.0.0".to_string(),
        };
        let mut network = P2PNetwork::new(config.clone());
        let peer_id = PeerId::random();
        let block = crate::blockchain::Block::genesis_with_config(config.clone());
        let message = ProtocolMessage::BlockBroadcast(block);

        // Should handle and return BlockReceived event
        let event = network.handle_message(peer_id, message);
        assert!(matches!(event, Some(P2PEvent::BlockReceived { .. })));
    }

    #[test]
    fn test_default_network_config() {
        let config = default_network_config();
        assert_eq!(config.network_id, "vc-mainnet");
        assert_eq!(config.chain_id, 1);
        assert_eq!(config.genesis_hash, ""); // Default config has empty genesis hash
        assert!(!config.protocol_version.is_empty());
    }

    #[test]
    fn test_format_peer_info() {
        let peer_id = PeerId::random();
        let handshake = HandshakeMessage {
            network_id: "test-net".to_string(),
            chain_id: 1,
            genesis_hash: "test-hash".to_string(),
            protocol_version: "1.0.0".to_string(),
            supported_protocols: vec!["sync".to_string(), "blocks".to_string()],
            node_version: "1.0.0".to_string(),
        };

        let info = format_peer_info(&peer_id, &Some(handshake));
        assert!(info.contains(&peer_id.to_string()));
        assert!(info.contains("test-net"));
        assert!(info.contains("1.0.0"));
    }

    #[test]
    fn test_format_peer_info_no_handshake() {
        let peer_id = PeerId::random();
        let info = format_peer_info(&peer_id, &None);
        assert!(info.contains(&peer_id.to_string()));
        assert!(info.contains("Handshake pending"));
    }

    #[test]
    fn test_certificate_expiry_in_discovery() {
        use crate::config::{BootnodeCertificate, NetworkAuthority};
        use chrono::Utc;

        // Create expired certificate
        let (authority, signing_key) = NetworkAuthority::new("vc-mainnet-authority".to_string());

        let mut cert = BootnodeCertificate::new(
            "node-123".to_string(),
            "vc-mainnet".to_string(),
            1,
            "http://bootnode.example.com:8080".to_string(),
            "/ip4/127.0.0.1/tcp/9090".to_string(),
            Utc::now().timestamp() - 86400, // Expired 1 day ago
        );
        cert.sign(&signing_key).expect("Failed to sign certificate");

        // Expired certificate should fail verification
        let result = cert.verify(&[authority]);
        assert!(
            result.is_err(),
            "Expired certificate should fail verification"
        );
    }
}
