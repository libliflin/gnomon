#!/usr/bin/env bash
set -euo pipefail

# macOS-safe timeout
if command -v gtimeout &>/dev/null; then
  TO="gtimeout"
elif command -v timeout &>/dev/null; then
  TO="timeout"
else
  TO=""
fi
run() { ${TO:+$TO 60} "$@"; }

echo "# Project Snapshot"
echo "Timestamp: $(date -u '+%Y-%m-%dT%H:%M:%SZ')"
echo ""

# --- Git status ---
echo "## Git Status"
git status --short || true
echo ""

# --- Recent commits ---
echo "## Recent Commits"
git log --oneline -10
echo ""

# --- Build ---
echo "## Build"
BUILD_OUT=$(run cargo build --workspace 2>&1) && BUILD_OK=true || BUILD_OK=false
if $BUILD_OK; then
  echo "OK — builds clean"
else
  echo "FAIL"
  echo '```'
  echo "$BUILD_OUT" | grep -E "^error" | head -10
  echo '```'
fi
echo ""

# --- Tests ---
echo "## Tests"
TEST_OUT=$(run cargo test --workspace 2>&1) && TEST_OK=true || TEST_OK=true  # parse below
# Aggregate across all test result lines
PASSED=$(echo "$TEST_OUT" | grep "test result:" | awk '{sum += $4} END {print sum+0}')
FAILED=$(echo "$TEST_OUT" | grep "test result:" | awk '{sum += $6} END {print sum+0}')
IGNORED=$(echo "$TEST_OUT" | grep "test result:" | awk '{sum += $8} END {print sum+0}')
echo "Pass: $PASSED | Fail: $FAILED | Skip: $IGNORED"
if [[ "$FAILED" -gt 0 ]]; then
  echo '```'
  echo "$TEST_OUT" | grep -E "^FAILED|^---- " | head -20
  echo '```'
fi
echo ""

# --- Clippy ---
echo "## Clippy"
CLIPPY_OUT=$(run cargo clippy --workspace --all-targets -- -D warnings 2>&1) && CLIPPY_OK=true || CLIPPY_OK=false
if $CLIPPY_OK; then
  echo "OK — no warnings"
else
  WARN_COUNT=$(echo "$CLIPPY_OUT" | grep -c "^error\[" || true)
  echo "FAIL — ${WARN_COUNT} error(s)"
  echo '```'
  echo "$CLIPPY_OUT" | grep -E "^error" | head -10
  echo '```'
fi
echo ""

# --- CI ---
echo "## CI"
for f in .github/workflows/*.yml .github/workflows/*.yaml; do
  [[ -e "$f" ]] && echo "- $f" || true
done
echo ""
