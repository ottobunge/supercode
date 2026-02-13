//! Session manager

use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::db::{repositories::session::{SessionRepository, SessionStatus as DbSessionStatus}, Database};
use crate::monitor::{SessionActivity, SessionMonitor, OpenCodeMonitor, ClaudeMonitor};
use super::{SessionProvider, SessionHandle, OpenCodeProvider, ClaudeProvider, SessionStatus as ProviderSessionStatus};

pub struct SessionManager {
    db: Database,
    session_repo: SessionRepository,
    opencode_provider: Arc<OpenCodeProvider>,
    claude_provider: Arc<ClaudeProvider>,
    /// Monitors for each provider type
    opencode_monitor: Arc<OpenCodeMonitor>,
    claude_monitor: Arc<ClaudeMonitor>,
    /// Cache of session activities
    activity_cache: Arc<RwLock<std::collections::HashMap<String, SessionActivity>>>,
}

impl SessionManager {
    pub fn new(db: Database) -> Self {
        let opencode_provider = Arc::new(OpenCodeProvider::with_url("http://localhost:9090"));
        let claude_provider = Arc::new(ClaudeProvider::with_defaults());
        
        // Create monitors
        let opencode_monitor = Arc::new(OpenCodeMonitor::new("http://localhost:9090"));
        let claude_monitor = Arc::new(ClaudeMonitor::new(
            dirs::data_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("."))
                .join("claude")
                .join("sessions")
        ));
        
