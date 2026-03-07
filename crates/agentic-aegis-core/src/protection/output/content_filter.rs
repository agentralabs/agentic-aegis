use regex::Regex;

use crate::types::{SecurityCategory, SecurityIssue, ThreatLevel};

pub struct ContentFilter {
    patterns: Vec<ContentPattern>,
}

struct ContentPattern {
    pattern: Regex,
    description: String,
    threat_level: ThreatLevel,
}

impl ContentFilter {
    pub fn new() -> Self {
        let patterns = vec![
            ContentPattern {
                pattern: Regex::new(r"(?i)(?:how\s+to\s+(?:make|build|create)\s+(?:a\s+)?(?:bomb|explosive|weapon))")
                    .unwrap_or_else(|_| Regex::new(r"$^").expect("fallback")),
                description: "dangerous content detected".to_string(),
                threat_level: ThreatLevel::Critical,
            },
            ContentPattern {
                pattern: Regex::new(r"(?i)(?:malware|ransomware|virus|trojan)\s+(?:source|code|script)")
                    .unwrap_or_else(|_| Regex::new(r"$^").expect("fallback")),
                description: "malware-related content detected".to_string(),
                threat_level: ThreatLevel::Critical,
            },
            ContentPattern {
                pattern: Regex::new(r"(?i)(?:phishing|social\s+engineering)\s+(?:template|script|page)")
                    .unwrap_or_else(|_| Regex::new(r"$^").expect("fallback")),
                description: "phishing content detected".to_string(),
                threat_level: ThreatLevel::High,
            },
        ];

        Self { patterns }
    }

    pub fn scan(&self, content: &str) -> Vec<SecurityIssue> {
        let mut issues = Vec::new();

        for pattern in &self.patterns {
            for (line_idx, line) in content.lines().enumerate() {
                if pattern.pattern.is_match(line) {
                    issues.push(
                        SecurityIssue::new(
                            SecurityCategory::InappropriateContent,
                            pattern.threat_level,
                            pattern.description.clone(),
                        )
                        .with_location(line_idx + 1, 1),
                    );
                }
            }
        }

        issues
    }

    pub fn is_safe(&self, content: &str) -> bool {
        self.scan(content).is_empty()
    }
}

impl Default for ContentFilter {
    fn default() -> Self {
        Self::new()
    }
}
