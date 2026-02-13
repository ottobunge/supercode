//! OpenCode session provider implementation

use anyhow::{Context, Result};
use async_trait::async_trait;
use uuid::Uuid;

use super::opencode::OpenCodeClient;
use super::provider::{SessionHandle, SessionProvider, SessionStatus};

pub struct OpenCodeProvider {
    client: OpenCodeClient,
}

impl OpenCodeProvider {
    pub fn new(client: OpenCodeClient) -> Self {
        Self { client }
    }

    pub fn with_url(url: impl Into<String>) -> Self {
        Self::new(OpenCodeClient::new(url))
    }
}

#[async_trait]
impl SessionProvider for OpenCodeProvider {
    async fn create_session(&self, system_prompt: Option<String>) -> Result<SessionHandle> {
        let internal_id = Uuid::new_v4().to_string();
        
        let response = self.client
            .create_session(system_prompt, None)
            .await
            .context("Failed to create OpenCode session")?;

        Ok(SessionHandle {
            internal_id,
            provider_id: response.id,
        })
    }

    async fn send_message(&self, session_id: &str, message: &str) -> Result<String> {
        let response = self.client
            .send_message(session_id, message)
            .await
            .context("Failed to send message to OpenCode session")?;

        // The response structure depends on OpenCode API
        // For now, return the JSON as string
        Ok(response.extra.to_string())
    }

    async fn get_status(&self, session_id: &str) -> Result<SessionStatus> {
        let info = self.client
            .get_session(session_id)
            .await
            .context("Failed to get OpenCode session status")?;

        // Parse status from response
        let status = match info.status.as_deref() {
            Some("running") | Some("active") => SessionStatus::Running,
            Some("completed") | Some("done") => SessionStatus::Completed,
            Some("failed") | Some("error") => SessionStatus::Failed,
            Some("terminated") | Some("cancelled") => SessionStatus::Terminated,
            _ => SessionStatus::Pending,
        };

        Ok(status)
    }

    async fn fork_session(&self, session_id: &str) -> Result<SessionHandle> {
        let internal_id = Uuid::new_v4().to_string();
        
        let response = self.client
            .fork_session(session_id)
            .await
            .context("Failed to fork OpenCode session")?;

        Ok(SessionHandle {
            internal_id,
            provider_id: response.id,
        })
    }

    async fn kill_session(&self, session_id: &str) -> Result<()> {
        self.client
            .kill_session(session_id)
            .await
            .context("Failed to kill OpenCode session")
    }

    async fn health_check(&self) -> Result<bool> {
        self.client.health_check().await
    }
}
