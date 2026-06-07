#!/bin/bash
# test-integration.sh — property-based tests from VOCABULARY.md
# Usage: ./test-integration.sh <binary>

set -e

BINARY="${1:?usage: test-integration.sh <binary>}"
PASS=0
FAIL=0
TOTAL=0

ok() { PASS=$((PASS+1)); TOTAL=$((TOTAL+1)); echo "  PASS: $1"; }
bad() { FAIL=$((FAIL+1)); TOTAL=$((TOTAL+1)); echo "  FAIL: $1"; }

echo "=== Integration Tests ==="
echo "Binary: $BINARY"
echo ""

# --- Should match (expect exit 1) ---
echo "--- Should match (expect flagged) ---"

echo "## Phase 1: Setup" | $BINARY --stdin >/dev/null 2>&1 && bad "## Phase 1: Setup" || ok "## Phase 1: Setup"
echo "// Pass 1: tokenize" | $BINARY --stdin >/dev/null 2>&1 && bad "// Pass 1: tokenize" || ok "// Pass 1: tokenize"
echo "Step 3 of 5" | $BINARY --stdin >/dev/null 2>&1 && bad "Step 3 of 5" || ok "Step 3 of 5"
echo "feat: phase 2 of auth refactor" | $BINARY --stdin >/dev/null 2>&1 && bad "feat: phase 2 of auth refactor" || ok "feat: phase 2 of auth refactor"
echo "Stage II, data migration" | $BINARY --stdin >/dev/null 2>&1 && bad "Stage II, data migration" || ok "Stage II, data migration"

# Additional should-match cases
echo "## Round 1: review" | $BINARY --stdin >/dev/null 2>&1 && bad "## Round 1: review" || ok "## Round 1: review"
echo "Iteration 5: optimize" | $BINARY --stdin >/dev/null 2>&1 && bad "Iteration 5: optimize" || ok "Iteration 5: optimize"
echo "Wave 2 of rollout" | $BINARY --stdin >/dev/null 2>&1 && bad "Wave 2 of rollout" || ok "Wave 2 of rollout"
echo "Batch 3 processing" | $BINARY --stdin >/dev/null 2>&1 && bad "Batch 3 processing" || ok "Batch 3 processing"
echo "## Part 1: intro" | $BINARY --stdin >/dev/null 2>&1 && bad "## Part 1: intro" || ok "## Part 1: intro"

# --- Must not match (expect exit 0) ---
echo ""
echo "--- Must not match (expect clean) ---"

echo "feat: add parser" | $BINARY --stdin >/dev/null 2>&1 && ok "feat: add parser" || bad "feat: add parser"
echo "nit: rename uc to userCount" | $BINARY --stdin >/dev/null 2>&1 && ok "nit: rename uc to userCount" || bad "nit: rename uc to userCount"
echo "fix(api): correct fee calculation" | $BINARY --stdin >/dev/null 2>&1 && ok "fix(api): correct fee calculation" || bad "fix(api): correct fee calculation"
echo "TODO: handle null input" | $BINARY --stdin >/dev/null 2>&1 && ok "TODO: handle null input" || bad "TODO: handle null input"
echo "chore: bump deps" | $BINARY --stdin >/dev/null 2>&1 && ok "chore: bump deps" || bad "chore: bump deps"
echo "WIP: draft, do not merge" | $BINARY --stdin >/dev/null 2>&1 && ok "WIP: draft, do not merge" || bad "WIP: draft, do not merge"
echo "// FIXME: race condition on shutdown" | $BINARY --stdin >/dev/null 2>&1 && ok "// FIXME: race condition" || bad "// FIXME: race condition"
echo "increment the retry counter" | $BINARY --stdin >/dev/null 2>&1 && ok "increment the retry counter" || bad "increment the retry counter"
echo "the first pass over the array" | $BINARY --stdin >/dev/null 2>&1 && ok "the first pass over the array" || bad "the first pass over the array"

# Additional must-not-match cases
echo "LGTM" | $BINARY --stdin >/dev/null 2>&1 && ok "LGTM" || bad "LGTM"
echo "PTAL" | $BINARY --stdin >/dev/null 2>&1 && ok "PTAL" || bad "PTAL"
echo "refactor: clean up module" | $BINARY --stdin >/dev/null 2>&1 && ok "refactor: clean up module" || bad "refactor: clean up module"
echo "docs: update README" | $BINARY --stdin >/dev/null 2>&1 && ok "docs: update README" || bad "docs: update README"
echo "NOTE: this is intentional" | $BINARY --stdin >/dev/null 2>&1 && ok "NOTE: this is intentional" || bad "NOTE: this is intentional"

# --- JSON output test ---
echo ""
echo "--- JSON output ---"
json=$(echo "## Phase 1: Setup" | $BINARY --stdin --json 2>/dev/null)
if echo "$json" | grep -q '"phase"'; then
    ok "JSON output contains term"
else
    bad "JSON output missing term"
fi

# --- Summary ---
echo ""
echo "=== Results ==="
echo "Passed: $PASS / $TOTAL"
echo "Failed: $FAIL / $TOTAL"

if [ $FAIL -gt 0 ]; then
    exit 1
fi
