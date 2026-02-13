//! OpenCode session monitor

use std::sync::Arc;

use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use tracing::{debug, warn};

use super::state::{AgentState, ApprovalType, SessionActivity};
use super::monitor_trait::SessionMonitor;
use crate::session::opencode::client::OpenCodeClient;

/// Monitor for OpenCode sessions
pub struct OpenCodeMonitor {
    client: Client,
    base_url: String,
    /// Cache of known sessions and their last known state
    #[allow(dead_code)]
    session_cache: std::sync::Mutex<std::collections::HashMap<String, SessionActivity>>,
}

impl OpenCodeMonitor {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .build()
                .expect("Failed to create HTTP client"),
            base_url: base_url.into(),
            session_cache: std::sync::Mutex::new(std::collections::HashMap::new()),
        }
    }

    /// Query OpenCode API for session status
    async fn query_session(&self, session_id: &str) -> Result<OpenCodeSessionInfo> {
        let url = format!("{}/session/{}", self.base_url, session_id);
        
        let response = self.client
            .get(&url)
            .send()
            .await
            .context("Failed to query OpenCode session")?;

        if response.status() == 404 {
            return Err(anyhow::anyhow!("Session not found: {}", session_id));
        }

        let info: OpenCodeSessionInfo = response
            .json()
            .await
            .context("Failed to parse OpenCode session response")?;

        Ok(info)
    }
}

#[async_trait]
impl SessionMonitor for OpenCodeMonitor {
    async fn get_state(&self, session_id: &str) -> Result<AgentState> {
        let info = self.query_session(session_id).await?;

        // OpenCode doesn't give us explicit "waiting for approval" state
        // We infer from status and other indicators
        let state = match info.status.as_deref() {
            Some("running") | Some("active") => {
                // Check if there's any pending tool call or user prompt
                if info.pending_tool_calls.unwrap_or(0) > 0 {
                    AgentState::WaitingForApproval {
                        approval_type: ApprovalType::ToolUse,
                        description: "Pending tool execution".to_string(),
                    }
                } else if info.awaiting_user_input.unwrap_or(false) {
                    AgentState::WaitingForInput
                } else {
                    AgentState::Processing
                }
            }
            Some("completed") | Some("done") => AgentState::Completed,
            Some("failed") | Some("error") => AgentState::Failed {
                error: info.error.clone().unwrap_or_else(|| "Unknown error".to_string()),
            },
            Some("terminated") | Some("cancelled") => AgentState::Terminated,
            _ => AgentState::Unknown,
        };

        Ok(state)
    }

    async fn get_activity(&self, session_id: &str) -> Result<SessionActivity> {
        let info = self.query_session(session_id).await?;

        let state = self.get_state(session_id).await?;

        let mut activity = SessionActivity::new(
            session_id.to_string(),
            session_id.to_string(),
        ).with_state(state);

        // Add last message if available
        if let Some(msg) = info.last_message {
            activity = activity.with_message(msg);
        }

        // Add metadata
        if let Some(id) = info.id {
            activity.metadata.insert("opencode_id".to_string(), id);
        }
        if let Some(slug) = info.slug {
            activity.metadata.insert("slug".to_string(), slug);
        }

        Ok(activity)
    }

    async fn is_waiting_for_approval(&self, session_id: &str) -> Result<bool> {
        let state = self.get_state(session_id).await?;
        
        Ok(matches!(
            state,
            AgentState::WaitingForApproval { .. }
        ))
    }

    async fn get_last_message(&self, session_id: &str) -> Result<Option<String>> {
        let info = self.query_session(session_id).await?;
        Ok(info.last_message)
    }

    async fn get_last_response(&self, session_id: &str) -> Result<Option<String>> {
        let info = self.query_session(session_id).await?;
        Ok(info.last_response)
    }

    async fn get_session_output(&self, session_id: &str) -> Result<Option<String>> {
        let info = self.query_session(session_id).await?;
        // OpenCode may provide output/logs in different fields
        Ok(info.conversation_history.or(info.output))
    }

    async fn is_alive(&self, session_id: &str) -> Result<bool> {
        let info = self.query_session(session_id).await?;
        
        match info.status.as_deref() {
            Some("running") | Some("active") => Ok(true),
            Some("completed") | Some("done") | Some("failed") | Some("error") 
            | Some("terminated") | Some("cancelled") => Ok(false),
            _ => Ok(false),
        }
    }
}

/// OpenCode session information from their API
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OpenCodeSessionInfo {
    id: Option<String>,
    slug: Option<String>,
    status: Option<String>,
    last_message: Option<String>,
    last_response: Option<String>,
    pending_tool_calls: Option<usize>,
    awaiting_user_input: Option<bool>,
    error: Option<String>,
    conversation_history: Option<String>,
    output: Option<String>,
}
