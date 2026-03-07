use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug)]
pub enum McpError {
    MethodNotFound { method: String },
    InvalidParams { message: String },
    ToolNotFound { tool: String },
    ToolExecutionError { message: String },
    InternalError { message: String },
}

impl McpError {
    pub fn code(&self) -> i32 {
        match self {
            Self::MethodNotFound { .. } => -32601,
            Self::InvalidParams { .. } => -32602,
            Self::ToolNotFound { .. } => -32803,
            Self::ToolExecutionError { .. } => -32000,
            Self::InternalError { .. } => -32603,
        }
    }

    pub fn message(&self) -> String {
        match self {
            Self::MethodNotFound { method } => format!("method not found: {}", method),
            Self::InvalidParams { message } => format!("invalid params: {}", message),
            Self::ToolNotFound { tool } => format!("tool not found: {}", tool),
            Self::ToolExecutionError { message } => format!("tool execution error: {}", message),
            Self::InternalError { message } => format!("internal error: {}", message),
        }
    }

    pub fn to_json_rpc_error(&self, id: Value) -> Value {
        serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": {
                "code": self.code(),
                "message": self.message()
            }
        })
    }
}

impl std::fmt::Display for McpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message())
    }
}

impl std::error::Error for McpError {}

pub type McpResult<T> = Result<T, McpError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolContent {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallResult {
    pub content: Vec<ToolContent>,
    #[serde(rename = "isError", skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}

impl ToolCallResult {
    pub fn success(text: String) -> Self {
        Self {
            content: vec![ToolContent {
                content_type: "text".to_string(),
                text,
            }],
            is_error: None,
        }
    }

    pub fn error(text: String) -> Self {
        Self {
            content: vec![ToolContent {
                content_type: "text".to_string(),
                text,
            }],
            is_error: Some(true),
        }
    }

    pub fn to_value(&self) -> Value {
        serde_json::to_value(self).unwrap_or_else(|_| serde_json::json!({"error": "serialization failed"}))
    }
}
