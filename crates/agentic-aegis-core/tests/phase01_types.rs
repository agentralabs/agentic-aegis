use agentic_aegis_core::types::error::*;
use agentic_aegis_core::types::ids::*;
use agentic_aegis_core::types::security::*;
use agentic_aegis_core::types::session::*;
use agentic_aegis_core::types::validation::*;

// === ID Tests ===

#[test]
fn test_aegis_id_new_unique() {
    let id1 = AegisId::new();
    let id2 = AegisId::new();
    assert_ne!(id1, id2);
}

#[test]
fn test_aegis_id_from_string_deterministic() {
    let id1 = AegisId::from_string("test-context");
    let id2 = AegisId::from_string("test-context");
    assert_eq!(id1, id2);
}

#[test]
fn test_aegis_id_from_different_strings() {
    let id1 = AegisId::from_string("context-a");
    let id2 = AegisId::from_string("context-b");
    assert_ne!(id1, id2);
}

#[test]
fn test_aegis_id_display() {
    let id = AegisId::new();
    let s = id.to_string();
    assert!(!s.is_empty());
    assert!(s.contains('-')); // UUID format
}

#[test]
fn test_aegis_id_to_hex() {
    let id = AegisId::new();
    let hex = id.to_hex();
    assert!(!hex.is_empty());
}

#[test]
fn test_session_id_new_unique() {
    let id1 = SessionId::new();
    let id2 = SessionId::new();
    assert_ne!(id1, id2);
}

#[test]
fn test_session_id_from_string() {
    let id1 = SessionId::from_string("session-1");
    let id2 = SessionId::from_string("session-1");
    assert_eq!(id1, id2);
}

#[test]
fn test_validation_id_new() {
    let id = ValidationId::new();
    assert!(!id.to_string().is_empty());
}

#[test]
fn test_snapshot_id_new() {
    let id = SnapshotId::new();
    assert!(!id.to_string().is_empty());
}

#[test]
fn test_rollback_id_new() {
    let id = RollbackId::new();
    assert!(!id.to_string().is_empty());
}

#[test]
fn test_id_default() {
    let id: AegisId = Default::default();
    assert!(!id.to_string().is_empty());
}

#[test]
fn test_id_clone() {
    let id1 = AegisId::new();
    let id2 = id1.clone();
    assert_eq!(id1, id2);
}

#[test]
fn test_id_hash() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    let id1 = AegisId::new();
    let id2 = AegisId::new();
    set.insert(id1.clone());
    set.insert(id2.clone());
    assert_eq!(set.len(), 2);
    set.insert(id1.clone());
    assert_eq!(set.len(), 2);
}

// === Error Tests ===

#[test]
fn test_aegis_error_display() {
    let err = AegisError::Validation("test error".to_string());
    assert_eq!(err.to_string(), "validation error: test error");
}

#[test]
fn test_aegis_error_session() {
    let err = AegisError::Session("session failed".to_string());
    assert!(err.to_string().contains("session"));
}

#[test]
fn test_aegis_error_shadow() {
    let err = AegisError::ShadowExecution("exec failed".to_string());
    assert!(err.to_string().contains("shadow"));
}

#[test]
fn test_aegis_error_protection() {
    let err = AegisError::Protection("threat detected".to_string());
    assert!(err.to_string().contains("protection"));
}

#[test]
fn test_aegis_error_not_found() {
    let err = AegisError::NotFound("missing item".to_string());
    assert!(err.to_string().contains("not found"));
}

#[test]
fn test_aegis_error_from_io() {
    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let err: AegisError = io_err.into();
    assert!(err.to_string().contains("io error"));
}

#[test]
fn test_aegis_result_ok() {
    let result: AegisResult<i32> = Ok(42);
    assert_eq!(result.unwrap(), 42);
}

#[test]
fn test_aegis_result_err() {
    let result: AegisResult<i32> = Err(AegisError::Validation("bad".to_string()));
    assert!(result.is_err());
}

// === Language Tests ===

#[test]
fn test_language_from_str_rust() {
    assert_eq!(Language::from_str_loose("rust"), Language::Rust);
    assert_eq!(Language::from_str_loose("rs"), Language::Rust);
    assert_eq!(Language::from_str_loose("Rust"), Language::Rust);
}

#[test]
fn test_language_from_str_python() {
    assert_eq!(Language::from_str_loose("python"), Language::Python);
    assert_eq!(Language::from_str_loose("py"), Language::Python);
}

