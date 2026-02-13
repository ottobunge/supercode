//! OpenCode HTTP API client

use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// OpenCode API client
pub struct OpenCodeClient {
    client: Client,
    base_url: String,
    /// Cache of active sessions: opencode_session_id -> supercode_session_id
    sessions: Arc<RwLock<std::collections::HashMap<String, String>>>,
}

#[derive(Debug, Serialize)]
struct CreateSessionRequest {
    #[serde(rename = "systemPrompt")]
    system_prompt: String,
    #[serde(rename = "resumeId")]
    resume_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateSessionResponse {
    pub id: String,
    pub slug: Option<String>,
    // OpenCode returns "id" as the session identifier
}

#[derive(Debug, Serialize)]
struct SendMessageRequest {
    parts: Vec<MessagePart>,
    #[serde(rename = "resumeId")]
    resume_id: Option<String>,
}

#[derive(Debug, Serialize)]
struct MessagePart {
    #[serde(rename = "type")]
    part_type: String,
    text: String,
}

#[derive(Debug, Deserialize)]
pub struct SendMessageResponse {
    // OpenCode returns the response content
    // The exact structure depends on the API
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct SessionInfo {
    pub id: String,
    #[serde(rename = "sessionId")]
    pub session_id: String,
    pub status: Option<String>,
    #[serde(rename = "children")]
    pub children: Option<Vec<SessionInfo>>,
}

impl OpenCodeClient {
    /// Create a new OpenCode client with timeouts
    pub fn new(base_url: impl Into<String>) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .connect_timeout(std::time::Duration::from_secs(5))
            .build()
            .unwrap_or_else(|_| Client::new()); // Fallback if config fails

        Self {
            client,
            base_url: base_url.into(),
            sessions: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Create a new session
    pub async fn create_session(
        &self,
        system_prompt: Option<String>,
        resume_id: Option<String>,
    ) -> Result<CreateSessionResponse> {
        let url = format!("{}/session", self.base_url);
        
        let request = CreateSessionRequest {
            system_prompt: system_prompt.unwrap_or_else(|| "You are a helpful coding assistant.".to_string()),
            resume_id,
        };

        debug!("Creating OpenCode session: {:?}", request);
        
        let response = match self.client
            .post(&url)
            .json(&request)
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                error!("OpenCode HTTP error: {}", e);
                return Err(e).context("Failed to connect to OpenCode server");
            }
        };

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("OpenCode API error: {} - {}", status, body);
        }

        let result: CreateSessionResponse = response
            .json()
            .await
            .context("Failed to parse OpenCode response")?;

        info!("Created OpenCode session: {}", result.id);
        
        // Cache the session mapping
        self.sessions.write().await.insert(
            result.id.clone(),
            result.id.clone(),
        );

        Ok(result)
    }

    /// Send a message to a session
    pub async fn send_message(
        &self,
        session_id: &str,
        message: impl Into<String>,
    ) -> Result<SendMessageResponse> {
        let url = format!("{}/session/{}/message", self.base_url, session_id);
        
        let request = SendMessageRequest {
            parts: vec![MessagePart {
                part_type: "text".to_string(),
                text: message.into(),
            }],
            resume_id: None,
        };

        debug!("Sending message to OpenCode session: {}", session_id);
        
        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to send message to OpenCode session")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("OpenCode API error: {} - {}", status, body);
        }

        let result: SendMessageResponse = response
            .json()
            .await
            .context("Failed to parse OpenCode response")?;

        Ok(result)
    }

    /// Get session info
    pub async fn get_session(&self, session_id: &str) -> Result<SessionInfo> {
        let url = format!("{}/session/{}", self.base_url, session_id);
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .context("Failed to get OpenCode session")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("OpenCode API error: {} - {}", status, body);
        }

        let result: SessionInfo = response
            .json()
            .await
            .context("Failed to parse OpenCode response")?;

        Ok(result)
    }

    /// List child sessions (for forks)
    pub async fn get_children(&self, session_id: &str) -> Result<Vec<SessionInfo>> {
        let url = format!("{}/session/{}/children", self.base_url, session_id);
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .context("Failed to get OpenCode children")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("OpenCode API error: {} - {}", status, body);
        }

        let result: Vec<SessionInfo> = response
            .json()
            .await
            .context("Failed to parse OpenCode response")?;

        Ok(result)
    }

    /// Fork a session
    pub async fn fork_session(&self, session_id: &str) -> Result<CreateSessionResponse> {
        let url = format!("{}/session/{}/fork", self.base_url, session_id);
        
        let response = self.client
            .post(&url)
            .send()
            .await
            .context("Failed to fork OpenCode session")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("OpenCode API error: {} - {}", status, body);
        }

        let result: CreateSessionResponse = response
            .json()
            .await
            .context("Failed to parse OpenCode response")?;

        info!("Forked OpenCode session: {} -> {}", session_id, result.id);
        
        Ok(result)
    }

    /// Kill/terminate a session
    pub async fn kill_session(&self, session_id: &str) -> Result<()> {
        let url = format!("{}/session/{}", self.base_url, session_id);
        
        let response = self.client
            .delete(&url)
            .send()
            .await
            .context("Failed to kill OpenCode session")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("OpenCode API error: {} - {}", status, body);
        }

        info!("Killed OpenCode session: {}", session_id);
        
        Ok(())
    }

    /// Check if OpenCode server is running
    pub async fn health_check(&self) -> Result<bool> {
        let url = format!("{}/health", self.base_url);
        
        match self.client.get(&url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(e) => {
                warn!("OpenCode health check failed: {}", e);
                Ok(false)
            }
        }
    }

    /// Get the base URL
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}

impl Default for OpenCodeClient {
    fn default() -> Self {
        Self::new("http://localhost:9090")
    }
}
