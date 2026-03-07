---
status: stable
---

# Command Surface

AgenticAegis exposes 30+ CLI commands and 12 MCP tools.

## CLI Commands

The `aegis` binary organizes commands into subcommand groups: `validate`, `shadow`, `scan`, `session`, `rollback`, `config`, and `completions`.

## MCP Tools

All 12 MCP tools follow verb-first imperative naming. Each tool accepts JSON input and returns structured JSON output with optional `isError` signaling.

## Coverage

Every invention in AgenticAegis is reachable through at least one CLI command and one MCP tool. The command surface is verified by `scripts/check-command-surface.sh`.

## Parity

The command surface maintains parity with the canonical sister pattern: CLI for humans, MCP for agents, FFI for polyglot integration.
