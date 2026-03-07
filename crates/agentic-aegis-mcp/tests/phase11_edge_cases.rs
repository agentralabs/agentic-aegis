//! Phase 11: MCP edge cases — malformed inputs, protocol abuse, concurrent access

use std::sync::Arc;
use tokio::sync::Mutex;

use agentic_aegis_mcp::protocol::ProtocolHandler;
use agentic_aegis_mcp::session::McpSessionManager;
use agentic_aegis_mcp::tools::registry::ToolRegistry;

fn make_session() -> Arc<Mutex<McpSessionManager>> {
    Arc::new(Mutex::new(McpSessionManager::new()))
}

fn make_handler() -> ProtocolHandler {
    ProtocolHandler::new(make_session())
}

// ═══════════════════════════════════════════════════════════════════════
// EDGE: Malformed JSON-RPC requests
// ═══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn edge_missing_method_field() {
    let handler = make_handler();
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1
    });
    let response = handler.handle_request(request).await;
    // Should handle gracefully — empty string method → method not found
    assert!(response["error"].is_object() || response.is_null());
}

#[tokio::test]
async fn edge_null_id() {
    let handler = make_handler();
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": null,
        "method": "initialize",
        "params": {}
    });
    let response = handler.handle_request(request).await;
    assert!(response["result"].is_object());
}

#[tokio::test]
async fn edge_string_id() {
    let handler = make_handler();
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": "abc-123",
        "method": "initialize",
        "params": {}
    });
    let response = handler.handle_request(request).await;
    assert_eq!(response["id"], "abc-123");
}

#[tokio::test]
async fn edge_missing_params() {
    let handler = make_handler();
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/list"
    });
    let response = handler.handle_request(request).await;
    // Should still work — params is optional for list
    let tools = response["result"]["tools"].as_array();
    assert!(tools.is_some());
}

#[tokio::test]
async fn edge_empty_params() {
    let handler = make_handler();
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {}
    });
    let response = handler.handle_request(request).await;
    // Missing tool name should return error
    assert!(response["error"].is_object());
}

#[tokio::test]
async fn edge_tool_call_null_arguments() {
    let handler = make_handler();
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "aegis_session_create",
            "arguments": null
        }
    });
    let response = handler.handle_request(request).await;
    // Should handle null arguments gracefully
    assert!(response["result"].is_object() || response["error"].is_object());
}

#[tokio::test]
async fn edge_tool_call_empty_arguments() {
    let session = make_session();
    let result = ToolRegistry::call(
        "aegis_confidence_score",
        Some(serde_json::json!({})),
        &session,
    )
    .await;
    // Missing required params should return error content, not panic
    assert!(result.is_ok());
    let val = result.unwrap();
    assert!(val.get("isError").is_some());
}

#[tokio::test]
async fn edge_tool_call_wrong_type_arguments() {
    let session = make_session();
    let result = ToolRegistry::call(
        "aegis_session_create",
        Some(serde_json::json!({"language": 42})), // number instead of string
        &session,
    )
    .await;
    // Should handle type mismatch gracefully
    assert!(result.is_ok());
    let val = result.unwrap();
    assert!(val.get("isError").is_some());
}

#[tokio::test]
async fn edge_validate_streaming_invalid_session() {
    let session = make_session();
    let result = ToolRegistry::call(
        "aegis_validate_streaming",
        Some(serde_json::json!({
            "session_id": "nonexistent-session-id",
            "chunk": "fn main() {}"
        })),
        &session,
    )
    .await;
    assert!(result.is_ok());
    let val = result.unwrap();
    assert!(val.get("isError").is_some());
}

