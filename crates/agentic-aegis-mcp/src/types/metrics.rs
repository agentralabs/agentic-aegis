use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpResponseMetrics {
    pub layer: String,
    pub tokens_used: u64,
    pub tokens_saved: u64,
    pub cache_hit: bool,
}