#[test]
fn test_language_from_str_javascript() {
    assert_eq!(Language::from_str_loose("javascript"), Language::JavaScript);
    assert_eq!(Language::from_str_loose("js"), Language::JavaScript);
}

#[test]
fn test_language_from_str_typescript() {
    assert_eq!(Language::from_str_loose("typescript"), Language::TypeScript);
    assert_eq!(Language::from_str_loose("ts"), Language::TypeScript);
}

#[test]
fn test_language_from_str_go() {
    assert_eq!(Language::from_str_loose("go"), Language::Go);
    assert_eq!(Language::from_str_loose("golang"), Language::Go);
}

#[test]
fn test_language_from_str_java() {
    assert_eq!(Language::from_str_loose("java"), Language::Java);
}

#[test]
fn test_language_from_str_csharp() {
    assert_eq!(Language::from_str_loose("csharp"), Language::CSharp);
    assert_eq!(Language::from_str_loose("c#"), Language::CSharp);
}

#[test]
fn test_language_from_str_unknown() {
    assert_eq!(Language::from_str_loose("brainfuck"), Language::Unknown);
}

#[test]
fn test_language_as_str() {
    assert_eq!(Language::Rust.as_str(), "rust");
    assert_eq!(Language::Python.as_str(), "python");
    assert_eq!(Language::Unknown.as_str(), "unknown");
}

#[test]
fn test_language_default() {
    assert_eq!(Language::default(), Language::Unknown);
}

// === ValidationContext Tests ===

#[test]
fn test_validation_context_new() {
    let ctx = ValidationContext::new(SessionId::new(), Language::Rust, "test.rs".to_string());
    assert_eq!(ctx.language, Language::Rust);
    assert_eq!(ctx.file_path, "test.rs");
    assert!(ctx.accumulated_code.is_empty());
    assert_eq!(ctx.chunk_index, 0);
}

#[test]
fn test_validation_context_append_chunk() {
    let mut ctx = ValidationContext::new(SessionId::new(), Language::Rust, "test.rs".to_string());
    ctx.append_chunk("fn main() {\n");
    assert_eq!(ctx.accumulated_code, "fn main() {\n");
    assert_eq!(ctx.chunk_index, 1);

    ctx.append_chunk("    println!(\"hello\");\n");
    assert!(ctx.accumulated_code.contains("println"));
    assert_eq!(ctx.chunk_index, 2);
}

// === ValidationError Tests ===

#[test]
fn test_validation_error_new() {
    let err = ValidationError::new("test".to_string(), ValidationSeverity::Error);
    assert_eq!(err.message, "test");
    assert_eq!(err.severity, ValidationSeverity::Error);
    assert!(err.line.is_none());
}

#[test]
fn test_validation_error_with_location() {
    let err = ValidationError::error("test".to_string()).with_location(10, 5);
    assert_eq!(err.line, Some(10));
    assert_eq!(err.column, Some(5));
}

#[test]
fn test_validation_error_with_suggestion() {
    let err = ValidationError::warning("test".to_string()).with_suggestion("fix it".to_string());
    assert_eq!(err.suggestion, Some("fix it".to_string()));
}

// === StreamingValidation Tests ===

#[test]
fn test_streaming_validation_ok() {
    let v = StreamingValidation::ok(0);
    assert!(v.valid);
    assert!(!v.should_stop);
    assert!(v.errors.is_empty());
    assert_eq!(v.confidence, 1.0);
}

#[test]
fn test_streaming_validation_fail() {
    let errors = vec![ValidationError::error("bad".to_string())];
    let v = StreamingValidation::fail(errors, 5);
    assert!(!v.valid);
    assert!(v.should_stop);
    assert_eq!(v.chunk_index, 5);
}

#[test]
fn test_streaming_validation_with_hint() {
    let v = StreamingValidation::ok(0).with_hint("try this".to_string());
    assert_eq!(v.correction_hint, Some("try this".to_string()));
}

#[test]
fn test_streaming_validation_with_confidence() {
    let v = StreamingValidation::ok(0).with_confidence(0.75);
    assert_eq!(v.confidence, 0.75);
}

#[test]
fn test_streaming_validation_confidence_clamped() {
    let v = StreamingValidation::ok(0).with_confidence(1.5);
    assert_eq!(v.confidence, 1.0);
    let v2 = StreamingValidation::ok(0).with_confidence(-0.5);
    assert_eq!(v2.confidence, 0.0);
}

