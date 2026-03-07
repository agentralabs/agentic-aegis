#!/usr/bin/env bash
set -euo pipefail
echo "Running AgenticAegis benchmarks..."
cargo bench -p agentic-aegis-core
echo "Benchmark complete."
