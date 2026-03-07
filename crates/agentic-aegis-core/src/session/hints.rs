use crate::types::{Language, ValidationError, ValidationSeverity};

pub struct CorrectionHintGenerator;

impl CorrectionHintGenerator {
    pub fn new() -> Self {
        Self
    }

    pub fn generate_hint(
        &self,
        error: &ValidationError,
        language: &Language,
        code_context: &str,
    ) -> Option<String> {
        let msg = error.message.to_lowercase();

        // Bracket mismatch hints
        if msg.contains("bracket")
            || msg.contains("unexpected '}'")
            || msg.contains("unexpected ')'")
        {
            return Some(self.bracket_hint(&msg, code_context));
        }

        // Type mismatch hints
        if msg.contains("type") || msg.contains("mismatch") {
            return Some(self.type_hint(&msg, language));
        }

        // Unclosed string hints
        if msg.contains("unclosed string") {
            return Some("close the string literal with a matching quote character".to_string());
        }

        // Indentation hints
        if msg.contains("indentation") {
            return Some("use consistent indentation (either all spaces or all tabs)".to_string());
        }

        // Unsafe code hints
        if msg.contains("unsafe") {
            return Some("consider using safe alternatives or wrapping in a safe API".to_string());
        }

        // Security hints
        if msg.contains("hardcoded")
            && (msg.contains("password") || msg.contains("secret") || msg.contains("key"))
        {
            return Some(
                "use environment variables or a secret manager instead of hardcoded values"
                    .to_string(),
            );
        }

        // Default hint based on severity
        match error.severity {
            ValidationSeverity::Error => Some("fix the error before continuing".to_string()),
            ValidationSeverity::Warning => Some("consider addressing this warning".to_string()),
            _ => None,
        }
    }

    pub fn generate_hints(
        &self,
        errors: &[ValidationError],
        language: &Language,
        code_context: &str,
    ) -> Vec<String> {
        errors
            .iter()
            .filter_map(|e| self.generate_hint(e, language, code_context))
            .collect()
    }

    fn bracket_hint(&self, msg: &str, _code_context: &str) -> String {
        if msg.contains("unexpected '}'") {
            "remove the extra closing brace or add a matching opening brace".to_string()
        } else if msg.contains("unexpected ')'") {
            "remove the extra closing parenthesis or add a matching opening parenthesis".to_string()
        } else if msg.contains("unexpected ']'") {
            "remove the extra closing bracket or add a matching opening bracket".to_string()
        } else if msg.contains("mismatched") {
            "ensure each opening bracket has a matching closing bracket of the same type"
                .to_string()
        } else {
            "check bracket pairing and nesting".to_string()
        }
    }

    fn type_hint(&self, msg: &str, language: &Language) -> String {
        match language {
            Language::Rust => {
                if msg.contains("out of range") {
                    "use a larger integer type (e.g., i32 instead of i8)".to_string()
                } else {
                    "check that the types match on both sides of the assignment".to_string()
                }
            }
            Language::TypeScript => {
                if msg.contains("null") {
                    "add '| null' to the type annotation or use the optional operator '?'"
                        .to_string()
                } else {
                    "verify the type annotation matches the assigned value".to_string()
                }
            }
            Language::Python => {
                if msg.contains("type: ignore") {
                    "fix the underlying type error instead of suppressing it".to_string()
                } else {
                    "verify the type hints match the actual values".to_string()
                }
            }
            _ => "check type annotations and value types".to_string(),
        }
    }
}

impl Default for CorrectionHintGenerator {
    fn default() -> Self {
        Self::new()
    }
}
