use agentic_aegis_core::shadow::*;
use agentic_aegis_core::types::Language;

// === ResourceUsage Tests ===

#[test]
fn test_resource_usage_default() {
    let usage = ResourceUsage::default();
    assert_eq!(usage.memory_bytes, 0);
    assert_eq!(usage.cpu_time_ms, 0);
    assert_eq!(usage.wall_time_ms, 0);
}

// === ResourceLimits Tests ===

#[test]
fn test_resource_limits_default() {
    let limits = ResourceLimits::default();
    assert!(limits.max_memory_bytes > 0);
    assert!(limits.max_cpu_time_ms > 0);
    assert!(limits.max_wall_time_ms > 0);
}

// === ExecutionResult Tests ===

#[test]
fn test_execution_result_compile_failed() {
    let result = ExecutionResult::compile_failed("error".to_string(), 100);
    assert!(!result.success);
    assert_eq!(result.stderr, "error");
    assert_eq!(result.exit_code, 1);
}

#[test]
fn test_execution_result_success() {
    let result = ExecutionResult::success("hello".to_string(), 50);
    assert!(result.success);
    assert_eq!(result.stdout, "hello");
    assert_eq!(result.exit_code, 0);
}

#[test]
fn test_execution_result_timeout() {
    let result = ExecutionResult::timeout(30000);
    assert!(!result.success);
    assert!(result.stderr.contains("timed out"));
    assert_eq!(result.exit_code, -1);
}

// === SideEffect Tests ===

#[test]
fn test_side_effect_file_write() {
    let effect = SideEffect::FileWrite {
        path: "/tmp/test".to_string(),
        bytes: 100,
    };
    assert!(!effect.is_dangerous());
    assert_eq!(effect.category(), "file_write");
}

#[test]
fn test_side_effect_file_delete_dangerous() {
    let effect = SideEffect::FileDelete {
        path: "/tmp/test".to_string(),
    };
    assert!(effect.is_dangerous());
    assert_eq!(effect.category(), "file_delete");
}

#[test]
fn test_side_effect_network_dangerous() {
    let effect = SideEffect::NetworkConnect {
        host: "example.com".to_string(),
        port: 443,
    };
    assert!(effect.is_dangerous());
    assert_eq!(effect.category(), "network");
}

#[test]
fn test_side_effect_process_dangerous() {
    let effect = SideEffect::ProcessSpawn {
        command: "rm -rf /".to_string(),
    };
    assert!(effect.is_dangerous());
    assert_eq!(effect.category(), "process");
}

#[test]
fn test_side_effect_env_not_dangerous() {
    let effect = SideEffect::EnvAccess {
        variable: "HOME".to_string(),
    };
    assert!(!effect.is_dangerous());
    assert_eq!(effect.category(), "env");
}

#[test]
fn test_side_effect_stdout() {
    let effect = SideEffect::StdoutWrite { bytes: 100 };
    assert!(!effect.is_dangerous());
    assert_eq!(effect.category(), "stdout");
}

#[test]
fn test_side_effect_stderr() {
    let effect = SideEffect::StderrWrite { bytes: 50 };
    assert!(!effect.is_dangerous());
    assert_eq!(effect.category(), "stderr");
}

// === EffectTracker Tests ===

#[test]
fn test_effect_tracker_detect_file_write_rust() {
    let tracker = EffectTracker::new();
    let code = "std::fs::write(\"file.txt\", data);";
    let effects = tracker.analyze(code, &Language::Rust);
    assert!(!effects.is_empty());
    assert!(effects.iter().any(|e| e.category() == "file_write"));
}

#[test]
fn test_effect_tracker_detect_file_read_rust() {
    let tracker = EffectTracker::new();
    let code = "let data = std::fs::read(\"file.txt\").unwrap();";
    let effects = tracker.analyze(code, &Language::Rust);
    assert!(!effects.is_empty());
}

#[test]
fn test_effect_tracker_detect_network() {
    let tracker = EffectTracker::new();
    let code = "let client = reqwest::Client::new();";
    let effects = tracker.analyze(code, &Language::Rust);
    assert!(effects.iter().any(|e| e.category() == "network"));
}

#[test]
fn test_effect_tracker_detect_process_spawn() {
    let tracker = EffectTracker::new();
    let code = "Command::new(\"ls\").arg(\"-la\").output();";
    let effects = tracker.analyze(code, &Language::Rust);
    assert!(effects.iter().any(|e| e.category() == "process"));
}

#[test]
fn test_effect_tracker_detect_env_access() {
    let tracker = EffectTracker::new();
    let code = "let home = env::var(\"HOME\").unwrap();";
    let effects = tracker.analyze(code, &Language::Rust);
    assert!(effects.iter().any(|e| e.category() == "env"));
}

