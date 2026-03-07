use regex::Regex;

use crate::types::{SecurityCategory, SecurityIssue, ThreatLevel};

pub struct PromptInjectionDetector {
    patterns: Vec<InjectionPattern>,
}

struct InjectionPattern {
    pattern: Regex,
    description: String,
    threat_level: ThreatLevel,
}

impl PromptInjectionDetector {
    pub fn new() -> Self {
        let patterns = vec![
            InjectionPattern {
                pattern: Regex::new(r"(?i)ignore\s+(all\s+)?previous\s+instructions")
                    .unwrap_or_else(|_| Regex::new(r"$^").expect("fallback")),
                description: "instruction override attempt".to_string(),
                threat_level: ThreatLevel::Critical,
            },
            InjectionPattern {
                pattern: Regex::new(r"(?i)you\s+are\s+now\s+(?:a|an|the)\s+\w+")
                    .unwrap_or_else(|_| Regex::new(r"$^").expect("fallback")),
                description: "role hijacking attempt".to_string(),
                threat_level: ThreatLevel::High,
            },
            InjectionPattern {
                pattern: Regex::new(r"(?i)forget\s+(?:all\s+)?(?:your|previous|the)\s+(?:instructions|rules|constraints)")
                    .unwrap_or_else(|_| Regex::new(r"$^").expect("fallback")),
                description: "instruction erasure attempt".to_string(),
                threat_level: ThreatLevel::Critical,
            },
            InjectionPattern {
                pattern: Regex::new(r"(?i)system\s*:\s*you\s+(?:are|must|should|will)")
                    .unwrap_or_else(|_| Regex::new(r"$^").expect("fallback")),
                description: "fake system prompt injection".to_string(),
                threat_level: ThreatLevel::Critical,
            },
            InjectionPattern {
                pattern: Regex::new(r"(?i)\[INST\]|\[/INST\]|<<SYS>>|<</SYS>>")
                    .unwrap_or_else(|_| Regex::new(r"$^").expect("fallback")),
                description: "model-specific control token injection".to_string(),
                threat_level: ThreatLevel::High,
            },
            InjectionPattern {
                pattern: Regex::new(r"(?i)act\s+as\s+(?:if|though)\s+(?:you|your)\s+(?:are|were)\s+(?:not|no)")
                    .unwrap_or_else(|_| Regex::new(r"$^").expect("fallback")),
                description: "constraint bypass attempt".to_string(),
                threat_level: ThreatLevel::High,
            },
            InjectionPattern {
                pattern: Regex::new(r"(?i)(?:do|please)\s+not\s+(?:follow|obey|respect)\s+(?:any|the|your)")
                    .unwrap_or_else(|_| Regex::new(r"$^").expect("fallback")),
                description: "rule violation instruction".to_string(),
                threat_level: ThreatLevel::High,
            },
            InjectionPattern {
                pattern: Regex::new(r"(?i)pretend\s+(?:that\s+)?(?:you|to)\s+(?:are|have|can|be|not)")
                    .unwrap_or_else(|_| Regex::new(r"$^").expect("fallback")),
                description: "identity manipulation attempt".to_string(),
                threat_level: ThreatLevel::Medium,
            },
            InjectionPattern {
                pattern: Regex::new(r"(?i)begin\s+(?:new|alternative|different)\s+(?:conversation|session|context)")
                    .unwrap_or_else(|_| Regex::new(r"$^").expect("fallback")),
                description: "context reset attempt".to_string(),
                threat_level: ThreatLevel::Medium,
            },
            InjectionPattern {
                pattern: Regex::new(r"(?i)(?:reveal|show|display|output)\s+(?:your|the)\s+(?:system|initial|original)\s+(?:prompt|instructions|message)")
                    .unwrap_or_else(|_| Regex::new(r"$^").expect("fallback")),
                description: "system prompt extraction attempt".to_string(),
                threat_level: ThreatLevel::Medium,
            },
        ];

        Self { patterns }
    }

    pub fn scan(&self, input: &str) -> Vec<SecurityIssue> {
        let mut issues = Vec::new();

        for pattern in &self.patterns {
            if pattern.pattern.is_match(input) {
                let evidence = pattern
                    .pattern
                    .find(input)
                    .map(|m| m.as_str().to_string());

                issues.push(
                    SecurityIssue::new(
                        SecurityCategory::PromptInjection,
                        pattern.threat_level,
                        pattern.description.clone(),
                    )
                    .with_evidence(evidence.unwrap_or_default()),
                );
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

impl Default for PromptInjectionDetector {
    fn default() -> Self {
        Self::new()
    }
}
