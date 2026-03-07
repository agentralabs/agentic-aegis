use agentic_aegis_core::session::*;
use agentic_aegis_core::types::*;

// === SessionManager Tests ===

#[tokio::test]
async fn test_session_manager_create() {
    let mut mgr = SessionManager::new();
    let config = SessionConfig {
        language: Language::Rust,
        ..Default::default()
    };
    let id = mgr.create_session(config).unwrap();
    assert!(!id.to_string().is_empty());
}

#[tokio::test]
async fn test_session_manager_get_session() {
    let mut mgr = SessionManager::new();
    let config = SessionConfig {
        language: Language::Python,
        ..Default::default()
    };
    let id = mgr.create_session(config).unwrap();
    let session = mgr.get_session(&id.to_string()).unwrap();
    assert_eq!(session.config.language, Language::Python);
}

#[tokio::test]
async fn test_session_manager_session_not_found() {
    let mgr = SessionManager::new();
    let result = mgr.get_session("nonexistent");
    assert!(result.is_err());
}

#[tokio::test]
async fn test_session_manager_validate_chunk() {
    let mut mgr = SessionManager::new();
    let config = SessionConfig {
        language: Language::Rust,
        ..Default::default()
    };
    let id = mgr.create_session(config).unwrap();
    let result = mgr
        .validate_chunk(&id.to_string(), "fn main() { }")
        .await
        .unwrap();
    assert!(result.valid);
}

#[tokio::test]
async fn test_session_manager_validate_multiple_chunks() {
    let mut mgr = SessionManager::new();
    let config = SessionConfig {
        language: Language::Rust,
        ..Default::default()
    };
    let id = mgr.create_session(config).unwrap();

    let r1 = mgr
        .validate_chunk(&id.to_string(), "fn main() {\n")
        .await
        .unwrap();
    assert!(r1.valid);

    let r2 = mgr
        .validate_chunk(&id.to_string(), "    let x = 5;\n")
        .await
        .unwrap();
    assert!(r2.valid);

    let r3 = mgr
        .validate_chunk(&id.to_string(), "}\n")
        .await
        .unwrap();
    assert!(r3.valid);
}

#[tokio::test]
async fn test_session_manager_end_session() {
    let mut mgr = SessionManager::new();
    let config = SessionConfig::default();
    let id = mgr.create_session(config).unwrap();
    assert!(mgr.end_session(&id.to_string()).is_ok());
}

#[tokio::test]
async fn test_session_manager_end_nonexistent() {
    let mut mgr = SessionManager::new();
    assert!(mgr.end_session("nonexistent").is_err());
}

#[tokio::test]
async fn test_session_manager_list_sessions() {
    let mut mgr = SessionManager::new();
    mgr.create_session(SessionConfig::default()).unwrap();
    mgr.create_session(SessionConfig::default()).unwrap();
    assert_eq!(mgr.list_sessions().len(), 2);
}

#[tokio::test]
async fn test_session_manager_active_sessions() {
    let mut mgr = SessionManager::new();
    let id1 = mgr.create_session(SessionConfig::default()).unwrap();
    mgr.create_session(SessionConfig::default()).unwrap();
    mgr.end_session(&id1.to_string()).unwrap();
    assert_eq!(mgr.active_sessions().len(), 1);
}

#[tokio::test]
async fn test_session_manager_session_count() {
    let mut mgr = SessionManager::new();
    assert_eq!(mgr.session_count(), 0);
    mgr.create_session(SessionConfig::default()).unwrap();
    assert_eq!(mgr.session_count(), 1);
}

#[tokio::test]
async fn test_session_manager_remove_session() {
    let mut mgr = SessionManager::new();
    let id = mgr.create_session(SessionConfig::default()).unwrap();
    let removed = mgr.remove_session(&id.to_string()).unwrap();
    assert_eq!(removed.id, id);
    assert_eq!(mgr.session_count(), 0);
}

