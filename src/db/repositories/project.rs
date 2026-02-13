//! Project repository

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::params;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::Database;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: Option<String>,
}

pub struct ProjectRepository {
    db: Database,
}

impl ProjectRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Get the database reference
    pub fn db(&self) -> &Database {
        &self.db
    }

    /// Create a new project
    pub async fn create(
        &self,
        name: String,
        description: Option<String>,
    ) -> Result<Project> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();

        let project = Project {
            id: id.clone(),
            name: name.clone(),
            description,
            created_at: now,
            updated_at: now,
            metadata: None,
        };

        let conn = self.db.lock().await;
        conn.execute(
            "INSERT INTO projects (id, name, description, created_at, updated_at, metadata)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                project.id,
                project.name,
                project.description,
                project.created_at.to_rfc3339(),
                project.updated_at.to_rfc3339(),
                project.metadata,
            ],
        ).context("Failed to insert project")?;

        tracing::debug!("Created project: {}", id);
        Ok(project)
    }

    /// Get a project by ID
    pub async fn get(&self, id: &str) -> Result<Option<Project>> {
        let conn = self.db.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, name, description, created_at, updated_at, metadata
             FROM projects WHERE id = ?1"
        )?;

        let result = stmt.query_row(params![id], |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                metadata: row.get(5)?,
            })
        });

        match result {
            Ok(project) => Ok(Some(project)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e).context("Failed to get project"),
        }
    }

    /// List all projects
    pub async fn list(&self) -> Result<Vec<Project>> {
        let conn = self.db.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, name, description, created_at, updated_at, metadata
             FROM projects ORDER BY created_at DESC"
        )?;

        let projects = stmt.query_map([], |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                updated_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                metadata: row.get(5)?,
            })
        })?.collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(projects)
    }

    /// Delete a project
    pub async fn delete(&self, id: &str) -> Result<()> {
        let conn = self.db.lock().await;
        conn.execute("DELETE FROM projects WHERE id = ?1", params![id])?;
        tracing::debug!("Deleted project: {}", id);
        Ok(())
    }
}