#[tokio::test]
async fn edge_session_end_already_ended() {
    let session = make_session();

    // Create session
    let create_result = ToolRegistry::call(
        "aegis_session_create",
        Some(serde_json::json!({"language": "rust"})),
        &session,
    )
    .await
    .unwrap();
    let text = create_result["content"][0]["text"].as_str().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(text).unwrap();
    let session_id = parsed["session_id"].as_str().unwrap();

    // End it
    let _ = ToolRegistry::call(
        "aegis_session_end",
        Some(serde_json::json!({"session_id": session_id})),
        &session,
    )
    .await;

    // Try to end again — should return error
    let result = ToolRegistry::call(
        "aegis_session_end",
        Some(serde_json::json!({"session_id": session_id})),
        &session,
    )
    .await;
    assert!(result.is_ok());
    let val = result.unwrap();
    assert!(val.get("isError").is_some());
}

#[tokio::test]
async fn edge_rollback_empty_session() {
    let session = make_session();
    let result = ToolRegistry::call(
        "aegis_rollback",
        Some(serde_json::json!({
            "session_id": "nonexistent",
            "target": "latest"
        })),
        &session,
    )
    .await;
    assert!(result.is_ok());
    let val = result.unwrap();
    assert!(val.get("isError").is_some());
}

// ═══════════════════════════════════════════════════════════════════════
// EDGE: Security scanning edge cases
// ═══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn edge_check_input_empty() {
    let session = make_session();
    let result = ToolRegistry::call(
        "aegis_check_input",
        Some(serde_json::json!({"input": ""})),
        &session,
    )
    .await
    .unwrap();
    let text = result["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("\"safe\": true"));
}

#[tokio::test]
async fn edge_check_output_empty() {
    let session = make_session();
    let result = ToolRegistry::call(
        "aegis_check_output",
        Some(serde_json::json!({"output": ""})),
        &session,
    )
    .await
    .unwrap();
    assert!(!result["content"][0]["text"].as_str().unwrap().is_empty());
}

#[tokio::test]
async fn edge_scan_security_empty_code() {
    let session = make_session();
    let result = ToolRegistry::call(
        "aegis_scan_security",
        Some(serde_json::json!({"code": "", "language": "rust"})),
        &session,
    )
    .await
    .unwrap();
    let text = result["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("\"is_safe\": true"));
}

#[tokio::test]
async fn edge_confidence_score_empty_code() {
    let session = make_session();
    let result = ToolRegistry::call(
        "aegis_confidence_score",
        Some(serde_json::json!({"code": "", "language": "rust"})),
        &session,
    )
    .await
    .unwrap();
    let text = result["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("confidence"));
}

#[tokio::test]
async fn edge_correction_hint_empty_error() {
    let session = make_session();
    let result = ToolRegistry::call(
        "aegis_correction_hint",
        Some(serde_json::json!({
            "error_message": "",
            "language": "rust"
        })),
        &session,
    )
    .await
    .unwrap();
    let text = result["content"][0]["text"].as_str().unwrap();
    assert!(text.contains("hint"));
}

// ═══════════════════════════════════════════════════════════════════════
// STRESS: Rapid sequential tool calls
// ═══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn stress_rapid_session_create_end() {
    let session = make_session();
    for _ in 0..50 {
        let create_result = ToolRegistry::call(
            "aegis_session_create",
            Some(serde_json::json!({"language": "rust"})),
            &session,
        )
        .await
        .unwrap();
        let text = create_result["content"][0]["text"].as_str().unwrap();
        let parsed: serde_json::Value = serde_json::from_str(text).unwrap();
        let sid = parsed["session_id"].as_str().unwrap();

        let _ = ToolRegistry::call(
            "aegis_session_end",
            Some(serde_json::json!({"session_id": sid})),
            &session,
        )
        .await;
    }
}

#[tokio::test]
async fn stress_rapid_validate_complete() {
    let session = make_session();
    for i in 0..30 {
        let code = format!("fn func_{i}() -> i32 {{ {i} }}");
        let result = ToolRegistry::call(
            "aegis_validate_complete",
            Some(serde_json::json!({"code": code, "language": "rust"})),
            &session,
        )
        .await
        .unwrap();
        assert!(result["content"][0]["text"].as_str().is_some());
    }
}

