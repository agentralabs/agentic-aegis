use std::sync::Arc;
use tokio::sync::Mutex;

use serde_json::Value;

use agentic_aegis_core::types::Language;

use crate::session::McpSessionManager;
use crate::types::{McpError, McpResult, ToolCallResult};

pub async fn handle_check_input(
    args: &Value,
    session: &Arc<Mutex<McpSessionManager>>,
) -> McpResult<ToolCallResult> {
    let input =
        args.get("input")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams {
                message: "input is required".to_string(),
            })?;

    let check_type = args
        .get("check_type")
        .and_then(|v| v.as_str())
        .unwrap_or("all");

    let mut guard = session.lock().await;

    // Rate limit check
    let rate_result = guard.rate_limiter.check("default");
    if !rate_result.is_allowed() {
        return Ok(ToolCallResult::error(
            "rate limit exceeded, please retry later".to_string(),
        ));
    }

    let mut all_issues = Vec::new();

    if check_type == "all" || check_type == "prompt_injection" {
        let issues = guard.prompt_detector.scan(input);
        all_issues.extend(issues);
    }

    if check_type == "all" || check_type == "payload" {
        let issues = guard.payload_scanner.scan(input);
        all_issues.extend(issues);
    }

    if check_type == "all" || check_type == "intent" {
        if let Some(stated_intent) = args.get("stated_intent").and_then(|v| v.as_str()) {
            let verification = guard.intent_verifier.verify(stated_intent, input);
            if !verification.matches {
                let result = serde_json::json!({
                    "safe": false,
                    "intent_match": false,
                    "confidence": verification.confidence,
                    "warnings": verification.warnings,
                    "issues": all_issues.len(),
                });
                return Ok(ToolCallResult::success(
                    serde_json::to_string_pretty(&result).map_err(|e| McpError::InternalError {
                        message: e.to_string(),
                    })?,
                ));
            }
        }
    }

    let max_threat = all_issues
        .iter()
        .map(|i| i.threat_level)
        .max()
        .unwrap_or(agentic_aegis_core::types::ThreatLevel::None);

    let issue_details: Vec<Value> = all_issues
        .iter()
        .map(|i| {
            serde_json::json!({
                "category": i.category.as_str(),
                "threat_level": format!("{:?}", i.threat_level),
                "message": i.message,
            })
        })
        .collect();

    let result = serde_json::json!({
        "safe": all_issues.is_empty(),
        "threat_level": format!("{:?}", max_threat),
        "issues_count": all_issues.len(),
        "issues": issue_details,
    });

    Ok(ToolCallResult::success(
        serde_json::to_string_pretty(&result).map_err(|e| McpError::InternalError {
            message: e.to_string(),
        })?,
    ))
}

pub async fn handle_check_output(
    args: &Value,
    session: &Arc<Mutex<McpSessionManager>>,
) -> McpResult<ToolCallResult> {
    let output =
        args.get("output")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McpError::InvalidParams {
                message: "output is required".to_string(),
            })?;

    let check_type = args
        .get("check_type")
        .and_then(|v| v.as_str())
        .unwrap_or("all");

    let guard = session.lock().await;
    let mut result_obj = serde_json::Map::new();

    if check_type == "all" || check_type == "pii" {
        let pii_matches = guard.pii_detector.scan(output);
        let pii_details: Vec<Value> = pii_matches
            .iter()
            .map(|m| {
                serde_json::json!({
                    "kind": m.kind.as_str(),
                    "masked": m.value_masked,
                    "line": m.line,
                })
            })
            .collect();
        result_obj.insert(
            "pii_found".to_string(),
            serde_json::json!(pii_matches.len()),
        );
        result_obj.insert("pii_matches".to_string(), serde_json::json!(pii_details));
    }

    if check_type == "all" || check_type == "content" {
        let content_issues = guard.content_filter.scan(output);
        result_obj.insert(
            "content_safe".to_string(),
            serde_json::json!(content_issues.is_empty()),
        );
        result_obj.insert(
            "content_issues".to_string(),
            serde_json::json!(content_issues.len()),
        );
    }

    if check_type == "all" || check_type == "sanitize" {
        let sanitized = guard.output_sanitizer.sanitize(output);
        result_obj.insert(
            "was_modified".to_string(),
            serde_json::json!(sanitized.was_modified),
        );
        result_obj.insert("actions".to_string(), serde_json::json!(sanitized.actions));
        result_obj.insert(
            "sanitized_content".to_string(),
            serde_json::json!(sanitized.content),
        );
    }

    let result = Value::Object(result_obj);

    Ok(ToolCallResult::success(
        serde_json::to_string_pretty(&result).map_err(|e| McpError::InternalError {
            message: e.to_string(),
        })?,
    ))
}

pub async fn handle_scan_security(
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

    let scan = guard.code_safety.analyze(code, &language);

    let issue_details: Vec<Value> = scan
        .issues
        .iter()
        .map(|i| {
            serde_json::json!({
                "category": i.category.as_str(),
                "threat_level": format!("{:?}", i.threat_level),
                "message": i.message,
                "line": i.line,
                "recommendation": i.recommendation,
            })
        })
        .collect();

    let result = serde_json::json!({
        "is_safe": scan.is_safe,
        "overall_threat": format!("{:?}", scan.overall_threat),
        "issues_count": scan.issues.len(),
        "lines_scanned": scan.lines_scanned,
        "scan_duration_ms": scan.scan_duration_ms,
        "issues": issue_details,
    });

    Ok(ToolCallResult::success(
        serde_json::to_string_pretty(&result).map_err(|e| McpError::InternalError {
            message: e.to_string(),
        })?,
    ))
}
