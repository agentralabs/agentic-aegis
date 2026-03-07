use regex::Regex;

use crate::types::{PiiKind, PiiMatch, SecurityCategory, SecurityIssue, ThreatLevel};

pub struct PiiDetector {
    patterns: Vec<PiiPattern>,
}

struct PiiPattern {
    pattern: Regex,
    kind: PiiKind,
}

impl PiiDetector {
    pub fn new() -> Self {
        let patterns = vec![
            PiiPattern {
                pattern: Regex::new(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}")
                    .unwrap_or_else(|_| Regex::new(r"$^").expect("fallback")),
                kind: PiiKind::Email,
            },
            PiiPattern {
                pattern: Regex::new(r"\b(?:\+?1[-.\s]?)?\(?\d{3}\)?[-.\s]?\d{3}[-.\s]?\d{4}\b")
                    .unwrap_or_else(|_| Regex::new(r"$^").expect("fallback")),
                kind: PiiKind::Phone,
            },
            PiiPattern {
                pattern: Regex::new(r"\b\d{3}[-\s]?\d{2}[-\s]?\d{4}\b")
                    .unwrap_or_else(|_| Regex::new(r"$^").expect("fallback")),
                kind: PiiKind::SocialSecurity,
            },
            PiiPattern {
                pattern: Regex::new(r"\b(?:\d{4}[-\s]?){3}\d{4}\b")
                    .unwrap_or_else(|_| Regex::new(r"$^").expect("fallback")),
                kind: PiiKind::CreditCard,
            },
            PiiPattern {
                pattern: Regex::new(r"\b(?:\d{1,3}\.){3}\d{1,3}\b")
                    .unwrap_or_else(|_| Regex::new(r"$^").expect("fallback")),
                kind: PiiKind::IpAddress,
            },
            PiiPattern {
                pattern: Regex::new(
                    r#"(?i)(?:api[_-]?key|apikey)\s*[:=]\s*['"]?[a-zA-Z0-9_-]{20,}['"]?"#,
                )
                .unwrap_or_else(|_| Regex::new(r"$^").expect("fallback")),
                kind: PiiKind::ApiKey,
            },
            PiiPattern {
                pattern: Regex::new(r"(?:AKIA|ASIA)[A-Z0-9]{16}")
                    .unwrap_or_else(|_| Regex::new(r"$^").expect("fallback")),
                kind: PiiKind::AwsKey,
            },
            PiiPattern {
                pattern: Regex::new(r"-----BEGIN\s+(?:RSA\s+)?PRIVATE\s+KEY-----")
                    .unwrap_or_else(|_| Regex::new(r"$^").expect("fallback")),
                kind: PiiKind::PrivateKey,
            },
        ];

        Self { patterns }
    }

    pub fn scan(&self, content: &str) -> Vec<PiiMatch> {
        let mut matches = Vec::new();

        for (line_idx, line) in content.lines().enumerate() {
            for pii_pattern in &self.patterns {
                for mat in pii_pattern.pattern.find_iter(line) {
                    let value = mat.as_str();
                    let masked = mask_value(value, &pii_pattern.kind);
                    matches.push(PiiMatch {
                        kind: pii_pattern.kind,
                        value_masked: masked,
                        line: line_idx + 1,
                        start_col: mat.start() + 1,
                        end_col: mat.end(),
                    });
                }
            }
        }

        matches
    }

    pub fn to_security_issues(&self, content: &str) -> Vec<SecurityIssue> {
        self.scan(content)
            .into_iter()
            .map(|m| {
                SecurityIssue::new(
                    SecurityCategory::PiiExposure,
                    ThreatLevel::Medium,
                    format!("{} detected: {}", m.kind.as_str(), m.value_masked),
                )
                .with_location(m.line, m.start_col)
            })
            .collect()
    }

    pub fn has_pii(&self, content: &str) -> bool {
        !self.scan(content).is_empty()
    }

    pub fn redact(&self, content: &str) -> String {
        let mut result = content.to_string();
        for pii_pattern in &self.patterns {
            result = pii_pattern
                .pattern
                .replace_all(&result, |_: &regex::Captures| {
                    format!("[REDACTED_{}]", pii_pattern.kind.as_str().to_uppercase())
                })
                .to_string();
        }
        result
    }
}

impl Default for PiiDetector {
    fn default() -> Self {
        Self::new()
    }
}

fn mask_value(value: &str, kind: &PiiKind) -> String {
    match kind {
        PiiKind::Email => {
            if let Some(at_pos) = value.find('@') {
                let local = &value[..at_pos];
                let domain = &value[at_pos..];
                if local.len() > 2 {
                    format!("{}***{}", &local[..1], domain)
                } else {
                    format!("***{}", domain)
                }
            } else {
                "***@***.***".to_string()
            }
        }
        PiiKind::Phone => {
            if value.len() > 4 {
                format!("***{}", &value[value.len() - 4..])
            } else {
                "****".to_string()
            }
        }
        PiiKind::CreditCard => {
            if value.len() > 4 {
                let last_four = value
                    .chars()
                    .rev()
                    .take(4)
                    .collect::<String>()
                    .chars()
                    .rev()
                    .collect::<String>();
                format!("****-****-****-{}", last_four)
            } else {
                "****".to_string()
            }
        }
        PiiKind::SocialSecurity => "***-**-****".to_string(),
        PiiKind::IpAddress => {
            let parts: Vec<&str> = value.split('.').collect();
            if parts.len() == 4 {
                format!("{}.***.***.{}", parts[0], parts[3])
            } else {
                "***.***.***.***".to_string()
            }
        }
        _ => {
            if value.len() > 8 {
                format!("{}...{}", &value[..4], &value[value.len() - 4..])
            } else {
                "********".to_string()
            }
        }
    }
}
