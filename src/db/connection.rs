//! Database connection management
//!
//! NOTE: This implementation uses synchronous rusqlite with tokio::Mutex.
//! For production use with 10+ concurrent sessions, switch to sqlx with
//! async-native SQLite support for true non-blocking operations.

use anyhow::{Context, Result};
use rusqlite::Connection;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

use super::schema::SCHEMA;

pub struct Database {
    /// NOTE: Using synchronous rusqlite with Mutex - this blocks the async
    /// runtime thread during DB operations. For 10+ sessions, use sqlx instead.
    conn: Arc<Mutex<Connection>>,
    path: String,
}

impl Database {
    /// Create a new database connection
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();

        // Create parent directories if needed
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(path)
            .with_context(|| format!("Failed to open database at {:?}", path))?;

        // Enable foreign keys
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;

        // Initialize schema
        conn.execute_batch(SCHEMA)?;

        info!("Database initialized at {:?}", path);

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            path: path.to_string_lossy().to_string(),
        })
    }

    /// Get a locked connection
    /// 
    /// WARNING: This holds the mutex for the duration of the operation,
    /// blocking other async tasks from accessing the database.
    /// 
    /// For production with high concurrency, migrate to sqlx:
    /// https://github.com/launchbadge/sqlx
    pub async fn lock(&self) -> tokio::sync::MutexGuard<'_, Connection> {
        self.conn.lock().await
    }

    /// Get the database path
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Check if database is accessible (for health checks)
    pub async fn health_check(&self) -> Result<bool> {
        let conn = self.lock().await;
        // Simple query to check connectivity
        match conn.execute("SELECT 1", []) {
            Ok(_) => Ok(true),
            Err(e) => {
                tracing::warn!("Database health check failed: {}", e);
                Ok(false)
            }
        }
    }
}

impl Clone for Database {
    fn clone(&self) -> Self {
        Self {
            conn: Arc::clone(&self.conn),
            path: self.path.clone(),
        }
    }
}