#[tokio::test]
async fn test_session_manager_validate_ended_session() {
    let mut mgr = SessionManager::new();
    let config = SessionConfig::default();
    let id = mgr.create_session(config).unwrap();
    mgr.end_session(&id.to_string()).unwrap();
    let result = mgr.validate_chunk(&id.to_string(), "code").await;
    assert!(result.is_err());
}

// === SessionStateMachine Tests ===

#[test]
fn test_state_machine_new() {
    let sm = SessionStateMachine::new();
    assert_eq!(*sm.current(), SessionState::Created);
    assert!(!sm.is_active());
    assert!(!sm.is_terminal());
}

#[test]
fn test_state_machine_transition_to_active() {
    let mut sm = SessionStateMachine::new();
    assert!(sm.transition(SessionState::Active).is_ok());
    assert!(sm.is_active());
}

#[test]
fn test_state_machine_invalid_transition() {
    let mut sm = SessionStateMachine::new();
    assert!(sm.transition(SessionState::Completed).is_err());
}

#[test]
fn test_state_machine_history() {
    let mut sm = SessionStateMachine::new();
    sm.transition(SessionState::Active).unwrap();
    sm.transition(SessionState::Paused).unwrap();
    assert_eq!(sm.history().len(), 3); // Created -> Active -> Paused
}

#[test]
fn test_state_machine_terminal() {
    let mut sm = SessionStateMachine::new();
    sm.transition(SessionState::Active).unwrap();
    sm.transition(SessionState::Completed).unwrap();
    assert!(sm.is_terminal());
}

#[test]
fn test_state_machine_duration() {
    let sm = SessionStateMachine::new();
    let duration = sm.duration_in_current_state();
    assert!(duration >= chrono::Duration::zero());
}

// === RollbackEngine Tests ===

#[test]
fn test_rollback_engine_new() {
    let engine = RollbackEngine::new();
    assert_eq!(engine.snapshot_count(), 0);
}

#[test]
fn test_rollback_engine_save_snapshot() {
    let mut engine = RollbackEngine::new();
    let snapshot = SessionSnapshot {
        code: "fn main() {}".to_string(),
        chunk_index: 0,
        timestamp: chrono::Utc::now(),
    };
    let id = engine.save_snapshot(snapshot);
    assert!(!id.to_string().is_empty());
    assert_eq!(engine.snapshot_count(), 1);
}

#[test]
fn test_rollback_engine_rollback_to_latest() {
    let mut engine = RollbackEngine::new();
    engine.save_snapshot(SessionSnapshot {
        code: "first".to_string(),
        chunk_index: 0,
        timestamp: chrono::Utc::now(),
    });
    engine.save_snapshot(SessionSnapshot {
        code: "second".to_string(),
        chunk_index: 1,
        timestamp: chrono::Utc::now(),
    });
    let snapshot = engine.rollback_to_latest().unwrap();
    assert_eq!(snapshot.code, "second");
}

#[test]
fn test_rollback_engine_rollback_to_chunk() {
    let mut engine = RollbackEngine::new();
    engine.save_snapshot(SessionSnapshot {
        code: "chunk0".to_string(),
        chunk_index: 0,
        timestamp: chrono::Utc::now(),
    });
    engine.save_snapshot(SessionSnapshot {
        code: "chunk1".to_string(),
        chunk_index: 1,
        timestamp: chrono::Utc::now(),
    });
    engine.save_snapshot(SessionSnapshot {
        code: "chunk2".to_string(),
        chunk_index: 2,
        timestamp: chrono::Utc::now(),
    });
    let snapshot = engine.rollback_to_chunk(1).unwrap();
    assert!(snapshot.chunk_index <= 1);
}

#[test]
fn test_rollback_engine_rollback_empty() {
    let engine = RollbackEngine::new();
    assert!(engine.rollback_to_latest().is_err());
}

