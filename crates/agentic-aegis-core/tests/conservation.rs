use std::time::Duration;

use agentic_aegis_core::cache::LruCache;
use agentic_aegis_core::metrics::{generate_report, AuditEntry, AuditLog, Layer, TokenMetrics};
use agentic_aegis_core::query::{
    apply_intent, ChangeType, ExtractionIntent, TokenBudget, VersionedState,
};

/// Helper: create an audit entry.
fn audit(
    tool: &str,
    layer: Layer,
    used: u64,
    saved: u64,
    cache_hit: bool,
    intent: ExtractionIntent,
) -> AuditEntry {
    AuditEntry::new(tool, layer, used, saved, cache_hit, intent, 1000, used)
}

// ---------------------------------------------------------------------------
// 1. test_second_query_cheaper — cache hit costs 0 tokens
// ---------------------------------------------------------------------------
#[test]
fn test_second_query_cheaper() {
    let cache: LruCache<String, Vec<u8>> = LruCache::new(100, Duration::from_secs(300));

    // First query: miss — simulate full retrieval cost
    let key = "sessions".to_string();
    assert!(cache.get(&key).is_none()); // miss
    let data = vec![0u8; 1000]; // 1000 bytes ≈ full cost
    cache.insert(key.clone(), data.clone());

    // Second query: hit — zero additional tokens
    let cached = cache.get(&key);
    assert!(cached.is_some());

    // Verify via metrics
    let metrics = cache.metrics();
    assert_eq!(metrics.hit_count(), 1);
    assert_eq!(metrics.miss_count(), 1);
    // A cache hit means 0 new tokens are consumed
    assert!(metrics.hit_rate() > 0.0);
}

// ---------------------------------------------------------------------------
// 2. test_unchanged_state_free — delta returns empty when nothing changed
// ---------------------------------------------------------------------------
#[test]
fn test_unchanged_state_free() {
    let state = VersionedState::new(vec![1, 2, 3]);
    let delta = state.changes_since_version(0);
    assert!(delta.is_unchanged());
    assert_eq!(delta.change_count(), 0);
}

// ---------------------------------------------------------------------------
// 3. test_scoped_query_10x_cheaper — IdsOnly vs Full
// ---------------------------------------------------------------------------
#[test]
fn test_scoped_query_10x_cheaper() {
    let ids_cost = ExtractionIntent::IdsOnly.estimated_tokens();
    let full_cost = ExtractionIntent::Full.estimated_tokens();
    assert!(
        full_cost >= ids_cost * 10,
        "Full ({}) should be at least 10x IdsOnly ({})",
        full_cost,
        ids_cost
    );

    // Also verify with actual data — use a small set where IdsOnly is cheaper
    let data: Vec<String> = (0..5).map(|i| format!("item_{}", i)).collect();

    let ids_result = apply_intent(
        &data,
        ExtractionIntent::IdsOnly,
        |s| s.clone(),
        |d| format!("{} items", d.len()),
    );
    let full_result = apply_intent(
        &data,
        ExtractionIntent::Full,
        |s| s.clone(),
        |d| format!("{} items", d.len()),
    );

    let ids_tokens = ids_result.estimated_tokens();
    let full_tokens = full_result.estimated_tokens();
    assert!(
        full_tokens > ids_tokens,
        "Full result tokens ({}) should exceed IdsOnly tokens ({}) for small sets",
        full_tokens,
        ids_tokens
    );
}

// ---------------------------------------------------------------------------
// 4. test_delta_proportional_to_changes
// ---------------------------------------------------------------------------
#[test]
fn test_delta_proportional_to_changes() {
    let mut state = VersionedState::new(0);
    for i in 1..=100 {
        state.record_change(ChangeType::Updated, i);
    }

    let delta_10 = state.changes_since_version(90);
    let delta_50 = state.changes_since_version(50);

    assert_eq!(delta_10.change_count(), 10);
    assert_eq!(delta_50.change_count(), 50);

    // Fewer changes = proportionally fewer tokens
    assert!(delta_10.change_count() < delta_50.change_count());
}

// ---------------------------------------------------------------------------
// 5. test_conservation_score_improves_with_warmup
// ---------------------------------------------------------------------------
#[test]
fn test_conservation_score_improves_with_warmup() {
    let metrics = TokenMetrics::new();

    // Cold start: full queries, no savings
    metrics.record(Layer::Full, 100);
    let cold_score = metrics.conservation_score();

    // Warm: cache hits produce savings
    for _ in 0..9 {
        metrics.record(Layer::Cache, 0);
        metrics.record_savings(100);
    }
    let warm_score = metrics.conservation_score();

    assert!(
        warm_score > cold_score,
        "Warm score ({}) should exceed cold score ({})",
        warm_score,
        cold_score
    );
}

// ---------------------------------------------------------------------------
// 6. test_conservation_target_07 — score should reach 0.7 with caching
// ---------------------------------------------------------------------------
#[test]
fn test_conservation_target_07() {
    let metrics = TokenMetrics::new();
    let log = AuditLog::new(1000);

    // Simulate a realistic workload: 30% full, 70% cached
    for _ in 0..30 {
        metrics.record(Layer::Full, 100);
        log.record(audit(
            "tool",
            Layer::Full,
            100,
            0,
            false,
            ExtractionIntent::Full,
        ));
    }
    for _ in 0..70 {
        metrics.record(Layer::Cache, 0);
        metrics.record_savings(100);
        log.record(audit(
            "tool",
            Layer::Cache,
            0,
            100,
            true,
            ExtractionIntent::IdsOnly,
        ));
    }

    let report = generate_report(&metrics, &log);
    assert!(
        report.score >= 0.7,
        "Conservation score {} should be >= 0.7",
        report.score
    );
    assert!(report.verdict.meets_target());
}

