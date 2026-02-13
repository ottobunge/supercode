//! Supercode configuration management
//! Handles loading, saving, and encrypting the config file

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

/// Supercode configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// This node's identity name (required for peering)
    #[serde(default)]
    pub name: String,

    /// Private key (base64 encoded) - REQUIRED for peering
    #[serde(default)]
    pub private_key: String,

    /// Public key (base64 encoded) - derived from private_key
    #[serde(default)]
    pub public_key: String,

    /// Database path
    #[serde(default = "default_db_path")]
    pub database_path: String,

    /// Server settings
    #[serde(default)]
    pub server: ServerConfig,

    /// Known peers
    #[serde(default)]
    pub peers: HashMap<String, PeerConfig>,

    /// Pending peer requests (runtime only, not serialized)
    #[serde(skip)]
    pub pending_requests: HashMap<String, PeerRequest>,
}

fn default_db_path() -> String {
    "~/.supercode/supercode.db".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            name: String::new(),
            private_key: String::new(),
            public_key: String::new(),
            database_path: default_db_path(),
            server: ServerConfig::default(),
            peers: HashMap::new(),
            pending_requests: HashMap::new(),
        }
    }
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    9091
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
        }
    }
}

/// Peer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerConfig {
    /// Auth info for connecting (can be password or token)
    pub auth: String,

    /// Hostnames/IPs to connect to
    pub hostnames: Vec<String>,

    /// Remote's public key
    #[serde(default)]
    pub public_key: String,

    /// Is this peer verified/trusted
    #[serde(default)]
    pub verified: bool,
}

/// A pending peer request (runtime only)
#[derive(Debug, Clone)]
pub struct PeerRequest {
    /// Requesting node's name
    pub name: String,
    /// Requesting node's public key
    pub public_key: String,
    /// IP address of requester
    pub from_addr: String,
    /// When the request was received
    pub received_at: chrono::DateTime<chrono::Utc>,
}

impl Config {
    /// Load config from the default location or specified path
    pub fn load(path: Option<&str>) -> Result<Self> {
        let config_path = Self::config_path(path)?;

        if !config_path.exists() {
            info!(
                "Config file not found, creating default at {:?}",
                config_path
            );
            let config = Config::default();
            config.save(path)?;
            return Ok(config);
        }

        let raw = fs::read_to_string(&config_path).context("Failed to read config file")?;

        let config: Config = serde_yaml::from_str(&raw).context("Failed to parse config file")?;

        debug!("Loaded config from {:?}", config_path);
        Ok(config)
    }

    /// Save config to the default location
    pub fn save(&self, path: Option<&str>) -> Result<()> {
        let config_path = Self::config_path(path)?;

        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_yaml::to_string(&self)?;
        fs::write(&config_path, content).context("Failed to write config file")?;

        info!("Saved config to {:?}", config_path);
        Ok(())
    }

    /// Get the config file path
    fn config_path(path: Option<&str>) -> Result<PathBuf> {
        // Check env override first
        if let Ok(env_path) = std::env::var("SUPERCODE_CONFIG") {
            return Ok(PathBuf::from(env_path));
        }

        if let Some(p) = path {
            return Ok(PathBuf::from(p));
        }

        let home = dirs::home_dir().context("Cannot find home directory")?;
        Ok(home.join(".supercode").join("config.yml"))
    }

    /// Check if this node is ready for peering
    pub fn can_peer(&self) -> bool {
        !self.name.is_empty() && !self.private_key.is_empty() && !self.public_key.is_empty()
    }

    /// Add a new peer
    pub fn add_peer(&mut self, name: &str, config: PeerConfig) {
        self.peers.insert(name.to_string(), config);
    }

    /// Remove a peer
    pub fn remove_peer(&mut self, name: &str) -> Option<PeerConfig> {
        self.peers.remove(name)
    }

    /// Get a peer by name
    pub fn get_peer(&self, name: &str) -> Option<&PeerConfig> {
        self.peers.get(name)
    }

    /// Add a pending peer request
    pub fn add_pending_request(&mut self, request: PeerRequest) {
        self.pending_requests.insert(request.name.clone(), request);
    }

    /// Get pending requests
    pub fn get_pending_requests(&self) -> Vec<&PeerRequest> {
        self.pending_requests.values().collect()
    }

    /// Clear a pending request
    pub fn clear_pending_request(&mut self, name: &str) {
        self.pending_requests.remove(name);
    }

    /// Deny a pending peer request
    pub fn deny_peer(&mut self, name: &str) {
        self.clear_pending_request(name);
    }

    /// Resolve database path (expand ~)
    pub fn resolve_db_path(&self) -> Result<PathBuf> {
        let home = dirs::home_dir().context("Cannot find home directory")?;
        let path = self.database_path.replace("~", &home.to_string_lossy());
        Ok(PathBuf::from(path))
    }
}

// Helper for debug logging
fn debug(_msg: &str) {
    // Debug logging - can be enabled later
}
