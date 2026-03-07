---
status: stable
---

# CLI Reference

The `aegis` binary provides 30+ commands for validation, execution, and security.

## Core Commands

- `aegis validate --streaming` — Validate a token stream in real time
- `aegis validate --complete` — Validate a complete code block
- `aegis shadow execute` — Run code in the shadow executor
- `aegis scan security` — Run all security scanners on input
- `aegis session create` — Start a new validation session
- `aegis session status` — Check session state
- `aegis session end` — End and summarize a session
- `aegis rollback` — Revert to a previous safe checkpoint

## Global Flags

- `--format json|text` — Output format (default: text)
- `--verbose` — Enable detailed logging
- `--config <path>` — Path to configuration file

## Shell Completions

Generate completions with `aegis completions <shell>` for bash, zsh, or fish.
