//! Quality gates for code verification

use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityGateResult {
    pub name: String,
    pub passed: bool,
    pub output: String,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Default)]
pub struct QualityGates;

/// Validate and canonicalize project directory path
/// Returns the canonical path or an error if invalid
fn validate_project_dir(project_dir: &str) -> Result<String, String> {
    let path = Path::new(project_dir);

    // Check path exists
    if !path.exists() {
        return Err(format!("Project directory does not exist: {}", project_dir));
    }

    // Check it's a directory
    if !path.is_dir() {
        return Err(format!("Project path is not a directory: {}", project_dir));
    }

    // Get canonical path to prevent path traversal
    match path.canonicalize() {
        Ok(canonical) => Ok(canonical.to_string_lossy().to_string()),
        Err(e) => Err(format!("Invalid path: {}", e)),
    }
}

impl QualityGates {
    /// Run all quality gates for a project
    pub fn run_all(project_dir: &str) -> Vec<QualityGateResult> {
        let mut results = Vec::new();

        // Validate path first
        let validated_dir = match validate_project_dir(project_dir) {
            Ok(dir) => dir,
            Err(e) => {
                results.push(QualityGateResult {
                    name: "path_validation".to_string(),
                    passed: false,
                    output: e,
                    duration_ms: 0,
                });
                return results;
            }
        };

        // Try to detect project type and run appropriate gates
        if Path::new(&validated_dir).join("Cargo.toml").exists() {
            results.push(Self::rust_check(&validated_dir));
            results.push(Self::rust_clippy(&validated_dir));
        }

        if Path::new(&validated_dir).join("package.json").exists() {
            results.push(Self::npm_lint(&validated_dir));
            results.push(Self::npm_typecheck(&validated_dir));
        }

        if Path::new(&validated_dir).join("pyproject.toml").exists()
            || Path::new(&validated_dir).join("requirements.txt").exists()
        {
            results.push(Self::python_ruff(&validated_dir));
            results.push(Self::python_mypy(&validated_dir));
        }

        results
    }

    /// Run rustc check
    pub fn rust_check(project_dir: &str) -> QualityGateResult {
        let start = std::time::Instant::now();

        let manifest_path = match Path::new(project_dir).join("Cargo.toml").canonicalize() {
            Ok(p) => p,
            Err(e) => {
                return QualityGateResult {
                    name: "rust-check".to_string(),
                    passed: false,
                    output: format!("Cannot resolve manifest path: {}", e),
                    duration_ms: start.elapsed().as_millis() as u64,
                };
            }
        };

        let output = Command::new("cargo")
            .args([
                "check",
                "--manifest-path",
                manifest_path.to_str().unwrap_or(""),
            ])
            .output();

        let (passed, output_str) = match output {
            Ok(o) => (
                o.status.success(),
                String::from_utf8_lossy(&o.stderr).to_string(),
            ),
            Err(e) => (false, format!("Command failed to run: {}", e)),
        };

        QualityGateResult {
            name: "rust-check".to_string(),
            passed,
            output: output_str,
            duration_ms: start.elapsed().as_millis() as u64,
        }
    }

    /// Run clippy
    pub fn rust_clippy(project_dir: &str) -> QualityGateResult {
        let start = std::time::Instant::now();

        let manifest_path = match Path::new(project_dir).join("Cargo.toml").canonicalize() {
            Ok(p) => p,
            Err(e) => {
                return QualityGateResult {
                    name: "rust-clippy".to_string(),
                    passed: false,
                    output: format!("Cannot resolve manifest path: {}", e),
                    duration_ms: start.elapsed().as_millis() as u64,
                };
            }
        };

        let output = Command::new("cargo")
            .args([
                "clippy",
                "--manifest-path",
                manifest_path.to_str().unwrap_or(""),
                "--",
                "-D",
                "warnings",
            ])
            .output();

        let (passed, output_str) = match output {
            Ok(o) => (
                o.status.success(),
                String::from_utf8_lossy(&o.stderr).to_string(),
            ),
            Err(e) => (false, format!("Command failed to run: {}", e)),
        };

        QualityGateResult {
            name: "rust-clippy".to_string(),
            passed,
            output: output_str,
            duration_ms: start.elapsed().as_millis() as u64,
        }
    }

