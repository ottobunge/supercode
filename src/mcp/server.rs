//! MCP server

use std::sync::Arc;

use anyhow::Result;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use serde_json::json;

use super::types::*;

pub struct McpServer {
    port: u16,
    session_manager: Arc<crate::session::SessionManager>,
}

impl McpServer {
    pub fn new(port: u16, session_manager: Arc<crate::session::SessionManager>) -> Self {
        Self { port, session_manager }
    }

    pub async fn run(&self) -> Result<()> {
        let addr = format!("127.0.0.1:{}", self.port);
        let listener = TcpListener::bind(&addr).await?;
        
        tracing::info!("MCP server listening on {}", addr);

        loop {
            let (stream, addr) = listener.accept().await?;
            tracing::debug!("Accepted connection from {}", addr);
            
            let session_manager = self.session_manager.clone();
            tokio::spawn(async move {
                if let Err(e) = Self::handle_connection(stream, session_manager).await {
                    tracing::error!("Error handling connection: {}", e);
                }
            });
        }
    }

    async fn handle_connection(mut stream: TcpStream, session_manager: Arc<crate::session::SessionManager>) -> Result<()> {
        let mut buffer = vec![0u8; 8192];
        
        loop {
            // Read request
            let n = stream.read(&mut buffer).await?;
            if n == 0 {
                break;
            }

            let request_str = String::from_utf8_lossy(&buffer[..n]);
            tracing::debug!("Received raw: {}", request_str);

            // Extract JSON body from HTTP request
            let json_body = match Self::extract_json_body(&request_str) {
                Some(body) => body,
                None => {
                    let response = JsonRpcResponse::error(
                        json!(null),
                        -32700,
                        "Invalid HTTP request"
                    );
                    send_response(&mut stream, response).await?;
                    continue;
                }
            };

            // Parse JSON-RPC request
            let request: JsonRpcRequest = match serde_json::from_str(&json_body) {
                Ok(r) => r,
                Err(e) => {
                    let response = JsonRpcResponse::error(
                        json!(null),
                        -32700,
                        &format!("Parse error: {}", e)
                    );
                    send_response(&mut stream, response).await?;
                    continue;
                }
            };

            // Handle request
            let response = Self::handle_request(request, &session_manager).await;
            send_response(&mut stream, response).await?;
        }

        Ok(())
    }

    /// Extract JSON body from HTTP request
    fn extract_json_body(http_request: &str) -> Option<String> {
        // Find the double newline that separates headers from body
        let parts: Vec<&str> = http_request.splitn(2, "\r\n\r\n").collect();
        if parts.len() < 2 {
            // Try without CRLF
            let parts: Vec<&str> = http_request.splitn(2, "\n\n").collect();
            if parts.len() < 2 {
                return None;
            }
            return Some(parts[1].to_string());
        }
        
        // Check if this is a POST request with JSON
        let headers = parts[0];
        if !headers.contains("Content-Type: application/json") {
            // For now, try to parse anyway
        }
        
        Some(parts[1].to_string())
    }
}

async fn send_response(stream: &mut TcpStream, response: JsonRpcResponse) -> Result<()> {
    let response_str = serde_json::to_string(&response)?;
    tracing::debug!("Sending: {}", response_str);
    
    // Send HTTP response
    let http_response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
        response_str.len(),
        response_str
    );
    
    stream.write_all(http_response.as_bytes()).await?;
    stream.flush().await?;
    
    Ok(())
}

