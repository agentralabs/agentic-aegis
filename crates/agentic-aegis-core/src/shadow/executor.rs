use std::time::Instant;
use tempfile::TempDir;

use crate::types::{AegisResult, Language};

use super::{ExecutionResult, ResourceLimits, ResourceUsage};

pub struct SandboxExecutor {
    limits: ResourceLimits,
}

impl SandboxExecutor {
    pub fn new() -> Self {
        Self {
            limits: ResourceLimits::default(),
        }
    }

    pub fn with_limits(mut self, limits: ResourceLimits) -> Self {
        self.limits = limits;
        self
    }

    pub fn limits(&self) -> &ResourceLimits {
        &self.limits
    }

    pub async fn execute(&self, code: &str, language: &Language) -> AegisResult<ExecutionResult> {
        let temp_dir = TempDir::new().map_err(|e| crate::types::AegisError::Io(e.to_string()))?;
        let start = Instant::now();

        let result = match language {
            Language::Python => self.execute_python(code, &temp_dir).await,
            Language::JavaScript => self.execute_javascript(code, &temp_dir).await,
            Language::Rust => self.execute_rust(code, &temp_dir).await,
            _ => Ok(ExecutionResult {
                success: false,
                stdout: String::new(),
                stderr: format!("unsupported language for execution: {}", language.as_str()),
                exit_code: 1,
                duration_ms: 0,
                resource_usage: ResourceUsage::default(),
                effects: Vec::new(),
            }),
        };

        let duration = start.elapsed().as_millis() as u64;

        match result {
            Ok(mut r) => {
                r.duration_ms = duration;
                // Truncate output if needed
                if r.stdout.len() as u64 > self.limits.max_output_bytes {
                    r.stdout.truncate(self.limits.max_output_bytes as usize);
                    r.stdout.push_str("\n... (output truncated)");
                }
                if r.stderr.len() as u64 > self.limits.max_output_bytes {
                    r.stderr.truncate(self.limits.max_output_bytes as usize);
                    r.stderr.push_str("\n... (output truncated)");
                }
                Ok(r)
            }
            Err(e) => Ok(ExecutionResult {
                success: false,
                stdout: String::new(),
                stderr: e.to_string(),
                exit_code: -1,
                duration_ms: duration,
                resource_usage: ResourceUsage::default(),
                effects: Vec::new(),
            }),
        }
    }

    async fn execute_python(&self, code: &str, temp_dir: &TempDir) -> AegisResult<ExecutionResult> {
        let src_path = temp_dir.path().join("main.py");
        std::fs::write(&src_path, code)?;

        let timeout = std::time::Duration::from_millis(self.limits.max_wall_time_ms);

        let output = tokio::time::timeout(
            timeout,
            tokio::process::Command::new("python3")
                .arg(&src_path)
                .env("PYTHONDONTWRITEBYTECODE", "1")
                .output(),
        )
        .await
        .map_err(|_| crate::types::AegisError::Timeout("python execution timed out".to_string()))?
        .map_err(|e| crate::types::AegisError::ShadowExecution(e.to_string()))?;

        let exit_code = output.status.code().unwrap_or(-1);
        Ok(ExecutionResult {
            success: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code,
            duration_ms: 0,
            resource_usage: ResourceUsage::default(),
            effects: Vec::new(),
        })
    }

    async fn execute_javascript(
        &self,
        code: &str,
        temp_dir: &TempDir,
    ) -> AegisResult<ExecutionResult> {
        let src_path = temp_dir.path().join("main.js");
        std::fs::write(&src_path, code)?;

        let timeout = std::time::Duration::from_millis(self.limits.max_wall_time_ms);

        let output = tokio::time::timeout(
            timeout,
            tokio::process::Command::new("node").arg(&src_path).output(),
        )
        .await
        .map_err(|_| crate::types::AegisError::Timeout("node execution timed out".to_string()))?
        .map_err(|e| crate::types::AegisError::ShadowExecution(e.to_string()))?;

        let exit_code = output.status.code().unwrap_or(-1);
        Ok(ExecutionResult {
            success: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code,
            duration_ms: 0,
            resource_usage: ResourceUsage::default(),
            effects: Vec::new(),
        })
    }

    async fn execute_rust(&self, code: &str, temp_dir: &TempDir) -> AegisResult<ExecutionResult> {
        let src_path = temp_dir.path().join("main.rs");
        std::fs::write(&src_path, code)?;

        // First compile
        let compile_output = tokio::process::Command::new("rustc")
            .arg("--edition=2021")
            .arg(&src_path)
            .arg("-o")
            .arg(temp_dir.path().join("main"))
            .output()
            .await
            .map_err(|e| crate::types::AegisError::ShadowExecution(e.to_string()))?;

        if !compile_output.status.success() {
            return Ok(ExecutionResult::compile_failed(
                String::from_utf8_lossy(&compile_output.stderr).to_string(),
                0,
            ));
        }

        // Then execute
        let timeout = std::time::Duration::from_millis(self.limits.max_wall_time_ms);
        let binary = temp_dir.path().join("main");

        let output = tokio::time::timeout(timeout, tokio::process::Command::new(&binary).output())
            .await
            .map_err(|_| crate::types::AegisError::Timeout("rust execution timed out".to_string()))?
            .map_err(|e| crate::types::AegisError::ShadowExecution(e.to_string()))?;

        let exit_code = output.status.code().unwrap_or(-1);
        Ok(ExecutionResult {
            success: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code,
            duration_ms: 0,
            resource_usage: ResourceUsage::default(),
            effects: Vec::new(),
        })
    }
}

impl Default for SandboxExecutor {
    fn default() -> Self {
        Self::new()
    }
}
