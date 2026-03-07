use std::time::Instant;
use tempfile::TempDir;

use crate::types::{AegisResult, Language};

pub struct ShadowCompiler;

impl ShadowCompiler {
    pub fn new() -> Self {
        Self
    }

    pub async fn compile(&self, code: &str, language: &Language) -> AegisResult<CompileResult> {
        let start = Instant::now();
        let temp_dir = TempDir::new().map_err(|e| crate::types::AegisError::Io(e.to_string()))?;

        let result = match language {
            Language::Rust => self.compile_rust(code, &temp_dir).await,
            Language::Python => self.check_python(code, &temp_dir).await,
            Language::JavaScript | Language::TypeScript => {
                self.check_javascript(code, &temp_dir).await
            }
            Language::Go => self.compile_go(code, &temp_dir).await,
            Language::C | Language::Cpp => self.compile_c(code, language, &temp_dir).await,
            _ => Ok(CompileResult {
                success: true,
                errors: Vec::new(),
                warnings: Vec::new(),
                binary_path: None,
                duration_ms: start.elapsed().as_millis() as u64,
            }),
        };

        result
    }

    async fn compile_rust(&self, code: &str, temp_dir: &TempDir) -> AegisResult<CompileResult> {
        let start = Instant::now();
        let src_path = temp_dir.path().join("main.rs");
        std::fs::write(&src_path, code)?;

        let output = tokio::process::Command::new("rustc")
            .arg("--edition=2021")
            .arg(&src_path)
            .arg("-o")
            .arg(temp_dir.path().join("main"))
            .output()
            .await
            .map_err(|e| crate::types::AegisError::ShadowExecution(e.to_string()))?;

        let duration_ms = start.elapsed().as_millis() as u64;
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if output.status.success() {
            let mut warnings = Vec::new();
            for line in stderr.lines() {
                if line.contains("warning") {
                    warnings.push(line.to_string());
                }
            }
            Ok(CompileResult {
                success: true,
                errors: Vec::new(),
                warnings,
                binary_path: Some(temp_dir.path().join("main").to_string_lossy().to_string()),
                duration_ms,
            })
        } else {
            let errors: Vec<String> = stderr
                .lines()
                .filter(|l| l.contains("error"))
                .map(|l| l.to_string())
                .collect();
            Ok(CompileResult {
                success: false,
                errors,
                warnings: Vec::new(),
                binary_path: None,
                duration_ms,
            })
        }
    }

    async fn check_python(&self, code: &str, temp_dir: &TempDir) -> AegisResult<CompileResult> {
        let start = Instant::now();
        let src_path = temp_dir.path().join("main.py");
        std::fs::write(&src_path, code)?;

        let output = tokio::process::Command::new("python3")
            .arg("-m")
            .arg("py_compile")
            .arg(&src_path)
            .output()
            .await
            .map_err(|e| crate::types::AegisError::ShadowExecution(e.to_string()))?;

        let duration_ms = start.elapsed().as_millis() as u64;

        if output.status.success() {
            Ok(CompileResult {
                success: true,
                errors: Vec::new(),
                warnings: Vec::new(),
                binary_path: None,
                duration_ms,
            })
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            Ok(CompileResult {
                success: false,
                errors: vec![stderr],
                warnings: Vec::new(),
                binary_path: None,
                duration_ms,
            })
        }
    }

    async fn check_javascript(&self, code: &str, temp_dir: &TempDir) -> AegisResult<CompileResult> {
        let start = Instant::now();
        let src_path = temp_dir.path().join("main.js");
        std::fs::write(&src_path, code)?;

        let output = tokio::process::Command::new("node")
            .arg("--check")
            .arg(&src_path)
            .output()
            .await
            .map_err(|e| crate::types::AegisError::ShadowExecution(e.to_string()))?;

        let duration_ms = start.elapsed().as_millis() as u64;

        if output.status.success() {
            Ok(CompileResult {
                success: true,
                errors: Vec::new(),
                warnings: Vec::new(),
                binary_path: None,
                duration_ms,
            })
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            Ok(CompileResult {
                success: false,
                errors: vec![stderr],
                warnings: Vec::new(),
                binary_path: None,
                duration_ms,
            })
        }
    }

    async fn compile_go(&self, code: &str, temp_dir: &TempDir) -> AegisResult<CompileResult> {
        let start = Instant::now();
        let src_path = temp_dir.path().join("main.go");
        std::fs::write(&src_path, code)?;

        let output = tokio::process::Command::new("go")
            .arg("build")
            .arg("-o")
            .arg(temp_dir.path().join("main"))
            .arg(&src_path)
            .output()
            .await
            .map_err(|e| crate::types::AegisError::ShadowExecution(e.to_string()))?;

        let duration_ms = start.elapsed().as_millis() as u64;

        if output.status.success() {
            Ok(CompileResult {
                success: true,
                errors: Vec::new(),
                warnings: Vec::new(),
                binary_path: Some(temp_dir.path().join("main").to_string_lossy().to_string()),
                duration_ms,
            })
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            Ok(CompileResult {
                success: false,
                errors: vec![stderr],
                warnings: Vec::new(),
                binary_path: None,
                duration_ms,
            })
        }
    }

    async fn compile_c(
        &self,
        code: &str,
        language: &Language,
        temp_dir: &TempDir,
    ) -> AegisResult<CompileResult> {
        let start = Instant::now();
        let ext = if matches!(language, Language::Cpp) {
            "cpp"
        } else {
            "c"
        };
        let compiler = if matches!(language, Language::Cpp) {
            "c++"
        } else {
            "cc"
        };

        let src_path = temp_dir.path().join(format!("main.{}", ext));
        std::fs::write(&src_path, code)?;

        let output = tokio::process::Command::new(compiler)
            .arg(&src_path)
            .arg("-o")
            .arg(temp_dir.path().join("main"))
            .output()
            .await
            .map_err(|e| crate::types::AegisError::ShadowExecution(e.to_string()))?;

        let duration_ms = start.elapsed().as_millis() as u64;

        if output.status.success() {
            Ok(CompileResult {
                success: true,
                errors: Vec::new(),
                warnings: Vec::new(),
                binary_path: Some(temp_dir.path().join("main").to_string_lossy().to_string()),
                duration_ms,
            })
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            Ok(CompileResult {
                success: false,
                errors: vec![stderr],
                warnings: Vec::new(),
                binary_path: None,
                duration_ms,
            })
        }
    }
}

impl Default for ShadowCompiler {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct CompileResult {
    pub success: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub binary_path: Option<String>,
    pub duration_ms: u64,
}