#[test]
fn test_streaming_validation_default() {
    let v = StreamingValidation::default();
    assert!(v.valid);
}

// === ValidationResult Tests ===

#[test]
fn test_validation_result_success() {
    let r = ValidationResult::success(Language::Rust, 10);
    assert!(r.valid);
    assert_eq!(r.total_chunks, 10);
    assert_eq!(r.confidence, 1.0);
}

#[test]
fn test_validation_result_failure() {
    let errors = vec![ValidationError::error("bad".to_string())];
    let r = ValidationResult::failure(errors, Language::Python);
    assert!(!r.valid);
    assert_eq!(r.language, Language::Python);
}

// === SessionState Tests ===

#[test]
fn test_session_state_default() {
    assert_eq!(SessionState::default(), SessionState::Created);
}

#[test]
fn test_session_state_is_active() {
    assert!(SessionState::Active.is_active());
    assert!(SessionState::Paused.is_active());
    assert!(!SessionState::Created.is_active());
    assert!(!SessionState::Completed.is_active());
}

#[test]
fn test_session_state_is_terminal() {
    assert!(SessionState::Completed.is_terminal());
    assert!(SessionState::Failed.is_terminal());
    assert!(SessionState::RolledBack.is_terminal());
    assert!(!SessionState::Active.is_terminal());
}

#[test]
fn test_session_state_transitions() {
    assert!(SessionState::Created.can_transition_to(&SessionState::Active));
    assert!(SessionState::Active.can_transition_to(&SessionState::Paused));
    assert!(SessionState::Active.can_transition_to(&SessionState::Completed));
    assert!(SessionState::Active.can_transition_to(&SessionState::Failed));
    assert!(SessionState::Paused.can_transition_to(&SessionState::Active));
    assert!(!SessionState::Completed.can_transition_to(&SessionState::Active));
    assert!(!SessionState::Created.can_transition_to(&SessionState::Completed));
}

// === SessionConfig Tests ===

#[test]
fn test_session_config_default() {
    let config = SessionConfig::default();
    assert_eq!(config.language, Language::Unknown);
    assert_eq!(config.max_errors, 50);
    assert!(!config.stop_on_first_error);
    assert!(config.enable_type_checking);
    assert!(config.enable_security_scan);
}

// === ValidationSession Tests ===

#[test]
fn test_validation_session_new() {
    let session = ValidationSession::new(SessionConfig::default());
    assert_eq!(session.state, SessionState::Created);
    assert_eq!(session.total_chunks_processed, 0);
    assert_eq!(session.total_errors, 0);
}

#[test]
fn test_validation_session_activate() {
    let mut session = ValidationSession::new(SessionConfig::default());
    assert!(session.activate().is_ok());
    assert_eq!(session.state, SessionState::Active);
}

#[test]
fn test_validation_session_pause() {
    let mut session = ValidationSession::new(SessionConfig::default());
    session.activate().unwrap();
    assert!(session.pause().is_ok());
    assert_eq!(session.state, SessionState::Paused);
}

#[test]
fn test_validation_session_complete() {
    let mut session = ValidationSession::new(SessionConfig::default());
    session.activate().unwrap();
    assert!(session.complete().is_ok());
    assert_eq!(session.state, SessionState::Completed);
}

#[test]
fn test_validation_session_fail() {
    let mut session = ValidationSession::new(SessionConfig::default());
    session.activate().unwrap();
    assert!(session.fail().is_ok());
    assert_eq!(session.state, SessionState::Failed);
}

#[test]
fn test_validation_session_invalid_transition() {
    let mut session = ValidationSession::new(SessionConfig::default());
    assert!(session.complete().is_err());
}

#[test]
fn test_validation_session_take_snapshot() {
    let mut session = ValidationSession::new(SessionConfig::default());
    session.activate().unwrap();
    session.context.append_chunk("fn main() {}");
    session.take_snapshot();
    assert_eq!(session.snapshots.len(), 1);
    assert_eq!(session.snapshots[0].code, "fn main() {}");
}

#[test]
fn test_validation_session_over_error_limit() {
    let config = SessionConfig {
        max_errors: 5,
        ..Default::default()
    };
    let mut session = ValidationSession::new(config);
    session.total_errors = 6;
    assert!(session.is_over_error_limit());
}

