---
status: stable
---

# MCP Prompts

AgenticAegis provides MCP prompts for guided interaction with AI agents.

## Available Prompts

- **validate-code** — Guide an agent through streaming validation of generated code
- **security-review** — Walk through a comprehensive security scan
- **session-workflow** — Create, monitor, and close a validation session

## Usage

MCP clients can list prompts via `prompts/list` and invoke them via `prompts/get`. Each prompt returns a structured message sequence that agents can follow.

## Custom Prompts

Extend the prompt set by adding entries to the `prompts` section of `aegis.toml`.
