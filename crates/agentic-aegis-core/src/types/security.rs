use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[derive(Default)]
pub enum ThreatLevel {
    #[default]
    None,
    Low,
    Medium,
    High,
    Critical,
}

impl ThreatLevel {
    pub fn score(&self) -> f64 {
        match self {
            ThreatLevel::None => 0.0,
            ThreatLevel::Low => 0.25,
            ThreatLevel::Medium => 0.5,
            ThreatLevel::High => 0.75,
            ThreatLevel::Critical => 1.0,
        }
    }

    pub fn from_score(score: f64) -> Self {
        if score >= 0.9 {
            ThreatLevel::Critical
        } else if score >= 0.7 {
            ThreatLevel::High
        } else if score >= 0.4 {
            ThreatLevel::Medium
        } else if score > 0.0 {
            ThreatLevel::Low
        } else {
            ThreatLevel::None
        }
    }

    pub fn is_blocking(&self) -> bool {
        matches!(self, ThreatLevel::High | ThreatLevel::Critical)
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityCategory {
    PromptInjection,
    CodeInjection,
    PathTraversal,
    CommandInjection,
    SqlInjection,
    XssAttack,
    SensitiveDataExposure,
    InsecureCrypto,
    ResourceExhaustion,
    MaliciousPayload,
    PiiExposure,
    InappropriateContent,
    UnsafeSystemCall,
    HardcodedCredential,
}

impl SecurityCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            SecurityCategory::PromptInjection => "prompt_injection",
            SecurityCategory::CodeInjection => "code_injection",
            SecurityCategory::PathTraversal => "path_traversal",
            SecurityCategory::CommandInjection => "command_injection",
            SecurityCategory::SqlInjection => "sql_injection",
            SecurityCategory::XssAttack => "xss_attack",
            SecurityCategory::SensitiveDataExposure => "sensitive_data_exposure",
            SecurityCategory::InsecureCrypto => "insecure_crypto",
            SecurityCategory::ResourceExhaustion => "resource_exhaustion",
            SecurityCategory::MaliciousPayload => "malicious_payload",
            SecurityCategory::PiiExposure => "pii_exposure",
            SecurityCategory::InappropriateContent => "inappropriate_content",
            SecurityCategory::UnsafeSystemCall => "unsafe_system_call",
            SecurityCategory::HardcodedCredential => "hardcoded_credential",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityIssue {
    pub category: SecurityCategory,
    pub threat_level: ThreatLevel,
    pub message: String,
    pub line: Option<usize>,
    pub column: Option<usize>,
    pub evidence: Option<String>,
    pub recommendation: Option<String>,
}

impl SecurityIssue {
    pub fn new(category: SecurityCategory, threat_level: ThreatLevel, message: String) -> Self {
        Self {
            category,
            threat_level,
            message,
            line: None,
            column: None,
            evidence: None,
            recommendation: None,
        }
    }

    pub fn with_location(mut self, line: usize, column: usize) -> Self {
        self.line = Some(line);
        self.column = Some(column);
        self
    }

    pub fn with_evidence(mut self, evidence: String) -> Self {
        self.evidence = Some(evidence);
        self
    }

    pub fn with_recommendation(mut self, rec: String) -> Self {
        self.recommendation = Some(rec);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityScan {
    pub issues: Vec<SecurityIssue>,
    pub overall_threat: ThreatLevel,
    pub scanned_at: DateTime<Utc>,
    pub scan_duration_ms: u64,
    pub lines_scanned: usize,
    pub is_safe: bool,
}

impl SecurityScan {
    pub fn clean(lines_scanned: usize, duration_ms: u64) -> Self {
        Self {
            issues: Vec::new(),
            overall_threat: ThreatLevel::None,
            scanned_at: Utc::now(),
            scan_duration_ms: duration_ms,
            lines_scanned,
            is_safe: true,
        }
    }

    pub fn with_issues(
        issues: Vec<SecurityIssue>,
        lines_scanned: usize,
        duration_ms: u64,
    ) -> Self {
        let overall_threat = issues
            .iter()
            .map(|i| i.threat_level)
            .max()
            .unwrap_or(ThreatLevel::None);
        let is_safe = !overall_threat.is_blocking();
        Self {
            issues,
            overall_threat,
            scanned_at: Utc::now(),
            scan_duration_ms: duration_ms,
            lines_scanned,
            is_safe,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PiiMatch {
    pub kind: PiiKind,
    pub value_masked: String,
    pub line: usize,
    pub start_col: usize,
    pub end_col: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PiiKind {
    Email,
    Phone,
    SocialSecurity,
    CreditCard,
    IpAddress,
    ApiKey,
    AwsKey,
    PrivateKey,
}

impl PiiKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            PiiKind::Email => "email",
            PiiKind::Phone => "phone",
            PiiKind::SocialSecurity => "ssn",
            PiiKind::CreditCard => "credit_card",
            PiiKind::IpAddress => "ip_address",
            PiiKind::ApiKey => "api_key",
            PiiKind::AwsKey => "aws_key",
            PiiKind::PrivateKey => "private_key",
        }
    }
}
