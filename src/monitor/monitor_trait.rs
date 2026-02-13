//! Session monitoring trait

use anyhow::Result;
use async_trait::async_trait;

use super::state::{AgentState, SessionActivity};

/// Trait for monitoring sub-agent sessions
/// 
/// This allows the manager to query the state of sub-agents
/// running on different providers (OpenCode, Claude, etc.)
#[async_trait]
pub trait SessionMonitor: Send + Sync {
    /// Get the current state of a session
    async fn get_state(&self, session_id: &str) -> Result<AgentState>;

    /// Get detailed activity information about a session
    async fn get_activity(&self, session_id: &str) -> Result<SessionActivity>;

    /// Check if the session is waiting for user approval/permission
    async fn is_waiting_for_approval(&self, session_id: &str) -> Result<bool>;

    /// Get the last message sent to the agent
    async fn get_last_message(&self, session_id: &str) -> Result<Option<String>>;

    /// Get the last response from the agent
    async fn get_last_response(&self, session_id: &str) -> Result<Option<String>>;

    /// Get session output/logs (if available)
    async fn get_session_output(&self, session_id: &str) -> Result<Option<String>>;

    /// Check if session is still alive/running
    async fn is_alive(&self, session_id: &str) -> Result<bool>;
}

/// Errors that can occur during monitoring
#[derive(Debug, thiserror::Error)]
pub enum MonitorError {
    #[error("Session not found: {0}")]
    SessionNotFound(String),

    #[error("Provider error: {0}")]
    ProviderError(String),

    #[error("Not implemented for this provider")]
    NotImplemented,

    #[error("Connection error: {0}")]
    ConnectionError(String),
}
