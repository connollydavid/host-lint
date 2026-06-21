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

# --- Symlink handling (--all over git-tracked files) ---
echo ""
echo "--- Symlink handling (--all) ---"
walk="$tmpdir/walk-repo"
mkdir -p "$walk/sub"
printf '## Stage 2 of rollout\n' > "$walk/sub/notes.md"
ln -s sub "$walk/link"
ln -s . "$walk/loop"
# `--all` audits tracked files (`git ls-files`); track the file and the symlinks
# (incl. the `loop -> .` cycle) so the scan exercises the symlink skip. git records
# symlinks by target without following them, so `add -A` does not recurse the cycle.
git init -q "$walk"
git -C "$walk" -c user.name=t -c user.email=t@t add -A
git -C "$walk" -c user.name=t -c user.email=t@t commit -q -m "init"
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

# --- Tier 1+2: decimal numerals and label prefix (expect flag, rc=1) ---
echo ""
echo "--- Decimal numerals + label prefix (expect flag) ---"
for s in 'entry point (Phase 5.0).' '5.5: exec/pty tools' '// 5.5: the pty exec tool' '## 5.5: error handling' 'section 2.1'; do
    printf '%s' "$s" | $BINARY --stdin >/dev/null 2>&1 && rc=0 || rc=$?
    [ "$rc" -eq 1 ] && ok "flag: $s" || bad "flag: $s (rc=$rc)"
done

# --- Tier 3: bare-numeral degenerate form (expect warn, rc=3) ---
echo ""
echo "--- Bare-numeral degenerate form (expect warn) ---"
for s in 'as decided in 2.1' 'exec tools (5.5)' 'the peek/poke tools arrive in 5.3' 'implements work-item 5.3'; do
    printf '%s' "$s" | $BINARY --stdin >/dev/null 2>&1 && rc=0 || rc=$?
    [ "$rc" -eq 3 ] && ok "warn: $s" || bad "warn: $s (rc=$rc)"
done

# --- Tier 3: leading code-as-name label (expect warn, rc=3) ---
echo ""
echo "--- Leading code label (expect warn) ---"
for s in 'F1: PE version stamp 3.10' 'F2: handle isolation' 'B3: the durable name follows'; do
    printf '%s' "$s" | $BINARY --stdin >/dev/null 2>&1 && rc=0 || rc=$?
    [ "$rc" -eq 3 ] && ok "warn: $s" || bad "warn: $s (rc=$rc)"
done

# --- Version / quantity not warned (expect clean, rc=0) ---
echo ""
echo "--- Version / quantity stay clean ---"
for s in 'bump to v2.1' 'requires Python 3.11' '5.5 seconds elapsed' 'increased by 2.1%' 'COM1 open, DCB seeding' 'the F1 key opens help' 'wire-respond on Windows NT 3.1' 'ships the SDK 2.1 headers'; do
    printf '%s' "$s" | $BINARY --stdin >/dev/null 2>&1 && rc=0 || rc=$?
    [ "$rc" -eq 0 ] && ok "clean: $s" || bad "clean: $s (rc=$rc)"
done

# --- Prose agentic tells (advisory warn, rc=3) ---
echo ""
echo "--- Prose tells warn, never block ---"
for s in 'a robust streamlined rollout' 'Let'"'"'s unpack the rollout' 'We delve into the tapestry'; do
    printf '%s' "$s" | $BINARY --stdin >/dev/null 2>&1 && rc=0 || rc=$?
    [ "$rc" -eq 3 ] && ok "warn: $s" || bad "warn: $s (rc=$rc)"
done
# Decoration on the --stdin subject line escalates to a flag (blocks), it does not
# merely warn — the subject-decoration rule. Body prose and --prose stay advisory.
for s in 'a clean title — with an em-dash' 'a curly “quoted” subject'; do
    printf '%s' "$s" | $BINARY --stdin >/dev/null 2>&1 && rc=0 || rc=$?
    [ "$rc" -eq 1 ] && ok "flag: subject decoration: $s" || bad "flag: subject decoration: $s (rc=$rc)"
