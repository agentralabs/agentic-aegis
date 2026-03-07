# Changelog

All notable changes to AgenticAegis will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-03-06

### Added

- Initial release of AgenticAegis (Sister #11 - The Shield)
- 20 inventions across 5 tiers (Streaming Validation, Shadow Execution, Input Protection, Output Protection, Validation Orchestration)
- 12 MCP tools for streaming validation, shadow execution, session management, and security scanning
- 30+ CLI commands via `aegis` binary
- 4 streaming validators: token, syntax, type, semantic
- Shadow execution engine with sandbox isolation and resource monitoring
- Input protection: prompt injection detection, intent verification, payload scanning, rate limiting
- Output protection: content filtering, PII detection, code safety analysis, output sanitization
- Session management with rollback engine and correction hint generation
- 11 bridge traits for sister integration (NoOp defaults for standalone operation)
- C FFI bindings
- 308 tests passing across 9 test phases
- Zero clippy warnings
- Full MCP quality compliance (zero unwraps, -32803 for unknown tools)
