use async_trait::async_trait;

use crate::types::{
    AegisResult, Language, StreamingValidation, ValidationContext, ValidationError,
    ValidationSeverity, ValidationWarning,
};

use super::StreamingValidator;

pub struct TokenValidator {
    max_line_length: usize,
    max_nesting_depth: usize,
}

impl TokenValidator {
    pub fn new() -> Self {
        Self {
            max_line_length: 200,
            max_nesting_depth: 20,
        }
    }

    pub fn with_max_line_length(mut self, len: usize) -> Self {
        self.max_line_length = len;
        self
    }

    pub fn with_max_nesting_depth(mut self, depth: usize) -> Self {
        self.max_nesting_depth = depth;
        self
    }

    fn check_brackets(code: &str) -> Vec<ValidationError> {
        let mut errors = Vec::new();
        let mut stack: Vec<(char, usize, usize)> = Vec::new();

        for (line_idx, line) in code.lines().enumerate() {
            let mut in_string = false;
            let mut string_char = ' ';
            let mut prev_char = ' ';

            for (col_idx, ch) in line.chars().enumerate() {
                if in_string {
                    if ch == string_char && prev_char != '\\' {
                        in_string = false;
                    }
                    prev_char = ch;
                    continue;
                }

                if ch == '"' || ch == '\'' {
                    in_string = true;
                    string_char = ch;
                    prev_char = ch;
                    continue;
                }

                match ch {
                    '(' | '[' | '{' => {
                        stack.push((ch, line_idx, col_idx));
                    }
                    ')' | ']' | '}' => {
                        let expected = match ch {
                            ')' => '(',
                            ']' => '[',
                            '}' => '{',
                            _ => unreachable!(),
                        };
                        match stack.pop() {
                            Some((open, _, _)) if open == expected => {}
                            Some((open, open_line, open_col)) => {
                                errors.push(
                                    ValidationError::new(
                                        format!(
                                            "mismatched bracket: expected closing for '{}' at line {}:{}, found '{}'",
                                            open, open_line + 1, open_col + 1, ch
                                        ),
                                        ValidationSeverity::Error,
                                    )
                                    .with_location(line_idx + 1, col_idx + 1),
                                );
                            }
                            None => {
                                errors.push(
                                    ValidationError::new(
                                        format!("unexpected closing bracket '{}'", ch),
                                        ValidationSeverity::Error,
                                    )
                                    .with_location(line_idx + 1, col_idx + 1),
                                );
                            }
                        }
                    }
                    _ => {}
                }
                prev_char = ch;
            }
        }

        errors
    }

    fn check_line_lengths(&self, code: &str) -> Vec<ValidationWarning> {
        let mut warnings = Vec::new();
        for (idx, line) in code.lines().enumerate() {
            if line.len() > self.max_line_length {
                warnings.push(ValidationWarning {
                    message: format!(
                        "line {} exceeds {} characters ({})",
                        idx + 1,
                        self.max_line_length,
                        line.len()
                    ),
                    line: Some(idx + 1),
                    code: Some("line-length".to_string()),
                });
            }
        }
        warnings
    }

    fn check_nesting_depth(&self, code: &str) -> Vec<ValidationWarning> {
        let mut warnings = Vec::new();
        let mut depth: usize = 0;
        let mut max_depth: usize = 0;

        for (line_idx, line) in code.lines().enumerate() {
            for ch in line.chars() {
                match ch {
                    '{' | '(' | '[' => {
                        depth += 1;
                        if depth > max_depth {
                            max_depth = depth;
                        }
                        if depth > self.max_nesting_depth {
                            warnings.push(ValidationWarning {
                                message: format!(
                                    "nesting depth {} exceeds maximum {} at line {}",
                                    depth,
                                    self.max_nesting_depth,
                                    line_idx + 1
                                ),
                                line: Some(line_idx + 1),
                                code: Some("nesting-depth".to_string()),
                            });
                        }
                    }
                    '}' | ')' | ']' => {
                        depth = depth.saturating_sub(1);
                    }
                    _ => {}
                }
            }
        }

        warnings
    }

    fn check_language_specific(
        &self,
        code: &str,
        language: &Language,
    ) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        match language {
            Language::Rust => {
                for (idx, line) in code.lines().enumerate() {
                    let trimmed = line.trim();
                    if trimmed.starts_with("fn ") && trimmed.contains("->")
                        && !trimmed.ends_with('{') && !trimmed.ends_with(';') && !trimmed.ends_with(',')
                            && !trimmed.contains("where") {
                                // Could be a multi-line function signature, just warn
                            }
                    if trimmed == "unsafe" || trimmed.starts_with("unsafe {") || trimmed.starts_with("unsafe fn") {
                        errors.push(
                            ValidationError::warning(format!(
                                "unsafe code detected at line {}",
                                idx + 1
                            ))
                            .with_location(idx + 1, 1),
                        );
                    }
                }
            }
            Language::Python => {
                for (idx, line) in code.lines().enumerate() {
                    let trimmed = line.trim();
                    if trimmed.starts_with("eval(") || trimmed.starts_with("exec(") {
                        errors.push(
                            ValidationError::warning(format!(
                                "dangerous function call at line {}",
                                idx + 1
                            ))
                            .with_location(idx + 1, 1),
                        );
                    }
                }
            }
            Language::JavaScript | Language::TypeScript => {
                for (idx, line) in code.lines().enumerate() {
                    let trimmed = line.trim();
                    if trimmed.contains("eval(") || trimmed.contains("Function(") {
                        errors.push(
                            ValidationError::warning(format!(
                                "dangerous function call at line {}",
                                idx + 1
                            ))
                            .with_location(idx + 1, 1),
                        );
                    }
                }
            }
            _ => {}
        }

        errors
    }
}

impl Default for TokenValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl StreamingValidator for TokenValidator {
    async fn validate_chunk(
        &self,
        context: &ValidationContext,
        chunk: &str,
    ) -> AegisResult<StreamingValidation> {
        let accumulated = format!("{}{}", context.accumulated_code, chunk);

        let bracket_errors = Self::check_brackets(&accumulated);
        let line_warnings = self.check_line_lengths(&accumulated);
        let nesting_warnings = self.check_nesting_depth(&accumulated);
        let lang_errors = self.check_language_specific(&accumulated, &context.language);

        let all_errors: Vec<ValidationError> = bracket_errors
            .into_iter()
            .chain(lang_errors)
            .collect();

        let all_warnings: Vec<_> = line_warnings
            .into_iter()
            .chain(nesting_warnings)
            .collect();

        let has_hard_errors = all_errors
            .iter()
            .any(|e| e.severity == ValidationSeverity::Error);

        if has_hard_errors {
            Ok(StreamingValidation::fail(all_errors, context.chunk_index)
                .with_warnings(all_warnings))
        } else {
            let mut result = StreamingValidation::ok(context.chunk_index);
            result.errors = all_errors;
            result.warnings = all_warnings;
            Ok(result)
        }
    }

    fn name(&self) -> &'static str {
        "token_validator"
    }
}
