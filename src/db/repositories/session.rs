//! Session repository

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::params;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::Database;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub project_id: Option<String>,
    pub agent_type: AgentType,
    pub session_type: SessionType,
    pub status: SessionStatus,
    pub working_dir: Option<String>,
    pub opencode_session_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AgentType {
    Manager,
    Developer,
    Reviewer,
}

impl AgentType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AgentType::Manager => "manager",
            AgentType::Developer => "developer",
            AgentType::Reviewer => "reviewer",
        }
    }

    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "manager" => Ok(AgentType::Manager),
            "developer" => Ok(AgentType::Developer),
            "reviewer" => Ok(AgentType::Reviewer),
            _ => anyhow::bail!("Unknown agent type: {}", s),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SessionType {
    OpenCode,
    Claude,
}

impl SessionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            SessionType::OpenCode => "opencode",
            SessionType::Claude => "claude",
        }
    }

    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "opencode" => Ok(SessionType::OpenCode),
            "claude" => Ok(SessionType::Claude),
            _ => anyhow::bail!("Unknown session type: {}", s),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SessionStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Terminated,
}

impl SessionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            SessionStatus::Pending => "pending",
            SessionStatus::Running => "running",
            SessionStatus::Completed => "completed",
            SessionStatus::Failed => "failed",
            SessionStatus::Terminated => "terminated",
        }
    }

    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "pending" => Ok(SessionStatus::Pending),
            "running" => Ok(SessionStatus::Running),
            "completed" => Ok(SessionStatus::Completed),
            "failed" => Ok(SessionStatus::Failed),
            "terminated" => Ok(SessionStatus::Terminated),
            _ => anyhow::bail!("Unknown session status: {}", s),
        }
    }
}

pub struct SessionRepository {
    db: Database,
}

