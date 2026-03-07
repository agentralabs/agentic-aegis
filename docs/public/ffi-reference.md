---
status: stable
---

# FFI Reference

The `agentic-aegis-ffi` crate exposes C-compatible bindings for use from Python, Ruby, and other languages.

## Available Functions

- `aegis_validate_token(token: *const c_char) -> c_int` — Validate a single token
- `aegis_session_create() -> *mut AegisSession` — Create a new session
- `aegis_session_free(session: *mut AegisSession)` — Free a session
- `aegis_scan_security(input: *const c_char) -> *mut c_char` — Scan input for threats
- `aegis_version() -> *const c_char` — Return the library version

## Python Usage

```python
from agentic_aegis import AegisValidator
validator = AegisValidator()
result = validator.validate_streaming(tokens)
```

## Memory Safety

All FFI functions follow Rust ownership rules. Callers must free returned pointers using the corresponding `_free` functions.
