use std::sync::Arc;
use tokio::sync::Mutex;

use agentic_aegis_mcp::protocol::ProtocolHandler;
use agentic_aegis_mcp::session::McpSessionManager;

fn make_handler() -> ProtocolHandler {
    let session = Arc::new(Mutex::new(McpSessionManager::new()));
    ProtocolHandler::new(session)
}

#[tokio::test]
async fn test_protocol_initialize() {
    let handler = make_handler();
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {}
    });
    let response = handler.handle_request(request).await;
    assert_eq!(response["jsonrpc"], "2.0");
    assert_eq!(response["id"], 1);
    assert!(response["result"]["capabilities"].is_object());
    assert!(response["result"]["serverInfo"]["name"]
        .as_str()
        .unwrap()
        .contains("aegis"));
}

#[tokio::test]
async fn test_protocol_list_tools() {
    let handler = make_handler();
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list",
        "params": {}
    });
    let response = handler.handle_request(request).await;
    assert_eq!(response["id"], 2);
    let tools = response["result"]["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 12);
}

#[tokio::test]
async fn test_protocol_list_resources() {
    let handler = make_handler();
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 3,
        "method": "resources/list",
        "params": {}
    });
    let response = handler.handle_request(request).await;
    assert_eq!(response["id"], 3);
    assert!(response["result"]["resources"]
        .as_array()
        .unwrap()
        .is_empty());
}

#[tokio::test]
async fn test_protocol_list_prompts() {
    let handler = make_handler();
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 4,
        "method": "prompts/list",
        "params": {}
    });
    let response = handler.handle_request(request).await;
    assert_eq!(response["id"], 4);
    assert!(response["result"]["prompts"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_protocol_unknown_method() {
    let handler = make_handler();
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 5,
        "method": "unknown/method",
        "params": {}
    });
    let response = handler.handle_request(request).await;
    assert_eq!(response["error"]["code"], -32601);
}

#[tokio::test]
async fn test_protocol_tool_call() {
    let handler = make_handler();
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 6,
        "method": "tools/call",
        "params": {
            "name": "aegis_session_create",
            "arguments": {
                "language": "rust"
            }
        }
    });
    let response = handler.handle_request(request).await;
    assert_eq!(response["id"], 6);
    assert!(response["result"].is_object());
}

#[tokio::test]
async fn test_protocol_tool_call_unknown_tool() {
    let handler = make_handler();
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 7,
        "method": "tools/call",
        "params": {
            "name": "nonexistent_tool",
            "arguments": {}
        }
    });
    let response = handler.handle_request(request).await;
    assert_eq!(response["error"]["code"], -32803);
}

#[tokio::test]
async fn test_protocol_tool_call_missing_name() {
    let handler = make_handler();
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 8,
        "method": "tools/call",
        "params": {}
    });
    let response = handler.handle_request(request).await;
    assert!(response["error"].is_object());
}

#[tokio::test]
async fn test_protocol_notification_initialized() {
    let handler = make_handler();
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized"
    });
    let response = handler.handle_request(request).await;
    assert!(response.is_null()); // Notifications don't get responses
}

#[tokio::test]
async fn test_protocol_notification_cancelled() {
    let handler = make_handler();
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "notifications/cancelled"
    });
    let response = handler.handle_request(request).await;
    assert!(response.is_null());
}

#[tokio::test]
async fn test_protocol_full_validation_flow() {
    let handler = make_handler();

    // Initialize
    let init = serde_json::json!({
        "jsonrpc": "2.0", "id": 1,
        "method": "initialize", "params": {}
    });
    let init_resp = handler.handle_request(init).await;
    assert!(init_resp["result"]["capabilities"].is_object());

    // Create session
    let create = serde_json::json!({
        "jsonrpc": "2.0", "id": 2,
        "method": "tools/call",
        "params": {
            "name": "aegis_session_create",
            "arguments": { "language": "rust" }
        }
    });
    let create_resp = handler.handle_request(create).await;
    let text = create_resp["result"]["content"][0]["text"]
        .as_str()
        .unwrap();
    let parsed: serde_json::Value = serde_json::from_str(text).unwrap();
    let session_id = parsed["session_id"].as_str().unwrap().to_string();

    // Validate chunk
    let validate = serde_json::json!({
        "jsonrpc": "2.0", "id": 3,
        "method": "tools/call",
        "params": {
            "name": "aegis_validate_streaming",
            "arguments": {
                "session_id": session_id,
                "chunk": "fn main() { }\n"
            }
        }
    });
    let val_resp = handler.handle_request(validate).await;
    assert!(val_resp["result"].is_object());

    // End session
    let end = serde_json::json!({
        "jsonrpc": "2.0", "id": 4,
        "method": "tools/call",
        "params": {
            "name": "aegis_session_end",
            "arguments": { "session_id": session_id }
        }
    });
    let end_resp = handler.handle_request(end).await;
    assert!(end_resp["result"].is_object());
}

#[tokio::test]
async fn test_protocol_version_in_initialize() {
    let handler = make_handler();
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {}
    });
    let response = handler.handle_request(request).await;
    assert_eq!(response["result"]["protocolVersion"], "2024-11-05");
}
