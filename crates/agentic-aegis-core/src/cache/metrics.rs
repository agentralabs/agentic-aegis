use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

use serde::{Deserialize, Serialize};

/// Thread-safe cache performance metrics using atomics.
pub struct CacheMetrics {
    hit_count: AtomicU64,
    miss_count: AtomicU64,
    eviction_count: AtomicU64,
    current_size: AtomicUsize,
}

/// Serializable snapshot of cache metrics at a point in time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMetricsSnapshot {
    pub hit_count: u64,
    pub miss_count: u64,
    pub eviction_count: u64,
    pub current_size: usize,
    pub hit_rate: f64,
}

impl CacheMetrics {
    /// Create a new zeroed metrics tracker.
    pub fn new() -> Self {
        Self {
            hit_count: AtomicU64::new(0),
            miss_count: AtomicU64::new(0),
            eviction_count: AtomicU64::new(0),
            current_size: AtomicUsize::new(0),
        }
    }

    /// Record a cache hit.
    pub fn record_hit(&self) {
        self.hit_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a cache miss.
    pub fn record_miss(&self) {
        self.miss_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Record an eviction.
    pub fn record_eviction(&self) {
        self.eviction_count.fetch_add(1, Ordering::Relaxed);
    }

    /// Set the current cache size.
    pub fn set_size(&self, size: usize) {
        self.current_size.store(size, Ordering::Relaxed);
    }

    /// Get total cache hits.
    pub fn hit_count(&self) -> u64 {
        self.hit_count.load(Ordering::Relaxed)
    }

    /// Get total cache misses.
    pub fn miss_count(&self) -> u64 {
        self.miss_count.load(Ordering::Relaxed)
    }

    /// Get total evictions.
    pub fn eviction_count(&self) -> u64 {
        self.eviction_count.load(Ordering::Relaxed)
    }

    /// Get current cache size.
    pub fn current_size(&self) -> usize {
        self.current_size.load(Ordering::Relaxed)
    }

    /// Calculate the cache hit rate as a ratio (0.0 to 1.0).
    /// Returns 0.0 if no lookups have been performed.
    pub fn hit_rate(&self) -> f64 {
        let hits = self.hit_count() as f64;
        let misses = self.miss_count() as f64;
        let total = hits + misses;
        if total == 0.0 {
            0.0
        } else {
            hits / total
        }
    }

    /// Take a serializable snapshot of the current metrics.
    pub fn snapshot(&self) -> CacheMetricsSnapshot {
        CacheMetricsSnapshot {
            hit_count: self.hit_count(),
            miss_count: self.miss_count(),
            eviction_count: self.eviction_count(),
            current_size: self.current_size(),
            hit_rate: self.hit_rate(),
        }
    }

    /// Reset all counters to zero.
    pub fn reset(&self) {
        self.hit_count.store(0, Ordering::Relaxed);
        self.miss_count.store(0, Ordering::Relaxed);
        self.eviction_count.store(0, Ordering::Relaxed);
        self.current_size.store(0, Ordering::Relaxed);
    }
}

impl Default for CacheMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_values() {
        let m = CacheMetrics::new();
        assert_eq!(m.hit_count(), 0);
        assert_eq!(m.miss_count(), 0);
        assert_eq!(m.eviction_count(), 0);
        assert_eq!(m.current_size(), 0);
    }

    #[test]
    fn test_record_hit() {
        let m = CacheMetrics::new();
        m.record_hit();
        m.record_hit();
        assert_eq!(m.hit_count(), 2);
    }

    #[test]
    fn test_record_miss() {
        let m = CacheMetrics::new();
        m.record_miss();
        assert_eq!(m.miss_count(), 1);
    }

    #[test]
    fn test_record_eviction() {
        let m = CacheMetrics::new();
        m.record_eviction();
        m.record_eviction();
        m.record_eviction();
        assert_eq!(m.eviction_count(), 3);
    }

    #[test]
    fn test_hit_rate_no_lookups() {
        let m = CacheMetrics::new();
        assert_eq!(m.hit_rate(), 0.0);
    }

    #[test]
    fn test_hit_rate_all_hits() {
        let m = CacheMetrics::new();
        m.record_hit();
        m.record_hit();
        assert_eq!(m.hit_rate(), 1.0);
    }

    #[test]
    fn test_hit_rate_mixed() {
        let m = CacheMetrics::new();
        m.record_hit();
        m.record_miss();
        assert!((m.hit_rate() - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_set_size() {
        let m = CacheMetrics::new();
        m.set_size(42);
        assert_eq!(m.current_size(), 42);
    }

    #[test]
    fn test_snapshot() {
        let m = CacheMetrics::new();
        m.record_hit();
        m.record_miss();
        m.record_eviction();
        m.set_size(5);
        let snap = m.snapshot();
        assert_eq!(snap.hit_count, 1);
        assert_eq!(snap.miss_count, 1);
        assert_eq!(snap.eviction_count, 1);
        assert_eq!(snap.current_size, 5);
        assert!((snap.hit_rate - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_reset() {
        let m = CacheMetrics::new();
        m.record_hit();
        m.record_miss();
        m.set_size(10);
        m.reset();
        assert_eq!(m.hit_count(), 0);
        assert_eq!(m.miss_count(), 0);
        assert_eq!(m.current_size(), 0);
    }
}
