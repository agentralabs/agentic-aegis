use crate::protection::output::pii_detector::PiiDetector;

pub struct OutputSanitizer {
    pii_detector: PiiDetector,
    max_output_length: usize,
    strip_ansi: bool,
}

impl OutputSanitizer {
    pub fn new() -> Self {
        Self {
            pii_detector: PiiDetector::new(),
            max_output_length: 1_000_000, // 1 MB
            strip_ansi: true,
        }
    }

    pub fn with_max_length(mut self, max: usize) -> Self {
        self.max_output_length = max;
        self
    }

    pub fn with_strip_ansi(mut self, strip: bool) -> Self {
        self.strip_ansi = strip;
        self
    }

    pub fn sanitize(&self, output: &str) -> SanitizedOutput {
        let mut sanitized = output.to_string();
        let mut actions = Vec::new();

        // Strip ANSI escape codes
        if self.strip_ansi {
            let ansi_re = regex::Regex::new(r"\x1B\[[0-9;]*[a-zA-Z]")
                .unwrap_or_else(|_| regex::Regex::new(r"$^").expect("fallback"));
            if ansi_re.is_match(&sanitized) {
                sanitized = ansi_re.replace_all(&sanitized, "").to_string();
                actions.push("stripped_ansi_codes".to_string());
            }
        }

        // Redact PII
        if self.pii_detector.has_pii(&sanitized) {
            sanitized = self.pii_detector.redact(&sanitized);
            actions.push("redacted_pii".to_string());
        }

        // Strip null bytes
        if sanitized.contains('\0') {
            sanitized = sanitized.replace('\0', "");
            actions.push("stripped_null_bytes".to_string());
        }

        // Truncate if needed
        if sanitized.len() > self.max_output_length {
            sanitized.truncate(self.max_output_length);
            sanitized.push_str("\n... (output truncated)");
            actions.push("truncated_output".to_string());
        }

        SanitizedOutput {
            content: sanitized,
            was_modified: !actions.is_empty(),
            actions,
        }
    }
}

impl Default for OutputSanitizer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct SanitizedOutput {
    pub content: String,
    pub was_modified: bool,
    pub actions: Vec<String>,
}
