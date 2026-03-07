---
status: stable
---

# Quickstart

Get started with AgenticAegis in under 5 minutes.

## Prerequisites

- Rust 1.75+ or Node.js 18+
- An AI agent or LLM pipeline that generates code

## Steps

1. Install AgenticAegis: `npm install @agenticamem/aegis`
2. Start the MCP server: `agentic-aegis-mcp`
3. Connect your agent to the MCP endpoint
4. Begin streaming code through the validator

## First Validation

```bash
echo '{"code": "print(1)"}' | aegis validate --streaming
```

AgenticAegis will validate the token stream in real time and report any issues.
