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

# Internal code-as-name tell (VOCABULARY.md, internal tracking codes)
echo "ci: fix the no-OS-comm guard's fail-open nm regex (review B1)" | $BINARY --stdin >/dev/null 2>&1 && bad "ci: ... (review B1)" || ok "ci: ... (review B1)"
echo "addresses finding #7" | $BINARY --stdin >/dev/null 2>&1 && bad "addresses finding #7" || ok "addresses finding #7"
echo "blocker B2 resolved" | $BINARY --stdin >/dev/null 2>&1 && bad "blocker B2 resolved" || ok "blocker B2 resolved"
echo "addresses review (B1)" | $BINARY --stdin >/dev/null 2>&1 && bad "addresses review (B1)" || ok "addresses review (B1)"

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

# Internal code-as-name gates (VOCABULARY.md, internal tracking codes)
echo "review 3 files" | $BINARY --stdin >/dev/null 2>&1 && ok "review 3 files" || bad "review 3 files"
echo "finding 0 results" | $BINARY --stdin >/dev/null 2>&1 && ok "finding 0 results" || bad "finding 0 results"
echo "fixes #18" | $BINARY --stdin >/dev/null 2>&1 && ok "fixes #18" || bad "fixes #18"
echo "closes #35" | $BINARY --stdin >/dev/null 2>&1 && ok "closes #35" || bad "closes #35"
echo "Finding #B1 was fixed upstream" | $BINARY --stdin >/dev/null 2>&1 && ok "Finding #B1 was fixed upstream" || bad "Finding #B1 was fixed upstream"

# --- Bare-numeral headers (markdown files only) ---
echo ""
echo "--- Bare-numeral headers ---"
tmpdir=$(mktemp -d)
trap 'rm -rf "$tmpdir"' EXIT

printf '## 3\n' > "$tmpdir/plan.md"
$BINARY "$tmpdir/plan.md" >/dev/null 2>&1 && bad "## 3 in .md" || ok "## 3 in .md"
printf '## 5.5\n' > "$tmpdir/plan.md"
$BINARY "$tmpdir/plan.md" >/dev/null 2>&1 && bad "## 5.5 in .md" || ok "## 5.5 in .md"
printf '## 1.2.3\n' > "$tmpdir/changelog.md"
$BINARY "$tmpdir/changelog.md" >/dev/null 2>&1 && ok "## 1.2.3 version heading" || bad "## 1.2.3 version heading"
printf '## Error handling\n' > "$tmpdir/plan.md"
$BINARY "$tmpdir/plan.md" >/dev/null 2>&1 && ok "## named header" || bad "## named header"
printf '# 3 retries by default\n' > "$tmpdir/script.sh"
$BINARY "$tmpdir/script.sh" >/dev/null 2>&1 && ok "numeral comment in .sh" || bad "numeral comment in .sh"

# --- JSON output test ---
echo ""
echo "--- JSON output ---"
json=$(echo "## Phase 1: Setup" | $BINARY --stdin --json 2>/dev/null) || true
if echo "$json" | grep -q '"phase"'; then
    ok "JSON output contains term"
else
    bad "JSON output missing term"
fi

# --- History scan (--log) ---
echo ""
echo "--- History scan (--log) ---"
BINARY_ABS="$(cd "$(dirname "$BINARY")" && pwd)/$(basename "$BINARY")"
repo="$tmpdir/log-repo"
git init -q "$repo"
git -C "$repo" -c user.name=t -c user.email=t@t commit -q --allow-empty -m "feat: add parser"
(cd "$repo" && "$BINARY_ABS" --log >/dev/null 2>&1) && ok "--log clean history" || bad "--log clean history"
git -C "$repo" -c user.name=t -c user.email=t@t commit -q --allow-empty -m "docs: phase 2 of rollout"
(cd "$repo" && "$BINARY_ABS" --log >/dev/null 2>&1) && bad "--log flagged history" || ok "--log flagged history"
out=$(cd "$repo" && "$BINARY_ABS" --log 2>&1) || true
if echo "$out" | grep -qE '^[0-9a-f]{7}:'; then
    ok "--log output labelled with commit sha"
else
    bad "--log output labelled with commit sha"
fi

# --- Symlink handling (--all) ---
echo ""
echo "--- Symlink handling (--all) ---"
walk="$tmpdir/walk-repo"
mkdir -p "$walk/sub"
printf '## Stage 2 of rollout\n' > "$walk/sub/notes.md"
ln -s sub "$walk/link"
ln -s . "$walk/loop"
out=$(cd "$walk" && timeout 10 "$BINARY_ABS" --all 2>&1) && status=0 || status=$?
if [ $status -eq 1 ]; then
    ok "--all terminates with cyclic symlink"
else
    bad "--all terminates with cyclic symlink (exit $status)"
fi
count=$(echo "$out" | grep -c 'Stage 2 of rollout' || true)
if [ "$count" -eq 1 ]; then
    ok "--all scans symlinked content once"
else
    bad "--all scans symlinked content once (got $count)"
fi

# --- Summary ---
echo ""
echo "=== Results ==="
echo "Passed: $PASS / $TOTAL"
echo "Failed: $FAIL / $TOTAL"

if [ $FAIL -gt 0 ]; then
    exit 1
fi
