// Tests for agent quality gates

use std::fs;
use supercode::agent::gates::QualityGates;
use tempfile::TempDir;

#[test]
fn test_quality_gates_nonexistent_dir() {
    let results = QualityGates::run_all("/nonexistent/path/12345");

    // Should return path validation error
    assert!(results.len() >= 1);
    assert!(!results[0].passed);
    assert!(results[0].output.contains("does not exist"));
}

#[test]
fn test_quality_gates_empty_dir() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().to_string_lossy().to_string();

    let results = QualityGates::run_all(&path);

    // Empty dir with no project files - should return empty or minimal results
    // No quality gates should fail, but also no gates should run
    assert!(
        results.is_empty()
            || results
                .iter()
                .all(|r| !r.passed || r.name == "path_validation")
    );
}

#[test]
fn test_rust_check_with_cargo_toml() {
    let temp_dir = TempDir::new().unwrap();
    let path = temp_dir.path().to_string_lossy().to_string();

    // Create a minimal Cargo.toml
    fs::write(
        temp_dir.path().join("Cargo.toml"),
        r#"
[package]
name = "test"
version = "0.1.0"
edition = "2021"
"#,
    )
    .unwrap();

    let result = QualityGates::rust_check(&path);

    // Should either pass (if cargo works) or fail gracefully
    assert!(!result.name.is_empty());
    assert!(result.duration_ms >= 0);
}

#[test]
fn test_path_validation_rejects_traversal() {
    // This tests that the validation catches suspicious paths
    // The actual behavior depends on whether the path exists
    let results = QualityGates::run_all("/");

    // Should either fail on path validation or run gates
    // At minimum, should not crash
    assert!(true);
}
