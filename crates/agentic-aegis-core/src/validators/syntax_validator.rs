use async_trait::async_trait;

use crate::types::{
    AegisResult, Language, StreamingValidation, ValidationContext, ValidationError,
    ValidationSeverity, ValidationWarning,
};

use super::StreamingValidator;

pub struct SyntaxValidator;

impl SyntaxValidator {
    pub fn new() -> Self {
        Self
    }

    fn validate_rust_syntax(code: &str) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        let mut brace_depth = 0i64;
        let mut paren_depth = 0i64;
        let mut bracket_depth = 0i64;
        let mut in_string = false;
        let mut in_char = false;
        let mut in_line_comment;
        let mut in_block_comment = false;
        let mut prev = '\0';

        for (line_idx, line) in code.lines().enumerate() {
            in_line_comment = false;
            let chars: Vec<char> = line.chars().collect();
            let len = chars.len();
            let mut i = 0;

            while i < len {
                let ch = chars[i];

                if in_block_comment {
                    if ch == '/' && prev == '*' {
                        in_block_comment = false;
                    }
                    prev = ch;
                    i += 1;
                    continue;
                }

                if in_line_comment {
                    prev = ch;
                    i += 1;
                    continue;
                }

                if in_string {
                    if ch == '"' && prev != '\\' {
                        in_string = false;
                    }
                    prev = ch;
                    i += 1;
                    continue;
                }

                if in_char {
                    if ch == '\'' && prev != '\\' {
                        in_char = false;
                    }
                    prev = ch;
                    i += 1;
                    continue;
                }

                if ch == '/' && i + 1 < len && chars[i + 1] == '/' {
                    in_line_comment = true;
                    i += 2;
                    continue;
                }

                if ch == '/' && i + 1 < len && chars[i + 1] == '*' {
                    in_block_comment = true;
                    i += 2;
                    prev = '*';
                    continue;
                }

                if ch == '"' {
                    in_string = true;
                } else if ch == '\'' {
                    in_char = true;
                } else {
                    match ch {
                        '{' => brace_depth += 1,
                        '}' => {
                            brace_depth -= 1;
                            if brace_depth < 0 {
                                errors.push(
                                    ValidationError::error(format!(
                                        "unexpected '}}' at line {}",
                                        line_idx + 1
                                    ))
                                    .with_location(line_idx + 1, i + 1),
                                );
                                brace_depth = 0;
                            }
                        }
                        '(' => paren_depth += 1,
                        ')' => {
                            paren_depth -= 1;
                            if paren_depth < 0 {
                                errors.push(
                                    ValidationError::error(format!(
                                        "unexpected ')' at line {}",
                                        line_idx + 1
                                    ))
                                    .with_location(line_idx + 1, i + 1),
                                );
                                paren_depth = 0;
                            }
                        }
                        '[' => bracket_depth += 1,
                        ']' => {
                            bracket_depth -= 1;
                            if bracket_depth < 0 {
                                errors.push(
                                    ValidationError::error(format!(
                                        "unexpected ']' at line {}",
                                        line_idx + 1
                                    ))
                                    .with_location(line_idx + 1, i + 1),
                                );
                                bracket_depth = 0;
                            }
                        }
                        _ => {}
                    }
                }

                prev = ch;
                i += 1;
            }
        }

        // Check for unclosed strings
        if in_string {
            errors.push(ValidationError::error(
                "unclosed string literal".to_string(),
            ));
        }
        if in_block_comment {
            errors.push(ValidationError::error("unclosed block comment".to_string()));
        }

        errors
    }

    fn validate_python_syntax(code: &str) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        // Check indentation consistency
        let mut indent_char: Option<char> = None;
        for (idx, line) in code.lines().enumerate() {
            if line.is_empty() || line.trim().is_empty() {
                continue;
            }
            let first_char = line.chars().next();
            if let Some(c) = first_char {
                if c == ' ' || c == '\t' {
                    match indent_char {
                        None => indent_char = Some(c),
                        Some(expected) if expected != c => {
                            errors.push(
                                ValidationError::error(format!(
                                    "mixed indentation at line {}",
                                    idx + 1
                                ))
                                .with_location(idx + 1, 1)
                                .with_suggestion("use consistent indentation".to_string()),
                            );
                            break;
                        }
                        _ => {}
                    }
                }
            }

            let trimmed = line.trim();

            // Check for common syntax issues
            if (trimmed.starts_with("def ") || trimmed.starts_with("class "))
                && !trimmed.ends_with(':')
                && !trimmed.ends_with('\\')
                && !trimmed.contains('#')
                && (!trimmed.contains('(') || trimmed.contains(')'))
            {
                errors.push(
                    ValidationError::warning(format!("possible missing colon at line {}", idx + 1))
                        .with_location(idx + 1, line.len()),
                );
            }
        }

        errors
    }

    fn validate_generic_syntax(code: &str) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        // Check for obviously incomplete strings
        let mut in_double_string;
        let mut in_single_string;
        let mut prev;

        for (line_idx, line) in code.lines().enumerate() {
            in_double_string = false;
            in_single_string = false;
            prev = '\0';

            for ch in line.chars() {
                if in_double_string {
                    if ch == '"' && prev != '\\' {
                        in_double_string = false;
                    }
                } else if in_single_string {
                    if ch == '\'' && prev != '\\' {
                        in_single_string = false;
                    }
                } else if ch == '"' {
                    in_double_string = true;
                } else if ch == '\'' {
                    in_single_string = true;
                }
                prev = ch;
            }

            if in_double_string || in_single_string {
                errors.push(
                    ValidationError::warning(format!(
                        "possible unclosed string at line {}",
                        line_idx + 1
                    ))
                    .with_location(line_idx + 1, line.len()),
                );
            }
        }

        errors
    }

    fn check_common_issues(code: &str) -> Vec<ValidationWarning> {
        let mut warnings = Vec::new();
        let line_count = code.lines().count();

        if line_count > 500 {
            warnings.push(ValidationWarning {
                message: format!("file has {} lines, consider splitting", line_count),
                line: None,
                code: Some("file-length".to_string()),
            });
        }

        // Check for TODO/FIXME
        for (idx, line) in code.lines().enumerate() {
            if line.contains("TODO") || line.contains("FIXME") || line.contains("HACK") {
                warnings.push(ValidationWarning {
                    message: format!("found TODO/FIXME marker at line {}", idx + 1),
                    line: Some(idx + 1),
                    code: Some("todo-marker".to_string()),
                });
            }
        }

        warnings
    }
}

impl Default for SyntaxValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl StreamingValidator for SyntaxValidator {
    async fn validate_chunk(
        &self,
        context: &ValidationContext,
        chunk: &str,
    ) -> AegisResult<StreamingValidation> {
        let accumulated = format!("{}{}", context.accumulated_code, chunk);

        let errors = match context.language {
            Language::Rust => Self::validate_rust_syntax(&accumulated),
            Language::Python => Self::validate_python_syntax(&accumulated),
            _ => Self::validate_generic_syntax(&accumulated),
        };

        let warnings = Self::check_common_issues(&accumulated);

        let has_errors = errors
            .iter()
            .any(|e| e.severity == ValidationSeverity::Error);

        if has_errors {
            Ok(StreamingValidation::fail(errors, context.chunk_index).with_warnings(warnings))
        } else {
            let mut result = StreamingValidation::ok(context.chunk_index);
            result.errors = errors;
            result.warnings = warnings;
            Ok(result)
        }
    }

    fn name(&self) -> &'static str {
        "syntax_validator"
    }
}
