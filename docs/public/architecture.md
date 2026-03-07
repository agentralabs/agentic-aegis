---
status: stable
---

# Architecture

AgenticAegis follows the canonical Agentra 4-crate workspace pattern.

## Crate Layout

- **agentic-aegis-core** — Domain types, 20 inventions, validation engine, session management
- **agentic-aegis-mcp** — JSON-RPC MCP server with 12 tools
- **agentic-aegis-cli** — 30+ CLI commands under the `aegis` binary
- **agentic-aegis-ffi** — C-compatible FFI for Python and other languages

## Data Flow

Tokens flow in through the streaming validator, pass through syntax accumulation and type tracking, then enter shadow execution if enabled. Security scanners run in parallel. Results are collected in the session and returned to the caller.

## Bridge System

Trait-based bridges connect to sister projects. Each bridge has a NoOp default implementation, ensuring standalone operation without any sibling dependencies.
