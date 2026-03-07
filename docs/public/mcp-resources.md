---
status: stable
---

# MCP Resources

AgenticAegis exposes MCP resources for read access to validation state.

## Available Resources

- `aegis://sessions` — List all active validation sessions
- `aegis://sessions/{id}` — Get details of a specific session
- `aegis://config` — Current server configuration
- `aegis://stats` — Validation and security statistics

## Resource Format

All resources return JSON with a consistent envelope:

```json
{
  "uri": "aegis://sessions",
  "mimeType": "application/json",
  "content": { ... }
}
```

Resources are read-only. Use MCP tools to modify state.
