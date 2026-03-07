---
status: stable
---

# Configuration

AgenticAegis is configured through environment variables and a TOML configuration file.

## Environment Variables

- `AEGIS_LOG_LEVEL` — Log level: trace, debug, info, warn, error (default: info)
- `AEGIS_STORAGE_DIR` — Storage directory (default: `~/.agentic-aegis/`)
- `AEGIS_MAX_SESSIONS` — Maximum concurrent sessions (default: 100)
- `AEGIS_SANDBOX_TIMEOUT_MS` — Shadow execution timeout in ms (default: 5000)
- `AEGIS_SECURITY_SENSITIVITY` — Security scan sensitivity: low, medium, high (default: medium)

## Configuration File

Place an `aegis.toml` in the storage directory or specify via `--config`:

```toml
[validation]
streaming_buffer_size = 4096
type_check_depth = 5

[security]
pii_detection = true
injection_detection = true
```
