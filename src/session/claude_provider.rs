//! Claude Code session provider implementation

use anyhow::{Context, Result};
use async_trait::async_trait;
use uuid::Uuid;

use super::claude::ClaudeClient;
use super::provider::{SessionHandle, SessionProvider, SessionStatus};

pub struct ClaudeProvider {
    client: ClaudeClient,
}

impl ClaudeProvider {
    pub fn new(client: ClaudeClient) -> Self {
        Self { client }
    }

    pub fn with_defaults() -> Self {
        Self::new(ClaudeClient::default())
    }
}

#[async_trait]
impl SessionProvider for ClaudeProvider {
    async fn create_session(&self, system_prompt: Option<String>) -> Result<SessionHandle> {
        let internal_id = Uuid::new_v4().to_string();
        
        let response = self.client
            .create_session(system_prompt, None)
            .await
            .context("Failed to create Claude Code session")?;

        Ok(SessionHandle {
            internal_id,
            provider_id: response.session_id,
        })
    }

    async fn send_message(&self, session_id: &str, message: &str) -> Result<String> {
        // Send message and get actual response from Claude Code
        let response = self.client
            .send_message(session_id, message)
            .await
            .context("Failed to send message to Claude Code session")?;

        // Return the actual response from Claude
        Ok(response)
    }

    async fn get_status(&self, session_id: &str) -> Result<SessionStatus> {
        let running = self.client.is_running(session_id).await;
        
        if running {
            Ok(SessionStatus::Running)
        } else {
            // Check if session exists
            let session = self.client.get_session(session_id).await?;
            match session {
                Some(_) => Ok(SessionStatus::Completed),
                None => Ok(SessionStatus::Terminated),
            }
        }
    }

    async fn fork_session(&self, _session_id: &str) -> Result<SessionHandle> {
        // Claude Code doesn't support forking in the same way
        // Create a new session instead
        let internal_id = Uuid::new_v4().to_string();
        
        let response = self.client
            .create_session(None, None)
            .await
            .context("Failed to create new Claude Code session")?;

        Ok(SessionHandle {
            internal_id,
            provider_id: response.session_id,
        })
    }

    async fn kill_session(&self, session_id: &str) -> Result<()> {
        self.client
            .kill_session(session_id)
            .map_err(|e| anyhow::anyhow!("Failed to kill Claude Code session: {}", e))
    }

    async fn health_check(&self) -> Result<bool> {
        self.client.health_check().map_err(|e| anyhow::anyhow!(e))
    }
}