impl McpServer {
    async fn handle_request(request: JsonRpcRequest, session_manager: &Arc<crate::session::SessionManager>) -> JsonRpcResponse {
        let id = request.id;
        
        let method = match McpMethod::from_str(&request.method) {
            Some(m) => m,
            None => {
                return JsonRpcResponse::error(id, -32601, "Method not found");
            }
        };

        match method {
            McpMethod::Initialize => {
                let result = InitializeResult {
                    protocol_version: "2024-11-05".to_string(),
                    capabilities: Capabilities {
                        tools: true,
                        resources: false,
                        prompts: false,
                    },
                    server_info: ServerInfo {
                        name: "supercode".to_string(),
                        version: env!("CARGO_PKG_VERSION").to_string(),
                    },
                };
                JsonRpcResponse::success(id, serde_json::to_value(result).unwrap())
            }
            
            McpMethod::ToolsList => {
                let tools = Self::get_tools();
                let result = ToolsListResult { tools };
                JsonRpcResponse::success(id, serde_json::to_value(result).unwrap())
            }
            
            McpMethod::ToolsCall => {
                let params: ToolCall = match serde_json::from_value(request.params) {
                    Ok(p) => p,
                    Err(e) => {
                        return JsonRpcResponse::error(id, -32602, &format!("Invalid params: {}", e));
                    }
                };

                match Self::call_tool(&params, session_manager).await {
                    Ok(result) => JsonRpcResponse::success(id, serde_json::to_value(result).unwrap()),
                    Err(e) => JsonRpcResponse::error(id, -32000, &e.to_string()),
                }
            }
            
            _ => JsonRpcResponse::error(id, -32601, "Method not implemented"),
        }
    }