done
# A trope-dense paragraph trips the density summary in --json.
dense="Let's unpack this. It's not a tweak, it's a revolution. We delve. We leverage. We harness. The result? Pure synergy. Fast, clean, and robust."
printf '%s' "$dense" | $BINARY --stdin --json 2>/dev/null | grep -q '"term": "tell-density"' \
    && ok "density summary emitted" || bad "density summary emitted"

# --- Markdown-aware --prose: code blocks and headings are not prose ---
echo ""
echo "--- Markdown awareness ---"
md=$(mktemp --suffix=.md)
printf 'Intro paragraph here.\n\n```\nIt'"'"'s not a tweak, it'"'"'s a revolution, we delve.\n```\n' > "$md"
$BINARY --prose "$md" >/dev/null 2>&1 && rc=0 || rc=$?
[ "$rc" -eq 0 ] && ok "code block in .md is not prose" || bad "code block in .md (rc=$rc)"
# the same text as plain stdin (not markdown) DOES warn
printf 'It'"'"'s not a tweak, it'"'"'s a revolution, we delve.' | $BINARY --stdin >/dev/null 2>&1 && rc=0 || rc=$?
[ "$rc" -eq 3 ] && ok "same text as plain prose warns" || bad "plain prose warns (rc=$rc)"
rm -f "$md"

# --- --docs: repo-wide prose lane (markdown only, honors .host-lintignore) ---
echo ""
echo "--- --docs (repo-wide prose) ---"
docs=$(mktemp -d)
git -C "$docs" init -q
# An authored doc with prose tropes (negative-parallelism + em-dash + tricolon).
printf "It's not a tweak, it's a revolution, we delve — fast, clean, and robust.\n" > "$docs/guide.md"
# Code with an em-dash in a comment: --docs must NOT prose-scan it (scope = docs).
printf 'fn f() {} // a comment — with a dash\n' > "$docs/src.rs"
# An immutable record excluded via .host-lintignore.
printf 'A frozen note — with a dash.\n' > "$docs/RECORD.md"
printf 'RECORD.md\n' > "$docs/.host-lintignore"
git -C "$docs" add -A
git -C "$docs" -c user.name=t -c user.email=t@t commit -q -m "init"
out=$(cd "$docs" && "$BINARY_ABS" --docs 2>&1) && rc=0 || rc=$?
echo "$out" | grep -q "guide.md" && ok "--docs flags authored markdown prose" || bad "--docs flags authored markdown prose"
[ "$rc" -eq 3 ] && ok "--docs warns (exit 3) on tropes" || bad "--docs warns exit 3 (rc=$rc)"
echo "$out" | grep -q "src.rs" && bad "--docs must not prose-scan code" || ok "--docs skips code (.rs)"
echo "$out" | grep -q "RECORD.md" && bad "--docs must honor .host-lintignore" || ok "--docs honors .host-lintignore"
rm -rf "$docs"

# --- plan/0031: prose output is located (line:col) + fix-hinted ---
echo ""
echo "--- prose output: col + fix hint ---"
pmd=$(mktemp --suffix=.md)
printf 'A sentence with a dash — right here in it.\n' > "$pmd"
pout=$($BINARY --prose "$pmd" 2>&1) || true
echo "$pout" | grep -qE ':[0-9]+:[0-9]+: warning: — .*\[fix:' \
    && ok "decoration carries line:col + fix hint" || bad "decoration line:col + fix hint (got: $pout)"
rm -f "$pmd"

# --- Warn output marker and JSON severity ---
echo ""
echo "--- Severity in output ---"
warn_out=$(printf '%s' 'exec tools (5.5)' | $BINARY --stdin 2>&1) || true
echo "$warn_out" | grep -q 'warning:' && ok "warn line marked 'warning:'" || bad "warn line marked 'warning:'"
warn_json=$(printf '%s' 'exec tools (5.5)' | $BINARY --stdin --json 2>/dev/null) || true
echo "$warn_json" | grep -q '"severity": "warn"' && ok "JSON severity warn" || bad "JSON severity warn"
flag_json=$(printf '%s' 'phase 2 of rollout' | $BINARY --stdin --json 2>/dev/null) || true
echo "$flag_json" | grep -q '"severity": "flag"' && ok "JSON severity flag" || bad "JSON severity flag"

