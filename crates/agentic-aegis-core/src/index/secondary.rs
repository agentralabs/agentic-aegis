use std::collections::HashMap;

/// Secondary index: maps attribute values to lists of entity IDs.
/// Used for queries like "all sessions with language=rust".
pub struct SecondaryIndex {
    entries: HashMap<String, HashMap<String, Vec<String>>>,
}

impl SecondaryIndex {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// Add an entry: attribute=value → entity_id
    pub fn add(&mut self, attribute: &str, value: &str, entity_id: String) {
        self.entries
            .entry(attribute.to_string())
            .or_default()
            .entry(value.to_string())
            .or_default()
            .push(entity_id);
    }

    /// Query: get all entity IDs with attribute=value
    pub fn query(&self, attribute: &str, value: &str) -> Vec<String> {
        self.entries
            .get(attribute)
            .and_then(|m| m.get(value))
            .cloned()
            .unwrap_or_default()
    }

    /// Remove an entity ID from a specific attribute=value entry
    pub fn remove(&mut self, attribute: &str, value: &str, entity_id: &str) {
        if let Some(attr_map) = self.entries.get_mut(attribute) {
            if let Some(ids) = attr_map.get_mut(value) {
                ids.retain(|id| id != entity_id);
                if ids.is_empty() {
                    attr_map.remove(value);
                }
            }
            if attr_map.is_empty() {
                self.entries.remove(attribute);
            }
        }
    }

    /// Remove an entity ID from all index entries
    pub fn remove_entity(&mut self, entity_id: &str) {
        for attr_map in self.entries.values_mut() {
            for ids in attr_map.values_mut() {
                ids.retain(|id| id != entity_id);
            }
        }
        // Clean up empty entries
        for attr_map in self.entries.values_mut() {
            attr_map.retain(|_, ids| !ids.is_empty());
        }
        self.entries.retain(|_, m| !m.is_empty());
    }

    /// Get all attribute names in the index
    pub fn attributes(&self) -> Vec<String> {
        self.entries.keys().cloned().collect()
    }

    /// Get all values for a given attribute
    pub fn values_for(&self, attribute: &str) -> Vec<String> {
        self.entries
            .get(attribute)
            .map(|m| m.keys().cloned().collect())
            .unwrap_or_default()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Default for SecondaryIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_query() {
        let mut idx = SecondaryIndex::new();
        idx.add("language", "rust", "session1".into());
        idx.add("language", "rust", "session2".into());
        idx.add("language", "python", "session3".into());

        let rust_sessions = idx.query("language", "rust");
        assert_eq!(rust_sessions.len(), 2);
        let python_sessions = idx.query("language", "python");
        assert_eq!(python_sessions.len(), 1);
    }

    #[test]
    fn test_query_empty() {
        let idx = SecondaryIndex::new();
        assert!(idx.query("language", "rust").is_empty());
    }

    #[test]
    fn test_remove_specific() {
        let mut idx = SecondaryIndex::new();
        idx.add("lang", "rust", "s1".into());
        idx.add("lang", "rust", "s2".into());
        idx.remove("lang", "rust", "s1");
        let result = idx.query("lang", "rust");
        assert_eq!(result, vec!["s2"]);
    }

    #[test]
    fn test_remove_entity() {
        let mut idx = SecondaryIndex::new();
        idx.add("lang", "rust", "s1".into());
        idx.add("state", "active", "s1".into());
        idx.remove_entity("s1");
        assert!(idx.query("lang", "rust").is_empty());
        assert!(idx.query("state", "active").is_empty());
    }

    #[test]
    fn test_attributes() {
        let mut idx = SecondaryIndex::new();
        idx.add("lang", "rust", "s1".into());
        idx.add("state", "active", "s1".into());
        let mut attrs = idx.attributes();
        attrs.sort();
        assert_eq!(attrs, vec!["lang", "state"]);
    }

    #[test]
    fn test_values_for() {
        let mut idx = SecondaryIndex::new();
        idx.add("lang", "rust", "s1".into());
        idx.add("lang", "python", "s2".into());
        let mut vals = idx.values_for("lang");
        vals.sort();
        assert_eq!(vals, vec!["python", "rust"]);
    }
}
