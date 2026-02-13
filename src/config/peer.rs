//! Peer management for remote Supercode instances

use std::collections::HashMap;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::net::tcp::OwnedReadHalf;
use tracing::{info, warn};

use super::{Config, PeerConfig, PeerRequest};

/// Peer manager for handling peer connections
pub struct PeerManager {
    config: Config,
}

impl PeerManager {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn config_mut(&mut self) -> &mut Config {
        &mut self.config
    }

    pub fn can_peer(&self) -> bool {
        self.config.can_peer()
    }

    /// Initiate peer connection to another instance
    pub async fn connect_to_peer(&self, peer_name: &str) -> Result<PeerConnection> {
        let peer = self.config.get_peer(peer_name)
            .ok_or_else(|| anyhow::anyhow!("Peer not found: {}", peer_name))?;

        for hostname in &peer.hostnames {
            match self.try_connect(peer_name, hostname, &peer.auth).await {
                Ok(conn) => return Ok(conn),
                Err(e) => {
                    warn!("Failed to connect to {} at {}: {}", peer_name, hostname, e);
                }
            }
        }

        anyhow::bail!("Could not connect to peer {} on any hostname", peer_name)
    }

    async fn try_connect(&self, peer_name: &str, hostname: &str, auth: &str) -> Result<PeerConnection> {
        let addr = format!("{}:9092", hostname);
        info!("Attempting to connect to peer at {}", addr);
        
        let stream = TcpStream::connect(&addr).await?;
        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::new(reader);

        let handshake = PeerHandshake {
            version: 1,
            name: self.config.name.clone(),
            public_key: self.config.public_key.clone(),
            auth: auth.to_string(),
        };

        let handshake_json = serde_json::to_string(&handshake)?;
        writer.write_all(handshake_json.as_bytes()).await?;
        writer.write_all(b"\n").await?;
        writer.flush().await?;

        let mut line = String::new();
        reader.read_line(&mut line).await?;

        let response: PeerHandshakeResponse = serde_json::from_str(&line)?;

        if !response.accepted {
            anyhow::bail!("Peer {} rejected connection: {}", peer_name, response.message);
        }

        info!("Successfully connected to peer {}", peer_name);

        Ok(PeerConnection {
            name: peer_name.to_string(),
            stream: reader,
            remote_public_key: response.public_key,
        })
    }

    pub fn handle_peer_request(&mut self, request: PeerRequest) {
        info!("Received peer request from {}", request.name);
        self.config.add_pending_request(request);
    }

    pub async fn accept_peer(&mut self, peer_name: &str) -> Result<()> {
        let request = self.config.pending_requests.get(peer_name)
            .ok_or_else(|| anyhow::anyhow!("No pending request from: {}", peer_name))?
            .clone();

        let peer_config = PeerConfig {
            auth: String::new(),
            hostnames: vec![request.from_addr.clone()],
            public_key: request.public_key.clone(),
            verified: true,
        };

        self.config.add_peer(peer_name, peer_config);
        self.config.clear_pending_request(peer_name);

        info!("Accepted peer request from {}", peer_name);
        Ok(())
    }

    pub fn deny_peer(&mut self, peer_name: &str) {
        self.config.clear_pending_request(peer_name);
        info!("Denied peer request from {}", peer_name);
    }

    pub fn get_pending_requests(&self) -> Vec<(&String, &PeerRequest)> {
        self.config.pending_requests.iter().collect()
    }
}

/// Peer connection handle
pub struct PeerConnection {
    pub name: String,
    pub stream: BufReader<OwnedReadHalf>,
    pub remote_public_key: String,
}

/// Peer handshake message
#[derive(Debug, Serialize, Deserialize)]
pub struct PeerHandshake {
    pub version: u32,
    pub name: String,
    pub public_key: String,
    pub auth: String,
}

/// Peer handshake response
#[derive(Debug, Serialize, Deserialize)]
pub struct PeerHandshakeResponse {
    pub accepted: bool,
    pub message: String,
    pub public_key: String,
}

/// Message from/to peer
#[derive(Debug, Serialize, Deserialize)]
pub struct PeerMessage {
    pub id: String,
    pub message_type: String,
    pub payload: String,
    pub from: String,
    pub timestamp: DateTime<Utc>,
    pub signature: Option<String>,
}
