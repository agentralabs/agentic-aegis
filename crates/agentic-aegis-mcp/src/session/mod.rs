use agentic_aegis_core::protection::{
    CodeSafetyAnalyzer, ContentFilter, IntentVerifier, OutputSanitizer, PayloadScanner,
    PiiDetector, PromptInjectionDetector, RateLimiter,
};
use agentic_aegis_core::session::CorrectionHintGenerator;
use agentic_aegis_core::session::RollbackEngine;
use agentic_aegis_core::session::SessionManager as CoreSessionManager;
use agentic_aegis_core::shadow::{EffectTracker, SandboxExecutor, ShadowCompiler};

pub struct McpSessionManager {
    pub core: CoreSessionManager,
    pub compiler: ShadowCompiler,
    pub executor: SandboxExecutor,
    pub effect_tracker: EffectTracker,
    pub prompt_detector: PromptInjectionDetector,
    pub intent_verifier: IntentVerifier,
    pub payload_scanner: PayloadScanner,
    pub rate_limiter: RateLimiter,
    pub content_filter: ContentFilter,
    pub pii_detector: PiiDetector,
    pub code_safety: CodeSafetyAnalyzer,
    pub output_sanitizer: OutputSanitizer,
    pub rollback_engine: RollbackEngine,
    pub hint_generator: CorrectionHintGenerator,
}

impl McpSessionManager {
    pub fn new() -> Self {
        Self {
            core: CoreSessionManager::new(),
            compiler: ShadowCompiler::new(),
            executor: SandboxExecutor::new(),
            effect_tracker: EffectTracker::new(),
            prompt_detector: PromptInjectionDetector::new(),
            intent_verifier: IntentVerifier::new(),
            payload_scanner: PayloadScanner::new(),
            rate_limiter: RateLimiter::default(),
            content_filter: ContentFilter::new(),
            pii_detector: PiiDetector::new(),
            code_safety: CodeSafetyAnalyzer::new(),
            output_sanitizer: OutputSanitizer::new(),
            rollback_engine: RollbackEngine::new(),
            hint_generator: CorrectionHintGenerator::new(),
        }
    }
}

impl Default for McpSessionManager {
    fn default() -> Self {
        Self::new()
    }
}
