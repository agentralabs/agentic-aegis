use serde_json::Value;

use crate::tools::registry::ToolDefinition;

/// Facade groupings for compact MCP tool mode.
/// Reduces 12 individual tools to 3 compact facades.

struct FacadeGroup {
    name: &'static str,
    description: &'static str,
    operations: &'static [&'static str],
}

const FACADES: &[FacadeGroup] = &[
    FacadeGroup {
        name: "aegis_validation",
        description: "Validate code, execute in shadow sandbox, and scan for security vulnerabilities",
        operations: &[
            "validate_streaming",
            "validate_complete",
            "shadow_execute",
            "scan_security",
        ],
    },
    FacadeGroup {
        name: "aegis_session",
        description: "Create, query, end validation sessions, and rollback to previous states",
        operations: &[
            "session_create",
            "session_status",
            "session_end",
            "rollback",
        ],
    },
    FacadeGroup {
        name: "aegis_analysis",
        description: "Check input/output security, get correction hints, and score code confidence",
        operations: &[
            "check_input",
            "check_output",
            "correction_hint",
            "confidence_score",
        ],
    },
];

/// Check whether compact tool mode is enabled via environment variable.
pub fn is_compact_mode() -> bool {
    std::env::var("AEGIS_COMPACT_TOOLS")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

/// Build the compact facade tool definitions for tools/list.
pub fn compact_tool_definitions() -> Vec<ToolDefinition> {
    FACADES
        .iter()
        .map(|f| {
            let ops: Vec<&str> = f.operations.to_vec();
            let ops_enum: Vec<Value> = ops
                .iter()
                .map(|o| Value::String(o.to_string()))
                .collect();

            ToolDefinition {
                name: f.name.to_string(),
                description: f.description.to_string(),
                input_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "operation": {
                            "type": "string",
                            "enum": ops_enum,
                            "description": "Operation to perform"
                        },
                        "params": {
                            "type": "object",
                            "description": "Parameters for the operation"
                        }
                    },
                    "required": ["operation"]
                }),
            }
        })
        .collect()
}

/// Normalize a compact facade call into the underlying tool name
/// and arguments. Returns `(real_tool_name, arguments)`.
///
/// Mapping: facade "aegis_validation" + operation "validate_streaming"
///          -> tool "aegis_validate_streaming"
pub fn normalize_compact_call(
    facade_name: &str,
    arguments: &Option<Value>,
) -> Option<(String, Option<Value>)> {
    let facade = FACADES.iter().find(|f| f.name == facade_name)?;

    let args = arguments.as_ref().unwrap_or(&Value::Null);
    let operation = args.get("operation").and_then(|v| v.as_str())?;

    if !facade.operations.contains(&operation) {
        return None;
    }

    let real_name = format!("aegis_{}", operation);
    let params = args.get("params").cloned();

    Some((real_name, params))
}

/// Check if a tool name is a compact facade name.
pub fn is_compact_facade(name: &str) -> bool {
    FACADES.iter().any(|f| f.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compact_definitions_count() {
        let defs = compact_tool_definitions();
        assert_eq!(defs.len(), 3);
        assert_eq!(defs[0].name, "aegis_validation");
        assert_eq!(defs[1].name, "aegis_session");
        assert_eq!(defs[2].name, "aegis_analysis");
    }

    #[test]
    fn test_normalize_validation_facade() {
        let args = Some(serde_json::json!({
            "operation": "validate_streaming",
            "params": { "session_id": "s1", "chunk": "let x = 1;" }
        }));
        let result = normalize_compact_call("aegis_validation", &args);
        assert!(result.is_some());
        let (name, params) = result.unwrap();
        assert_eq!(name, "aegis_validate_streaming");
        assert_eq!(
            params.unwrap().get("session_id").unwrap().as_str().unwrap(),
            "s1"
        );
    }

    #[test]
    fn test_normalize_session_facade() {
        let args = Some(serde_json::json!({
            "operation": "rollback",
            "params": { "session_id": "s1" }
        }));
        let result = normalize_compact_call("aegis_session", &args);
        assert!(result.is_some());
        let (name, _) = result.unwrap();
        assert_eq!(name, "aegis_rollback");
    }

    #[test]
    fn test_normalize_analysis_facade() {
        let args = Some(serde_json::json!({
            "operation": "confidence_score",
            "params": { "code": "x = 1", "language": "python" }
        }));
        let result = normalize_compact_call("aegis_analysis", &args);
        assert!(result.is_some());
        let (name, _) = result.unwrap();
        assert_eq!(name, "aegis_confidence_score");
    }

    #[test]
    fn test_normalize_unknown_facade() {
        let args = Some(serde_json::json!({ "operation": "validate_streaming" }));
        assert!(normalize_compact_call("aegis_unknown", &args).is_none());
    }

    #[test]
    fn test_normalize_invalid_operation() {
        let args = Some(serde_json::json!({ "operation": "nonexistent" }));
        assert!(normalize_compact_call("aegis_validation", &args).is_none());
    }

    #[test]
    fn test_is_compact_facade() {
        assert!(is_compact_facade("aegis_validation"));
        assert!(is_compact_facade("aegis_session"));
        assert!(is_compact_facade("aegis_analysis"));
        assert!(!is_compact_facade("aegis_validate_streaming"));
    }
}
