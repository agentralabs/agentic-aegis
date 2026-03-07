use agentic_aegis_core::bridges::*;
use agentic_aegis_core::bridges::hydra::*;

// === NoOp Bridge Tests ===

#[test]
fn test_noop_aegis_bridge_name() {
    let bridge = NoOpBridges;
    assert_eq!(bridge.name(), "aegis");
}

#[test]
fn test_noop_aegis_bridge_version() {
    let bridge = NoOpBridges;
    assert!(!bridge.version().is_empty());
}

#[test]
fn test_noop_time_bridge() {
    let bridge = NoOpBridges;
    assert!(bridge.check_deadline("session1").is_ok());
    assert!(bridge.record_validation_time("session1", 100).is_ok());
}

#[test]
fn test_noop_contract_bridge() {
    let bridge = NoOpBridges;
    assert!(bridge.check_validation_policy("code").unwrap());
    assert!(bridge.report_validation_result("session1", true).is_ok());
}

#[test]
fn test_noop_identity_bridge() {
    let bridge = NoOpBridges;
    assert!(bridge.verify_agent_identity("agent1").unwrap());
    let signature = bridge.sign_validation_result("result").unwrap();
    assert!(signature.is_empty());
}

#[test]
fn test_noop_memory_bridge() {
    let bridge = NoOpBridges;
    assert!(bridge.store_validation_context("session1", "ctx").is_ok());
    assert!(bridge.recall_validation_pattern("pattern").unwrap().is_none());
}

#[test]
fn test_noop_cognition_bridge() {
    let bridge = NoOpBridges;
    assert_eq!(bridge.assess_code_quality("code").unwrap(), 1.0);
    assert!(bridge.get_user_preferences().unwrap().is_none());
}

#[test]
fn test_noop_comm_bridge() {
    let bridge = NoOpBridges;
    assert!(bridge.broadcast_validation_event("event", "payload").is_ok());
    assert!(bridge.notify_validation_failure("session1", "error").is_ok());
}

#[test]
fn test_noop_codebase_bridge() {
    let bridge = NoOpBridges;
    assert!(bridge.get_file_context("file.rs").unwrap().is_none());
    assert!(bridge.get_project_types().unwrap().is_empty());
}

#[test]
fn test_noop_vision_bridge() {
    let bridge = NoOpBridges;
    let state = bridge.capture_validation_state("session1").unwrap();
    assert!(state.is_empty());
}

#[test]
fn test_noop_planning_bridge() {
    let bridge = NoOpBridges;
    assert!(bridge.register_validation_constraint("constraint").is_ok());
    assert!(bridge.get_generation_plan().unwrap().is_none());
}

#[test]
fn test_noop_reality_bridge() {
    let bridge = NoOpBridges;
    assert!(bridge.check_resource_availability().unwrap());
    assert!(bridge.get_deployment_context().unwrap().is_none());
}

#[test]
fn test_noop_hydra_adapter() {
    let bridge = NoOpBridges;
    assert!(bridge.register_with_hydra().is_ok());
    assert!(bridge.report_to_hydra("event", "payload").is_ok());
}

#[test]
fn test_noop_ghost_writer() {
    let mut bridge = NoOpBridges;
    let snapshot = bridge.snapshot().unwrap();
    assert!(snapshot.is_empty());
    assert!(bridge.restore(&[]).is_ok());
}

// === Foundation Bridge Tests ===

#[test]
fn test_foundation_bridges_time() {
    let bridge = agentic_aegis_core::bridges::foundation::FoundationBridges;
    assert!(bridge.check_deadline("session1").is_ok());
}

#[test]
fn test_foundation_bridges_contract() {
    let bridge = agentic_aegis_core::bridges::foundation::FoundationBridges;
    assert!(bridge.check_validation_policy("code").unwrap());
}

#[test]
fn test_foundation_bridges_identity() {
    let bridge = agentic_aegis_core::bridges::foundation::FoundationBridges;
    assert!(bridge.verify_agent_identity("agent").unwrap());
}

#[test]
fn test_foundation_bridges_memory() {
    let bridge = agentic_aegis_core::bridges::foundation::FoundationBridges;
    assert!(bridge.store_validation_context("s", "c").is_ok());
}

#[test]
fn test_foundation_bridges_hydra() {
    let bridge = agentic_aegis_core::bridges::foundation::FoundationBridges;
    assert!(bridge.register_with_hydra().is_ok());
}
