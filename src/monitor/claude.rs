//! Claude session monitor

use std::path::PathBuf;
use std::process::Command;

use anyhow::{Context, Result};
use async_trait::async_trait;
use tracing::{debug, warn};

use super::state::{AgentState, SessionActivity};
use super::monitor_trait::SessionMonitor;

/// Monitor for Claude Code sessions
/// 
/// Claude doesn't have an HTTP API like OpenCode, so we monitor
/// via process status and output parsing.
pub struct ClaudeMonitor {
    /// Where Claude sessions are stored
    sessions_dir: PathBuf,
}

impl ClaudeMonitor {
    pub fn new(sessions_dir: impl Into<PathBuf>) -> Self {
        Self {
            sessions_dir: sessions_dir.into(),
        }
    }

    /// Check if a Claude process is running for this session
    fn check_process(&self, session_id: &str) -> bool {
        // Use pgrep to find Claude processes for this session
        let output = Command::new("pgrep")
            .args(["-f", &format!("claude.*session.*{}", session_id)])
            .output();

        match output {
            Ok(o) => o.status.success(),
            Err(_) => false,
        }
    }

    /// Try to get session info from Claude's session directory
    fn get_session_info(&self, session_id: &str) -> Option<ClaudeSessionInfo> {
        let session_path = self.sessions_dir.join(session_id);
        
        if !session_path.exists() {
            return None;
        }

        // Read session state file if it exists
        let state_file = session_path.join("state.json");
        if state_file.exists() {
            if let Ok(content) = std::fs::read_to_string(&state_file) {
                if let Ok(info) = serde_json::from_str(&content) {
                    return Some(info);
                }
            }
        }

        None
    }
}

#[derive(Debug, serde::Deserialize)]
struct ClaudeSessionInfo {
    status: Option<String>,
    last_message: Option<String>,
    last_response: Option<String>,
}

#[async_trait]
impl SessionMonitor for ClaudeMonitor {
    async fn get_state(&self, session_id: &str) -> Result<AgentState> {
        // Check if process is running
        let is_running = self.check_process(session_id);
        
        if !is_running {
            // Try to get final status from session files
            if let Some(info) = self.get_session_info(session_id) {
                let state = match info.status.as_deref() {
                    Some("completed") => AgentState::Completed,
                    Some("failed") | Some("error") => AgentState::Failed {
                        error: "Session failed".to_string(),
                    },
                    _ => AgentState::Terminated,
                };
                return Ok(state);
            }
            
            return Ok(AgentState::Terminated);
        }

        // Process is running - we can't easily tell if it's waiting for approval
        // Claude doesn't expose this state externally
        // For now, assume it's processing
        Ok(AgentState::Processing)
    }

    async fn get_activity(&self, session_id: &str) -> Result<SessionActivity> {
        let state = self.get_state(session_id).await?;
        
        let mut activity = SessionActivity::new(
            session_id.to_string(),
            session_id.to_string(),
        ).with_state(state);

        // Try to get last message
        if let Some(info) = self.get_session_info(session_id) {
            if let Some(msg) = info.last_message {
                activity = activity.with_message(msg);
            }
            if let Some(resp) = info.last_response {
                activity = activity.with_response(resp);
            }
        }

        Ok(activity)
    }

    async fn is_waiting_for_approval(&self, session_id: &str) -> Result<bool> {
        // Claude doesn't expose this directly
        // For now, we assume it's not waiting (would need process inspection)
        let state = self.get_state(session_id).await?;
        
        Ok(matches!(state, AgentState::WaitingForApproval { .. }))
    }

    async fn get_last_message(&self, session_id: &str) -> Result<Option<String>> {
        if let Some(info) = self.get_session_info(session_id) {
            return Ok(info.last_message);
        }
        Ok(None)
    }

    async fn get_last_response(&self, session_id: &str) -> Result<Option<String>> {
        if let Some(info) = self.get_session_info(session_id) {
            return Ok(info.last_response);
        }
        Ok(None)
    }

    async fn get_session_output(&self, session_id: &str) -> Result<Option<String>> {
        // Try to read Claude's session logs
        let log_path = self.sessions_dir.join(session_id).join("logs");
        
        if log_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&log_path) {
                return Ok(Some(content));
            }
        }
        
        Ok(None)
    }

    async fn is_alive(&self, session_id: &str) -> Result<bool> {
        Ok(self.check_process(session_id))
    }
}
