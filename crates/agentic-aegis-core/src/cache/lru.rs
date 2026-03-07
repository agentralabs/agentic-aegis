use std::collections::HashMap;
use std::hash::Hash;
use std::sync::RwLock;
use std::time::{Duration, Instant};

use super::metrics::CacheMetrics;

/// Entry stored in the LRU cache with TTL tracking.
struct CacheEntry<V> {
    value: V,
    inserted_at: Instant,
    last_accessed: Instant,
}

/// A generic LRU cache with TTL-based expiration.
///
/// Thread-safe via `RwLock<HashMap>`. When the cache is full,
/// the least-recently-used entry is evicted on insert.
pub struct LruCache<K, V> {
    store: RwLock<HashMap<K, CacheEntry<V>>>,
    max_size: usize,
    ttl: Duration,
    metrics: CacheMetrics,
}

impl<K, V> LruCache<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    /// Create a new LRU cache with the given capacity and TTL.
    pub fn new(max_size: usize, ttl: Duration) -> Self {
        Self {
            store: RwLock::new(HashMap::with_capacity(max_size)),
            max_size,
            ttl,
            metrics: CacheMetrics::new(),
        }
    }

    /// Retrieve a value from the cache, returning `None` on miss or expiry.
    pub fn get(&self, key: &K) -> Option<V> {
        let now = Instant::now();
        // First try a read lock to check existence and TTL
        {
            let store = self.store.read().unwrap();
            if let Some(entry) = store.get(key) {
                if now.duration_since(entry.inserted_at) > self.ttl {
                    // Expired — will remove below
                    drop(store);
                    let mut store = self.store.write().unwrap();
                    store.remove(key);
                    self.metrics.record_eviction();
                    self.metrics.set_size(store.len());
                    self.metrics.record_miss();
                    return None;
                }
                // Clone value, then upgrade to write to update last_accessed
                let value = entry.value.clone();
                drop(store);
                if let Ok(mut store) = self.store.write() {
                    if let Some(entry) = store.get_mut(key) {
                        entry.last_accessed = now;
                    }
                }
                self.metrics.record_hit();
                return Some(value);
            }
        }
        self.metrics.record_miss();
        None
    }

    /// Insert a key-value pair, evicting the LRU entry if at capacity.
    pub fn insert(&self, key: K, value: V) {
        let now = Instant::now();
        let mut store = self.store.write().unwrap();

        // Purge expired entries first
        let expired_keys: Vec<K> = store
            .iter()
            .filter(|(_, e)| now.duration_since(e.inserted_at) > self.ttl)
            .map(|(k, _)| k.clone())
            .collect();
        for k in &expired_keys {
            store.remove(k);
            self.metrics.record_eviction();
        }

        // Evict LRU if still at capacity
        if store.len() >= self.max_size && !store.contains_key(&key) {
            self.evict_lru(&mut store);
        }

        store.insert(
            key,
            CacheEntry {
                value,
                inserted_at: now,
                last_accessed: now,
            },
        );
        self.metrics.set_size(store.len());
    }

    /// Invalidate (remove) a specific key.
    pub fn invalidate(&self, key: &K) -> bool {
        let mut store = self.store.write().unwrap();
        let removed = store.remove(key).is_some();
        if removed {
            self.metrics.record_eviction();
            self.metrics.set_size(store.len());
        }
        removed
    }

    /// Clear all entries.
    pub fn clear(&self) {
        let mut store = self.store.write().unwrap();
        let count = store.len();
        store.clear();
        for _ in 0..count {
            self.metrics.record_eviction();
        }
        self.metrics.set_size(0);
    }

    /// Check if the cache contains a live (non-expired) entry for the key.
    pub fn contains(&self, key: &K) -> bool {
        let store = self.store.read().unwrap();
        if let Some(entry) = store.get(key) {
            Instant::now().duration_since(entry.inserted_at) <= self.ttl
        } else {
            false
        }
    }

    /// Return the number of entries (including possibly expired ones).
    pub fn len(&self) -> usize {
        self.store.read().unwrap().len()
    }

    /// Check if the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Access the cache metrics.
    pub fn metrics(&self) -> &CacheMetrics {
        &self.metrics
    }

    /// Evict the least-recently-used entry from the store.
    fn evict_lru(&self, store: &mut HashMap<K, CacheEntry<V>>) {
        if store.is_empty() {
            return;
        }
        let lru_key = store
            .iter()
            .min_by_key(|(_, e)| e.last_accessed)
            .map(|(k, _)| k.clone());
        if let Some(key) = lru_key {
            store.remove(&key);
            self.metrics.record_eviction();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_insert_and_get() {
        let cache = LruCache::new(10, Duration::from_secs(60));
        cache.insert("key1".to_string(), 42);
        assert_eq!(cache.get(&"key1".to_string()), Some(42));
    }

    #[test]
    fn test_miss_returns_none() {
        let cache: LruCache<String, i32> = LruCache::new(10, Duration::from_secs(60));
        assert_eq!(cache.get(&"missing".to_string()), None);
    }

    #[test]
    fn test_ttl_expiration() {
        let cache = LruCache::new(10, Duration::from_millis(50));
        cache.insert("key".to_string(), 1);
        thread::sleep(Duration::from_millis(100));
        assert_eq!(cache.get(&"key".to_string()), None);
    }

    #[test]
    fn test_eviction_on_full() {
        let cache = LruCache::new(2, Duration::from_secs(60));
        cache.insert("a".to_string(), 1);
        cache.insert("b".to_string(), 2);
        cache.insert("c".to_string(), 3);
        // One of a or b should be evicted
        assert_eq!(cache.len(), 2);
        assert!(cache.contains(&"c".to_string()));
    }

    #[test]
    fn test_lru_evicts_oldest_accessed() {
        let cache = LruCache::new(2, Duration::from_secs(60));
        cache.insert("a".to_string(), 1);
        cache.insert("b".to_string(), 2);
        // Access "a" to make it more recent
        let _ = cache.get(&"a".to_string());
        cache.insert("c".to_string(), 3);
        // "b" should have been evicted (least recently used)
        assert_eq!(cache.get(&"a".to_string()), Some(1));
        assert_eq!(cache.get(&"b".to_string()), None);
        assert_eq!(cache.get(&"c".to_string()), Some(3));
    }

    #[test]
    fn test_invalidate() {
        let cache = LruCache::new(10, Duration::from_secs(60));
        cache.insert("key".to_string(), 42);
        assert!(cache.invalidate(&"key".to_string()));
        assert_eq!(cache.get(&"key".to_string()), None);
    }

    #[test]
    fn test_clear() {
        let cache = LruCache::new(10, Duration::from_secs(60));
        cache.insert("a".to_string(), 1);
        cache.insert("b".to_string(), 2);
        cache.clear();
        assert!(cache.is_empty());
    }

    #[test]
    fn test_contains() {
        let cache = LruCache::new(10, Duration::from_secs(60));
        cache.insert("key".to_string(), 1);
        assert!(cache.contains(&"key".to_string()));
        assert!(!cache.contains(&"other".to_string()));
    }

    #[test]
    fn test_metrics_tracking() {
        let cache = LruCache::new(10, Duration::from_secs(60));
        cache.insert("key".to_string(), 1);
        let _ = cache.get(&"key".to_string());
        let _ = cache.get(&"missing".to_string());
        assert_eq!(cache.metrics().hit_count(), 1);
        assert_eq!(cache.metrics().miss_count(), 1);
    }

    #[test]
    fn test_overwrite_existing_key() {
        let cache = LruCache::new(10, Duration::from_secs(60));
        cache.insert("key".to_string(), 1);
        cache.insert("key".to_string(), 2);
        assert_eq!(cache.get(&"key".to_string()), Some(2));
        assert_eq!(cache.len(), 1);
    }
}
