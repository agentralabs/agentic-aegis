use agentic_aegis_mcp::types::*;

#[test]
fn test_mcp_error_method_not_found() {
    let err = McpError::MethodNotFound {
        method: "foo".to_string(),
    };
    assert_eq!(err.code(), -32601);
    assert!(err.message().contains("foo"));
}

#[test]
fn test_mcp_error_invalid_params() {
    let err = McpError::InvalidParams {
        message: "missing field".to_string(),
    };
    assert_eq!(err.code(), -32602);
    assert!(err.message().contains("missing field"));
}

#[test]
fn test_mcp_error_tool_not_found() {
    let err = McpError::ToolNotFound {
        tool: "unknown_tool".to_string(),
    };
    assert_eq!(err.code(), -32803);
    assert!(err.message().contains("unknown_tool"));
}

#[test]
fn test_mcp_error_tool_execution() {
    let err = McpError::ToolExecutionError {
        message: "failed".to_string(),
    };
    assert_eq!(err.code(), -32000);
}

#[test]
fn test_mcp_error_internal() {
    let err = McpError::InternalError {
        message: "boom".to_string(),
    };
    assert_eq!(err.code(), -32603);
}

#[test]
fn test_mcp_error_to_json_rpc() {
    let err = McpError::ToolNotFound {
        tool: "test".to_string(),
    };
    let json = err.to_json_rpc_error(serde_json::json!(1));
    assert_eq!(json["jsonrpc"], "2.0");
    assert_eq!(json["id"], 1);
    assert_eq!(json["error"]["code"], -32803);
}

#[test]
fn test_mcp_error_display() {
    let err = McpError::ToolNotFound {
        tool: "test".to_string(),
    };
    assert!(err.to_string().contains("test"));
}

#[test]
fn test_tool_call_result_success() {
    let result = ToolCallResult::success("hello".to_string());
    assert!(result.is_error.is_none());
    assert_eq!(result.content.len(), 1);
    assert_eq!(result.content[0].text, "hello");
}

#[test]
fn test_tool_call_result_error() {
    let result = ToolCallResult::error("bad".to_string());
    assert_eq!(result.is_error, Some(true));
    assert_eq!(result.content[0].text, "bad");
}

#[test]
fn test_tool_call_result_to_value() {
    let result = ToolCallResult::success("test".to_string());
    let value = result.to_value();
    assert!(value.is_object());
}

#[test]
fn test_tool_content_type() {
    let content = ToolContent {
        content_type: "text".to_string(),
        text: "hello".to_string(),
    };
    assert_eq!(content.content_type, "text");
}
