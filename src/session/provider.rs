//! Session provider trait

use anyhow::Result;
use async_trait::async_trait;

/// Session provider trait for different agent backends
#[async_trait]
pub trait SessionProvider: Send + Sync {
    /// Create a new session
    async fn create_session(&self, system_prompt: Option<String>) -> Result<SessionHandle>;

    /// Send a message to a session
    async fn send_message(&self, session_id: &str, message: &str) -> Result<String>;

    /// Get session status
    async fn get_status(&self, session_id: &str) -> Result<SessionStatus>;

    /// Fork a session
    async fn fork_session(&self, session_id: &str) -> Result<SessionHandle>;

    /// Kill/terminate a session
    async fn kill_session(&self, session_id: &str) -> Result<()>;

    /// Check if the provider is healthy
    async fn health_check(&self) -> Result<bool>;
}

/// Handle to a created session
#[derive(Debug, Clone)]
pub struct SessionHandle {
    /// Our internal session ID
    pub internal_id: String,
    /// The provider's session ID
    pub provider_id: String,
}

/// Session status
#[derive(Debug, Clone)]
pub enum SessionStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Terminated,
}