#[test]
fn test_effect_tracker_detect_file_delete() {
    let tracker = EffectTracker::new();
    let code = "fs::remove_file(\"tmp.txt\");";
    let effects = tracker.analyze(code, &Language::Rust);
    assert!(effects.iter().any(|e| e.category() == "file_delete"));
}

#[test]
fn test_effect_tracker_has_dangerous_true() {
    let tracker = EffectTracker::new();
    let code = "Command::new(\"rm\").arg(\"-rf\").output();";
    assert!(tracker.has_dangerous_effects(code, &Language::Rust));
}

#[test]
fn test_effect_tracker_has_dangerous_false() {
    let tracker = EffectTracker::new();
    let code = "let x = 5;\nprintln!(\"{}\", x);";
    assert!(!tracker.has_dangerous_effects(code, &Language::Rust));
}

#[test]
fn test_effect_tracker_python_file_read() {
    let tracker = EffectTracker::new();
    let code = "f = open('file.txt', 'r')";
    let effects = tracker.analyze(code, &Language::Python);
    assert!(!effects.is_empty());
}

#[test]
fn test_effect_tracker_python_subprocess() {
    let tracker = EffectTracker::new();
    let code = "subprocess.run(['ls', '-la'])";
    let effects = tracker.analyze(code, &Language::Python);
    assert!(effects.iter().any(|e| e.category() == "process"));
}

#[test]
fn test_effect_tracker_js_fetch() {
    let tracker = EffectTracker::new();
    let code = "fetch('https://example.com')";
    let effects = tracker.analyze(code, &Language::JavaScript);
    assert!(effects.iter().any(|e| e.category() == "network"));
}

#[test]
fn test_effect_tracker_empty_code() {
    let tracker = EffectTracker::new();
    let effects = tracker.analyze("", &Language::Rust);
    assert!(effects.is_empty());
}

// === ResourceMonitor Tests ===

#[test]
fn test_resource_monitor_default() {
    let monitor = ResourceMonitor::default();
    assert!(monitor.is_within_limits());
}

#[test]
fn test_resource_monitor_check_memory_ok() {
    let monitor = ResourceMonitor::default();
    let check = monitor.check_memory();
    assert!(check.is_ok());
}

#[test]
fn test_resource_monitor_check_memory_exceeded() {
    let limits = ResourceLimits {
        max_memory_bytes: 100,
        ..Default::default()
    };
    let mut monitor = ResourceMonitor::new(limits);
    monitor.update(ResourceUsage {
        memory_bytes: 200,
        ..Default::default()
    });
    let check = monitor.check_memory();
    assert!(check.is_exceeded());
}

#[test]
fn test_resource_monitor_check_memory_warning() {
    let limits = ResourceLimits {
        max_memory_bytes: 100,
        ..Default::default()
    };
    let mut monitor = ResourceMonitor::new(limits);
    monitor.update(ResourceUsage {
        memory_bytes: 85,
        ..Default::default()
    });
    let check = monitor.check_memory();
    assert!(matches!(check, monitor::ResourceCheck::Warning { .. }));
}

#[test]
fn test_resource_monitor_check_all() {
    let monitor = ResourceMonitor::default();
    let checks = monitor.check_all();
    assert_eq!(checks.len(), 3);
    assert!(checks.iter().all(|c| c.is_ok()));
}

#[test]
fn test_resource_monitor_reset() {
    let mut monitor = ResourceMonitor::default();
    monitor.update(ResourceUsage {
        memory_bytes: 1000,
        ..Default::default()
    });
    monitor.reset();
    assert_eq!(monitor.current_usage().memory_bytes, 0);
}

#[test]
fn test_resource_monitor_exceeded_not_within_limits() {
    let limits = ResourceLimits {
        max_memory_bytes: 10,
        ..Default::default()
    };
    let mut monitor = ResourceMonitor::new(limits);
    monitor.update(ResourceUsage {
        memory_bytes: 100,
        ..Default::default()
    });
    assert!(!monitor.is_within_limits());
}

// === ShadowCompiler Tests ===

#[test]
fn test_shadow_compiler_new() {
    let compiler = ShadowCompiler::new();
    assert_eq!(std::mem::size_of_val(&compiler), 0); // Unit struct
}

// === SandboxExecutor Tests ===

#[test]
fn test_sandbox_executor_new() {
    let executor = SandboxExecutor::new();
    assert!(executor.limits().max_wall_time_ms > 0);
}

#[test]
fn test_sandbox_executor_with_limits() {
    let limits = ResourceLimits {
        max_wall_time_ms: 5000,
        ..Default::default()
    };
    let executor = SandboxExecutor::new().with_limits(limits);
    assert_eq!(executor.limits().max_wall_time_ms, 5000);
}
