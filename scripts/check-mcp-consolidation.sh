#!/usr/bin/env bash
set -uo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
MCP_DIR="$ROOT/crates/agentic-aegis-mcp/src"
ERRORS=0

echo "=== MCP Consolidation Check ==="

# Check for unwrap() calls
UNWRAP_COUNT=$(grep -rn '\.unwrap()' "$MCP_DIR" --include='*.rs' 2>/dev/null | grep -v test | grep -v '// ok:' | wc -l | tr -d ' ')
if [ "$UNWRAP_COUNT" -gt 0 ]; then
    echo "FAIL: $UNWRAP_COUNT .unwrap() calls found in MCP crate"
    grep -rn '\.unwrap()' "$MCP_DIR" --include='*.rs' | grep -v test || true
    ERRORS=$((ERRORS + 1))
else
    echo "PASS: zero .unwrap() calls"
fi

# Check for expect() calls
EXPECT_COUNT=$(grep -rn '\.expect(' "$MCP_DIR" --include='*.rs' 2>/dev/null | grep -v test | wc -l | tr -d ' ')
if [ "$EXPECT_COUNT" -gt 0 ]; then
    echo "FAIL: $EXPECT_COUNT .expect() calls found in MCP crate"
    ERRORS=$((ERRORS + 1))
else
    echo "PASS: zero .expect() calls"
fi

# Check for -32803 error code
TOOL_NOT_FOUND=$(grep -rn '32803' "$MCP_DIR" --include='*.rs' 2>/dev/null | wc -l | tr -d ' ')
if [ "$TOOL_NOT_FOUND" -eq 0 ]; then
    echo "FAIL: -32803 (TOOL_NOT_FOUND) error code not found"
    ERRORS=$((ERRORS + 1))
else
    echo "PASS: -32803 error code present"
fi

echo ""
if [ "$ERRORS" -gt 0 ]; then
    echo "FAILED: $ERRORS check(s) failed"
    exit 1
else
    echo "ALL CHECKS PASSED"
fi
