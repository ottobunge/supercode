//! Peer server for handling incoming peer connections

use std::sync::Arc;

use anyhow::Result;
use chrono::Utc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;
use tracing::{error, info, warn};

use crate::config::{Config, PeerHandshake, PeerHandshakeResponse, PeerManager, PeerRequest};

/// Peer server that handles incoming peer connections
pub struct PeerServer {
    port: u16,
    config: Arc<RwLock<Config>>,
    peer_manager: Arc<RwLock<Option<PeerManager>>>,
}

impl PeerServer {
    pub fn new(port: u16, config: Arc<RwLock<Config>>) -> Self {
        Self {
            port,
            config,
            peer_manager: Arc::new(RwLock::new(None)),
        }
    }

    /// Start the peer server
    pub async fn start(&self) -> Result<()> {
        let addr = format!("0.0.0.0:{}", self.port);
        let listener = TcpListener::bind(&addr).await?;
        
        info!("Peer server listening on {}", addr);

        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    let config = self.config.clone();
                    let peer_manager = self.peer_manager.clone();
                    
                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_peer_connection(stream, addr, config, peer_manager).await {
                            error!("Error handling peer connection from {}: {}", addr, e);
                        }
                    });
                }
                Err(e) => {
                    warn!("Error accepting peer connection: {}", e);
                }
            }
        }
    }

    /// Handle an incoming peer connection
    async fn handle_peer_connection(
        stream: TcpStream,
        addr: std::net::SocketAddr,
        config: Arc<RwLock<Config>>,
        peer_manager: Arc<RwLock<Option<PeerManager>>>,
    ) -> Result<()> {
        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::new(reader);

        // Read handshake
        let mut line = String::new();
        reader.read_line(&mut line).await?;

        let handshake: PeerHandshake = serde_json::from_str(&line.trim())?;
        info!("Received peer handshake from {} ({})", handshake.name, addr.ip());

        // Check if we have a config with keys
        let config_guard = config.read().await;
        
        // Check auth
        let peer_config = config_guard.get_peer(&handshake.name);
        let auth_valid = peer_config.map(|p| p.auth == handshake.auth).unwrap_or(false);
        
        // Also accept if it's a new request (no existing peer)
        let is_new_request = peer_config.is_none();

        let response = if !config_guard.can_peer() {
            // We don't have keys, can't peer
            PeerHandshakeResponse {
                accepted: false,
                message: "This node does not have keys configured. Run 'supercode keygen' first.".to_string(),
                public_key: String::new(),
            }
        } else if is_new_request {
            // New peer request - add to pending
            info!("Adding pending peer request from {}", handshake.name);
            
            let request = PeerRequest {
                name: handshake.name.clone(),
                public_key: handshake.public_key.clone(),
                from_addr: addr.ip().to_string(),
                received_at: Utc::now(),
            };
            
            // Store pending request
            drop(config_guard);
            let mut config_guard = config.write().await;
            config_guard.add_pending_request(request);
            drop(config_guard);

            PeerHandshakeResponse {
                accepted: false,
                message: "Pending approval. Use 'supercode peer accept' to approve.".to_string(),
                public_key: config.read().await.public_key.clone(),
            }
        } else if auth_valid {
            // Existing peer with valid auth
            info!("Accepted peer connection from {}", handshake.name);
            
            PeerHandshakeResponse {
                accepted: true,
                message: "Connected successfully".to_string(),
                public_key: config_guard.public_key.clone(),
            }
        } else {
            // Auth failed
            warn!("Peer connection rejected: invalid auth from {}", handshake.name);
            PeerHandshakeResponse {
                accepted: false,
                message: "Invalid auth credentials".to_string(),
                public_key: String::new(),
            }
        };

        // Send response
        let response_json = serde_json::to_string(&response)?; 
        writer.write_all(response_json.as_bytes()).await?;
        writer.write_all(b"\n").await?;
        writer.flush().await?;

        if response.accepted {
            info!("Successfully established peer connection with {}", handshake.name);
        }

        Ok(())
    }

    /// Update the peer manager
    pub async fn set_peer_manager(&self, manager: PeerManager) {
        let mut pm = self.peer_manager.write().await;
        *pm = Some(manager);
    }
}
