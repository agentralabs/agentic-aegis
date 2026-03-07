use std::sync::Arc;
use tokio::sync::Mutex;

use serde_json::Value;

use crate::session::McpSessionManager;
use crate::tools::registry::ToolRegistry;
use crate::types::McpError;

pub struct ProtocolHandler {
    session: Arc<Mutex<McpSessionManager>>,
}

impl ProtocolHandler {
    pub fn new(session: Arc<Mutex<McpSessionManager>>) -> Self {
        Self { session }
    }

    pub async fn handle_request(&self, request: Value) -> Value {
        let id = request.get("id").cloned().unwrap_or(Value::Null);
        let method = request.get("method").and_then(|v| v.as_str()).unwrap_or("");

        match method {
            "initialize" => self.handle_initialize(id),
            "tools/list" => self.handle_list_tools(id),
            "tools/call" => self.handle_tool_call(id, &request).await,
            "resources/list" => self.handle_list_resources(id),
            "prompts/list" => self.handle_list_prompts(id),
            "notifications/initialized" | "notifications/cancelled" => Value::Null,
            _ => McpError::MethodNotFound {
                method: method.to_string(),
            }
            .to_json_rpc_error(id),
        }
    }

    fn handle_initialize(&self, id: Value) -> Value {
        serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": { "listChanged": false },
                    "resources": { "listChanged": false },
                    "prompts": { "listChanged": false }
                },
                "serverInfo": {
                    "name": "agentic-aegis",
                    "version": env!("CARGO_PKG_VERSION")
                }
            }
        })
    }

    fn handle_list_tools(&self, id: Value) -> Value {
        let tools = ToolRegistry::list_tools();
        let tool_list: Vec<Value> = tools
            .iter()
            .map(|t| {
                serde_json::json!({
                    "name": t.name,
                    "description": t.description,
                    "inputSchema": t.input_schema
                })
            })
            .collect();

        serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "tools": tool_list
            }
        })
    }

    async fn handle_tool_call(&self, id: Value, request: &Value) -> Value {
        let params = request
            .get("params")
            .cloned()
            .unwrap_or(serde_json::json!({}));

        let tool_name = match params.get("name").and_then(|v| v.as_str()) {
            Some(name) => name.to_string(),
            None => {
                return McpError::InvalidParams {
                    message: "tool name is required".to_string(),
                }
                .to_json_rpc_error(id);
            }
        };

        let arguments = params.get("arguments").cloned();

        match ToolRegistry::call(&tool_name, arguments, &self.session).await {
            Ok(result) => {
                serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": result
                })
            }
            Err(e) => e.to_json_rpc_error(id),
        }
    }

    fn handle_list_resources(&self, id: Value) -> Value {
        serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "resources": []
            }
        })
    }

    fn handle_list_prompts(&self, id: Value) -> Value {
        serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "prompts": []
            }
        })
    }
}
