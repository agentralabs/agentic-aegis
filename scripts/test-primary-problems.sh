#!/usr/bin/env bash
set -euo pipefail

# test-primary-problems.sh
# Runs the critical test suite and validates primary invariants for
# AgenticAegis before any push or release.

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
MCP_SRC="$REPO_ROOT/crates/agentic-aegis-mcp/src"
FAILED=0

echo "=== AgenticAegis Primary Problem Tests ==="
echo ""

# 1. All workspace tests pass
echo "--- Running workspace tests ---"
if cargo test --workspace --quiet 2>&1; then
    echo "  PASS  All workspace tests pass"
else
    echo "  FAIL  Workspace tests failed"
    FAILED=1
fi

# 2. Clippy clean
echo ""
echo "--- Running clippy ---"
if cargo clippy --workspace -- -D warnings 2>&1 | tail -1 | grep -q "Finished"; then
    echo "  PASS  Clippy clean"
else
    echo "  FAIL  Clippy warnings/errors"
    FAILED=1
fi

# 3. MCP consolidation
echo ""
echo "--- MCP consolidation check ---"
bash "$REPO_ROOT/scripts/check-mcp-consolidation.sh" 2>&1

echo ""
if [ "$FAILED" -ne 0 ]; then
    echo "RESULT: FAIL"
    exit 1
else
    echo "RESULT: ALL PRIMARY PROBLEMS VALIDATED"
fi
