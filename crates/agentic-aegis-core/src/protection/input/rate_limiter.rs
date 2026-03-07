use std::collections::HashMap;
use std::time::Instant;

pub struct RateLimiter {
    window_ms: u64,
    max_requests: usize,
    clients: HashMap<String, Vec<Instant>>,
}

impl RateLimiter {
    pub fn new(window_ms: u64, max_requests: usize) -> Self {
        Self {
            window_ms,
            max_requests,
            clients: HashMap::new(),
        }
    }

    pub fn check(&mut self, client_id: &str) -> RateLimitResult {
        let now = Instant::now();
        let window = std::time::Duration::from_millis(self.window_ms);

        let timestamps = self.clients.entry(client_id.to_string()).or_default();

        // Remove expired entries
        timestamps.retain(|t| now.duration_since(*t) < window);

        if timestamps.len() >= self.max_requests {
            let oldest = timestamps.first().copied();
            let retry_after_ms = oldest
                .map(|t| {
                    let elapsed = now.duration_since(t);
                    if elapsed < window {
                        (window - elapsed).as_millis() as u64
                    } else {
                        0
                    }
                })
                .unwrap_or(self.window_ms);

            RateLimitResult::Limited {
                retry_after_ms,
                remaining: 0,
            }
        } else {
            timestamps.push(now);
            let remaining = self.max_requests - timestamps.len();
            RateLimitResult::Allowed { remaining }
        }
    }

    pub fn reset(&mut self, client_id: &str) {
        self.clients.remove(client_id);
    }

    pub fn reset_all(&mut self) {
        self.clients.clear();
    }

    pub fn window_ms(&self) -> u64 {
        self.window_ms
    }

    pub fn max_requests(&self) -> usize {
        self.max_requests
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new(60_000, 100) // 100 requests per minute
    }
}

#[derive(Debug, Clone)]
pub enum RateLimitResult {
    Allowed { remaining: usize },
    Limited { retry_after_ms: u64, remaining: usize },
}

impl RateLimitResult {
    pub fn is_allowed(&self) -> bool {
        matches!(self, RateLimitResult::Allowed { .. })
    }

    pub fn remaining(&self) -> usize {
        match self {
            RateLimitResult::Allowed { remaining } => *remaining,
            RateLimitResult::Limited { remaining, .. } => *remaining,
        }
    }
}
