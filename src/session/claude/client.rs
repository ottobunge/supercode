//! Claude Code CLI client

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::process::{Child, Command, Stdio};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Claude Code CLI client
pub struct ClaudeClient {
    /// Path to claude CLI binary
    claude_path: String,
    /// Working directory for sessions
    work_dir: PathBuf,
    /// Active processes: session_id -> child process
    processes: Arc<Mutex<HashMap<String, Child>>>,
    /// Session state: session_id -> session metadata
    sessions: Arc<RwLock<HashMap<String, ClaudeSession>>>,
}

#[derive(Debug, Clone)]
pub struct ClaudeSession {
    pub id: String,
    pub session_id: String,
    pub working_dir: PathBuf,
}

impl ClaudeClient {
    /// Create a new Claude client
    pub fn new(claude_path: impl Into<String>, work_dir: impl Into<PathBuf>) -> Self {
        Self {
            claude_path: claude_path.into(),
            work_dir: work_dir.into(),
            processes: Arc::new(Mutex::new(HashMap::new())),
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new session - starts an interactive Claude session
    pub async fn create_session(
        &self,
        system_prompt: Option<String>,
        resume_id: Option<String>,
    ) -> Result<ClaudeSessionResponse> {
        let session_id = uuid::Uuid::new_v4().to_string();
        let work_dir = self.work_dir.join(&session_id);
        
        // Create working directory
        std::fs::create_dir_all(&work_dir)
            .context("Failed to create session working directory")?;

        // Build the command
        let mut cmd = Command::new(&self.claude_path);
        cmd.arg("-p"); // Print mode (non-interactive)
        cmd.arg("--output-format");
        cmd.arg("json");
        
        if let Some(resume) = resume_id {
            cmd.arg("--resume");
            cmd.arg(&resume);
        }
        
        // Set working directory
        cmd.current_dir(&work_dir);
        
        // Set up pipes for communication
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        debug!("Starting Claude Code session: {:?}", cmd);

        let mut child = cmd.spawn()
            .context("Failed to start Claude Code process")?;

        // Get pipes
        let stdin = child.stdin.take()
            .context("Failed to get stdin")?;
        
        // Store the process immediately to avoid leak
        self.processes.lock().unwrap().insert(session_id.clone(), child);

        // Send system prompt first if provided
        if let Some(prompt) = system_prompt {
            use std::io::Write;
            let mut stdin = stdin;
            let init_message = format!("{}\n\n", prompt);
            stdin.write_all(init_message.as_bytes())?;
            stdin.flush()?;
            // Keep stdin alive by not dropping
            drop(stdin);
        }

        // Store session info
        let session = ClaudeSession {
            id: session_id.clone(),
            session_id: session_id.clone(),
            working_dir: work_dir.clone(),
        };
        self.sessions.write().await.insert(session_id.clone(), session);

        info!("Created Claude Code session: {}", session_id);

        Ok(ClaudeSessionResponse {
            id: session_id.clone(),
            session_id,
            working_dir: work_dir.to_string_lossy().to_string(),
        })
    }

    /// Send a message to a session - uses a new process for each message
    /// This is simpler than maintaining a persistent connection
    pub async fn send_message(&self, session_id: &str, message: &str) -> Result<String> {
        // Check if session exists
        let session = {
            let sessions = self.sessions.read().await;
            sessions.get(session_id).cloned()
        };

        let session = session.context("Session not found")?;
        let work_dir = session.working_dir.clone();

        // Build command for single-shot interaction
        let mut cmd = Command::new(&self.claude_path);
        cmd.arg("-p"); // Print mode
        cmd.arg("--output-format");
        cmd.arg("json");
        cmd.current_dir(&work_dir);
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        debug!("Sending message to Claude Code session: {}", session_id);

        // Use a new process for this message
        let mut child = cmd.spawn()
            .context("Failed to start Claude Code process")?;

        // Get stdin and send message
        {
            use std::io::Write;
            if let Some(mut stdin) = child.stdin.take() {
                stdin.write_all(message.as_bytes())?;
                stdin.write_all(b"\n")?;
                stdin.flush()?;
            }
        }

        // Read stdout
        let output = child.wait_with_output()
            .context("Failed to read Claude Code output")?;

        // Parse JSON output if possible
        let response_text = String::from_utf8_lossy(&output.stdout).to_string();
        
        // Try to extract meaningful content from JSON response
        let content = if let Ok(json) = serde_json::from_str::<serde_json::Value>(&response_text) {
            // Try to extract text content from Claude's JSON response
            json.get("content")
                .or_else(|| json.get("text"))
                .or_else(|| json.get("message"))
                .and_then(|v| v.as_str())
                .unwrap_or(&response_text)
                .to_string()
        } else {
            // If not JSON, return the raw text
            response_text.clone()
        };

        debug!("Received response from Claude Code: {}", content.chars().take(200).collect::<String>());

        Ok(content)
    }

    /// Get session status
    pub async fn get_session(&self, session_id: &str) -> Result<Option<ClaudeSession>> {
        let sessions = self.sessions.read().await;
        Ok(sessions.get(session_id).cloned())
    }

    /// Check if a session is running
    pub async fn is_running(&self, session_id: &str) -> bool {
        let mut processes = self.processes.lock().unwrap();
        if let Some(child) = processes.get_mut(session_id) {
            match child.try_wait() {
                Ok(Some(_)) => false, // Process has exited
                Ok(None) => true,      // Process is still running
                Err(_) => false,       // Error checking status
            }
        } else {
            false
        }
    }

    /// Terminate a session
    pub fn kill_session(&self, session_id: &str) -> Result<()> {
        let mut processes = self.processes.lock().unwrap();
        if let Some(mut child) = processes.remove(session_id) {
            child.kill()?;
            info!("Killed Claude Code session: {}", session_id);
        }
        
        // Also clean up from sessions
        // Note: This requires &mut which we don't have here
        // Sessions will be cleaned up on next access
        
        Ok(())
    }

    /// Check if Claude Code is available
    pub fn health_check(&self) -> Result<bool> {
        match Command::new(&self.claude_path).arg("--version").output() {
            Ok(output) => Ok(output.status.success()),
            Err(e) => {
                warn!("Claude Code health check failed: {}", e);
                Ok(false)
            }
        }
    }

    /// Get the CLI path
    pub fn claude_path(&self) -> &str {
        &self.claude_path
    }
}

impl Default for ClaudeClient {
    fn default() -> Self {
        let claude_path = which::which("claude")
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| "claude".to_string());

        let work_dir = std::env::var("CLAUDE_WORK_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| std::env::temp_dir().join("supercode-claude"));

        Self::new(claude_path, work_dir)
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ClaudeSessionResponse {
    pub id: String,
    pub session_id: String,
    pub working_dir: String,
}
