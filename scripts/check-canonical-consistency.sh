#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ERRORS=0

echo "=== Canonical Consistency Check ==="

# Check Cargo.toml files
for crate in core mcp cli ffi; do
    CARGO="$ROOT/crates/agentic-aegis-$crate/Cargo.toml"
    if [ -f "$CARGO" ]; then
        echo "PASS: $crate Cargo.toml exists"
    else
        echo "FAIL: $crate Cargo.toml missing"
        ERRORS=$((ERRORS + 1))
    fi
done

# Check workspace Cargo.toml
if [ -f "$ROOT/Cargo.toml" ]; then
    echo "PASS: workspace Cargo.toml exists"
else
    echo "FAIL: workspace Cargo.toml missing"
    ERRORS=$((ERRORS + 1))
fi

# Check CLAUDE.md
if [ -f "$ROOT/CLAUDE.md" ]; then
    echo "PASS: CLAUDE.md exists"
else
    echo "FAIL: CLAUDE.md missing"
    ERRORS=$((ERRORS + 1))
fi

# Check sister.manifest.json
if [ -f "$ROOT/sister.manifest.json" ]; then
    echo "PASS: sister.manifest.json exists"
else
    echo "FAIL: sister.manifest.json missing"
    ERRORS=$((ERRORS + 1))
fi

echo ""
if [ "$ERRORS" -gt 0 ]; then
    echo "FAILED: $ERRORS check(s) failed"
    exit 1
else
    echo "ALL CHECKS PASSED"
fi
