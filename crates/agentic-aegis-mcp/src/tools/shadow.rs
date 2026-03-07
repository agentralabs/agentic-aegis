use std::sync::Arc;
use tokio::sync::Mutex;

use serde_json::Value;

use agentic_aegis_core::types::Language;

use crate::session::McpSessionManager;
use crate::types::{McpError, McpResult, ToolCallResult};

pub async fn handle_shadow_execute(
    args: &Value,
    session: &Arc<Mutex<McpSessionManager>>,
) -> McpResult<ToolCallResult> {
    let code =
        args.get("code")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams {
                message: "code is required".to_string(),
            })?;

    let language_str = args
        .get("language")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidParams {
            message: "language is required".to_string(),
        })?;

    let language = Language::from_str_loose(language_str);

    let guard = session.lock().await;

    // Track side effects first
    let effects = guard.effect_tracker.analyze(code, &language);
    let has_dangerous = effects.iter().any(|e| e.is_dangerous());

    if has_dangerous {
        let effect_desc: Vec<String> = effects
            .iter()
            .filter(|e| e.is_dangerous())
            .map(|e| e.category().to_string())
            .collect();

        return Ok(ToolCallResult::error(format!(
            "code contains dangerous side effects: {}. Shadow execution blocked for safety.",
            effect_desc.join(", ")
        )));
    }

    // Execute in sandbox
    let result = guard.executor.execute(code, &language).await.map_err(|e| {
        McpError::ToolExecutionError {
            message: e.to_string(),
        }
    })?;

    let response = serde_json::json!({
        "success": result.success,
        "stdout": result.stdout,
        "stderr": result.stderr,
        "exit_code": result.exit_code,
        "duration_ms": result.duration_ms,
        "effects": effects.iter().map(|e| e.category()).collect::<Vec<_>>(),
    });

    let json = serde_json::to_string_pretty(&response).map_err(|e| McpError::InternalError {
        message: e.to_string(),
    })?;

    if result.success {
        Ok(ToolCallResult::success(json))
    } else {
        Ok(ToolCallResult::error(json))
    }
}