# --- Co-Authored-By trailer exemption (expect clean, rc=0) ---
echo ""
echo "--- Co-Authored-By trailers exempt ---"
for s in 'Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>' 'co-authored-by: someone 2.1' 'Co-Authored-By: Phase 2 Bot <bot@example.com>'; do
    printf '%s' "$s" | $BINARY --stdin >/dev/null 2>&1 && rc=0 || rc=$?
    [ "$rc" -eq 0 ] && ok "exempt: $s" || bad "exempt: $s (rc=$rc)"
done

# --- LEXICON: CRUD guards + strict escalation (issue #13) ---
echo ""
echo "--- LEXICON ---"
LEX=$(mktemp -d)
export GIT_DIR="$LEX/.git"   # redirect repo_root to the temp dir (no .git needed)
$BINARY lexicon add "Windows 3.1" >/dev/null 2>&1 && ok "add vocab" || bad "add vocab"
$BINARY lexicon add "#7" --url https://github.com/connollydavid/host/issues/7 >/dev/null 2>&1 && ok "add cited ref" || bad "add cited ref"
# Each guard refuses with exit 1 (the tool, not the prompt, owns the decision).
$BINARY lexicon add "5.5" >/dev/null 2>&1 && bad "G1 master key rejected" || ok "G1 master key rejected"
$BINARY lexicon add "Phase 5.5" >/dev/null 2>&1 && bad "G2 laundering rejected" || ok "G2 laundering rejected"
$BINARY lexicon add "#999" >/dev/null 2>&1 && bad "citation gate rejected" || ok "citation gate rejected"
$BINARY lexicon list 2>/dev/null | grep -q "Windows 3.1" && ok "list shows vocab" || bad "list shows vocab"
$BINARY lexicon --check >/dev/null 2>&1 && ok "--check clean" || bad "--check clean"
$BINARY lexicon rm "Windows 3.1" >/dev/null 2>&1 && ok "rm entry" || bad "rm entry"
$BINARY lexicon list 2>/dev/null | grep -q "Windows 3.1" && bad "rm removed it" || ok "rm removed it"
# jira-key gating is opt-in: declaring a project key gates PROJ-NNNN, nothing else.
printf '# host-lint: jira-key PROJ\n' > "$LEX/LEXICON"
$BINARY lexicon add "PROJ-1" >/dev/null 2>&1 && bad "declared jira-key needs URL" || ok "declared jira-key needs URL"
$BINARY lexicon add "PROJ-1" --url https://jira.example/PROJ-1 >/dev/null 2>&1 && ok "declared jira-key with URL" || bad "declared jira-key with URL"
$BINARY lexicon add "RFC-2119" >/dev/null 2>&1 && ok "undeclared key stays vocab" || bad "undeclared key stays vocab"
# Strict escalation: the directive turns an undeclared warn-tier code into a block.
printf '# host-lint: strict\nDecision 2.1\n' > "$LEX/LEXICON"
printf 'see Decision 2.1 here' | $BINARY --stdin >/dev/null 2>&1 && rc=0 || rc=$?
[ "$rc" -eq 0 ] && ok "strict: declared phrase clears" || bad "strict: declared phrase clears (rc=$rc)"
printf 'see Decision 2.4 here' | $BINARY --stdin >/dev/null 2>&1 && rc=0 || rc=$?
[ "$rc" -eq 1 ] && ok "strict: undeclared code escalates to flag" || bad "strict: escalation (rc=$rc)"
printf 'Decision 2.1\n' > "$LEX/LEXICON"   # same vocab, no strict directive
printf 'see Decision 2.4 here' | $BINARY --stdin >/dev/null 2>&1 && rc=0 || rc=$?
[ "$rc" -eq 3 ] && ok "non-strict: undeclared code only warns" || bad "non-strict warn (rc=$rc)"
unset GIT_DIR
rm -rf "$LEX"

# --- Summary ---
echo ""
echo "=== Results ==="
echo "Passed: $PASS / $TOTAL"
echo "Failed: $FAIL / $TOTAL"

if [ $FAIL -gt 0 ]; then
    exit 1
fi
