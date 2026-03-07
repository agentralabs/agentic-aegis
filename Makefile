.PHONY: all build build-debug test test-unit lint lint-fmt lint-clippy clean install check

all: build

build:
	cargo build --workspace --release

build-debug:
	cargo build --workspace

test:
	cargo test --workspace

test-unit:
	cargo test --lib --workspace

lint: lint-fmt lint-clippy

lint-fmt:
	cargo fmt --all -- --check

lint-clippy:
	cargo clippy --workspace --all-targets -- -D warnings

clean:
	cargo clean

install:
	cargo install --path crates/agentic-aegis-mcp
	cargo install --path crates/agentic-aegis-cli

check:
	bash scripts/check-mcp-consolidation.sh
	bash scripts/check-canonical-consistency.sh
	bash scripts/check-command-surface.sh
