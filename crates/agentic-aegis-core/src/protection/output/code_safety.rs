use regex::Regex;

use crate::types::{Language, SecurityCategory, SecurityIssue, SecurityScan, ThreatLevel};

pub struct CodeSafetyAnalyzer {
    patterns: Vec<SafetyPattern>,
}

struct SafetyPattern {
    pattern: Regex,
    category: SecurityCategory,
    description: String,
    threat_level: ThreatLevel,
    recommendation: String,
    languages: Vec<Language>,
}

impl CodeSafetyAnalyzer {
    pub fn new() -> Self {
        let patterns = vec![
            SafetyPattern {
                pattern: Regex::new(r"(?i)(?:rm\s+-rf|rmdir\s+/s|del\s+/f)")
                    .unwrap_or_else(|_| Regex::new(r"$^").expect("fallback")),
                category: SecurityCategory::UnsafeSystemCall,
                description: "destructive system command".to_string(),
                threat_level: ThreatLevel::Critical,
                recommendation: "avoid destructive file system commands in generated code".to_string(),
                languages: vec![],
            },
            SafetyPattern {
                pattern: Regex::new(r"(?i)(?:chmod\s+777|chmod\s+666|chmod\s+a\+rwx)")
                    .unwrap_or_else(|_| Regex::new(r"$^").expect("fallback")),
                category: SecurityCategory::UnsafeSystemCall,
                description: "overly permissive file permissions".to_string(),
                threat_level: ThreatLevel::High,
                recommendation: "use restrictive file permissions".to_string(),
                languages: vec![],
            },
            SafetyPattern {
                pattern: Regex::new(r#"(?i)(?:password|secret|token|api_key)\s*=\s*["'][^"']{8,}["']"#)
                    .unwrap_or_else(|_| Regex::new(r"$^").expect("fallback")),
                category: SecurityCategory::HardcodedCredential,
                description: "hardcoded credential".to_string(),
                threat_level: ThreatLevel::High,
                recommendation: "use environment variables or secure secret management".to_string(),
                languages: vec![],
            },
            SafetyPattern {
                pattern: Regex::new(r"(?:eval|exec)\s*\(")
                    .unwrap_or_else(|_| Regex::new(r"$^").expect("fallback")),
                category: SecurityCategory::CodeInjection,
                description: "dynamic code execution".to_string(),
                threat_level: ThreatLevel::High,
                recommendation: "avoid eval/exec; use safer alternatives".to_string(),
                languages: vec![Language::Python, Language::JavaScript, Language::TypeScript],
            },
            SafetyPattern {
                pattern: Regex::new(r"(?i)(?:MD5|SHA1)\s*[\.(]")
                    .unwrap_or_else(|_| Regex::new(r"$^").expect("fallback")),
                category: SecurityCategory::InsecureCrypto,
                description: "weak cryptographic hash".to_string(),
                threat_level: ThreatLevel::Medium,
                recommendation: "use SHA-256 or stronger hashing algorithms".to_string(),
                languages: vec![],
            },
            SafetyPattern {
                pattern: Regex::new(r"(?i)(?:DES|RC4|Blowfish)\s*[\.(]")
                    .unwrap_or_else(|_| Regex::new(r"$^").expect("fallback")),
                category: SecurityCategory::InsecureCrypto,
                description: "weak encryption algorithm".to_string(),
                threat_level: ThreatLevel::High,
                recommendation: "use AES-256 or ChaCha20 for encryption".to_string(),
                languages: vec![],
            },
            SafetyPattern {
                pattern: Regex::new(r"(?i)(?:innerHTML|outerHTML|document\.write)\s*=")
                    .unwrap_or_else(|_| Regex::new(r"$^").expect("fallback")),
                category: SecurityCategory::XssAttack,
                description: "potential XSS via DOM manipulation".to_string(),
                threat_level: ThreatLevel::High,
                recommendation: "use textContent or sanitize HTML before insertion".to_string(),
                languages: vec![Language::JavaScript, Language::TypeScript],
            },
            SafetyPattern {
                pattern: Regex::new(r#"format!\s*\(\s*"[^"]*\{[^}]*\}[^"]*"\s*,.*\).*(?:query|sql|exec)"#)
                    .unwrap_or_else(|_| Regex::new(r"$^").expect("fallback")),
                category: SecurityCategory::SqlInjection,
                description: "potential SQL injection via string formatting".to_string(),
                threat_level: ThreatLevel::High,
                recommendation: "use parameterized queries".to_string(),
                languages: vec![Language::Rust],
            },
        ];

        Self { patterns }
    }

    pub fn analyze(&self, code: &str, language: &Language) -> SecurityScan {
        let start = std::time::Instant::now();
        let mut issues = Vec::new();
        let lines_scanned = code.lines().count();

        for pattern in &self.patterns {
            if !pattern.languages.is_empty() && !pattern.languages.contains(language) {
                continue;
            }

            for (line_idx, line) in code.lines().enumerate() {
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
                        .with_evidence(evidence.unwrap_or_default())
                        .with_recommendation(pattern.recommendation.clone()),
                    );
                }
            }
        }

        let duration_ms = start.elapsed().as_millis() as u64;

        if issues.is_empty() {
            SecurityScan::clean(lines_scanned, duration_ms)
        } else {
            SecurityScan::with_issues(issues, lines_scanned, duration_ms)
        }
    }
}

impl Default for CodeSafetyAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
