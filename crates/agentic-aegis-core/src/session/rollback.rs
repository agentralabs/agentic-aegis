use crate::types::{AegisError, AegisResult, RollbackId, SessionSnapshot};

pub struct RollbackEngine {
    snapshots: Vec<IndexedSnapshot>,
}

struct IndexedSnapshot {
    id: RollbackId,
    snapshot: SessionSnapshot,
}

impl RollbackEngine {
    pub fn new() -> Self {
        Self {
            snapshots: Vec::new(),
        }
    }

    pub fn save_snapshot(&mut self, snapshot: SessionSnapshot) -> RollbackId {
        let id = RollbackId::new();
        self.snapshots.push(IndexedSnapshot {
            id: id.clone(),
            snapshot,
        });
        id
    }

    pub fn rollback_to(&self, rollback_id: &str) -> AegisResult<&SessionSnapshot> {
        self.snapshots
            .iter()
            .find(|s| s.id.to_string() == rollback_id)
            .map(|s| &s.snapshot)
            .ok_or_else(|| AegisError::Rollback(format!("snapshot not found: {}", rollback_id)))
    }

    pub fn rollback_to_latest(&self) -> AegisResult<&SessionSnapshot> {
        self.snapshots
            .last()
            .map(|s| &s.snapshot)
            .ok_or_else(|| AegisError::Rollback("no snapshots available".to_string()))
    }

    pub fn rollback_to_chunk(&self, chunk_index: usize) -> AegisResult<&SessionSnapshot> {
        self.snapshots
            .iter()
            .rev()
            .find(|s| s.snapshot.chunk_index <= chunk_index)
            .map(|s| &s.snapshot)
            .ok_or_else(|| {
                AegisError::Rollback(format!(
                    "no snapshot found for chunk index {}",
                    chunk_index
                ))
            })
    }

    pub fn snapshot_count(&self) -> usize {
        self.snapshots.len()
    }

    pub fn list_snapshots(&self) -> Vec<(String, usize)> {
        self.snapshots
            .iter()
            .map(|s| (s.id.to_string(), s.snapshot.chunk_index))
            .collect()
    }

    pub fn clear(&mut self) {
        self.snapshots.clear();
    }

    pub fn prune(&mut self, keep_last: usize) {
        if self.snapshots.len() > keep_last {
            let drain_count = self.snapshots.len() - keep_last;
            self.snapshots.drain(..drain_count);
        }
    }
}

impl Default for RollbackEngine {
    fn default() -> Self {
        Self::new()
    }
}
