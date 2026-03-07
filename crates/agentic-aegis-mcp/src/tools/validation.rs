use std::sync::Arc;
use tokio::sync::Mutex;

use serde_json::Value;

use agentic_aegis_core::types::{Language, SessionConfig, ValidationError};
use agentic_aegis_core::validators::{
    SemanticValidator, StreamingValidator, SyntaxValidator, TokenValidator, TypeValidator,
};

use crate::session::McpSessionManager;
use crate::types::{McpError, McpResult, ToolCallResult};

pub async fn handle_validate_streaming(
    args: &Value,
    session: &Arc<Mutex<McpSessionManager>>,
) -> McpResult<ToolCallResult> {
    let session_id = args
        .get("session_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidParams {
            message: "session_id is required".to_string(),
        })?;

    let chunk =
        args.get("chunk")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams {
                message: "chunk is required".to_string(),
            })?;

    let mut guard = session.lock().await;
    let result = guard
        .core
        .validate_chunk(session_id, chunk)
        .await
        .map_err(|e| McpError::ToolExecutionError {
            message: e.to_string(),
        })?;

    let json = serde_json::to_string_pretty(&result).map_err(|e| McpError::InternalError {
        message: e.to_string(),
    })?;

    Ok(ToolCallResult::success(json))
}

pub async fn handle_validate_complete(
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

    // Create a temporary session for complete validation
    let config = SessionConfig {
        language,
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

    let result = guard
        .core
        .validate_chunk(&session_id.to_string(), code)
        .await
        .map_err(|e| McpError::ToolExecutionError {
            message: e.to_string(),
        })?;

    let _ = guard.core.end_session(&session_id.to_string());

    let json = serde_json::to_string_pretty(&result).map_err(|e| McpError::InternalError {
        message: e.to_string(),
    })?;

    Ok(ToolCallResult::success(json))
}

pub async fn handle_correction_hint(
    args: &Value,
    session: &Arc<Mutex<McpSessionManager>>,
) -> McpResult<ToolCallResult> {
    let error_message = args
        .get("error_message")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidParams {
            message: "error_message is required".to_string(),
        })?;

    let language_str = args
        .get("language")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::InvalidParams {
            message: "language is required".to_string(),
        })?;

    let code_context = args
        .get("code_context")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let language = Language::from_str_loose(language_str);
    let error = ValidationError::error(error_message.to_string());

    let guard = session.lock().await;
    let hint = guard
        .hint_generator
        .generate_hint(&error, &language, code_context);

    let result = serde_json::json!({
        "hint": hint,
        "error": error_message,
        "language": language.as_str(),
    });

    Ok(ToolCallResult::success(
        serde_json::to_string_pretty(&result).map_err(|e| McpError::InternalError {
            message: e.to_string(),
        })?,
    ))
}

pub async fn handle_confidence_score(
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

    // Run all validators to compute confidence
    let context = agentic_aegis_core::types::ValidationContext::new(
        agentic_aegis_core::types::SessionId::new(),
        language,
        String::new(),
    );

    let validators: Vec<Box<dyn StreamingValidator>> = vec![
        Box::new(TokenValidator::new()),
        Box::new(SyntaxValidator::new()),
        Box::new(TypeValidator::new()),
        Box::new(SemanticValidator::new()),
    ];

    let mut total_confidence = 1.0f64;
    let mut total_errors = 0usize;
    let mut total_warnings = 0usize;

    for validator in &validators {
        if let Ok(result) = validator.validate_chunk(&context, code).await {
            total_confidence = total_confidence.min(result.confidence);
            total_errors += result.errors.len();
            total_warnings += result.warnings.len();
        }
    }

    // Also run security scan
    let guard = session.lock().await;
    let security_scan = guard.code_safety.analyze(code, &language);
    if !security_scan.is_safe {
        total_confidence *= 0.5;
    }

    let result = serde_json::json!({
        "confidence": total_confidence,
        "errors": total_errors,
        "warnings": total_warnings,
        "security_issues": security_scan.issues.len(),
        "is_safe": security_scan.is_safe,
        "language": language.as_str(),
        "lines": code.lines().count(),
    });

    Ok(ToolCallResult::success(
        serde_json::to_string_pretty(&result).map_err(|e| McpError::InternalError {
            message: e.to_string(),
        })?,
    ))
}
