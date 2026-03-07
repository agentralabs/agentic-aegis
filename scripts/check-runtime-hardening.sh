#!/usr/bin/env bash
set -uo pipefail

# check-runtime-hardening.sh
# Validates runtime hardening requirements for AgenticAegis MCP server.
# Exit 0 if all checks pass, exit 1 on first failure.

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
MCP_SRC="$REPO_ROOT/crates/agentic-aegis-mcp/src"
CORE_SRC="$REPO_ROOT/crates/agentic-aegis-core/src"

FAIL=0
pass() { echo "  PASS  $1"; }
fail() { echo "  FAIL  $1"; FAIL=1; }

echo "=== Runtime Hardening Check ==="

# 1. No .unwrap() in MCP server
COUNT=$(grep -rn '\.unwrap()' "$MCP_SRC" --include='*.rs' 2>/dev/null | grep -v test | wc -l | tr -d ' ')
if [ "$COUNT" -eq 0 ]; then pass "zero .unwrap() in MCP server"; else fail "$COUNT .unwrap() in MCP server"; fi

# 2. No .expect() in MCP server
COUNT=$(grep -rn '\.expect(' "$MCP_SRC" --include='*.rs' 2>/dev/null | grep -v test | wc -l | tr -d ' ')
if [ "$COUNT" -eq 0 ]; then pass "zero .expect() in MCP server"; else fail "$COUNT .expect() in MCP server"; fi

# 3. -32803 error code present
if grep -rn '32803' "$MCP_SRC" --include='*.rs' >/dev/null 2>&1; then
    pass "-32803 TOOL_NOT_FOUND present"
else
    fail "-32803 TOOL_NOT_FOUND missing"
fi

# 4. JSON-RPC version check
if grep -rn 'jsonrpc' "$MCP_SRC" --include='*.rs' >/dev/null 2>&1; then
    pass "JSON-RPC version handling present"
else
    fail "JSON-RPC version handling missing"
fi

# 5. Content-length handling
if grep -rn 'content-length' "$MCP_SRC" --include='*.rs' >/dev/null 2>&1; then
    pass "content-length handling present"
else
    fail "content-length handling missing"
fi

echo ""
if [ "$FAIL" -ne 0 ]; then
    echo "RESULT: FAIL"
    exit 1
else
    echo "RESULT: ALL CHECKS PASSED"
fi
