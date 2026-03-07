---
status: stable
---

# API Reference

AgenticAegis exposes its functionality through Rust library APIs, MCP tools, CLI commands, and FFI bindings.

## Core Library

The `agentic_aegis_core` crate provides all types, validators, and engines:

- `TokenStreamValidator` — validates tokens as they arrive
- `ShadowCompiler` — compiles code in isolation
- `SandboxExecutor` — executes code in a restricted environment
- `PromptInjectionDetector` — detects injection attempts
- `ValidationSession` — manages lifecycle of a validation run
- `RollbackEngine` — reverts to a previous safe state

## Error Types

All operations return `AegisResult<T>` which wraps domain-specific error variants for validation failures, security alerts, and session errors.

## Configuration

Use `AegisConfig` to customize validation thresholds, security sensitivity, and resource limits.
