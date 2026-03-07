use serde::{Deserialize, Serialize};

use super::ids::{SessionId, ValidationId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Language {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    Go,
    Java,
    CSharp,
    Cpp,
    C,
    Ruby,
    Swift,
    Kotlin,
    #[default]
    Unknown,
}

impl Language {
    pub fn from_str_loose(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "rust" | "rs" => Language::Rust,
            "python" | "py" => Language::Python,
            "javascript" | "js" => Language::JavaScript,
            "typescript" | "ts" => Language::TypeScript,
            "go" | "golang" => Language::Go,
            "java" => Language::Java,
            "csharp" | "cs" | "c#" => Language::CSharp,
            "cpp" | "c++" => Language::Cpp,
            "c" => Language::C,
            "ruby" | "rb" => Language::Ruby,
            "swift" => Language::Swift,
            "kotlin" | "kt" => Language::Kotlin,
            _ => Language::Unknown,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Language::Rust => "rust",
            Language::Python => "python",
            Language::JavaScript => "javascript",
            Language::TypeScript => "typescript",
            Language::Go => "go",
            Language::Java => "java",
            Language::CSharp => "csharp",
            Language::Cpp => "cpp",
            Language::C => "c",
            Language::Ruby => "ruby",
            Language::Swift => "swift",
            Language::Kotlin => "kotlin",
            Language::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationContext {
    pub session_id: SessionId,
    pub language: Language,
    pub file_path: String,
    pub blueprint_id: Option<String>,
    pub accumulated_code: String,
    pub expected_types: Vec<String>,
    pub line_offset: usize,
    pub chunk_index: usize,
}

impl ValidationContext {
    pub fn new(session_id: SessionId, language: Language, file_path: String) -> Self {
        Self {
            session_id,
            language,
            file_path,
            blueprint_id: None,
            accumulated_code: String::new(),
            expected_types: Vec::new(),
            line_offset: 0,
            chunk_index: 0,
        }
    }

    pub fn append_chunk(&mut self, chunk: &str) {
        self.accumulated_code.push_str(chunk);
        self.line_offset += chunk.lines().count();
        self.chunk_index += 1;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationSeverity {
    Error,
    Warning,
    Info,
    Hint,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub message: String,
    pub line: Option<usize>,
    pub column: Option<usize>,
    pub severity: ValidationSeverity,
    pub code: Option<String>,
    pub suggestion: Option<String>,
}

impl ValidationError {
    pub fn new(message: String, severity: ValidationSeverity) -> Self {
        Self {
            message,
            line: None,
            column: None,
            severity,
            code: None,
            suggestion: None,
        }
    }

    pub fn error(message: String) -> Self {
        Self::new(message, ValidationSeverity::Error)
    }

    pub fn warning(message: String) -> Self {
        Self::new(message, ValidationSeverity::Warning)
    }

    pub fn with_location(mut self, line: usize, column: usize) -> Self {
        self.line = Some(line);
        self.column = Some(column);
        self
    }

    pub fn with_suggestion(mut self, suggestion: String) -> Self {
        self.suggestion = Some(suggestion);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    pub message: String,
    pub line: Option<usize>,
    pub code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingValidation {
    pub id: ValidationId,
    pub valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
    pub should_stop: bool,
    pub correction_hint: Option<String>,
    pub confidence: f64,
    pub chunk_index: usize,
}

impl StreamingValidation {
    pub fn ok(chunk_index: usize) -> Self {
        Self {
            id: ValidationId::new(),
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            should_stop: false,
            correction_hint: None,
            confidence: 1.0,
            chunk_index,
        }
    }

    pub fn fail(errors: Vec<ValidationError>, chunk_index: usize) -> Self {
        let should_stop = errors
            .iter()
            .any(|e| e.severity == ValidationSeverity::Error);
        Self {
            id: ValidationId::new(),
            valid: false,
            errors,
            warnings: Vec::new(),
            should_stop,
            correction_hint: None,
            confidence: 0.0,
            chunk_index,
        }
    }

    pub fn with_hint(mut self, hint: String) -> Self {
        self.correction_hint = Some(hint);
        self
    }

    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }

    pub fn with_warnings(mut self, warnings: Vec<ValidationWarning>) -> Self {
        self.warnings = warnings;
        self
    }
}

impl Default for StreamingValidation {
    fn default() -> Self {
        Self::ok(0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
    pub total_chunks: usize,
    pub confidence: f64,
    pub language: Language,
}

impl ValidationResult {
    pub fn success(language: Language, total_chunks: usize) -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            total_chunks,
            confidence: 1.0,
            language,
        }
    }

    pub fn failure(errors: Vec<ValidationError>, language: Language) -> Self {
        Self {
            valid: false,
            errors,
            warnings: Vec::new(),
            total_chunks: 0,
            confidence: 0.0,
            language,
        }
    }
}
