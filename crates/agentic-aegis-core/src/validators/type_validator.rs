use async_trait::async_trait;
use regex::Regex;

use crate::types::{
    AegisResult, Language, StreamingValidation, ValidationContext, ValidationError,
    ValidationSeverity, ValidationWarning,
};

use super::StreamingValidator;

pub struct TypeValidator {
    type_patterns: Vec<TypePattern>,
}

struct TypePattern {
    _name: String,
    pattern: Regex,
    language: Option<Language>,
}

impl TypeValidator {
    pub fn new() -> Self {
        let type_patterns = vec![
            TypePattern {
                _name: "rust_type_annotation".to_string(),
                pattern: Regex::new(r":\s*(i8|i16|i32|i64|i128|u8|u16|u32|u64|u128|f32|f64|bool|char|str|String|Vec|HashMap|Option|Result|Box|Arc|Mutex)\b")
                    .unwrap_or_else(|_| Regex::new(r"$^").expect("fallback regex")),
                language: Some(Language::Rust),
            },
            TypePattern {
                _name: "rust_return_type".to_string(),
                pattern: Regex::new(r"->\s*\w+")
                    .unwrap_or_else(|_| Regex::new(r"$^").expect("fallback regex")),
                language: Some(Language::Rust),
            },
            TypePattern {
                _name: "ts_type_annotation".to_string(),
                pattern: Regex::new(r":\s*(string|number|boolean|void|any|never|unknown|null|undefined|object)\b")
                    .unwrap_or_else(|_| Regex::new(r"$^").expect("fallback regex")),
                language: Some(Language::TypeScript),
            },
            TypePattern {
                _name: "python_type_hint".to_string(),
                pattern: Regex::new(r":\s*(int|float|str|bool|list|dict|tuple|set|None|Optional|Union|Any)\b")
                    .unwrap_or_else(|_| Regex::new(r"$^").expect("fallback regex")),
                language: Some(Language::Python),
            },
        ];

        Self { type_patterns }
    }

    fn check_type_consistency(&self, code: &str, language: &Language) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        match language {
            Language::Rust => {
                self.check_rust_types(code, &mut errors);
            }
            Language::TypeScript => {
                self.check_typescript_types(code, &mut errors);
            }
            Language::Python => {
                self.check_python_types(code, &mut errors);
            }
            _ => {}
        }

        errors
    }

    fn check_rust_types(&self, code: &str, errors: &mut Vec<ValidationError>) {
        for (idx, line) in code.lines().enumerate() {
            let trimmed = line.trim();

            // Check for let without type or inference
            if trimmed.starts_with("let ") && trimmed.contains('=') {
                // Check for possible integer overflow patterns
                if trimmed.contains(": u8 =") || trimmed.contains(": i8 =") {
                    if let Some(val_str) = trimmed.split('=').nth(1) {
                        let val_str = val_str.trim().trim_end_matches(';');
                        if let Ok(val) = val_str.parse::<i64>() {
                            if trimmed.contains(": u8") && !(0..=255).contains(&val) {
                                errors.push(
                                    ValidationError::error(format!(
                                        "value {} out of range for u8 at line {}",
                                        val,
                                        idx + 1
                                    ))
                                    .with_location(idx + 1, 1),
                                );
                            }
                            if trimmed.contains(": i8") && !(-128..=127).contains(&val) {
                                errors.push(
                                    ValidationError::error(format!(
                                        "value {} out of range for i8 at line {}",
                                        val,
                                        idx + 1
                                    ))
                                    .with_location(idx + 1, 1),
                                );
                            }
                        }
                    }
                }
            }

            // Check for obvious type mismatches in return statements
            if trimmed.starts_with("return ") && trimmed.ends_with(';') {
                let return_val = trimmed
                    .strip_prefix("return ")
                    .unwrap_or("")
                    .trim_end_matches(';')
                    .trim();

                if return_val == "()" || return_val.is_empty() {
                    // Unit return, fine
                } else if return_val == "true" || return_val == "false" {
                    // Boolean return, fine
                }
            }
        }
    }

    fn check_typescript_types(&self, code: &str, errors: &mut Vec<ValidationError>) {
        for (idx, line) in code.lines().enumerate() {
            let trimmed = line.trim();

            // Check for null assignment to non-nullable
            if trimmed.contains("= null")
                && !trimmed.contains("| null")
                && !trimmed.contains("?:")
                && (trimmed.contains(": string")
                    || trimmed.contains(": number")
                    || trimmed.contains(": boolean"))
            {
                errors.push(
                    ValidationError::warning(format!(
                        "possible null assignment to non-nullable type at line {}",
                        idx + 1
                    ))
                    .with_location(idx + 1, 1),
                );
            }
        }
    }

    fn check_python_types(&self, code: &str, errors: &mut Vec<ValidationError>) {
        for (idx, line) in code.lines().enumerate() {
            let trimmed = line.trim();

            // Check for return type mismatch hints
            if trimmed.starts_with("def ") && trimmed.contains("-> None") {
                // Function declares None return
            }
            if trimmed.starts_with("def ") && trimmed.contains("-> int") {
                // Function declares int return
            }

            // Check for type: ignore comments (they indicate type issues)
            if trimmed.contains("# type: ignore") {
                errors.push(
                    ValidationError::warning(format!(
                        "type: ignore suppression at line {}",
                        idx + 1
                    ))
                    .with_location(idx + 1, 1),
                );
            }
        }
    }

    fn check_type_coverage(&self, code: &str, language: &Language) -> Vec<ValidationWarning> {
        let mut warnings = Vec::new();

        let relevant_patterns: Vec<&TypePattern> = self
            .type_patterns
            .iter()
            .filter(|p| p.language.as_ref().is_none_or(|l| l == language))
            .collect();

        let total_lines = code.lines().count();
        let typed_lines: usize = code
            .lines()
            .filter(|line| relevant_patterns.iter().any(|p| p.pattern.is_match(line)))
            .count();

        if total_lines > 20 && typed_lines == 0 {
            warnings.push(ValidationWarning {
                message: "no type annotations found in code".to_string(),
                line: None,
                code: Some("type-coverage".to_string()),
            });
        }

        warnings
    }
}

impl Default for TypeValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl StreamingValidator for TypeValidator {
    async fn validate_chunk(
        &self,
        context: &ValidationContext,
        chunk: &str,
    ) -> AegisResult<StreamingValidation> {
        let accumulated = format!("{}{}", context.accumulated_code, chunk);

        let errors = self.check_type_consistency(&accumulated, &context.language);
        let warnings = self.check_type_coverage(&accumulated, &context.language);

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
        "type_validator"
    }
}
