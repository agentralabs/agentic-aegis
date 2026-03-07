#!/usr/bin/env bash
set -euo pipefail

# check-install-commands.sh — Verify install script and documentation.
# Exits 1 on any failure.

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
FAIL=0

pass() { echo "  PASS  $1"; }
fail() { echo "  FAIL  $1"; FAIL=1; }

echo "=== Install Commands Check ==="
echo ""

# --- 1. scripts/install.sh exists and is executable ---
echo "--- Install script presence and permissions ---"
INSTALL_SCRIPT="$REPO_ROOT/scripts/install.sh"
if [[ -f "$INSTALL_SCRIPT" ]]; then
    pass "scripts/install.sh exists"
    if [[ -x "$INSTALL_SCRIPT" ]]; then
        pass "scripts/install.sh is executable"
    else
        fail "scripts/install.sh is not executable"
    fi
else
    fail "scripts/install.sh does not exist"
fi

# --- 2. install.sh contains profile support (desktop, terminal, server) ---
echo ""
echo "--- Install profile support ---"
if [[ -f "$INSTALL_SCRIPT" ]]; then
    PROFILES=("desktop" "terminal" "server")
    for profile in "${PROFILES[@]}"; do
        if grep -qi "$profile" "$INSTALL_SCRIPT"; then
            pass "install.sh contains '$profile' profile support"
        else
            fail "install.sh missing '$profile' profile support"
        fi
    done
else
    fail "cannot check profiles — install.sh not found"
fi

# --- 3. README.md contains install instructions ---
echo ""
echo "--- README install instructions ---"
README="$REPO_ROOT/README.md"
if [[ -f "$README" ]]; then
    # Check for common install-related keywords
    if grep -qi 'install' "$README"; then
        pass "README.md contains install instructions"
    else
        fail "README.md does not mention installation"
    fi
else
    fail "README.md not found"
fi

# --- Summary ---
echo ""
if [[ $FAIL -ne 0 ]]; then
    echo "RESULT: FAIL — install command issues detected."
    exit 1
else
    echo "RESULT: PASS — install commands are correct."
fi