// === ThreatLevel Tests ===

#[test]
fn test_threat_level_score() {
    assert_eq!(ThreatLevel::None.score(), 0.0);
    assert_eq!(ThreatLevel::Low.score(), 0.25);
    assert_eq!(ThreatLevel::Medium.score(), 0.5);
    assert_eq!(ThreatLevel::High.score(), 0.75);
    assert_eq!(ThreatLevel::Critical.score(), 1.0);
}

#[test]
fn test_threat_level_from_score() {
    assert_eq!(ThreatLevel::from_score(0.0), ThreatLevel::None);
    assert_eq!(ThreatLevel::from_score(0.1), ThreatLevel::Low);
    assert_eq!(ThreatLevel::from_score(0.5), ThreatLevel::Medium);
    assert_eq!(ThreatLevel::from_score(0.8), ThreatLevel::High);
    assert_eq!(ThreatLevel::from_score(0.95), ThreatLevel::Critical);
}

#[test]
fn test_threat_level_is_blocking() {
    assert!(!ThreatLevel::None.is_blocking());
    assert!(!ThreatLevel::Low.is_blocking());
    assert!(!ThreatLevel::Medium.is_blocking());
    assert!(ThreatLevel::High.is_blocking());
    assert!(ThreatLevel::Critical.is_blocking());
}

#[test]
fn test_threat_level_ordering() {
    assert!(ThreatLevel::None < ThreatLevel::Low);
    assert!(ThreatLevel::Low < ThreatLevel::Medium);
    assert!(ThreatLevel::Medium < ThreatLevel::High);
    assert!(ThreatLevel::High < ThreatLevel::Critical);
}

// === SecurityCategory Tests ===

#[test]
fn test_security_category_as_str() {
    assert_eq!(
        SecurityCategory::PromptInjection.as_str(),
        "prompt_injection"
    );
    assert_eq!(SecurityCategory::SqlInjection.as_str(), "sql_injection");
    assert_eq!(SecurityCategory::XssAttack.as_str(), "xss_attack");
    assert_eq!(SecurityCategory::PiiExposure.as_str(), "pii_exposure");
}

// === SecurityIssue Tests ===

#[test]
fn test_security_issue_new() {
    let issue = SecurityIssue::new(
        SecurityCategory::PromptInjection,
        ThreatLevel::High,
        "test".to_string(),
    );
    assert_eq!(issue.category, SecurityCategory::PromptInjection);
    assert_eq!(issue.threat_level, ThreatLevel::High);
}

#[test]
fn test_security_issue_with_location() {
    let issue = SecurityIssue::new(
        SecurityCategory::XssAttack,
        ThreatLevel::Medium,
        "xss".to_string(),
    )
    .with_location(5, 10);
    assert_eq!(issue.line, Some(5));
    assert_eq!(issue.column, Some(10));
}

#[test]
fn test_security_issue_with_evidence() {
    let issue = SecurityIssue::new(
        SecurityCategory::SqlInjection,
        ThreatLevel::High,
        "sql".to_string(),
    )
    .with_evidence("DROP TABLE".to_string());
    assert_eq!(issue.evidence, Some("DROP TABLE".to_string()));
}

// === SecurityScan Tests ===

#[test]
fn test_security_scan_clean() {
    let scan = SecurityScan::clean(100, 5);
    assert!(scan.is_safe);
    assert_eq!(scan.overall_threat, ThreatLevel::None);
    assert_eq!(scan.lines_scanned, 100);
}

#[test]
fn test_security_scan_with_issues() {
    let issues = vec![SecurityIssue::new(
        SecurityCategory::SqlInjection,
        ThreatLevel::High,
        "sql".to_string(),
    )];
    let scan = SecurityScan::with_issues(issues, 50, 10);
    assert!(!scan.is_safe);
    assert_eq!(scan.overall_threat, ThreatLevel::High);
    assert_eq!(scan.issues.len(), 1);
}

// === PiiKind Tests ===

#[test]
fn test_pii_kind_as_str() {
    assert_eq!(PiiKind::Email.as_str(), "email");
    assert_eq!(PiiKind::Phone.as_str(), "phone");
    assert_eq!(PiiKind::CreditCard.as_str(), "credit_card");
    assert_eq!(PiiKind::ApiKey.as_str(), "api_key");
    assert_eq!(PiiKind::PrivateKey.as_str(), "private_key");
}
