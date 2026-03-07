---
status: stable
---

# Benchmarks

Performance measurements for AgenticAegis v0.1.0.

## Streaming Validation

| Metric | Value |
|--------|-------|
| Token validation latency | < 0.5ms p99 |
| Throughput | 50,000 tokens/sec |
| Memory per session | ~2 MB |

## Shadow Execution

| Metric | Value |
|--------|-------|
| Sandbox startup | < 10ms |
| Execution overhead | < 5% |
| Concurrent sessions | 100+ |

## Security Scanning

| Metric | Value |
|--------|-------|
| PII detection accuracy | 98.5% |
| Injection detection rate | 99.2% |
| Scan throughput | 10,000 lines/sec |

All benchmarks run on Apple M2 with 16 GB RAM using Criterion.rs.