impl SessionRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Get the database reference
    pub fn db(&self) -> &Database {
        &self.db
    }

    /// Create a new session
    pub async fn create(
        &self,
        agent_type: AgentType,
        session_type: SessionType,
        project_id: Option<String>,
        working_dir: Option<String>,
    ) -> Result<Session> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();

        let session = Session {
            id: id.clone(),
            project_id,
            agent_type,
            session_type,
            status: SessionStatus::Pending,
            working_dir,
            opencode_session_id: None,
            created_at: now,
            updated_at: now,
            metadata: None,
        };

        let conn = self.db.lock().await;
        conn.execute(
            "INSERT INTO sessions (id, project_id, agent_type, session_type, status, working_dir, created_at, updated_at, metadata)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                session.id,
                session.project_id,
                session.agent_type.as_str(),
                session.session_type.as_str(),
                session.status.as_str(),
                session.working_dir,
                session.created_at.to_rfc3339(),
                session.updated_at.to_rfc3339(),
                session.metadata,
            ],
        ).context("Failed to insert session")?;

        tracing::debug!("Created session: {}", id);
        Ok(session)
    }

    /// Get a session by ID
    pub async fn get(&self, id: &str) -> Result<Option<Session>> {
        let conn = self.db.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, project_id, agent_type, session_type, status, working_dir, 
                    opencode_session_id, created_at, updated_at, metadata
             FROM sessions WHERE id = ?1"
        )?;

        let result = stmt.query_row(params![id], |row| {
            let agent_type_str = row.get::<_, String>(2)?;
            let session_type_str = row.get::<_, String>(3)?;
            let status_str = row.get::<_, String>(4)?;
            
            let agent_type = AgentType::from_str(&agent_type_str)
                .unwrap_or(AgentType::Developer); // Default to Developer for unknown
            let session_type = SessionType::from_str(&session_type_str)
                .unwrap_or(SessionType::OpenCode); // Default to OpenCode for unknown
            let status = SessionStatus::from_str(&status_str)
                .unwrap_or(SessionStatus::Pending); // Default to Pending for unknown

            let created_at = DateTime::parse_from_rfc3339(&row.get::<_, String>(7)?)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());
            let updated_at = DateTime::parse_from_rfc3339(&row.get::<_, String>(8)?)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());

            Ok(Session {
                id: row.get(0)?,
                project_id: row.get(1)?,
                agent_type,
                session_type,
                status,
                working_dir: row.get(5)?,
                opencode_session_id: row.get(6)?,
                created_at,
                updated_at,
                metadata: row.get(9)?,
            })
        });

        match result {
            Ok(session) => Ok(Some(session)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e).context("Failed to get session"),
        }
    }

    /// List all sessions, optionally filtered
    pub async fn list(
        &self,
        project_id: Option<&str>,
        status: Option<SessionStatus>,
    ) -> Result<Vec<Session>> {
        let conn = self.db.lock().await;

        let mut query = String::from(
            "SELECT id, project_id, agent_type, session_type, status, working_dir,
                    opencode_session_id, created_at, updated_at, metadata
             FROM sessions WHERE 1=1"
        );

        let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if project_id.is_some() {
            query.push_str(" AND project_id = ?1");
        }
        if status.is_some() {
            // Use correct parameter number based on what's in the query
            let param_num = if project_id.is_some() { 2 } else { 1 };
            query.push_str(&format!(" AND status = ?{}", param_num));
        }
        query.push_str(" ORDER BY created_at DESC");

        let mut stmt = conn.prepare(&query)?;

        if let Some(pid) = project_id {
            params.push(Box::new(pid.to_string()));
        }
        if let Some(st) = status {
            params.push(Box::new(st.as_str().to_string()));
        }

        let params_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p.as_ref()).collect();

        let sessions = stmt.query_map(params_refs.as_slice(), Self::map_row)?;

        let sessions_list: Vec<Session> = sessions.collect::<std::result::Result<Vec<_>, _>>()
            .context("Failed to collect sessions")?;

        Ok(sessions_list)
    }

    /// Update session status
    pub async fn update_status(&self, id: &str, status: SessionStatus) -> Result<()> {
        let conn = self.db.lock().await;
        let now = Utc::now().to_rfc3339();

        conn.execute(
            "UPDATE sessions SET status = ?1, updated_at = ?2 WHERE id = ?3",
            params![status.as_str(), now, id],
        )?;

        tracing::debug!("Updated session {} status to {}", id, status.as_str());
        Ok(())
    }

    /// Update session with OpenCode session ID
    pub async fn set_opencode_session_id(&self, id: &str, opencode_session_id: &str) -> Result<()> {
        let conn = self.db.lock().await;
        let now = Utc::now().to_rfc3339();

        conn.execute(
            "UPDATE sessions SET opencode_session_id = ?1, status = ?2, updated_at = ?3 WHERE id = ?4",
            params![opencode_session_id, SessionStatus::Running.as_str(), now, id],
        )?;

        Ok(())
    }

    /// Delete a session
    pub async fn delete(&self, id: &str) -> Result<()> {
        let conn = self.db.lock().await;
        conn.execute("DELETE FROM sessions WHERE id = ?1", params![id])?;
        tracing::debug!("Deleted session: {}", id);
        Ok(())
    }

    fn map_row(row: &rusqlite::Row) -> rusqlite::Result<Session> {
        Ok(Session {
            id: row.get(0)?,
            project_id: row.get(1)?,
            agent_type: AgentType::from_str(&row.get::<_, String>(2).unwrap_or_default()).unwrap_or(AgentType::Developer),
            session_type: SessionType::from_str(&row.get::<_, String>(3).unwrap_or_default()).unwrap_or(SessionType::OpenCode),
            status: SessionStatus::from_str(&row.get::<_, String>(4).unwrap_or_default()).unwrap_or(SessionStatus::Pending),
            working_dir: row.get(5)?,
            opencode_session_id: row.get(6)?,
            created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(7).unwrap_or_default())
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(8).unwrap_or_default())
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            metadata: row.get(9)?,
        })
    }
}
