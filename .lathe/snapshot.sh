#!/usr/bin/env bash
set -euo pipefail

# macOS-compatible timeout: use gtimeout if available, else timeout, else plain exec
_timeout() {
  if command -v gtimeout &>/dev/null; then
    gtimeout "$@"
  elif command -v timeout &>/dev/null; then
    timeout "$@"
  else
    shift; "$@"
  fi
}

echo "# Project Snapshot"
echo "Generated: $(date -u '+%Y-%m-%d %H:%M:%S UTC')"
echo

# ── Git status ────────────────────────────────────────────────────────────────
echo "## Git Status"
git status --short || true
echo

# ── Recent commits ────────────────────────────────────────────────────────────
echo "## Recent Commits"
git log --oneline -10
echo

# ── Build ─────────────────────────────────────────────────────────────────────
echo "## Build"
BUILD_OUT=$(_timeout 120 cargo build 2>&1) && BUILD_OK=true || BUILD_OK=false
if $BUILD_OK; then
  echo "OK — builds clean"
else
  echo "FAILED"
  echo '```'
  echo "$BUILD_OUT" | grep -E "^error" | head -10
  echo '```'
fi
echo

# ── Tests ─────────────────────────────────────────────────────────────────────
echo "## Tests"
TEST_OUT=$(_timeout 120 cargo test 2>&1) || true

# Sum counts across all test suites (unit + doc + integration).
# Use grep -oE to extract "NNN <field>" — avoids greedy sed capturing only
# the last digit of multi-digit numbers (e.g. "85 passed" → "5").
_sum_field() {
  echo "$TEST_OUT" | grep -E "^test result:" \
    | grep -oE "[0-9]+ $1" \
    | awk '{s+=$1} END {print s+0}'
}
PASS=$(_sum_field "passed")
FAIL=$(_sum_field "failed")
IGNORED=$(_sum_field "ignored")

echo "Pass: $PASS | Fail: $FAIL | Ignored: $IGNORED"

if [[ "$FAIL" != "0" && "$FAIL" != "" ]]; then
  echo
  echo "Failures:"
  echo '```'
  echo "$TEST_OUT" | grep -E "^(FAILED|failures:|---- )" | head -20
  echo '```'
fi
echo

# ── Clippy (lint) ─────────────────────────────────────────────────────────────
echo "## Clippy"
CLIPPY_OUT=$(_timeout 120 cargo clippy --all-targets --message-format=short 2>&1) && CLIPPY_OK=true || CLIPPY_OK=false
WARN_COUNT=$(echo "$CLIPPY_OUT" | grep -c "^warning" || true)
ERROR_COUNT=$(echo "$CLIPPY_OUT" | grep -c "^error" || true)

if $CLIPPY_OK && [[ "$ERROR_COUNT" -eq 0 ]]; then
  echo "OK — ${WARN_COUNT} warning(s)"
else
  echo "FAILED — ${ERROR_COUNT} error(s), ${WARN_COUNT} warning(s)"
  echo '```'
  echo "$CLIPPY_OUT" | grep -E "^(error|warning)" | head -10
  echo '```'
fi
echo

# ── CI ────────────────────────────────────────────────────────────────────────
echo "## CI"
CI_FILES=$(find .github/workflows -name "*.yml" -o -name "*.yaml" 2>/dev/null | sort || true)
if [[ -n "$CI_FILES" ]]; then
  echo "$CI_FILES"
else
  echo "No CI config found (.github/workflows)"
fi
