---
status: stable
---

# Runtime Install Sync

AgenticAegis keeps its runtime state synchronized with the installed version.

## How It Works

On startup, the CLI and MCP server check the stored version in `~/.agentic-aegis/version.json` against the binary version. If a mismatch is detected, automatic migration runs to update stored data formats.

## Migration Safety

Migrations are idempotent and non-destructive. A backup of the previous state is created before any migration step.

## Manual Sync

Force a sync check with:

```bash
aegis config sync
```

## Version Pinning

Set `AEGIS_SKIP_MIGRATION=1` to prevent automatic migration during development.
