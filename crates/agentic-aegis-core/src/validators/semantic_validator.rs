use async_trait::async_trait;
use regex::Regex;

use crate::types::{
    AegisResult, Language, StreamingValidation, ValidationContext, ValidationError,
    ValidationSeverity, ValidationWarning,
};

use super::StreamingValidator;

pub struct SemanticValidator {
    banned_patterns: Vec<BannedPattern>,
}

struct BannedPattern {
    name: String,
    pattern: Regex,
    message: String,
    severity: ValidationSeverity,
    languages: Vec<Language>,
}

impl SemanticValidator {
    pub fn new() -> Self {
        let banned_patterns = vec![
            BannedPattern {
                name: "infinite_loop".to_string(),
                pattern: Regex::new(r"while\s*\(\s*true\s*\)|loop\s*\{|while\s+True\s*:")
                    .unwrap_or_else(|_| Regex::new(r"$^").expect("fallback regex")),
                message: "potential infinite loop detected".to_string(),
                severity: ValidationSeverity::Warning,
                languages: vec![],
            },
            BannedPattern {
                name: "hardcoded_password".to_string(),
                pattern: Regex::new(r#"(?i)(password|passwd|pwd)\s*=\s*["'][^"']+["']"#)
                    .unwrap_or_else(|_| Regex::new(r"$^").expect("fallback regex")),
                message: "hardcoded password detected".to_string(),
                severity: ValidationSeverity::Error,
                languages: vec![],
            },
            BannedPattern {
                name: "hardcoded_secret".to_string(),
                pattern: Regex::new(r#"(?i)(secret|api_key|apikey|token)\s*=\s*["'][^"']+["']"#)
                    .unwrap_or_else(|_| Regex::new(r"$^").expect("fallback regex")),
                message: "hardcoded secret/key detected".to_string(),
                severity: ValidationSeverity::Error,
                languages: vec![],
            },
            BannedPattern {
                name: "debug_print".to_string(),
                pattern: Regex::new(r"console\.log\(|print\(.*debug|dbg!\(")
                    .unwrap_or_else(|_| Regex::new(r"$^").expect("fallback regex")),
                message: "debug output detected".to_string(),
                severity: ValidationSeverity::Info,
                languages: vec![],
            },
            BannedPattern {
                name: "empty_catch".to_string(),
                pattern: Regex::new(r"catch\s*\([^)]*\)\s*\{\s*\}")
                    .unwrap_or_else(|_| Regex::new(r"$^").expect("fallback regex")),
                message: "empty catch block detected".to_string(),
                severity: ValidationSeverity::Warning,
                languages: vec![Language::JavaScript, Language::TypeScript, Language::Java],
            },
            BannedPattern {
                name: "bare_except".to_string(),
                pattern: Regex::new(r"except\s*:")
                    .unwrap_or_else(|_| Regex::new(r"$^").expect("fallback regex")),
                message: "bare except clause detected".to_string(),
                severity: ValidationSeverity::Warning,
                languages: vec![Language::Python],
            },
            BannedPattern {
                name: "unwrap_call".to_string(),
                pattern: Regex::new(r"\.unwrap\(\)")
                    .unwrap_or_else(|_| Regex::new(r"$^").expect("fallback regex")),
                message: "unwrap() call detected — prefer proper error handling".to_string(),
                severity: ValidationSeverity::Warning,
                languages: vec![Language::Rust],
            },
        ];

        Self { banned_patterns }
    }

    fn check_duplicate_functions(&self, code: &str, language: &Language) -> Vec<ValidationWarning> {
        let mut warnings = Vec::new();
        let mut function_names: Vec<(String, usize)> = Vec::new();

        let fn_pattern = match language {
            Language::Rust => Some(
                Regex::new(r"fn\s+(\w+)")
                    .unwrap_or_else(|_| Regex::new(r"$^").expect("fallback regex")),
            ),
            Language::Python => Some(
                Regex::new(r"def\s+(\w+)")
                    .unwrap_or_else(|_| Regex::new(r"$^").expect("fallback regex")),
            ),
            Language::JavaScript | Language::TypeScript => Some(
                Regex::new(r"function\s+(\w+)")
                    .unwrap_or_else(|_| Regex::new(r"$^").expect("fallback regex")),
            ),
            _ => None,
        };

        if let Some(pattern) = fn_pattern {
            for (idx, line) in code.lines().enumerate() {
                if let Some(cap) = pattern.captures(line) {
                    if let Some(name) = cap.get(1) {
                        let fn_name = name.as_str().to_string();
                        if let Some((_, prev_line)) =
                            function_names.iter().find(|(n, _)| n == &fn_name)
                        {
                            warnings.push(ValidationWarning {
                                message: format!(
                                    "duplicate function '{}' at line {} (first at line {})",
                                    fn_name,
                                    idx + 1,
                                    prev_line + 1
                                ),
                                line: Some(idx + 1),
                                code: Some("duplicate-function".to_string()),
                            });
                        } else {
                            function_names.push((fn_name, idx));
                        }
                    }
                }
            }
        }

        warnings
    }

    fn check_unreachable_code(&self, code: &str, language: &Language) -> Vec<ValidationWarning> {
        let mut warnings = Vec::new();

        let return_pattern = match language {
            Language::Rust | Language::JavaScript | Language::TypeScript | Language::Java => {
                Some("return ")
            }
            Language::Python => Some("return "),
            _ => None,
        };

        if let Some(ret) = return_pattern {
            let lines: Vec<&str> = code.lines().collect();
            for i in 0..lines.len().saturating_sub(1) {
                let trimmed = lines[i].trim();
                if trimmed.starts_with(ret) {
                    let next = lines.get(i + 1).map_or("", |l| l.trim());
                    if !next.is_empty() && next != "}" && !next.starts_with("//") && !next.starts_with('#') {
                        warnings.push(ValidationWarning {
                            message: format!("possible unreachable code after return at line {}", i + 1),
                            line: Some(i + 2),
                            code: Some("unreachable-code".to_string()),
                        });
                    }
                }
            }
        }

        warnings
    }
}

impl Default for SemanticValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl StreamingValidator for SemanticValidator {
    async fn validate_chunk(
        &self,
        context: &ValidationContext,
        chunk: &str,
    ) -> AegisResult<StreamingValidation> {
        let accumulated = format!("{}{}", context.accumulated_code, chunk);

        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Check banned patterns
        for pattern in &self.banned_patterns {
            if !pattern.languages.is_empty()
                && !pattern.languages.contains(&context.language)
            {
                continue;
            }

            for (idx, line) in accumulated.lines().enumerate() {
                if pattern.pattern.is_match(line) {
                    match pattern.severity {
                        ValidationSeverity::Error => {
                            errors.push(
                                ValidationError::new(
                                    format!("{} at line {}", pattern.message, idx + 1),
                                    pattern.severity,
                                )
                                .with_location(idx + 1, 1),
                            );
                        }
                        _ => {
                            warnings.push(ValidationWarning {
                                message: format!("{} at line {}", pattern.message, idx + 1),
                                line: Some(idx + 1),
                                code: Some(pattern.name.clone()),
                            });
                        }
                    }
                }
            }
        }

        // Check for duplicate functions
        warnings.extend(self.check_duplicate_functions(&accumulated, &context.language));

        // Check for unreachable code
        warnings.extend(self.check_unreachable_code(&accumulated, &context.language));

        let has_hard_errors = errors
            .iter()
            .any(|e| e.severity == ValidationSeverity::Error);

        if has_hard_errors {
            Ok(StreamingValidation::fail(errors, context.chunk_index).with_warnings(warnings))
        } else {
            let mut result = StreamingValidation::ok(context.chunk_index);
            result.errors = errors;
            result.warnings = warnings;
            Ok(result)
        }
    }

    fn name(&self) -> &'static str {
        "semantic_validator"
    }
}
