# AgenticAegis Guide

## Getting Started

1. Install AgenticAegis (see INSTALL.md)
2. Start a validation session: `aegis session create`
3. Stream tokens through the validator: `aegis validate --streaming`
4. Review results: `aegis session status`

## MCP Server

Start the MCP server for agent integration:

```bash
agentic-aegis-mcp
```

Connect any MCP-compatible client to stdin/stdout.

## Security Scanning

Run a comprehensive security scan:

```bash
aegis scan security <file>
```

This checks for prompt injection, PII leakage, unsafe patterns, and payload attacks.

## Shadow Execution

Test generated code safely:

```bash
aegis shadow execute <file>
```

Code runs in an isolated sandbox with resource limits.

## Configuration

Set environment variables or create `~/.agentic-aegis/aegis.toml`. See `docs/public/configuration.md` for details.
