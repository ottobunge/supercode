// Tests for Supercode

use supercode::db::{Database, repositories::session::{SessionRepository, AgentType, SessionType, SessionStatus}};
use tempfile::TempDir;

fn create_test_db() -> (Database, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");
    let db = Database::new(db_path).unwrap();
    (db, temp_dir)
}

#[tokio::test]
async fn test_database_initialization() {
    let (db, _temp) = create_test_db();
    // Basic smoke test - if we get here, DB initialized
    assert!(db.path().contains("test.db"));
}

#[tokio::test]
async fn test_create_session() {
    let (db, _temp) = create_test_db();
    let repo = SessionRepository::new(db);
    
    let session = repo.create(
        AgentType::Developer,
        SessionType::OpenCode,
        None,
        None,
    ).await.unwrap();
    
    assert!(!session.id.is_empty());
    assert_eq!(session.agent_type, AgentType::Developer);
    assert_eq!(session.session_type, SessionType::OpenCode);
    assert_eq!(session.status, SessionStatus::Pending);
}

#[tokio::test]
async fn test_list_sessions() {
    let (db, _temp) = create_test_db();
    let repo = SessionRepository::new(db);
    
    // Create a session
    repo.create(
        AgentType::Developer,
        SessionType::OpenCode,
        None,
        None,
    ).await.unwrap();
    
    // List sessions
    let sessions = repo.list(None, None).await.unwrap();
    assert_eq!(sessions.len(), 1);
}

#[tokio::test]
async fn test_update_session_status() {
    let (db, _temp) = create_test_db();
    let repo = SessionRepository::new(db);
    
    let session = repo.create(
        AgentType::Developer,
        SessionType::OpenCode,
        None,
        None,
    ).await.unwrap();
    
    // Update status
    repo.update_status(&session.id, SessionStatus::Running).await.unwrap();
    
    // Check updated status
    let updated = repo.get(&session.id).await.unwrap().unwrap();
    assert_eq!(updated.status, SessionStatus::Running);
}

#[tokio::test]
async fn test_get_session() {
    let (db, _temp) = create_test_db();
    let repo = SessionRepository::new(db);
    
    let session = repo.create(
        AgentType::Reviewer,
        SessionType::Claude,
        None,
        Some("/tmp/test".to_string()),
    ).await.unwrap();
    
    let retrieved = repo.get(&session.id).await.unwrap().unwrap();
    
    assert_eq!(retrieved.id, session.id);
    assert_eq!(retrieved.agent_type, AgentType::Reviewer);
    assert_eq!(retrieved.session_type, SessionType::Claude);
    assert_eq!(retrieved.working_dir, Some("/tmp/test".to_string()));
}

#[tokio::test]
async fn test_session_not_found() {
    let (db, _temp) = create_test_db();
    let repo = SessionRepository::new(db);
    
    let result = repo.get("nonexistent-id").await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_filter_by_status() {
    let (db, _temp) = create_test_db();
    let repo = SessionRepository::new(db);
    
    // Create sessions with different statuses
    let s1 = repo.create(AgentType::Developer, SessionType::OpenCode, None, None).await.unwrap();
    let s2 = repo.create(AgentType::Developer, SessionType::OpenCode, None, None).await.unwrap();
    
    repo.update_status(&s1.id, SessionStatus::Running).await.unwrap();
    repo.update_status(&s2.id, SessionStatus::Completed).await.unwrap();
    
    // Filter by running
    let running = repo.list(None, Some(SessionStatus::Running)).await.unwrap();
    assert_eq!(running.len(), 1);
    
    // Filter by completed
    let completed = repo.list(None, Some(SessionStatus::Completed)).await.unwrap();
    assert_eq!(completed.len(), 1);
    
    // All sessions
    let all = repo.list(None, None).await.unwrap();
    assert_eq!(all.len(), 2);
}
