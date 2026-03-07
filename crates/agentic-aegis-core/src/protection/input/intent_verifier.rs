use crate::types::ThreatLevel;

pub struct IntentVerifier;

impl IntentVerifier {
    pub fn new() -> Self {
        Self
    }

    pub fn verify(&self, stated_intent: &str, actual_code: &str) -> IntentVerification {
        let mut score = 1.0f64;
        let mut warnings = Vec::new();

        // Check if the code contains operations not mentioned in intent
        let intent_lower = stated_intent.to_lowercase();
        let code_lower = actual_code.to_lowercase();

        // Check for file operations not in intent
        if (code_lower.contains("write") || code_lower.contains("delete") || code_lower.contains("remove"))
            && !intent_lower.contains("file")
            && !intent_lower.contains("write")
            && !intent_lower.contains("delete")
            && !intent_lower.contains("save")
        {
            score -= 0.3;
            warnings.push("code contains file operations not mentioned in intent".to_string());
        }

        // Check for network operations not in intent
        if (code_lower.contains("http") || code_lower.contains("socket") || code_lower.contains("fetch") || code_lower.contains("request"))
            && !intent_lower.contains("network")
            && !intent_lower.contains("http")
            && !intent_lower.contains("api")
            && !intent_lower.contains("request")
            && !intent_lower.contains("fetch")
        {
            score -= 0.3;
            warnings.push("code contains network operations not mentioned in intent".to_string());
        }

        // Check for process operations not in intent
        if (code_lower.contains("exec") || code_lower.contains("spawn") || code_lower.contains("system("))
            && !intent_lower.contains("process")
            && !intent_lower.contains("execute")
            && !intent_lower.contains("command")
            && !intent_lower.contains("run")
        {
            score -= 0.4;
            warnings.push("code contains process spawn not mentioned in intent".to_string());
        }

        // Check for crypto/security operations
        if (code_lower.contains("encrypt") || code_lower.contains("decrypt") || code_lower.contains("hash"))
            && !intent_lower.contains("encrypt")
            && !intent_lower.contains("hash")
            && !intent_lower.contains("crypto")
            && !intent_lower.contains("security")
        {
            score -= 0.2;
            warnings.push("code contains crypto operations not mentioned in intent".to_string());
        }

        score = score.clamp(0.0, 1.0);

        let threat_level = if score < 0.3 {
            ThreatLevel::High
        } else if score < 0.6 {
            ThreatLevel::Medium
        } else if score < 0.8 {
            ThreatLevel::Low
        } else {
            ThreatLevel::None
        };

        IntentVerification {
            matches: score >= 0.6,
            confidence: score,
            threat_level,
            warnings,
        }
    }
}

impl Default for IntentVerifier {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct IntentVerification {
    pub matches: bool,
    pub confidence: f64,
    pub threat_level: ThreatLevel,
    pub warnings: Vec<String>,
}
