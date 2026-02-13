//! Session state definitions

use serde::{Deserialize, Serialize};

/// Detailed state of a sub-agent session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentState {
    /// Session is idle, not doing anything
    Idle,

    /// Session is actively processing a task
    Processing,

    /// Session is waiting for user approval/permission
    /// This is the key state for monitoring - indicates the sub-agent
    /// is blocked waiting for something from the human
    WaitingForApproval {
        /// What type of approval is needed
        approval_type: ApprovalType,
        /// Description of what's being requested
        description: String,
    },

    /// Session is waiting for input from the user
    WaitingForInput,

    /// Session completed successfully
    Completed,

    /// Session failed with an error
    Failed { error: String },

    /// Session was terminated
    Terminated,

    /// State is unknown (could not determine)
    Unknown,
}

/// Types of approvals a sub-agent might need
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApprovalType {
    /// File system operation (read/write/execute)
    FileAccess,
    /// Running a shell command
    CommandExecution,
    /// Network request
    NetworkAccess,
    /// Using a specific tool
    ToolUse,
    /// General permission
    General,
    /// Unknown approval type
    Unknown,
}

impl ApprovalType {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "file" | "file_access" | "filesystem" => ApprovalType::FileAccess,
            "command" | "shell" | "execute" => ApprovalType::CommandExecution,
            "network" | "http" | "request" => ApprovalType::NetworkAccess,
            "tool" | "tool_use" => ApprovalType::ToolUse,
            "general" => ApprovalType::General,
            _ => ApprovalType::Unknown,
        }
    }
}

/// Detailed activity information about a session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionActivity {
    /// Our internal session ID
    pub session_id: String,
    /// Provider's session ID (OpenCode/Claude)
    pub provider_session_id: String,
    /// Current state
    pub state: AgentState,
    /// Last message sent to the agent
    pub last_message: Option<String>,
    /// Last response from the agent
    pub last_response: Option<String>,
    /// Timestamp when state last changed
    pub state_changed_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Additional metadata
    pub metadata: std::collections::HashMap<String, String>,
}

impl Default for SessionActivity {
    fn default() -> Self {
        Self {
            session_id: String::new(),
            provider_session_id: String::new(),
            state: AgentState::Unknown,
            last_message: None,
            last_response: None,
            state_changed_at: None,
            metadata: std::collections::HashMap::new(),
        }
    }
}

impl SessionActivity {
    pub fn new(session_id: String, provider_session_id: String) -> Self {
        Self {
            session_id,
            provider_session_id,
            state: AgentState::Idle,
            last_message: None,
            last_response: None,
            state_changed_at: Some(chrono::Utc::now()),
            metadata: std::collections::HashMap::new(),
        }
    }

    pub fn with_state(mut self, state: AgentState) -> Self {
        self.state = state;
        self.state_changed_at = Some(chrono::Utc::now());
        self
    }

    pub fn with_message(mut self, message: String) -> Self {
        self.last_message = Some(message);
        self
    }

    pub fn with_response(mut self, response: String) -> Self {
        self.last_response = Some(response);
        self
    }

    /// Check if the agent is waiting for something from the human
    pub fn is_blocked(&self) -> bool {
        matches!(
            self.state,
            AgentState::WaitingForApproval { .. } | AgentState::WaitingForInput
        )
    }

    /// Check if the agent is actively working
    pub fn is_active(&self) -> bool {
        matches!(self.state, AgentState::Processing)
    }
}