        Self {
            db: db.clone(),
            session_repo: SessionRepository::new(db),
            opencode_provider,
            claude_provider,
            opencode_monitor,
            claude_monitor,
            activity_cache: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    pub fn with_opencode_url(db: Database, url: impl Into<String>) -> Self {
        let opencode_url = url.into();
        let opencode_provider = Arc::new(OpenCodeProvider::with_url(&opencode_url));
        let claude_provider = Arc::new(ClaudeProvider::with_defaults());
        let opencode_monitor = Arc::new(OpenCodeMonitor::new(&opencode_url));
        let claude_monitor = Arc::new(ClaudeMonitor::new(
            dirs::data_dir()
                .unwrap_or_else(|| std::path::PathBuf::from("."))
                .join("claude")
                .join("sessions")
        ));
        
        Self {
            db: db.clone(),
            session_repo: SessionRepository::new(db),
            opencode_provider,
            claude_provider,
            opencode_monitor,
            claude_monitor,
            activity_cache: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }

    /// Get the database repository
    pub fn repository(&self) -> &SessionRepository {
        &self.session_repo
    }

    /// Get the appropriate provider for a session type
    fn get_provider(&self, session_type: &str) -> Result<&dyn SessionProvider> {
        match session_type {
            "opencode" => Ok(self.opencode_provider.as_ref() as &dyn SessionProvider),
            "claude" => Ok(self.claude_provider.as_ref() as &dyn SessionProvider),
            _ => anyhow::bail!("Unknown session type: {}", session_type),
        }
    }

    /// Get the monitor for a specific provider type
    fn get_monitor(&self, session_type: &str) -> Result<Box<&dyn SessionMonitor>> {
        match session_type {
            "opencode" => Ok(Box::new(self.opencode_monitor.as_ref())),
            "claude" => Ok(Box::new(self.claude_monitor.as_ref())),
            _ => anyhow::bail!("Unknown session type: {}", session_type),
        }
    }

    // ==================== Session Monitoring ====================

    /// Get detailed activity for a session
    pub async fn get_session_activity(&self, session_id: &str) -> Result<SessionActivity> {
        // First get the session from DB to know its type
        let session = self.session_repo.get(session_id).await?
            .ok_or_else(|| anyhow::anyhow!("Session not found: {}", session_id))?;

        let session_type = session.session_type.as_str();
        let provider_session_id = session.opencode_session_id
            .ok_or_else(|| anyhow::anyhow!("Session has no provider ID"))?;

        let monitor = self.get_monitor(session_type)?;
        
        // Try to get activity from provider
        match monitor.get_activity(&provider_session_id).await {
            Ok(mut activity) => {
                activity.session_id = session_id.to_string();
                
                // Cache it
                let mut cache = self.activity_cache.write().await;
                cache.insert(session_id.to_string(), activity.clone());
                
                Ok(activity)
            }
            Err(e) => {
                // If provider query fails, return cached or create basic activity
                let cache = self.activity_cache.read().await;
                if let Some(cached) = cache.get(session_id) {
                    Ok(cached.clone())
                } else {
                    // Return basic activity from DB state
                    let state = match session.status.as_str() {
                        "running" | "active" => crate::monitor::AgentState::Processing,
                        "completed" => crate::monitor::AgentState::Completed,
                        "failed" => crate::monitor::AgentState::Failed { error: e.to_string() },
                        _ => crate::monitor::AgentState::Unknown,
                    };
                    
                    Ok(SessionActivity::new(
                        session_id.to_string(),
                        provider_session_id,
                    ).with_state(state))
                }
            }
        }
    }

    /// Check if a session is waiting for human approval
    pub async fn is_session_waiting(&self, session_id: &str) -> Result<bool> {
        let activity = self.get_session_activity(session_id).await?;
        Ok(activity.is_blocked())
    }

    /// List all sessions with their current activity
    pub async fn list_sessions_with_activity(&self) -> Result<Vec<SessionActivity>> {
        let sessions = self.session_repo.list(None, None).await?;
        
        let mut activities = Vec::new();
        
        for session in sessions {
            match self.get_session_activity(&session.id).await {
                Ok(activity) => activities.push(activity),
                Err(_) => {
                    // If we can't get activity, add basic one
                    activities.push(SessionActivity::new(
                        session.id.clone(),
                        session.opencode_session_id.unwrap_or_default(),
                    ).with_state(match session.status.as_str() {
                        "running" | "active" => crate::monitor::AgentState::Processing,
                        "completed" => crate::monitor::AgentState::Completed,
                        _ => crate::monitor::AgentState::Unknown,
                    }));
                }
            }
        }
        
        Ok(activities)
    }

    /// Get all sessions that are waiting for human input/approval
    pub async fn get_blocked_sessions(&self) -> Result<Vec<SessionActivity>> {
        let activities = self.list_sessions_with_activity().await?;
        Ok(activities.into_iter().filter(|a| a.is_blocked()).collect())
    }

    /// Spawn a new session using the appropriate provider
    pub async fn spawn_session(
        &self,
        session_id: &str,
        agent_type: &str,
        session_type: &str,
        name: Option<&str>,
        extra_prompt: Option<&str>,
    ) -> Result<SessionHandle> {
        let provider = self.get_provider(session_type)?;

        // Build the agent prompt from type + extra_prompt + compaction note
        let agent_prompt = build_agent_prompt(agent_type, name, extra_prompt);

        // Create the session with empty system prompt (we'll send the full prompt as first message)
        let handle = provider.create_session(None).await?;

        // Update the database with the provider session ID
        self.session_repo
            .set_opencode_session_id(session_id, &handle.provider_id)
            .await?;

        // Send the initial prompt as the first message
        self.send_message(
            session_id,
            &handle.provider_id,
            session_type,
            &agent_prompt,
        ).await?;

        Ok(handle)
    }

    /// Send a message to a session
    pub async fn send_message(
        &self,
        session_id: &str,
        provider_session_id: &str,
        session_type: &str,
        message: &str,
    ) -> Result<String> {
        let provider = self.get_provider(session_type)?;

        provider.send_message(provider_session_id, message).await
    }

    /// Get session status from provider
    pub async fn get_session_status(
        &self,
        provider_session_id: &str,
        session_type: &str,
    ) -> Result<ProviderSessionStatus> {
        let provider = self.get_provider(session_type)?;

        provider.get_status(provider_session_id).await
    }

    /// Fork a session
    pub async fn fork_session(
        &self,
        provider_session_id: &str,
        session_type: &str,
    ) -> Result<SessionHandle> {
        let provider = self.get_provider(session_type)?;

        provider.fork_session(provider_session_id).await
    }

    /// Kill a session at the provider level
    pub async fn kill_provider_session(
        &self,
        provider_session_id: &str,
        session_type: &str,
    ) -> Result<()> {
        let provider = self.get_provider(session_type)?;
        provider.kill_session(provider_session_id).await
    }

    /// Check OpenCode provider health
    pub async fn check_opencode_health(&self) -> Result<bool> {
        self.opencode_provider.health_check().await
    }

    /// Check Claude provider health
    pub async fn check_claude_health(&self) -> Result<bool> {
        self.claude_provider.health_check().await
    }
}

/// Build the agent prompt from type, extra_prompt, and compaction note
fn build_agent_prompt(agent_type: &str, name: Option<&str>, extra_prompt: Option<&str>) -> String {
    // Determine role name from agent_type
    let role = match agent_type {
        "manager" => "Manager",
        "developer" => "Developer", 
        "reviewer" => "Reviewer",
        _ => "Agent",
    };

    // Get the agent's name or use "Unnamed" as fallback
    let agent_name = name.unwrap_or("Unnamed");

    let base_prompt = match agent_type {
        "manager" => r#"You are a Manager Agent for Supercode. Your role is to coordinate development work across multiple parallel sessions, maintain high-level context, delegate tasks to specialized agents (developer, reviewer), and ensure quality gates pass.

Primary Responsibilities:
1. Task Decomposition - Break down complex requests into manageable subtasks
2. Parallel Execution - Spawn multiple sessions for concurrent work
3. Delegation - Route work to the appropriate specialist
4. Context Preservation - Maintain bird's-eye view of ongoing work
5. Quality Gates - Ensure linting, type checking, and tests pass
6. Aggregation - Combine results from multiple parallel sessions

When delegating:
- Provide clear context and requirements
- Specify quality gates
- Aggregate results from multiple agents

When tasks are complete, summarize what was accomplished."#,
        
        "developer" => r#"You are a Developer Agent for Supercode. Your role is to implement features, fix bugs, write tests, and produce high-quality code.

Core Principles:
1. Follow Existing Patterns - Study similar code before writing
2. Type Everything - Complete type annotations required
3. Test Your Code - Write tests alongside implementation
4. Keep It Simple - Avoid over-engineering
5. Document Decisions - Add docstrings where logic isn't obvious

Before marking work complete, ensure:
- Code compiles
- Linting passes
- Type checking passes
- Tests pass
- No security issues"#,
        
        "reviewer" => r#"You are a Review Agent for Supercode. Your role is to review code changes, ensure quality standards, identify issues, and maintain code quality.

Review Responsibilities:
1. Code Quality - Check for bugs, anti-patterns
2. Type Safety - Ensure complete type annotations
3. Architecture Alignment - Verify project patterns
4. Security - Identify vulnerabilities
5. Testing - Verify adequate test coverage

Severity Levels:
- Critical (🔴): Security vulnerability, data loss - Must fix
- High (🟠): Bug, architecture violation - Should fix
- Medium (🟡): Code smell, maintainability - Recommend fix
- Low (🟢): Style, optimization - Optional

Provide specific, actionable feedback with file paths and line numbers."#,
        
        _ => "You are an agent for Supercode.",
    };

    let compaction_note = format!(r#"

---
IMPORTANT - SURVIVE COMPACTIONS:
This is the ONLY time you will receive these instructions. Remember this: 
- Your name is: {}
- Your role is: {}
- You coordinate other agents to complete tasks
- When asked to do work, break it down and delegate to developer/reviewer agents
- Always ensure quality gates pass before marking work complete
- Keep track of what all your sub-agents are doing

When you need to spawn a sub-agent, use the spawn_session tool with the appropriate agent_type.
When you need to communicate with a sub-agent, use the send_message tool.
When you need information about existing sessions, use the list_sessions tool.
When work is complete, summarize what was accomplished."#, agent_name, role);

    // Build the final prompt
    let mut prompt = base_prompt.to_string();
    
    // Add extra prompt if provided
    if let Some(extra) = extra_prompt {
        prompt.push_str("\n\n---\n\nAdditional Instructions:\n");
        prompt.push_str(extra);
    }
    
    // Add compaction note
    prompt.push_str(&compaction_note);
    
    prompt
}
