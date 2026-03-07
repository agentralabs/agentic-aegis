# AgenticAegis — Claude Code Instructions

Sister #11 — Streaming Validation (The Shield)

## Quick Reference

- **Binary:** `aegis` (CLI), `agentic-aegis-mcp` (MCP server)
- **File Extension:** `.aegis`
- **Version:** 0.1.0
- **Storage:** `~/.agentic-aegis/`

## Workspace

4 crates:
- `agentic-aegis-core` — Types, validators, shadow execution, protection, session, bridges
- `agentic-aegis-mcp` — 12 MCP tools, JSON-RPC protocol handler
- `agentic-aegis-cli` — 30+ CLI commands
- `agentic-aegis-ffi` — C FFI bindings

## MCP Quality Standard

- Tool descriptions: verb-first imperative, no trailing periods
- Error handling: tool execution errors → `isError: true`; protocol errors → JSON-RPC error
- Unknown tool: error code `-32803` (TOOL_NOT_FOUND)
- Zero `.unwrap()` calls in MCP crate

## Commit Style

- Conventional prefixes: `feat:`, `fix:`, `chore:`, `docs:`
- Never add "Co-Authored-By: Claude"

## Testing

```bash
cargo test --workspace           # 250+ tests
cargo clippy --workspace -- -D warnings  # 0 warnings
```

## 20 Inventions (5 Tiers)

1. Token Stream Validator
2. Syntax Accumulator
3. Type Flow Tracker
4. Error Predictor
5. Shadow Compiler
6. Sandbox Executor
7. Effect Tracker
8. Resource Monitor
9. Prompt Injection Detector
10. Intent Verifier
11. Payload Scanner
12. Rate Limiter
13. Content Filter
14. PII Detector
15. Code Safety Analyzer
16. Output Sanitizer
17. Validation Session Manager
18. Correction Hint Generator
19. Confidence Scorer
20. Rollback Engine

## 12 MCP Tools

1. aegis_validate_streaming
2. aegis_validate_complete
3. aegis_shadow_execute
4. aegis_check_input
5. aegis_check_output
6. aegis_session_create
7. aegis_session_status
8. aegis_session_end
9. aegis_correction_hint
10. aegis_confidence_score
11. aegis_rollback
12. aegis_scan_security
