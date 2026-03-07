---
status: stable
---

# Troubleshooting

Common issues and solutions for AgenticAegis.

## MCP Server Not Responding

Ensure the server is running: `pgrep agentic-aegis-mcp`. Check stderr logs for startup errors. Verify the port is not in use.

## High Validation Latency

Reduce `streaming_buffer_size` in configuration. Ensure no other process is saturating CPU. Check session count with `aegis session status`.

## Shadow Execution Timeout

Increase `AEGIS_SANDBOX_TIMEOUT_MS`. Complex code may need longer execution windows. Check for infinite loops in generated code.

## Security False Positives

Lower sensitivity with `AEGIS_SECURITY_SENSITIVITY=low`. Fine-tune individual scanners in `aegis.toml`.

## Session Limit Reached

Increase `AEGIS_MAX_SESSIONS` or end idle sessions with `aegis session end`.
