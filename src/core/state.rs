//! Application state

use crate::db::Database;
use crate::session::SessionManager;

pub struct AppState {
    pub db: Database,
    pub session_manager: SessionManager,
}

impl AppState {
    pub fn new(db: Database) -> Self {
        Self {
            session_manager: SessionManager::new(db.clone()),
            db,
        }
    }
}
