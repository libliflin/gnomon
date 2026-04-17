#!/usr/bin/env bash
set -euo pipefail

# macOS-safe timeout
if command -v gtimeout &>/dev/null; then
  TO=gtimeout
elif command -v timeout &>/dev/null; then
  TO=timeout
else
  TO=""
fi
run() { ${TO:+$TO 60} "$@"; }

echo "# Project Snapshot"
echo "Generated: $(date -u '+%Y-%m-%dT%H:%M:%SZ')"
echo

# ── Git status ──────────────────────────────────────────────────────────────
echo "## Git Status"
git status --short || true
echo

# ── Recent commits ───────────────────────────────────────────────────────────
echo "## Recent Commits"
git log --oneline -10 || true
echo

# ── Build ────────────────────────────────────────────────────────────────────
echo "## Build"
BUILD_OK=true; BUILD_OUT=$(run cargo build 2>&1) || BUILD_OK=false
if $BUILD_OK; then
  echo "OK — builds clean"
else
  echo "FAIL"
  echo "$BUILD_OUT" | { grep -E "^error" || true; } | head -10
fi
echo

# ── Tests ────────────────────────────────────────────────────────────────────
echo "## Tests"
TEST_OK=true; TEST_OUT=$(run cargo test --lib --bins 2>&1) || TEST_OK=false
PASS=$(echo "$TEST_OUT" | { grep -oE "test .* \.\.\. ok" || true; } | wc -l | tr -d ' ')
FAIL=$(echo "$TEST_OUT" | { grep -oE "test .* \.\.\. FAILED" || true; } | wc -l | tr -d ' ')
IGNORED=$(echo "$TEST_OUT" | { grep -oE "test .* \.\.\. ignored" || true; } | wc -l | tr -d ' ')
echo "Pass: $PASS | Fail: $FAIL | Ignored: $IGNORED"
if ! $TEST_OK; then
  echo "$TEST_OUT" | { grep -A 5 "FAILED\|^error" || true; } | head -20
fi
echo

# ── Clippy (lint) ────────────────────────────────────────────────────────────
echo "## Clippy"
CLIPPY_OK=true; CLIPPY_OUT=$(run cargo clippy -- -D warnings 2>&1) || CLIPPY_OK=false
if $CLIPPY_OK; then
  echo "OK"
else
  WARN_COUNT=$(echo "$CLIPPY_OUT" | { grep -c "^error\|^warning" || true; })
  echo "FAIL — $WARN_COUNT diagnostics"
  echo "$CLIPPY_OUT" | { grep "^error" || true; } | head -8
fi
echo

# ── CI ───────────────────────────────────────────────────────────────────────
echo "## CI"
CI_FILES=$(find .github/workflows -name "*.yml" -o -name "*.yaml" 2>/dev/null | sort || true)
if [ -n "$CI_FILES" ]; then
  echo "$CI_FILES"
else
  echo "No CI config found (.github/workflows missing)"
fi