    fn get_tools() -> Vec<Tool> {
        vec![
            Tool {
                name: "spawn_session".to_string(),
                description: "Create a new agent session".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "name": {
                            "type": "string",
                            "description": "Name for this agent (will be used to identify it)"
                        },
                        "agent_type": {
                            "type": "string",
                            "enum": ["manager", "developer", "reviewer"],
                            "description": "Type of agent to spawn"
                        },
                        "session_type": {
                            "type": "string",
                            "enum": ["opencode", "claude"],
                            "description": "Session backend type"
                        },
                        "project_id": {
                            "type": "string",
                            "description": "Optional project ID to assign session to"
                        },
                        "working_dir": {
                            "type": "string",
                            "description": "Working directory for the agent"
                        },
                        "extra_prompt": {
                            "type": "string",
                            "description": "Optional additional instructions for the agent"
                        }
                    },
                    "required": ["agent_type", "session_type", "working_dir", "name"]
                }),
            },
            Tool {
                name: "list_sessions".to_string(),
                description: "List all sessions, optionally filtered".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "project_id": {
                            "type": "string",
                            "description": "Filter by project ID"
                        },
                        "status": {
                            "type": "string",
                            "description": "Filter by status"
                        },
                        "agent_type": {
                            "type": "string",
                            "description": "Filter by agent type"
                        }
                    }
                }),
            },
            Tool {
                name: "send_message".to_string(),
                description: "Send a message to a session".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "session_id": {
                            "type": "string",
                            "description": "The session ID"
                        },
                        "content": {
                            "type": "string",
                            "description": "Message content to send"
                        }
                    },
                    "required": ["session_id", "content"]
                }),
            },
            Tool {
                name: "kill_session".to_string(),
                description: "Terminate a running session".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "session_id": {
                            "type": "string",
                            "description": "The session ID to kill"
                        }
                    },
                    "required": ["session_id"]
                }),
            },
            Tool {
                name: "get_session".to_string(),
                description: "Get session details and history".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "session_id": {
                            "type": "string",
                            "description": "The session ID"
                        }
                    },
                    "required": ["session_id"]
                }),
            },
            Tool {
                name: "fork_session".to_string(),
                description: "Fork an existing session for parallel work".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "session_id": {
                            "type": "string",
                            "description": "The session ID to fork"
                        },
                        "fork_message": {
                            "type": "string",
                            "description": "Optional message for the forked session"
                        }
                    },
                    "required": ["session_id"]
                }),
            },
            Tool {
                name: "list_projects".to_string(),
                description: "List all projects".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {}
                }),
            },
            Tool {
                name: "create_project".to_string(),
                description: "Create a new project".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "name": {
                            "type": "string",
                            "description": "Project name"
                        },
                        "description": {
                            "type": "string",
                            "description": "Project description"
                        }
                    },
                    "required": ["name"]
                }),
            },
            Tool {
                name: "run_quality_gates".to_string(),
                description: "Run quality gates on a project directory".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "project_dir": {
                            "type": "string",
                            "description": "Path to the project directory"
                        },
                        "gate": {
                            "type": "string",
                            "enum": ["all", "rust_check", "rust_clippy", "npm_lint", "npm_typecheck", "python_ruff", "python_mypy", "python_pytest"],
                            "description": "Which gate to run (default: all)"
                        }
                    },
                    "required": ["project_dir"]
                }),
            },
        ]
    }

    async fn call_tool(tool_call: &ToolCall, session_manager: &Arc<crate::session::SessionManager>) -> Result<ToolCallResult> {
        let args = &tool_call.arguments;
        
        match tool_call.name.as_str() {
            "spawn_session" => {
                // Validate required fields with proper error messages
                let agent_type = args["agent_type"].as_str()
                    .ok_or_else(|| anyhow::anyhow!("agent_type is required"))?;
                let session_type = args["session_type"].as_str()
                    .ok_or_else(|| anyhow::anyhow!("session_type is required"))?;
                let working_dir = args["working_dir"].as_str()
                    .ok_or_else(|| anyhow::anyhow!("working_dir is required"))?;
                let name = args["name"].as_str()
                    .ok_or_else(|| anyhow::anyhow!("name is required"))?;
                let project_id = args["project_id"].as_str().map(String::from);
                let extra_prompt = args["extra_prompt"].as_str();

                // Validate agent_type enum
                let agent_type_enum = crate::db::repositories::session::AgentType::from_str(agent_type)
                    .map_err(|_| anyhow::anyhow!("Invalid agent_type: {}. Must be one of: manager, developer, reviewer", agent_type))?;
                
                // Validate session_type enum  
                let session_type_enum = crate::db::repositories::session::SessionType::from_str(session_type)
                    .map_err(|_| anyhow::anyhow!("Invalid session_type: {}. Must be one of: opencode, claude", session_type))?;
                
                // Create DB session record
                let db = session_manager.repository().db().clone();
                let session_repo = crate::db::repositories::session::SessionRepository::new(db);
                
                let session = session_repo.create(
                    agent_type_enum,
                    session_type_enum,
                    project_id,
                    Some(working_dir.to_string()),
                ).await?;

                // Use provided name (now required)
                let agent_name = name;

                // Try to spawn with the provider (name will be included in initial prompt)
                match session_manager.spawn_session(&session.id, agent_type, session_type, Some(agent_name), extra_prompt).await {
                    Ok(handle) => {
                        Ok(ToolCallResult {
                            content: vec![ContentBlock::Text {
                                text: json!({
                                    "session_id": session.id,
                                    "name": agent_name,
                                    "provider_session_id": handle.provider_id,
                                    "status": "running"
                                }).to_string()
                            }]
                        })
                    }
                    Err(e) => {
                        // Session created in DB but provider failed
                        Ok(ToolCallResult {
                            content: vec![ContentBlock::Text {
                                text: json!({
                                    "session_id": session.id,
                                    "name": agent_name,
                                    "status": "error",
                                    "error": e.to_string()
                                }).to_string()
                            }]
                        })
                    }
                }
            }
            
            "list_sessions" => {
                let project_id = args["project_id"].as_str();
                let status = args["status"].as_str()
                    .map(|s| crate::db::repositories::session::SessionStatus::from_str(s))
                    .transpose()?;
                let agent_type = args["agent_type"].as_str();

                let sessions = session_manager.repository().list(project_id, status).await?;

                let session_list: Vec<serde_json::Value> = sessions.iter().map(|s| {
                    json!({
                        "id": s.id,
                        "agent_type": s.agent_type.as_str(),
                        "session_type": s.session_type.as_str(),
                        "status": s.status.as_str(),
                        "project_id": s.project_id,
                        "working_dir": s.working_dir,
                        "created_at": s.created_at.to_rfc3339()
                    })
                }).collect();

                Ok(ToolCallResult {
                    content: vec![ContentBlock::Text {
                        text: json!({ "sessions": session_list }).to_string()
                    }]
                })
            }
            
            "send_message" => {
                let session_id = args["session_id"].as_str()
                    .ok_or_else(|| anyhow::anyhow!("session_id is required"))?;
                let content = args["content"].as_str()
                    .ok_or_else(|| anyhow::anyhow!("content is required"))?;

                if session_id.is_empty() {
                    return Err(anyhow::anyhow!("session_id cannot be empty"));
                }
                if content.is_empty() {
                    return Err(anyhow::anyhow!("content cannot be empty"));
                }

                // Get the session to find provider session ID
                let session = session_manager.repository().get(session_id).await?
                    .ok_or_else(|| anyhow::anyhow!("Session not found"))?;

                let provider_session_id = session.opencode_session_id
                    .ok_or_else(|| anyhow::anyhow!("No provider session ID"))?;

                let response = session_manager.send_message(
                    session_id,
                    &provider_session_id,
                    session.session_type.as_str(),
                    content
                ).await?;

                Ok(ToolCallResult {
                    content: vec![ContentBlock::Text { text: response }]
                })
            }
            
            "kill_session" => {
                let session_id = args["session_id"].as_str()
                    .ok_or_else(|| anyhow::anyhow!("session_id is required"))?;
                
                if session_id.is_empty() {
                    return Err(anyhow::anyhow!("session_id cannot be empty"));
                }

                // Kill the provider session first
                if let Some(session) = session_manager.repository().get(session_id).await? {
                    if let Some(provider_id) = session.opencode_session_id {
                        let _ = session_manager.kill_provider_session(&provider_id, session.session_type.as_str()).await;
                    }
                }
                
                session_manager.repository().update_status(
                    session_id,
                    crate::db::repositories::session::SessionStatus::Terminated
                ).await?;

                Ok(ToolCallResult {
                    content: vec![ContentBlock::Text {
                        text: json!({ "success": true, "session_id": session_id }).to_string()
                    }]
                })
            }
            
            "get_session" => {
                let session_id = args["session_id"].as_str()
                    .ok_or_else(|| anyhow::anyhow!("session_id is required"))?;

                if session_id.is_empty() {
                    return Err(anyhow::anyhow!("session_id cannot be empty"));
                }
                
                let session = session_manager.repository().get(session_id).await?
                    .ok_or_else(|| anyhow::anyhow!("Session not found"))?;

                Ok(ToolCallResult {
                    content: vec![ContentBlock::Text {
                        text: json!({
                            "id": session.id,
                            "agent_type": session.agent_type.as_str(),
                            "session_type": session.session_type.as_str(),
                            "status": session.status.as_str(),
                            "project_id": session.project_id,
                            "working_dir": session.working_dir,
                            "opencode_session_id": session.opencode_session_id,
                            "created_at": session.created_at.to_rfc3339(),
                            "updated_at": session.updated_at.to_rfc3339()
                        }).to_string()
                    }]
                })
            }
            
            "fork_session" => {
                let session_id = args["session_id"].as_str().unwrap_or("");
                
                // Get original session
                let original = session_manager.repository().get(session_id).await?
                    .ok_or_else(|| anyhow::anyhow!("Session not found"))?;

                let provider_session_id = original.opencode_session_id
                    .ok_or_else(|| anyhow::anyhow!("No provider session ID"))?;

                // Fork with provider
                let handle = session_manager.fork_session(
                    &provider_session_id,
                    original.session_type.as_str()
                ).await?;

                // Create new DB record
                let db = session_manager.repository().db().clone();
                let session_repo = crate::db::repositories::session::SessionRepository::new(db);
                
                let new_session = session_repo.create(
                    original.agent_type,
                    original.session_type,
                    original.project_id.clone(),
                    original.working_dir.clone(),
                ).await?;

                session_repo.set_opencode_session_id(&new_session.id, &handle.provider_id).await?;

                Ok(ToolCallResult {
                    content: vec![ContentBlock::Text {
                        text: json!({
                            "session_id": new_session.id,
                            "forked_from": session_id,
                            "status": "running"
                        }).to_string()
                    }]
                })
            }
            
            "list_projects" => {
                let db = session_manager.repository().db().clone();
                let project_repo = crate::db::repositories::project::ProjectRepository::new(db);
                
                let projects = project_repo.list().await?;

                let project_list: Vec<serde_json::Value> = projects.iter().map(|p| {
                    json!({
                        "id": p.id,
                        "name": p.name,
                        "description": p.description,
                        "created_at": p.created_at.to_rfc3339()
                    })
                }).collect();

                Ok(ToolCallResult {
                    content: vec![ContentBlock::Text {
                        text: json!({ "projects": project_list }).to_string()
                    }]
                })
            }
            
            "create_project" => {
                let name = args["name"].as_str().unwrap_or("");
                let description = args["description"].as_str().map(String::from);

                let db = session_manager.repository().db().clone();
                let project_repo = crate::db::repositories::project::ProjectRepository::new(db);
                
                let project = project_repo.create(name.to_string(), description).await?;

                Ok(ToolCallResult {
                    content: vec![ContentBlock::Text {
                        text: json!({
                            "id": project.id,
                            "name": project.name,
                            "description": project.description
                        }).to_string()
                    }]
                })
            }
            
            "run_quality_gates" => {
                let project_dir = args["project_dir"].as_str()
                    .ok_or_else(|| anyhow::anyhow!("project_dir is required"))?;
                
                let gate = args["gate"].as_str().unwrap_or("all");

                if project_dir.is_empty() {
                    return Err(anyhow::anyhow!("project_dir cannot be empty"));
                }

                use crate::agent::gates::QualityGates;

                let results = match gate {
                    "all" => {
                        QualityGates::run_all(project_dir)
                            .into_iter()
                            .map(|r| json!({
                                "gate": r.name,
                                "passed": r.passed,
                                "duration_ms": r.duration_ms,
                                "output": r.output
                            }))
                            .collect::<Vec<_>>()
                    }
                    "rust_check" => {
                        let r = QualityGates::rust_check(project_dir);
                        vec![json!({
                            "gate": r.name,
                            "passed": r.passed,
                            "duration_ms": r.duration_ms,
                            "output": r.output
                        })]
                    }
                    "rust_clippy" => {
                        let r = QualityGates::rust_clippy(project_dir);
                        vec![json!({
                            "gate": r.name,
                            "passed": r.passed,
                            "duration_ms": r.duration_ms,
                            "output": r.output
                        })]
                    }
                    "npm_lint" => {
                        let r = QualityGates::npm_lint(project_dir);
                        vec![json!({
                            "gate": r.name,
                            "passed": r.passed,
                            "duration_ms": r.duration_ms,
                            "output": r.output
                        })]
                    }
                    "npm_typecheck" => {
                        let r = QualityGates::npm_typecheck(project_dir);
                        vec![json!({
                            "gate": r.name,
                            "passed": r.passed,
                            "duration_ms": r.duration_ms,
                            "output": r.output
                        })]
                    }
                    "python_ruff" => {
                        let r = QualityGates::python_ruff(project_dir);
                        vec![json!({
                            "gate": r.name,
                            "passed": r.passed,
                            "duration_ms": r.duration_ms,
                            "output": r.output
                        })]
                    }
                    "python_mypy" => {
                        let r = QualityGates::python_mypy(project_dir);
                        vec![json!({
                            "gate": r.name,
                            "passed": r.passed,
                            "duration_ms": r.duration_ms,
                            "output": r.output
                        })]
                    }
                    "python_pytest" => {
                        let r = QualityGates::python_pytest(project_dir);
                        vec![json!({
                            "gate": r.name,
                            "passed": r.passed,
                            "duration_ms": r.duration_ms,
                            "output": r.output
                        })]
                    }
                    _ => return Err(anyhow::anyhow!("Unknown gate: {}. Valid options: all, rust_check, rust_clippy, npm_lint, npm_typecheck, python_ruff, python_mypy, python_pytest", gate)),
                };

                Ok(ToolCallResult {
                    content: vec![ContentBlock::Text {
                        text: json!({ "results": results }).to_string()
                    }]
                })
            }
            
            _ => Err(anyhow::anyhow!("Unknown tool: {}", tool_call.name)),
        }
    }
}
