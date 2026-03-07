use regex::Regex;

use crate::types::{SecurityCategory, SecurityIssue, ThreatLevel};

pub struct PayloadScanner {
    patterns: Vec<PayloadPattern>,
}

struct PayloadPattern {
    pattern: Regex,
    category: SecurityCategory,
    description: String,
    threat_level: ThreatLevel,
}

impl PayloadScanner {
    pub fn new() -> Self {
        let patterns = vec![
            PayloadPattern {
                pattern: Regex::new(r"(?i)(?:SELECT|INSERT|UPDATE|DELETE|DROP|ALTER|CREATE)\s+.+(?:FROM|INTO|TABLE|SET)")
                    .unwrap_or_else(|_| Regex::new(r"$^").expect("fallback")),
                category: SecurityCategory::SqlInjection,
                description: "SQL injection pattern detected".to_string(),
                threat_level: ThreatLevel::High,
            },
            PayloadPattern {
                pattern: Regex::new(r"(?i)(?:UNION\s+SELECT|OR\s+1\s*=\s*1|;\s*DROP\s+TABLE)")
                    .unwrap_or_else(|_| Regex::new(r"$^").expect("fallback")),
                category: SecurityCategory::SqlInjection,
                description: "SQL injection attack pattern".to_string(),
                threat_level: ThreatLevel::Critical,
            },
            PayloadPattern {
                pattern: Regex::new(r"<script[^>]*>|javascript:|on\w+\s*=")
                    .unwrap_or_else(|_| Regex::new(r"$^").expect("fallback")),
                category: SecurityCategory::XssAttack,
                description: "XSS attack pattern detected".to_string(),
                threat_level: ThreatLevel::High,
            },
            PayloadPattern {
                pattern: Regex::new(r"\.\./|\.\.\\|%2e%2e|%252e%252e")
                    .unwrap_or_else(|_| Regex::new(r"$^").expect("fallback")),
                category: SecurityCategory::PathTraversal,
                description: "path traversal pattern detected".to_string(),
                threat_level: ThreatLevel::High,
            },
            PayloadPattern {
                pattern: Regex::new(r"(?:;|\||&&)\s*(?:cat|ls|rm|curl|wget|nc|bash|sh|cmd|powershell)")
                    .unwrap_or_else(|_| Regex::new(r"$^").expect("fallback")),
                category: SecurityCategory::CommandInjection,
                description: "command injection pattern detected".to_string(),
                threat_level: ThreatLevel::Critical,
            },
            PayloadPattern {
                pattern: Regex::new(r"`[^`]*(?:cat|ls|rm|curl|wget)[^`]*`|\$\([^)]*(?:cat|ls|rm|curl|wget)[^)]*\)")
                    .unwrap_or_else(|_| Regex::new(r"$^").expect("fallback")),
                category: SecurityCategory::CommandInjection,
                description: "shell command substitution detected".to_string(),
                threat_level: ThreatLevel::High,
            },
            PayloadPattern {
                pattern: Regex::new(r"(?i)(?:base64\s+--decode|base64\s+-d)\s+|atob\(|Buffer\.from\(.+base64")
                    .unwrap_or_else(|_| Regex::new(r"$^").expect("fallback")),
                category: SecurityCategory::MaliciousPayload,
                description: "obfuscated payload (base64 decode)".to_string(),
                threat_level: ThreatLevel::Medium,
            },
        ];

        Self { patterns }
    }

    pub fn scan(&self, input: &str) -> Vec<SecurityIssue> {
        let mut issues = Vec::new();

        for pattern in &self.patterns {
            for (line_idx, line) in input.lines().enumerate() {
                if pattern.pattern.is_match(line) {
                    let evidence = pattern
                        .pattern
                        .find(line)
                        .map(|m| m.as_str().to_string());

                    issues.push(
                        SecurityIssue::new(
                            pattern.category,
                            pattern.threat_level,
                            pattern.description.clone(),
                        )
                        .with_location(line_idx + 1, 1)
                        .with_evidence(evidence.unwrap_or_default()),
                    );
                }
            }
        }

        issues
    }

    pub fn is_safe(&self, input: &str) -> bool {
        self.scan(input).is_empty()
    }

    pub fn max_threat_level(&self, input: &str) -> ThreatLevel {
        self.scan(input)
            .iter()
            .map(|i| i.threat_level)
            .max()
            .unwrap_or(ThreatLevel::None)
    }
}

impl Default for PayloadScanner {
    fn default() -> Self {
        Self::new()
    }
}
