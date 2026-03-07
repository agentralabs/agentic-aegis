use std::sync::Arc;
use tokio::sync::Mutex;

use serde_json::Value;

use agentic_aegis_core::types::{Language, SessionConfig};

use crate::session::McpSessionManager;
use crate::types::{McpError, McpResult, ToolCallResult};

pub async fn handle_session_create(
    args: &Value,
    session: &Arc<Mutex<McpSessionManager>>,
) -> McpResult<ToolCallResult> {
    let language_str = args
        .get("language")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidParams {
            message: "language is required".to_string(),
        })?;

    let language = Language::from_str_loose(language_str);
    let file_path = args
        .get("file_path")
        .and_then(|v| v.as_str())
        .map(String::from);
    let max_errors = args
        .get("max_errors")
        .and_then(|v| v.as_u64())
        .map(|v| v as usize)
        .unwrap_or(50);

    let config = SessionConfig {
        language,
        file_path,
        max_errors,
        ..Default::default()
    };

    let mut guard = session.lock().await;
    let session_id =
        guard
            .core
            .create_session(config)
            .map_err(|e| McpError::ToolExecutionError {
                message: e.to_string(),
            })?;

    let result = serde_json::json!({
        "session_id": session_id.to_string(),
        "language": language.as_str(),
        "status": "active",
    });

    Ok(ToolCallResult::success(
        serde_json::to_string_pretty(&result).map_err(|e| McpError::InternalError {
            message: e.to_string(),
        })?,
    ))
}

pub async fn handle_session_status(
    args: &Value,
    session: &Arc<Mutex<McpSessionManager>>,
) -> McpResult<ToolCallResult> {
    let session_id = args
        .get("session_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidParams {
            message: "session_id is required".to_string(),
        })?;

    let guard = session.lock().await;
    let s = guard
        .core
        .get_session(session_id)
        .map_err(|e| McpError::ToolExecutionError {
            message: e.to_string(),
        })?;

    let result = serde_json::json!({
        "session_id": s.id.to_string(),
        "state": format!("{:?}", s.state),
        "language": s.config.language.as_str(),
        "total_chunks": s.total_chunks_processed,
        "total_errors": s.total_errors,
        "total_warnings": s.total_warnings,
        "code_length": s.context.accumulated_code.len(),
        "created_at": s.created_at.to_rfc3339(),
        "updated_at": s.updated_at.to_rfc3339(),
    });

    Ok(ToolCallResult::success(
        serde_json::to_string_pretty(&result).map_err(|e| McpError::InternalError {
            message: e.to_string(),
        })?,
    ))
}

pub async fn handle_session_end(
    args: &Value,
    session: &Arc<Mutex<McpSessionManager>>,
) -> McpResult<ToolCallResult> {
    let session_id = args
        .get("session_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidParams {
            message: "session_id is required".to_string(),
        })?;

    let mut guard = session.lock().await;

    // Get session info before ending
    let info = {
        let s = guard
            .core
            .get_session(session_id)
            .map_err(|e| McpError::ToolExecutionError {
                message: e.to_string(),
            })?;
        serde_json::json!({
            "session_id": s.id.to_string(),
            "total_chunks": s.total_chunks_processed,
            "total_errors": s.total_errors,
            "total_warnings": s.total_warnings,
            "code_length": s.context.accumulated_code.len(),
        })
    };

    guard
        .core
        .end_session(session_id)
        .map_err(|e| McpError::ToolExecutionError {
            message: e.to_string(),
        })?;

    let mut result = info;
    if let Some(o) = result.as_object_mut() {
        o.insert("state".to_string(), serde_json::json!("Completed"));
    }

    Ok(ToolCallResult::success(
        serde_json::to_string_pretty(&result).map_err(|e| McpError::InternalError {
            message: e.to_string(),
        })?,
    ))
}

pub async fn handle_rollback(
    args: &Value,
    session: &Arc<Mutex<McpSessionManager>>,
) -> McpResult<ToolCallResult> {
    let session_id = args
        .get("session_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidParams {
            message: "session_id is required".to_string(),
        })?;

    let target = args
        .get("target")
        .and_then(|v| v.as_str())
        .unwrap_or("latest");

    let guard = session.lock().await;

    let snapshot = match target {
        "latest" => guard.rollback_engine.rollback_to_latest().map_err(|e| {
            McpError::ToolExecutionError {
                message: e.to_string(),
            }
        })?,
        "chunk" => {
            let chunk_index = args
                .get("value")
                .and_then(|v| v.as_str())
                .and_then(|v| v.parse::<usize>().ok())
                .ok_or_else(|| McpError::InvalidParams {
                    message: "value must be a valid chunk index".to_string(),
                })?;
            guard
                .rollback_engine
                .rollback_to_chunk(chunk_index)
                .map_err(|e| McpError::ToolExecutionError {
                    message: e.to_string(),
                })?
        }
        _ => {
            let snapshot_id = args.get("value").and_then(|v| v.as_str()).unwrap_or(target);
            guard
                .rollback_engine
                .rollback_to(snapshot_id)
                .map_err(|e| McpError::ToolExecutionError {
                    message: e.to_string(),
                })?
        }
    };

    let result = serde_json::json!({
        "session_id": session_id,
        "rolled_back_to_chunk": snapshot.chunk_index,
        "code_length": snapshot.code.len(),
        "timestamp": snapshot.timestamp.to_rfc3339(),
    });

    Ok(ToolCallResult::success(
        serde_json::to_string_pretty(&result).map_err(|e| McpError::InternalError {
            message: e.to_string(),
        })?,
    ))
}
