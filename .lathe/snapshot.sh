#!/usr/bin/env bash
set -euo pipefail

# macOS ships 'gtimeout' from coreutils; Linux has 'timeout'
if command -v gtimeout &>/dev/null; then
  TO=gtimeout
else
  TO=timeout
fi

echo "# Project Snapshot"
echo "Timestamp: $(date -u '+%Y-%m-%dT%H:%M:%SZ')"
echo

# ── Git status ───────────────────────────────────────────────────────────────
echo "## Git Status"
git status --short
echo

# ── Recent commits ───────────────────────────────────────────────────────────
echo "## Recent Commits"
git log --oneline -10
echo

# ── Build ────────────────────────────────────────────────────────────────────
echo "## Build"
BUILD_OUT=$($TO 120 cargo build 2>&1) && BUILD_OK=true || BUILD_OK=false
if $BUILD_OK; then
  echo "OK — builds clean"
else
  echo "FAILED"
  echo '```'
  echo "$BUILD_OUT" | tail -20
  echo '```'
fi
echo

# ── Tests ────────────────────────────────────────────────────────────────────
echo "## Tests"
TEST_OUT=$($TO 120 cargo test 2>&1) && TEST_OK=true || TEST_OK=false

PASS=$(echo "$TEST_OUT" | grep -c '^test .* ok$' || true)
FAIL=$(echo "$TEST_OUT" | grep -c '^test .* FAILED$' || true)
IGNORED=$(echo "$TEST_OUT" | grep -c '^test .* ignored$' || true)

echo "Pass: $PASS | Fail: $FAIL | Ignored: $IGNORED"

if ! $TEST_OK || [ "$FAIL" -gt 0 ]; then
  echo '```'
  echo "$TEST_OUT" | grep -E '^test .* FAILED$|^FAILED$|^error' | head -10
  echo '```'
fi
echo

# ── Clippy (lint) ────────────────────────────────────────────────────────────
echo "## Clippy"
CLIPPY_OUT=$($TO 120 cargo clippy --workspace --all-targets -- -D warnings 2>&1) && CLIPPY_OK=true || CLIPPY_OK=false
if $CLIPPY_OK; then
  echo "OK — no warnings"
else
  WARN_COUNT=$(echo "$CLIPPY_OUT" | grep -c '^error\[' || true)
  echo "FAILED — $WARN_COUNT error(s)"
  echo '```'
  echo "$CLIPPY_OUT" | grep -E '^error' | head -10
  echo '```'
fi
echo

# ── CI config ────────────────────────────────────────────────────────────────
echo "## CI"
CI_FILES=$(find .github/workflows -name '*.yml' 2>/dev/null | sort)
if [ -n "$CI_FILES" ]; then
  echo "$CI_FILES"
else
  echo "No CI config found"
fi
