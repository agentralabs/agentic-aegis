#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
ERRORS=0

echo "=== Command Surface Check ==="

# Check MCP tools count
TOOL_COUNT=$(grep -c '"aegis_' "$ROOT/crates/agentic-aegis-mcp/src/tools/registry.rs" | head -1 || echo "0")
echo "MCP tools found: $TOOL_COUNT (target: 12)"
if [ "$TOOL_COUNT" -lt 12 ]; then
    echo "WARN: fewer than 12 MCP tools"
    ERRORS=$((ERRORS + 1))
fi

# Check CLI commands
CLI_COMMANDS=$(grep -c 'Subcommand' "$ROOT/crates/agentic-aegis-cli/src/main.rs" || echo "0")
echo "CLI command groups: $CLI_COMMANDS"

echo ""
if [ "$ERRORS" -gt 0 ]; then
    echo "WARNINGS: $ERRORS"
else
    echo "ALL CHECKS PASSED"
fi
