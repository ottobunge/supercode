//! Session manager

use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;

use crate::db::{repositories::session::{SessionRepository, SessionStatus as DbSessionStatus}, Database};
use super::{SessionProvider, SessionHandle, OpenCodeProvider, ClaudeProvider, SessionStatus as ProviderSessionStatus};

pub struct SessionManager {
    db: Database,
    session_repo: SessionRepository,
    opencode_provider: Arc<OpenCodeProvider>,
    claude_provider: Arc<ClaudeProvider>,
}

impl SessionManager {
    pub fn new(db: Database) -> Self {
        let opencode_provider = Arc::new(OpenCodeProvider::with_url("http://localhost:9090"));
        let claude_provider = Arc::new(ClaudeProvider::with_defaults());
        
        Self {
            db: db.clone(),
            session_repo: SessionRepository::new(db),
            opencode_provider,
            claude_provider,
        }
    }

    pub fn with_opencode_url(db: Database, url: impl Into<String>) -> Self {
        let opencode_provider = Arc::new(OpenCodeProvider::with_url(url));
        let claude_provider = Arc::new(ClaudeProvider::with_defaults());
        
        Self {
            db: db.clone(),
            session_repo: SessionRepository::new(db),
            opencode_provider,
            claude_provider,
        }
    }

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