#[test]
fn test_rollback_engine_list_snapshots() {
    let mut engine = RollbackEngine::new();
    engine.save_snapshot(SessionSnapshot {
        code: "a".to_string(),
        chunk_index: 0,
        timestamp: chrono::Utc::now(),
    });
    engine.save_snapshot(SessionSnapshot {
        code: "b".to_string(),
        chunk_index: 1,
        timestamp: chrono::Utc::now(),
    });
    let list = engine.list_snapshots();
    assert_eq!(list.len(), 2);
}

#[test]
fn test_rollback_engine_clear() {
    let mut engine = RollbackEngine::new();
    engine.save_snapshot(SessionSnapshot {
        code: "a".to_string(),
        chunk_index: 0,
        timestamp: chrono::Utc::now(),
    });
    engine.clear();
    assert_eq!(engine.snapshot_count(), 0);
}

#[test]
fn test_rollback_engine_prune() {
    let mut engine = RollbackEngine::new();
    for i in 0..10 {
        engine.save_snapshot(SessionSnapshot {
            code: format!("chunk{}", i),
            chunk_index: i,
            timestamp: chrono::Utc::now(),
        });
    }
    engine.prune(3);
    assert_eq!(engine.snapshot_count(), 3);
}

// === CorrectionHintGenerator Tests ===

#[test]
fn test_hint_generator_bracket_error() {
    let gen = CorrectionHintGenerator::new();
    let error = ValidationError::error("unexpected '}' at line 5".to_string());
    let hint = gen.generate_hint(&error, &Language::Rust, "");
    assert!(hint.is_some());
    assert!(hint.unwrap().contains("brace"));
}

#[test]
fn test_hint_generator_type_error() {
    let gen = CorrectionHintGenerator::new();
    let error = ValidationError::error("type mismatch".to_string());
    let hint = gen.generate_hint(&error, &Language::Rust, "");
    assert!(hint.is_some());
}

#[test]
fn test_hint_generator_unclosed_string() {
    let gen = CorrectionHintGenerator::new();
    let error = ValidationError::error("unclosed string literal".to_string());
    let hint = gen.generate_hint(&error, &Language::Rust, "");
    assert!(hint.is_some());
    assert!(hint.unwrap().contains("string"));
}

#[test]
fn test_hint_generator_indentation() {
    let gen = CorrectionHintGenerator::new();
    let error = ValidationError::error("mixed indentation at line 5".to_string());
    let hint = gen.generate_hint(&error, &Language::Python, "");
    assert!(hint.is_some());
    assert!(hint.unwrap().contains("indentation"));
}

#[test]
fn test_hint_generator_unsafe() {
    let gen = CorrectionHintGenerator::new();
    let error = ValidationError::warning("unsafe code detected".to_string());
    let hint = gen.generate_hint(&error, &Language::Rust, "");
    assert!(hint.is_some());
}

#[test]
fn test_hint_generator_hardcoded_password() {
    let gen = CorrectionHintGenerator::new();
    let error = ValidationError::error("hardcoded password detected".to_string());
    let hint = gen.generate_hint(&error, &Language::Rust, "");
    assert!(hint.is_some());
    assert!(hint.unwrap().contains("environment"));
}

#[test]
fn test_hint_generator_multiple() {
    let gen = CorrectionHintGenerator::new();
    let errors = vec![
        ValidationError::error("bracket mismatch".to_string()),
        ValidationError::error("unclosed string".to_string()),
    ];
    let hints = gen.generate_hints(&errors, &Language::Rust, "");
    assert_eq!(hints.len(), 2);
}

#[test]
fn test_hint_generator_type_error_rust_overflow() {
    let gen = CorrectionHintGenerator::new();
    let error = ValidationError::error("type: value out of range for u8".to_string());
    let hint = gen.generate_hint(&error, &Language::Rust, "");
    assert!(hint.is_some());
    assert!(hint.unwrap().contains("type"));
}

#[test]
fn test_hint_generator_type_error_ts_null() {
    let gen = CorrectionHintGenerator::new();
    let error = ValidationError::error("null assignment type mismatch".to_string());
    let hint = gen.generate_hint(&error, &Language::TypeScript, "");
    assert!(hint.is_some());
}
