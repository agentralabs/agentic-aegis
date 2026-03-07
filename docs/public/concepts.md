---
status: stable
---

# Core Concepts

AgenticAegis provides three foundational capabilities for securing AI-generated code.

## Streaming Validation

Validates code as it is generated token-by-token, catching syntax errors and type mismatches before the full output is complete.

## Shadow Execution

Runs generated code in an isolated sandbox to detect runtime errors, resource exhaustion, and unexpected side effects without impacting production.

## Protection

Scans for prompt injection, PII leakage, unsafe patterns, and payload attacks. The multi-layer defense ensures that malicious or dangerous code never reaches execution.

## Sessions

All validation activity is tracked within sessions, enabling rollback, confidence scoring, and correction hints across the lifecycle of a generation request.
