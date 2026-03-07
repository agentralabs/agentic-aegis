use std::sync::Arc;
use tokio::sync::Mutex;

use serde_json::Value;

use crate::session::McpSessionManager;
use crate::types::{McpError, McpResult, ToolCallResult};

use super::security;
use super::session as session_tools;
use super::shadow;
use super::validation;

pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

pub struct ToolRegistry;

impl ToolRegistry {
    pub fn list_tools() -> Vec<ToolDefinition> {
        vec![
            ToolDefinition {
                name: "aegis_validate_streaming".into(),
                description: "Validate a code chunk during streaming generation".into(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "session_id": { "type": "string", "description": "Validation session ID" },
                        "chunk": { "type": "string", "description": "Code chunk to validate" }
                    },
                    "required": ["session_id", "chunk"]
                }),
            },
            ToolDefinition {
                name: "aegis_validate_complete".into(),
                description: "Validate a complete code block".into(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "code": { "type": "string", "description": "Complete code to validate" },
                        "language": { "type": "string", "description": "Programming language" }
                    },
                    "required": ["code", "language"]
                }),
            },
            ToolDefinition {
                name: "aegis_shadow_execute".into(),
                description: "Execute code in a shadow sandbox environment".into(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "code": { "type": "string", "description": "Code to execute" },
                        "language": { "type": "string", "description": "Programming language" }
                    },
                    "required": ["code", "language"]
                }),
            },
            ToolDefinition {
                name: "aegis_check_input".into(),
                description: "Check input for security threats and prompt injection".into(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "input": { "type": "string", "description": "Input to check" },
                        "check_type": { "type": "string", "description": "Type of check: prompt_injection, payload, intent" },
                        "include_content": { "type": "boolean", "default": false },
                        "intent": { "type": "string", "enum": ["exists", "ids", "summary", "full"] },
                        "since": { "type": "integer" },
                        "token_budget": { "type": "integer" },
                        "max_results": { "type": "integer", "default": 10 },
                        "cursor": { "type": "string" }
                    },
                    "required": ["input"]
                }),
            },
            ToolDefinition {
                name: "aegis_check_output".into(),
                description: "Check output before delivery for PII and safety issues".into(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "output": { "type": "string", "description": "Output to check" },
                        "check_type": { "type": "string", "description": "Type of check: pii, content, sanitize" },
                        "include_content": { "type": "boolean", "default": false },
                        "intent": { "type": "string", "enum": ["exists", "ids", "summary", "full"] },
                        "since": { "type": "integer" },
                        "token_budget": { "type": "integer" },
                        "max_results": { "type": "integer", "default": 10 },
                        "cursor": { "type": "string" }
                    },
                    "required": ["output"]
                }),
            },
            ToolDefinition {
                name: "aegis_session_create".into(),
                description: "Create a new validation session".into(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "language": { "type": "string", "description": "Programming language" },
                        "file_path": { "type": "string", "description": "Optional file path context" },
                        "max_errors": { "type": "integer", "description": "Maximum errors before stopping" }
                    },
                    "required": ["language"]
                }),
            },
            ToolDefinition {
                name: "aegis_session_status".into(),
                description: "Get the status of a validation session".into(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "session_id": { "type": "string", "description": "Session ID to query" },
                        "include_content": { "type": "boolean", "default": false },
                        "intent": { "type": "string", "enum": ["exists", "ids", "summary", "full"] },
                        "since": { "type": "integer" },
                        "token_budget": { "type": "integer" },
                        "max_results": { "type": "integer", "default": 10 },
                        "cursor": { "type": "string" }
                    },
                    "required": ["session_id"]
                }),
            },
            ToolDefinition {
                name: "aegis_session_end".into(),
                description: "End a validation session and get final results".into(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "session_id": { "type": "string", "description": "Session ID to end" }
                    },
                    "required": ["session_id"]
                }),
            },
            ToolDefinition {
                name: "aegis_correction_hint".into(),
                description: "Get a correction hint for a validation error".into(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "error_message": { "type": "string", "description": "The error message" },
                        "language": { "type": "string", "description": "Programming language" },
                        "code_context": { "type": "string", "description": "Surrounding code context" },
                        "include_content": { "type": "boolean", "default": false },
                        "intent": { "type": "string", "enum": ["exists", "ids", "summary", "full"] },
                        "since": { "type": "integer" },
                        "token_budget": { "type": "integer" },
                        "max_results": { "type": "integer", "default": 10 },
                        "cursor": { "type": "string" }
                    },
                    "required": ["error_message", "language"]
                }),
            },
            ToolDefinition {
                name: "aegis_confidence_score".into(),
                description: "Get a confidence score for generated code".into(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "code": { "type": "string", "description": "Code to score" },
                        "language": { "type": "string", "description": "Programming language" },
                        "include_content": { "type": "boolean", "default": false },
                        "intent": { "type": "string", "enum": ["exists", "ids", "summary", "full"] },
                        "since": { "type": "integer" },
                        "token_budget": { "type": "integer" },
                        "max_results": { "type": "integer", "default": 10 },
                        "cursor": { "type": "string" }
                    },
                    "required": ["code", "language"]
                }),
            },
            ToolDefinition {
                name: "aegis_rollback".into(),
                description: "Rollback to a previous valid state in the session".into(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "session_id": { "type": "string", "description": "Session ID" },
                        "target": { "type": "string", "description": "Rollback target: latest, chunk_index, or snapshot_id" },
                        "value": { "type": "string", "description": "Target value (chunk index or snapshot ID)" }
                    },
                    "required": ["session_id"]
                }),
            },
            ToolDefinition {
                name: "aegis_scan_security".into(),
                description: "Scan code for security vulnerabilities".into(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "code": { "type": "string", "description": "Code to scan" },
                        "language": { "type": "string", "description": "Programming language" },
                        "include_content": { "type": "boolean", "default": false },
                        "intent": { "type": "string", "enum": ["exists", "ids", "summary", "full"] },
                        "since": { "type": "integer" },
                        "token_budget": { "type": "integer" },
                        "max_results": { "type": "integer", "default": 10 },
                        "cursor": { "type": "string" }
                    },
                    "required": ["code", "language"]
                }),
            },
        ]
    }

    pub async fn call(
        name: &str,
        arguments: Option<Value>,
        session: &Arc<Mutex<McpSessionManager>>,
    ) -> McpResult<Value> {
        let args = arguments.unwrap_or(serde_json::json!({}));

        let result = match name {
            "aegis_validate_streaming" => {
                validation::handle_validate_streaming(&args, session).await
            }
            "aegis_validate_complete" => validation::handle_validate_complete(&args, session).await,
            "aegis_shadow_execute" => shadow::handle_shadow_execute(&args, session).await,
            "aegis_check_input" => security::handle_check_input(&args, session).await,
            "aegis_check_output" => security::handle_check_output(&args, session).await,
            "aegis_session_create" => session_tools::handle_session_create(&args, session).await,
            "aegis_session_status" => session_tools::handle_session_status(&args, session).await,
            "aegis_session_end" => session_tools::handle_session_end(&args, session).await,
            "aegis_correction_hint" => validation::handle_correction_hint(&args, session).await,
            "aegis_confidence_score" => validation::handle_confidence_score(&args, session).await,
            "aegis_rollback" => session_tools::handle_rollback(&args, session).await,
            "aegis_scan_security" => security::handle_scan_security(&args, session).await,
            _ => {
                return Err(McpError::ToolNotFound {
                    tool: name.to_string(),
                })
            }
        };

        match result {
            Ok(r) => Ok(r.to_value()),
            Err(e) => Ok(ToolCallResult::error(e.to_string()).to_value()),
        }
    }
}
