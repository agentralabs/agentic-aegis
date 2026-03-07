# Contributing to AgenticAegis

Thank you for your interest in contributing to AgenticAegis! This document provides guidelines for contributing to the project.

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/YOUR_USERNAME/agentic-aegis.git`
3. Create a feature branch: `git checkout -b my-feature`
4. Make your changes
5. Run the tests (see below)
6. Commit and push
7. Open a pull request

## Development Setup

This is a Cargo workspace monorepo. All Rust crates are under `crates/`.

### Building

```bash
# Build everything (core + MCP server + CLI + FFI)
cargo build

# Build release
cargo build --release

# Core library only
cargo build -p agentic-aegis-core

# MCP server only
cargo build -p agentic-aegis-mcp
```

### Testing

```bash
# Run all tests
cargo test --workspace

# Core library only
cargo test -p agentic-aegis-core

# MCP server only
cargo test -p agentic-aegis-mcp

# CLI only
cargo test -p agentic-aegis-cli

# Run with verbose output
cargo test --workspace -- --nocapture
```

### Running the CLI

```bash
# Run the CLI binary (aegis)
cargo run -p agentic-aegis-cli -- info
cargo run -p agentic-aegis-cli -- validate complete --code "fn main() {}" --language rust

# Run the MCP server
cargo run -p agentic-aegis-cli -- serve
```

## Ways to Contribute

### Report Bugs

File an issue with:
- Steps to reproduce
- Expected behavior
- Actual behavior
- System info (OS, Rust version)

### Add an MCP Tool

1. Create a new tool handler in `crates/agentic-aegis-mcp/src/tools/`
2. Register it in `crates/agentic-aegis-mcp/src/tools/registry.rs`
3. Add tests
4. Update `docs/public/mcp-tools.md`

### Add a CLI Command

1. Add the subcommand in `crates/agentic-aegis-cli/src/`
2. Add tests
3. Update `docs/public/cli-reference.md`

### Write Examples

1. Add a new example
2. Ensure it runs without errors
3. Add a docstring explaining what it demonstrates

### Improve Documentation

All docs are in `docs/`. Fix typos, add examples, clarify explanations -- all welcome.

## Code Guidelines

- **Rust**: Follow standard Rust conventions. Run `cargo clippy` and `cargo fmt`.
- **Tests**: Every feature needs tests. We target 250+ tests across the workspace.
- **Documentation**: Update docs when changing public APIs.
- **MCP Quality**: Tool descriptions must be verb-first imperative, no trailing periods. Zero `.unwrap()` or `.expect()` in MCP code. Unknown tool error code is `-32803`.

## Commit Messages

Use conventional commit prefixes:
- `feat: add streaming validation for Go`
- `fix: correct bracket matching in nested strings`
- `chore: update dependencies`
- `docs: improve quickstart guide`
- `test: add phase07 MCP protocol tests`
- `refactor: simplify shadow execution pipeline`

**Important:** Never add `Co-Authored-By: Claude` or any AI co-author lines to commits.

## Pull Request Guidelines

- Keep PRs focused -- one feature or fix per PR
- Include tests for new functionality
- Update documentation if needed
- Ensure all tests pass before submitting
- Ensure guardrail scripts pass: `bash scripts/check-canonical-consistency.sh`
- Write a clear PR description

## Guardrails

Before submitting, run these guardrail scripts:

```bash
bash scripts/check-canonical-consistency.sh
bash scripts/check-command-surface.sh
bash scripts/check-mcp-consolidation.sh
```

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
