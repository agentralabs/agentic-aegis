use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::sync::RwLock;

use serde::{Deserialize, Serialize};

/// Tracks dependencies between cache keys and supports cascade invalidation.
///
/// When a key is invalidated, all keys that depend on it are also invalidated.
pub struct CacheInvalidator<K> {
    /// Maps a key to the set of keys that depend on it.
    /// If key A is invalidated, all keys in dependents[A] are also invalidated.
    dependents: RwLock<HashMap<K, HashSet<K>>>,
}

/// Result of a cascade invalidation operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvalidationResult {
    /// Number of keys directly invalidated.
    pub direct: usize,
    /// Number of keys invalidated via cascade.
    pub cascaded: usize,
    /// Total keys invalidated.
    pub total: usize,
}

impl<K> CacheInvalidator<K>
where
    K: Eq + Hash + Clone,
{
    /// Create a new invalidator with no dependencies.
    pub fn new() -> Self {
        Self {
            dependents: RwLock::new(HashMap::new()),
        }
    }

    /// Register that `dependent` depends on `dependency`.
    /// When `dependency` is invalidated, `dependent` will also be invalidated.
    pub fn add_dependency(&self, dependency: K, dependent: K) {
        let mut deps = self.dependents.write().unwrap();
        deps.entry(dependency).or_default().insert(dependent);
    }

    /// Remove a specific dependency relationship.
    pub fn remove_dependency(&self, dependency: &K, dependent: &K) {
        let mut deps = self.dependents.write().unwrap();
        if let Some(set) = deps.get_mut(dependency) {
            set.remove(dependent);
            if set.is_empty() {
                deps.remove(dependency);
            }
        }
    }

    /// Compute the full set of keys that would be invalidated if `key` is invalidated.
    /// Includes the key itself plus all transitive dependents.
    pub fn cascade(&self, key: &K) -> Vec<K> {
        let deps = self.dependents.read().unwrap();
        let mut result = Vec::new();
        let mut visited = HashSet::new();
        let mut stack = vec![key.clone()];

        while let Some(current) = stack.pop() {
            if !visited.insert(current.clone()) {
                continue;
            }
            result.push(current.clone());
            if let Some(dependents) = deps.get(&current) {
                for dep in dependents {
                    if !visited.contains(dep) {
                        stack.push(dep.clone());
                    }
                }
            }
        }
        result
    }

    /// Invalidate a key and return the full set of keys that should be removed
    /// from the cache (including cascaded dependents). Also cleans up dependency
    /// tracking for the invalidated keys.
    pub fn invalidate(&self, key: &K) -> InvalidationResult {
        let to_remove = self.cascade(key);
        let direct = 1;
        let total = to_remove.len();
        let cascaded = total.saturating_sub(direct);

        // Clean up dependency entries for invalidated keys
        let mut deps = self.dependents.write().unwrap();
        for k in &to_remove {
            deps.remove(k);
        }

        InvalidationResult {
            direct,
            cascaded,
            total,
        }
    }

    /// Get the number of tracked dependency sources.
    pub fn dependency_count(&self) -> usize {
        self.dependents.read().unwrap().len()
    }

    /// Clear all dependency tracking.
    pub fn clear(&self) {
        self.dependents.write().unwrap().clear();
    }

    /// Get the set of direct dependents for a key.
    pub fn dependents_of(&self, key: &K) -> Vec<K> {
        let deps = self.dependents.read().unwrap();
        deps.get(key)
            .map(|s| s.iter().cloned().collect())
            .unwrap_or_default()
    }
}

impl<K> Default for CacheInvalidator<K>
where
    K: Eq + Hash + Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_dependencies() {
        let inv = CacheInvalidator::<String>::new();
        let result = inv.invalidate(&"key".to_string());
        assert_eq!(result.direct, 1);
        assert_eq!(result.cascaded, 0);
        assert_eq!(result.total, 1);
    }

    #[test]
    fn test_single_dependency() {
        let inv = CacheInvalidator::new();
        inv.add_dependency("parent".to_string(), "child".to_string());
        let cascade = inv.cascade(&"parent".to_string());
        assert_eq!(cascade.len(), 2);
    }

    #[test]
    fn test_cascade_chain() {
        let inv = CacheInvalidator::new();
        inv.add_dependency("a".to_string(), "b".to_string());
        inv.add_dependency("b".to_string(), "c".to_string());
        let cascade = inv.cascade(&"a".to_string());
        assert_eq!(cascade.len(), 3);
    }

    #[test]
    fn test_cascade_diamond() {
        let inv = CacheInvalidator::new();
        inv.add_dependency("a".to_string(), "b".to_string());
        inv.add_dependency("a".to_string(), "c".to_string());
        inv.add_dependency("b".to_string(), "d".to_string());
        inv.add_dependency("c".to_string(), "d".to_string());
        let cascade = inv.cascade(&"a".to_string());
        // a, b, c, d — no duplicates
        assert_eq!(cascade.len(), 4);
    }

    #[test]
    fn test_invalidate_cleans_up() {
        let inv = CacheInvalidator::new();
        inv.add_dependency("a".to_string(), "b".to_string());
        inv.invalidate(&"a".to_string());
        assert_eq!(inv.dependency_count(), 0);
    }

    #[test]
    fn test_remove_dependency() {
        let inv = CacheInvalidator::new();
        inv.add_dependency("a".to_string(), "b".to_string());
        inv.remove_dependency(&"a".to_string(), &"b".to_string());
        assert_eq!(inv.dependents_of(&"a".to_string()).len(), 0);
    }

    #[test]
    fn test_clear() {
        let inv = CacheInvalidator::new();
        inv.add_dependency("a".to_string(), "b".to_string());
        inv.add_dependency("c".to_string(), "d".to_string());
        inv.clear();
        assert_eq!(inv.dependency_count(), 0);
    }

    #[test]
    fn test_cycle_protection() {
        let inv = CacheInvalidator::new();
        inv.add_dependency("a".to_string(), "b".to_string());
        inv.add_dependency("b".to_string(), "a".to_string());
        // Should not infinite loop
        let cascade = inv.cascade(&"a".to_string());
        assert_eq!(cascade.len(), 2);
    }
}
