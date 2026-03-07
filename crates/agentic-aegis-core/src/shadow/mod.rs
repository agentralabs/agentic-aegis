pub mod compiler;
pub mod executor;
pub mod monitor;
pub mod tracker;

pub use compiler::ShadowCompiler;
pub use executor::SandboxExecutor;
pub use monitor::ResourceMonitor;
pub use tracker::EffectTracker;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResourceUsage {
    pub memory_bytes: u64,
    pub cpu_time_ms: u64,
    pub wall_time_ms: u64,
    pub disk_bytes_written: u64,
    pub disk_bytes_read: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_memory_bytes: u64,
    pub max_cpu_time_ms: u64,
    pub max_wall_time_ms: u64,
    pub max_disk_bytes: u64,
    pub max_output_bytes: u64,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_bytes: 256 * 1024 * 1024, // 256 MB
            max_cpu_time_ms: 10_000,             // 10 seconds
            max_wall_time_ms: 30_000,            // 30 seconds
            max_disk_bytes: 64 * 1024 * 1024,    // 64 MB
            max_output_bytes: 1024 * 1024,       // 1 MB
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub duration_ms: u64,
    pub resource_usage: ResourceUsage,
    pub effects: Vec<SideEffect>,
}

impl ExecutionResult {
    pub fn compile_failed(stderr: String, duration_ms: u64) -> Self {
        Self {
            success: false,
            stdout: String::new(),
            stderr,
            exit_code: 1,
            duration_ms,
            resource_usage: ResourceUsage::default(),
            effects: Vec::new(),
        }
    }

    pub fn success(stdout: String, duration_ms: u64) -> Self {
        Self {
            success: true,
            stdout,
            stderr: String::new(),
            exit_code: 0,
            duration_ms,
            resource_usage: ResourceUsage::default(),
            effects: Vec::new(),
        }
    }

    pub fn timeout(duration_ms: u64) -> Self {
        Self {
            success: false,
            stdout: String::new(),
            stderr: "execution timed out".to_string(),
            exit_code: -1,
            duration_ms,
            resource_usage: ResourceUsage::default(),
            effects: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SideEffect {
    FileWrite { path: String, bytes: u64 },
    FileRead { path: String },
    FileDelete { path: String },
    NetworkConnect { host: String, port: u16 },
    ProcessSpawn { command: String },
    EnvAccess { variable: String },
    StdoutWrite { bytes: u64 },
    StderrWrite { bytes: u64 },
}

impl SideEffect {
    pub fn is_dangerous(&self) -> bool {
        matches!(
            self,
            SideEffect::FileDelete { .. }
                | SideEffect::NetworkConnect { .. }
                | SideEffect::ProcessSpawn { .. }
        )
    }

    pub fn category(&self) -> &'static str {
        match self {
            SideEffect::FileWrite { .. } => "file_write",
            SideEffect::FileRead { .. } => "file_read",
            SideEffect::FileDelete { .. } => "file_delete",
            SideEffect::NetworkConnect { .. } => "network",
            SideEffect::ProcessSpawn { .. } => "process",
            SideEffect::EnvAccess { .. } => "env",
            SideEffect::StdoutWrite { .. } => "stdout",
            SideEffect::StderrWrite { .. } => "stderr",
        }
    }
}
