<p align="center">
  <img src="assets/github-hero-pane.svg" width="980" alt="AgenticAegis Hero">
</p>

<p align="center">
  <strong>AgenticAegis</strong> — Sister #11: The Shield<br>
  Streaming validation, shadow execution, and security protection for AI-generated code.
</p>

<p align="center">
  <a href="#quickstart">Quickstart</a> |
  <a href="#problems-solved">Problems Solved</a> |
  <a href="#how-it-works">How It Works</a> |
  <a href="#benchmarks">Benchmarks</a> |
  <a href="#install">Install</a>
</p>

<p align="center">
  <img src="assets/github-terminal-pane.svg" width="980" alt="AgenticAegis Terminal Demo">
</p>

---

## Install

```bash
npm install @agenticamem/aegis
```

Or with Cargo:

```bash
cargo install agentic-aegis
```

## Quickstart

```bash
# Start the MCP server
agentic-aegis-mcp

# Validate a token stream
echo '{"code": "print(1)"}' | aegis validate --streaming

# Run security scan
aegis scan security input.py
```

## How It Works

<img src="assets/architecture-agentra.svg" width="980" alt="AgenticAegis Architecture">

AgenticAegis validates AI-generated code in real time as tokens stream from the LLM. It consists of four crates:

- **agentic-aegis-core** — 20 inventions covering validation, execution, and security
- **agentic-aegis-mcp** — 12 MCP tools for agent integration
- **agentic-aegis-cli** — 30+ CLI commands under the `aegis` binary
- **agentic-aegis-ffi** — C FFI bindings for Python and other languages

## Problems Solved

- **Latent syntax errors** caught mid-stream, not after full generation
- **Security vulnerabilities** blocked by multi-layer scanning
- **Runtime failures** prevented by shadow execution in sandboxed environments
- **No rollback path** solved with session-based checkpointing
- **Lack of trust metrics** addressed with confidence scoring

## Benchmarks

<img src="assets/benchmark-chart.svg" width="980" alt="AgenticAegis Benchmarks">

| Metric | Value |
|--------|-------|
| Token validation latency | < 0.5ms p99 |
| Throughput | 50,000 tokens/sec |
| Shadow execution startup | < 10ms |
| Security scan rate | 10,000 lines/sec |

## Standalone Guarantee

AgenticAegis works fully independently. No other Agentra sister is required. Bridge connections to siblings like AgenticMemory or AgenticCognition are optional enhancements that activate automatically when available.

## License

MIT