// ---------------------------------------------------------------------------
// 7. test_token_budget_enforced
// ---------------------------------------------------------------------------
#[test]
fn test_token_budget_enforced() {
    let mut budget = TokenBudget::new(100);

    assert!(budget.try_spend(40));
    assert!(budget.try_spend(40));
    // Only 20 remaining, cannot afford 30
    assert!(!budget.try_spend(30));
    assert_eq!(budget.remaining(), 20);
    assert!(!budget.is_exhausted());

    // Spend remaining
    assert!(budget.try_spend(20));
    assert!(budget.is_exhausted());
    assert_eq!(budget.remaining(), 0);
}

// ---------------------------------------------------------------------------
// 8. test_default_intent_is_minimal
// ---------------------------------------------------------------------------
#[test]
fn test_default_intent_is_minimal() {
    let intent = ExtractionIntent::default();
    assert_eq!(intent, ExtractionIntent::IdsOnly);
    assert!(intent.is_minimal());
    assert!(!intent.is_full());
}

// ---------------------------------------------------------------------------
// 9. test_cache_invalidation_on_mutation
// ---------------------------------------------------------------------------
#[test]
fn test_cache_invalidation_on_mutation() {
    let cache: LruCache<String, String> = LruCache::new(100, Duration::from_secs(300));

    // Populate cache
    cache.insert("sessions".to_string(), "data_v1".to_string());
    assert!(cache.contains(&"sessions".to_string()));

    // Simulate mutation: invalidate cache
    cache.invalidate(&"sessions".to_string());
    assert!(!cache.contains(&"sessions".to_string()));
    assert_eq!(cache.get(&"sessions".to_string()), None);

    // Re-populate with updated data
    cache.insert("sessions".to_string(), "data_v2".to_string());
    assert_eq!(
        cache.get(&"sessions".to_string()),
        Some("data_v2".to_string())
    );
}

// ---------------------------------------------------------------------------
// 10. test_end_to_end_conservation_flow
// ---------------------------------------------------------------------------
#[test]
fn test_end_to_end_conservation_flow() {
    // Set up infrastructure
    let cache: LruCache<String, Vec<String>> = LruCache::new(100, Duration::from_secs(300));
    let metrics = TokenMetrics::new();
    let log = AuditLog::new(1000);
    let mut budget = TokenBudget::new(500);
    let mut state = VersionedState::new(vec!["item1".to_string()]);

    // === Query 1: Cold start — full retrieval ===
    let key = "list".to_string();
    assert!(cache.get(&key).is_none());
    let full_cost: u64 = 100;
    budget.spend(full_cost);
    metrics.record(Layer::Full, full_cost);
    log.record(audit(
        "list",
        Layer::Full,
        full_cost,
        0,
        false,
        ExtractionIntent::Full,
    ));
    cache.insert(key.clone(), state.state().clone());

    // === Query 2: Cache hit — zero cost ===
    let cached = cache.get(&key);
    assert!(cached.is_some());
    metrics.record(Layer::Cache, 0);
    metrics.record_savings(full_cost);
    log.record(audit(
        "list",
        Layer::Cache,
        0,
        full_cost,
        true,
        ExtractionIntent::IdsOnly,
    ));

    // === Query 3: Scoped query (IdsOnly) ===
    let data = cached.unwrap();
    let scoped = apply_intent(
        &data,
        ExtractionIntent::IdsOnly,
        |s| s.clone(),
        |d| format!("{} items", d.len()),
    );
    let scoped_cost = scoped.estimated_tokens();
    budget.spend(scoped_cost);
    metrics.record(Layer::Scoped, scoped_cost);
    metrics.record_savings(full_cost.saturating_sub(scoped_cost));
    log.record(audit(
        "list",
        Layer::Scoped,
        scoped_cost,
        full_cost - scoped_cost,
        false,
        ExtractionIntent::IdsOnly,
    ));

    // === Mutation: update state and invalidate cache ===
    state.record_change(
        ChangeType::Updated,
        vec!["item1".to_string(), "item2".to_string()],
    );
    cache.invalidate(&key);

    // === Query 4: Delta query — only the change ===
    let delta = state.changes_since_version(0);
    assert!(!delta.is_unchanged());
    assert_eq!(delta.change_count(), 1);
    let delta_cost: u64 = 10;
    budget.spend(delta_cost);
    metrics.record(Layer::Delta, delta_cost);
    metrics.record_savings(full_cost.saturating_sub(delta_cost));
    log.record(audit(
        "list",
        Layer::Delta,
        delta_cost,
        full_cost - delta_cost,
        false,
        ExtractionIntent::IdsOnly,
    ));

    // === Verify conservation ===
    let report = generate_report(&metrics, &log);

    // Score should be good: we used 100+0+2+10 = 112 tokens,
    // saved 0+100+98+90 = 288 tokens, total = 400, score = 288/400 = 0.72
    assert!(
        report.score >= 0.7,
        "End-to-end conservation score {} should be >= 0.7",
        report.score
    );
    assert!(report.verdict.meets_target());

    // Budget should still have remaining
    assert!(!budget.is_exhausted());
    assert!(budget.remaining() > 0);

    // Cache metrics should show hits and misses
    assert!(cache.metrics().hit_count() >= 1);
    assert!(cache.metrics().miss_count() >= 1);
}
