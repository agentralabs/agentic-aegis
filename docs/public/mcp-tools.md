---
status: stable
---

# MCP Tools

AgenticAegis exposes 12 MCP tools via JSON-RPC.

## Validation

1. **aegis_validate_streaming** — Validate a token stream in real time
2. **aegis_validate_complete** — Validate a complete code block

## Execution

3. **aegis_shadow_execute** — Execute code in an isolated sandbox

## Input/Output

4. **aegis_check_input** — Scan input for prompt injection and threats
5. **aegis_check_output** — Scan output for PII and unsafe patterns

## Sessions

6. **aegis_session_create** — Create a new validation session
7. **aegis_session_status** — Query session state and metrics
8. **aegis_session_end** — End a session and return summary

## Analysis

9. **aegis_correction_hint** — Generate correction hints for failed validation
10. **aegis_confidence_score** — Compute confidence score for generated code
11. **aegis_rollback** — Rollback to a previous safe checkpoint
12. **aegis_scan_security** — Run all security scanners on input
