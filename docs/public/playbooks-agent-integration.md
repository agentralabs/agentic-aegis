---
status: stable
---

# Agent Integration Playbooks

Step-by-step guides for integrating AgenticAegis with AI agent frameworks.

## Playbook 1: MCP Client Integration

1. Start the MCP server: `agentic-aegis-mcp`
2. Connect your MCP client to stdin/stdout
3. Call `aegis_session_create` to start a session
4. Stream tokens through `aegis_validate_streaming`
5. End with `aegis_session_end` to get a summary

## Playbook 2: CLI Pipeline

1. Pipe LLM output through `aegis validate --streaming`
2. Check exit code for pass/fail
3. Use `aegis scan security` for additional checks

## Playbook 3: Python SDK

1. Install: `pip install agentic-aegis`
2. Import `AegisValidator` and create an instance
3. Call `validate_streaming()` with your token iterator
4. Handle `ValidationResult` for errors and warnings
