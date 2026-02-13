//! MCP protocol types

use serde::{Deserialize, Serialize};

/// JSON-RPC request
#[derive(Debug, Deserialize, Serialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: serde_json::Value,
    pub method: String,
    #[serde(default)]
    pub params: serde_json::Value,
}

/// JSON-RPC response
#[derive(Debug, Deserialize, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

impl JsonRpcResponse {
    pub fn success(id: serde_json::Value, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn error(id: serde_json::Value, code: i32, message: &str) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.to_string(),
                data: None,
            }),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// MCP method types
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum McpMethod {
    Initialize,
    ToolsList,
    ToolsCall,
    ResourcesList,
    ResourcesRead,
    PromptGet,
}

impl McpMethod {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "initialize" => Some(Self::Initialize),
            "tools/list" => Some(Self::ToolsList),
            "tools/call" => Some(Self::ToolsCall),
            "resources/list" => Some(Self::ResourcesList),
            "resources/read" => Some(Self::ResourcesRead),
            "prompt/get" => Some(Self::PromptGet),
            _ => None,
        }
    }
}

/// Tool definition
#[derive(Debug, Deserialize, Serialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

/// Tool call parameters
#[derive(Debug, Deserialize, Serialize)]
pub struct ToolCall {
    pub name: String,
    pub arguments: serde_json::Value,
}

/// Initialize request params
#[derive(Debug, Deserialize, Serialize)]
pub struct InitializeParams {
    pub protocol_version: Option<String>,
    pub capabilities: Option<serde_json::Value>,
    pub client_info: Option<serde_json::Value>,
}

/// Initialize result
#[derive(Debug, Deserialize, Serialize)]
pub struct InitializeResult {
    pub protocol_version: String,
    pub capabilities: Capabilities,
    pub server_info: ServerInfo,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Capabilities {
    #[serde(default)]
    pub tools: bool,
    #[serde(default)]
    pub resources: bool,
    #[serde(default)]
    pub prompts: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
}

/// Tools list result
#[derive(Debug, Deserialize, Serialize)]
pub struct ToolsListResult {
    pub tools: Vec<Tool>,
}

/// Tool call result
#[derive(Debug, Deserialize, Serialize)]
pub struct ToolCallResult {
    pub content: Vec<ContentBlock>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum ContentBlock {
    Text { text: String },
    Image { data: String, mime_type: String },
    Resource { uri: String, mime_type: String },
}