    /// Run npm lint
    pub fn npm_lint(project_dir: &str) -> QualityGateResult {
        let start = std::time::Instant::now();

        // Validate directory exists
        if !Path::new(project_dir).join("package.json").exists() {
            return QualityGateResult {
                name: "npm-lint".to_string(),
                passed: false,
                output: "package.json not found".to_string(),
                duration_ms: start.elapsed().as_millis() as u64,
            };
        }

        let output = Command::new("npm")
            .args(["run", "lint"])
            .current_dir(project_dir)
            .output();

        let (passed, output_str) = match output {
            Ok(o) => {
                let mut s = String::from_utf8_lossy(&o.stdout).to_string();
                s.push_str(&String::from_utf8_lossy(&o.stderr));
                (o.status.success(), s)
            }
            Err(e) => (false, format!("Command failed to run: {}", e)),
        };

        QualityGateResult {
            name: "npm-lint".to_string(),
            passed,
            output: output_str,
            duration_ms: start.elapsed().as_millis() as u64,
        }
    }

    /// Run npm typecheck
    pub fn npm_typecheck(project_dir: &str) -> QualityGateResult {
        let start = std::time::Instant::now();

        // Validate directory exists
        if !Path::new(project_dir).join("package.json").exists() {
            return QualityGateResult {
                name: "npm-typecheck".to_string(),
                passed: false,
                output: "package.json not found".to_string(),
                duration_ms: start.elapsed().as_millis() as u64,
            };
        }

        let output = Command::new("npm")
            .args(["run", "typecheck"])
            .current_dir(project_dir)
            .output();

        let (passed, output_str) = match output {
            Ok(o) => {
                let mut s = String::from_utf8_lossy(&o.stdout).to_string();
                s.push_str(&String::from_utf8_lossy(&o.stderr));
                (o.status.success(), s)
            }
            Err(e) => (false, format!("Command failed to run: {}", e)),
        };

        QualityGateResult {
            name: "npm-typecheck".to_string(),
            passed,
            output: output_str,
            duration_ms: start.elapsed().as_millis() as u64,
        }
    }

    /// Run ruff (Python linter)
    pub fn python_ruff(project_dir: &str) -> QualityGateResult {
        let start = std::time::Instant::now();
        let output = Command::new("ruff").arg("check").arg(project_dir).output();

        let (passed, output_str) = match output {
            Ok(o) => (
                o.status.success(),
                String::from_utf8_lossy(&o.stdout).to_string(),
            ),
            Err(e) => (false, format!("Command failed to run: {}", e)),
        };

        QualityGateResult {
            name: "ruff".to_string(),
            passed,
            output: output_str,
            duration_ms: start.elapsed().as_millis() as u64,
        }
    }

    /// Run mypy (Python type checker)
    pub fn python_mypy(project_dir: &str) -> QualityGateResult {
        let start = std::time::Instant::now();
        let output = Command::new("mypy").arg(project_dir).output();

        let (passed, output_str) = match output {
            Ok(o) => (
                o.status.success(),
                String::from_utf8_lossy(&o.stdout).to_string(),
            ),
            Err(e) => (false, format!("Command failed to run: {}", e)),
        };

        QualityGateResult {
            name: "mypy".to_string(),
            passed,
            output: output_str,
            duration_ms: start.elapsed().as_millis() as u64,
        }
    }

    /// Run pytest (Python tests)
    pub fn python_pytest(project_dir: &str) -> QualityGateResult {
        let start = std::time::Instant::now();
        let output = Command::new("pytest").arg(project_dir).output();

        let (passed, output_str) = match output {
            Ok(o) => {
                let mut s = String::from_utf8_lossy(&o.stdout).to_string();
                s.push_str(&String::from_utf8_lossy(&o.stderr));
                (o.status.success(), s)
            }
            Err(e) => (false, format!("Command failed to run: {}", e)),
        };

        QualityGateResult {
            name: "pytest".to_string(),
            passed,
            output: output_str,
            duration_ms: start.elapsed().as_millis() as u64,
        }
    }
}
