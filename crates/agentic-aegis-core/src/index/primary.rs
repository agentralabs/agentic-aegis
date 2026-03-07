use std::collections::HashMap;

/// Primary index: maps entity IDs to stored values.
/// O(1) lookup by ID — the cheapest possible retrieval.
pub struct PrimaryIndex<T> {
    entries: HashMap<String, T>,
}

impl<T: Clone> PrimaryIndex<T> {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn get(&self, id: &str) -> Option<&T> {
        self.entries.get(id)
    }

    pub fn insert(&mut self, id: String, value: T) {
        self.entries.insert(id, value);
    }

    pub fn remove(&mut self, id: &str) -> Option<T> {
        self.entries.remove(id)
    }

    pub fn contains(&self, id: &str) -> bool {
        self.entries.contains_key(id)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn ids(&self) -> Vec<String> {
        self.entries.keys().cloned().collect()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

impl<T: Clone> Default for PrimaryIndex<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_get() {
        let mut idx = PrimaryIndex::new();
        idx.insert("id1".into(), 42);
        assert_eq!(idx.get("id1"), Some(&42));
    }

    #[test]
    fn test_remove() {
        let mut idx = PrimaryIndex::new();
        idx.insert("id1".into(), 42);
        assert_eq!(idx.remove("id1"), Some(42));
        assert!(idx.get("id1").is_none());
    }

    #[test]
    fn test_contains() {
        let mut idx = PrimaryIndex::new();
        idx.insert("id1".into(), 1);
        assert!(idx.contains("id1"));
        assert!(!idx.contains("id2"));
    }

    #[test]
    fn test_len() {
        let mut idx = PrimaryIndex::new();
        assert_eq!(idx.len(), 0);
        idx.insert("a".into(), 1);
        idx.insert("b".into(), 2);
        assert_eq!(idx.len(), 2);
    }

    #[test]
    fn test_ids() {
        let mut idx = PrimaryIndex::new();
        idx.insert("a".into(), 1);
        idx.insert("b".into(), 2);
        let mut ids = idx.ids();
        ids.sort();
        assert_eq!(ids, vec!["a", "b"]);
    }
}
