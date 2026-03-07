use super::{ResourceLimits, ResourceUsage};

pub struct ResourceMonitor {
    limits: ResourceLimits,
    current: ResourceUsage,
}

impl ResourceMonitor {
    pub fn new(limits: ResourceLimits) -> Self {
        Self {
            limits,
            current: ResourceUsage::default(),
        }
    }

    pub fn limits(&self) -> &ResourceLimits {
        &self.limits
    }

    pub fn current_usage(&self) -> &ResourceUsage {
        &self.current
    }

    pub fn update(&mut self, usage: ResourceUsage) {
        self.current = usage;
    }

    pub fn check_memory(&self) -> ResourceCheck {
        if self.current.memory_bytes > self.limits.max_memory_bytes {
            ResourceCheck::Exceeded {
                resource: "memory".to_string(),
                current: self.current.memory_bytes,
                limit: self.limits.max_memory_bytes,
            }
        } else if self.current.memory_bytes as f64 > self.limits.max_memory_bytes as f64 * 0.8 {
            ResourceCheck::Warning {
                resource: "memory".to_string(),
                current: self.current.memory_bytes,
                limit: self.limits.max_memory_bytes,
                percent: (self.current.memory_bytes as f64 / self.limits.max_memory_bytes as f64)
                    * 100.0,
            }
        } else {
            ResourceCheck::Ok
        }
    }

    pub fn check_cpu_time(&self) -> ResourceCheck {
        if self.current.cpu_time_ms > self.limits.max_cpu_time_ms {
            ResourceCheck::Exceeded {
                resource: "cpu_time".to_string(),
                current: self.current.cpu_time_ms,
                limit: self.limits.max_cpu_time_ms,
            }
        } else {
            ResourceCheck::Ok
        }
    }

    pub fn check_disk(&self) -> ResourceCheck {
        let disk_total = self.current.disk_bytes_written + self.current.disk_bytes_read;
        if disk_total > self.limits.max_disk_bytes {
            ResourceCheck::Exceeded {
                resource: "disk".to_string(),
                current: disk_total,
                limit: self.limits.max_disk_bytes,
            }
        } else {
            ResourceCheck::Ok
        }
    }

    pub fn check_all(&self) -> Vec<ResourceCheck> {
        vec![
            self.check_memory(),
            self.check_cpu_time(),
            self.check_disk(),
        ]
    }

    pub fn is_within_limits(&self) -> bool {
        self.check_all()
            .iter()
            .all(|c| matches!(c, ResourceCheck::Ok | ResourceCheck::Warning { .. }))
    }

    pub fn reset(&mut self) {
        self.current = ResourceUsage::default();
    }
}

impl Default for ResourceMonitor {
    fn default() -> Self {
        Self::new(ResourceLimits::default())
    }
}

#[derive(Debug, Clone)]
pub enum ResourceCheck {
    Ok,
    Warning {
        resource: String,
        current: u64,
        limit: u64,
        percent: f64,
    },
    Exceeded {
        resource: String,
        current: u64,
        limit: u64,
    },
}

impl ResourceCheck {
    pub fn is_ok(&self) -> bool {
        matches!(self, ResourceCheck::Ok)
    }

    pub fn is_exceeded(&self) -> bool {
        matches!(self, ResourceCheck::Exceeded { .. })
    }
}