#[tokio::test]
async fn stress_rapid_security_scans() {
    let session = make_session();
    for i in 0..30 {
        let code = format!("let x_{i} = compute({i});");
        let result = ToolRegistry::call(
            "aegis_scan_security",
            Some(serde_json::json!({"code": code, "language": "rust"})),
            &session,
        )
        .await
        .unwrap();
        let text = result["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("is_safe"));
    }
}

#[tokio::test]
async fn stress_full_streaming_session() {
    let session = make_session();

    // Create session
    let create = ToolRegistry::call(
        "aegis_session_create",
        Some(serde_json::json!({"language": "rust"})),
        &session,
    )
    .await
    .unwrap();
    let text = create["content"][0]["text"].as_str().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(text).unwrap();
    let sid = parsed["session_id"].as_str().unwrap().to_string();

    // Stream 20 chunks
    let chunks = vec![
        "fn main() {\n",
        "    let mut sum: i32 = 0;\n",
        "    for i in 0..100 {\n",
        "        sum += i;\n",
        "    }\n",
        "    println!(\"{}\", sum);\n",
        "}\n",
        "\n",
        "fn add(a: i32, b: i32) -> i32 {\n",
        "    a + b\n",
        "}\n",
        "\n",
        "fn multiply(a: i32, b: i32) -> i32 {\n",
        "    a * b\n",
        "}\n",
        "\n",
        "fn divide(a: f64, b: f64) -> f64 {\n",
        "    a / b\n",
        "}\n",
        "\n",
    ];

    for chunk in &chunks {
        let result = ToolRegistry::call(
            "aegis_validate_streaming",
            Some(serde_json::json!({"session_id": &sid, "chunk": chunk})),
            &session,
        )
        .await
        .unwrap();
        assert!(result["content"][0]["text"].as_str().is_some());
    }

    // Check status
    let status = ToolRegistry::call(
        "aegis_session_status",
        Some(serde_json::json!({"session_id": &sid})),
        &session,
    )
    .await
    .unwrap();
    let status_text = status["content"][0]["text"].as_str().unwrap();
    let status_parsed: serde_json::Value = serde_json::from_str(status_text).unwrap();
    assert_eq!(status_parsed["total_chunks"].as_u64().unwrap(), 20);

    // End session
    let end = ToolRegistry::call(
        "aegis_session_end",
        Some(serde_json::json!({"session_id": &sid})),
        &session,
    )
    .await
    .unwrap();
    let end_text = end["content"][0]["text"].as_str().unwrap();
    assert!(end_text.contains("Completed"));
}

// ═══════════════════════════════════════════════════════════════════════
// EDGE: Protocol handler edge cases
// ═══════════════════════════════════════════════════════════════════════

#[tokio::test]
async fn edge_protocol_batch_requests() {
    let handler = make_handler();
    // Send multiple requests in sequence
    for i in 1..=10 {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": i,
            "method": "tools/list",
            "params": {}
        });
        let response = handler.handle_request(request).await;
        assert_eq!(response["id"], i);
        assert_eq!(response["result"]["tools"].as_array().unwrap().len(), 12);
    }
}

#[tokio::test]
async fn edge_protocol_initialize_multiple() {
    let handler = make_handler();
    // Initialize can be called multiple times
    for _ in 0..5 {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {}
        });
        let response = handler.handle_request(request).await;
        assert_eq!(response["result"]["protocolVersion"], "2024-11-05");
    }
}

#[tokio::test]
async fn edge_all_12_tools_callable() {
    let session = make_session();
    let tools = ToolRegistry::list_tools();
    assert_eq!(tools.len(), 12);
    for tool in &tools {
        // Call each tool with empty args — should not panic, just return error
        let result = ToolRegistry::call(&tool.name, Some(serde_json::json!({})), &session).await;
        assert!(result.is_ok(), "tool {} panicked on empty args", tool.name);
    }
}
