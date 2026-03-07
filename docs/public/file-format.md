---
status: stable
---

# File Format

AgenticAegis uses the `.aegis` file extension for validation session snapshots.

## Structure

An `.aegis` file is a binary format containing:

1. **Header** (8 bytes) — Magic bytes `AEGIS\x00\x01\x00` and version
2. **Session metadata** — JSON-encoded session info (creation time, config, status)
3. **Validation log** — Sequence of validation events with timestamps
4. **Checkpoint data** — Serialized state for rollback support

## Reading

```bash
aegis inspect file session.aegis
```

## Compatibility

The format is versioned. Older files are automatically migrated on read.
