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

# Assert the exact exit code of a --stdin scan, so a flag (1) -> warn (3) regression
# is caught rather than passing on any nonzero (plan/0055).
flag()  { local rc; printf '%s\n' "$1" | $BINARY --stdin >/dev/null 2>&1 && rc=0 || rc=$?; [ "$rc" -eq 1 ] && ok "flag: $1"  || bad "flag (want rc=1, got $rc): $1"; }
warn()  { local rc; printf '%s\n' "$1" | $BINARY --stdin >/dev/null 2>&1 && rc=0 || rc=$?; [ "$rc" -eq 3 ] && ok "warn: $1"  || bad "warn (want rc=3, got $rc): $1"; }
clean() { local rc; printf '%s\n' "$1" | $BINARY --stdin >/dev/null 2>&1 && rc=0 || rc=$?; [ "$rc" -eq 0 ] && ok "clean: $1" || bad "clean (want rc=0, got $rc): $1"; }

echo "=== Integration Tests ==="
echo "Binary: $BINARY"
echo ""

# --- Should flag (expect exit 1, blocking) ---
echo "--- Should flag (expect rc=1) ---"

flag "## Phase 1: Setup"
flag "feat: phase 2 of auth refactor"
flag "Stage 2, data migration"
flag "Iteration 5: optimize"
flag "Wave 2 of rollout"
flag "Sprint 3 backlog"

# Internal code-as-name tell (VOCABULARY.md, internal tracking codes)
flag "ci: fix the no-OS-comm guard's fail-open nm regex (review B1)"
flag "addresses finding #7"
flag "blocker B2 resolved"
flag "addresses review (B1)"

# Positional checklist-item references (host#16): box/boxes/steps + numeral or range
flag "plan/0001: box 7 [x] (deploy path landed)"
flag "plan/0001 boxes 4-8 blocked"
flag "box 3 root cause localized"
flag "plan steps 3-5 updated"

# --- Should warn (expect exit 3, advisory) — the corpus-grounded demotion (plan/0055) ---
echo "--- Should warn (expect rc=3) ---"

warn "// Pass 1: tokenize"
warn "Round 1 of review"
warn "Batch 3 processing"
warn "see Part 1 of the file"
warn "see section 3 of the spec"
warn "train for epoch 0 then stop"
warn "step 3-5 closed"        # singular step is advisory; the range still warns

# --- Roman / verb false flags that must NOT block (plan/0055) ---
echo "--- Blocking-tier false flags now clean (expect rc=0 or rc=3, never rc=1) ---"
clean "in this pass I fixed the parser bug"   # pronoun "I" is not a Roman numeral
clean "port the lexer to C"                   # single language letter
clean "step into 3 dimensions of design"      # numeral two words away
clean "phase iv intravenous line"             # lowercase roman: not a label
clean "phase DC offset rejection"             # EE abbreviation, phase's home domain
clean "wave XL of the rollout"                # size abbreviation
clean "boxes MM apart on the board"           # millimetres
clean "wave 12-07 release"                    # date, not an ascending checklist range

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

# host#16 boundaries: the literal checklist mark, the disposition verb, and a
# content-named reference all stay clean (only the positional citation flags).
echo "- [x] deploy path landed" | $BINARY --stdin >/dev/null 2>&1 && ok "- [x] literal mark" || bad "- [x] literal mark"
echo "1. [x] native MSVC build verified" | $BINARY --stdin >/dev/null 2>&1 && ok "1. [x] ordered mark" || bad "1. [x] ordered mark"
echo "box an irreducible citation in a fence" | $BINARY --stdin >/dev/null 2>&1 && ok "box (verb)" || bad "box (verb)"
echo "the deploy-path box landed" | $BINARY --stdin >/dev/null 2>&1 && ok "content-named box" || bad "content-named box"
echo "what is in the box" | $BINARY --stdin >/dev/null 2>&1 && ok "box (no numeral)" || bad "box (no numeral)"

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

# --- host-lint#23: an explicit file argument that cannot be scanned fails closed ---
echo ""
echo "--- Explicit file args fail closed (host-lint#23) ---"
$BINARY "$tmpdir/definitely-missing.md" >/dev/null 2>&1 && rc=0 || rc=$?
[ "$rc" -eq 2 ] && ok "missing file arg exits 2" || bad "missing file arg exits 2 (rc=$rc)"
out=$($BINARY "$tmpdir/definitely-missing.md" 2>&1) || true
echo "$out" | grep -q "definitely-missing.md" && ok "diagnostic names the path" || bad "diagnostic names the path (got: $out)"
mkdir -p "$tmpdir/adir"
$BINARY "$tmpdir/adir" >/dev/null 2>&1 && rc=0 || rc=$?
[ "$rc" -eq 2 ] && ok "directory arg exits 2" || bad "directory arg exits 2 (rc=$rc)"
# The deliberate skips stay policy, not errors: an unscannable extension passes.
printf 'not text\n' > "$tmpdir/pic.png"
$BINARY "$tmpdir/pic.png" >/dev/null 2>&1 && rc=0 || rc=$?
[ "$rc" -eq 0 ] && ok "unscannable extension stays a policy skip" || bad "unscannable extension policy skip (rc=$rc)"

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

# --- flag tier: decimal numerals and label prefix (expect flag, rc=1) ---
echo ""
echo "--- Decimal numerals + label prefix (expect flag) ---"
for s in 'entry point (Phase 5.0).' '5.5: exec/pty tools' '// 5.5: the pty exec tool' '## 5.5: error handling'; do
    printf '%s' "$s" | $BINARY --stdin >/dev/null 2>&1 && rc=0 || rc=$?
    [ "$rc" -eq 1 ] && ok "flag: $s" || bad "flag: $s (rc=$rc)"
done

# --- warn tier: bare-numeral degenerate form (expect warn, rc=3) ---
echo ""
echo "--- Bare-numeral degenerate form (expect warn) ---"
for s in 'as decided in 2.1' 'exec tools (5.5)' 'the peek/poke tools arrive in 5.3' 'implements work-item 5.3' 'section 2.1 of the spec'; do
    printf '%s' "$s" | $BINARY --stdin >/dev/null 2>&1 && rc=0 || rc=$?
    [ "$rc" -eq 3 ] && ok "warn: $s" || bad "warn: $s (rc=$rc)"
done

# --- warn tier: leading code-as-name label (expect warn, rc=3) ---
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

# --- issue #16: the prose lane consults the LEXICON ---
echo ""
echo "--- prose LEXICON (issue #16) ---"
plx=$(mktemp -d)
printf 'The rehost harness logs to disk; a second harness runs nightly.\n' > "$plx/doc.md"
# No LEXICON: both harness occurrences flag as ai-diction (exit 3).
out=$(cd "$plx" && "$BINARY_ABS" --prose doc.md 2>&1) && rc=0 || rc=$?
n=$(printf '%s\n' "$out" | grep -c 'harness')
{ [ "$rc" -eq 3 ] && [ "$n" -eq 2 ]; } && ok "prose: harness flags twice with no LEXICON" || bad "prose no-LEXICON (rc=$rc n=$n)"
# Declare the phrase: the occurrence inside it is masked, the standalone still flags.
printf 'rehost harness\n' > "$plx/LEXICON"
out=$(cd "$plx" && "$BINARY_ABS" --prose doc.md 2>&1) && rc=0 || rc=$?
n=$(printf '%s\n' "$out" | grep -c 'harness')
{ [ "$rc" -eq 3 ] && [ "$n" -eq 1 ]; } && ok "prose: declared phrase masked, standalone still flags" || bad "prose LEXICON mask (rc=$rc n=$n)"
rm -rf "$plx"

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

# --- host-lint#20: --prose (no args) and --all audit tracked docs, not silently clean ---
echo ""
echo "--- #20: --prose no-args and --all audit tracked docs ---"
issue20=$(mktemp -d)
git -C "$issue20" init -q
printf '# Doc\n\nThe default becomes 0xFC, replacing 0xD8.\n' > "$issue20/d.md" # sentence-final ing-tail
git -C "$issue20" add -A
git -C "$issue20" -c user.name=t -c user.email=t@t commit -q -m "init"
# --prose with no file args audits the tracked docs (matching host-lifecycle prose),
# neither a silent clean nor an error.
out=$(cd "$issue20" && "$BINARY_ABS" --prose 2>&1) && rc=0 || rc=$?
{ [ "$rc" -eq 3 ] && echo "$out" | grep -q "d.md"; } && ok "--prose (no args) audits tracked docs" || bad "--prose (no args) audits tracked docs (rc=$rc)"
# --all is the comprehensive audit: the naming lane AND the prose lane over the tracked set.
out=$(cd "$issue20" && "$BINARY_ABS" --all 2>&1) && rc=0 || rc=$?
{ [ "$rc" -eq 3 ] && echo "$out" | grep -q "d.md"; } && ok "--all runs the prose lane too" || bad "--all runs the prose lane too (rc=$rc)"
# --all --prose behaves like --all rather than erroring.
out=$(cd "$issue20" && "$BINARY_ABS" --all --prose 2>&1) && rc=0 || rc=$?
{ [ "$rc" -eq 3 ] && echo "$out" | grep -q "d.md"; } && ok "--all --prose audits (not an error)" || bad "--all --prose audits (rc=$rc)"
rm -rf "$issue20"

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
$BINARY lexicon add "5.5" >/dev/null 2>&1 && bad "master-key guard rejects a bare numeral" || ok "master-key guard rejects a bare numeral"
$BINARY lexicon add "Phase 5.5" >/dev/null 2>&1 && bad "no-laundering guard rejects a tell" || ok "no-laundering guard rejects a tell"
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

# --- host-lint:ignore naming fence (plan/0055) ---
echo ""
echo "--- host-lint:ignore naming fence ---"
fence_dir="$tmpdir/fence"; mkdir -p "$fence_dir"
printf 'intro line\n```host-lint:ignore\ncited Phase 1 reference\n```\nclean tail\n' > "$fence_dir/ok.md"
$BINARY "$fence_dir/ok.md" >/dev/null 2>&1 && ok "ignore-fence: quarantined tell is clean" || bad "ignore-fence: quarantined tell should be clean (rc=$?)"
printf 'intro line\nPhase 1 inline tell here\n' > "$fence_dir/inline.md"
$BINARY "$fence_dir/inline.md" >/dev/null 2>&1 && rc=0 || rc=$?
[ "$rc" -eq 1 ] && ok "ignore-fence: inline tell still flags" || bad "ignore-fence: inline tell should flag (rc=$rc)"
printf '```python\nPhase 1 in a code fence\n```\n' > "$fence_dir/codefence.md"
$BINARY "$fence_dir/codefence.md" >/dev/null 2>&1 && rc=0 || rc=$?
[ "$rc" -eq 1 ] && ok "ignore-fence: a plain code fence stays linted" || bad "ignore-fence: a plain code fence should stay linted (rc=$rc)"
printf 'intro\n```host-lint:ignore\nPhase 2 ships here\n' > "$fence_dir/unclosed.md"
$BINARY "$fence_dir/unclosed.md" >/dev/null 2>&1 && rc=0 || rc=$?
[ "$rc" -eq 1 ] && ok "ignore-fence: unclosed fence fails loud" || bad "ignore-fence: unclosed fence should fail loud (rc=$rc)"

# --- pre-commit hook end-to-end (plan/0055): scans the staged blob, fails closed ---
echo ""
echo "--- pre-commit hook end-to-end ---"
HOOK_SCRIPT="$(cd "$(dirname "$0")" && pwd)/pre-commit"
if [ -f "$HOOK_SCRIPT" ]; then
    hookrepo="$tmpdir/hook-repo"
    git init -q "$hookrepo"
    cp "$HOOK_SCRIPT" "$hookrepo/.git/hooks/pre-commit"
    cp "$BINARY_ABS" "$hookrepo/.git/hooks/host-lint"
    chmod +x "$hookrepo/.git/hooks/pre-commit" "$hookrepo/.git/hooks/host-lint"
    # a clean file commits
    printf 'ordinary content\n' > "$hookrepo/a.txt"
    (cd "$hookrepo" && git add a.txt && git -c user.name=t -c user.email=t@t commit -q -m "add a") \
        && ok "hook: a clean staged file commits" || bad "hook: a clean staged file should commit"
    # a staged tell is blocked (the staged blob, not the working tree)
    printf '## Phase 1: setup\n' > "$hookrepo/plan.md"
    (cd "$hookrepo" && git add plan.md && git -c user.name=t -c user.email=t@t commit -q -m "add plan" 2>/dev/null) \
        && bad "hook: a staged tell should block the commit" || ok "hook: a staged tell blocks the commit"
    # the staged-blob property: stage the tell, then edit it out of the working tree
    # unstaged — the hook must still block on the staged bytes (plan/0055)
    (cd "$hookrepo" && printf 'clean now\n' > plan.md \
        && git -c user.name=t -c user.email=t@t commit -q -m "commit staged tell" 2>/dev/null) \
        && bad "hook: must lint the staged blob, not the clean working tree" || ok "hook: lints the staged blob, not the working tree"
    # a tell in a NON-ASCII-named staged file must block: core.quotePath C-quotes
    # such a path, which (without -z + pipefail) fed empty stdin and committed
    # the tell unseen (plan/0055 cast review). Fresh repo to isolate the index.
    nrepo="$tmpdir/hook-nonascii"
    git init -q "$nrepo"
    cp "$HOOK_SCRIPT" "$nrepo/.git/hooks/pre-commit"; cp "$BINARY_ABS" "$nrepo/.git/hooks/host-lint"
    chmod +x "$nrepo/.git/hooks/pre-commit" "$nrepo/.git/hooks/host-lint"
    printf '## Phase 1: setup\n' > "$nrepo/café.md"
    (cd "$nrepo" && git add "café.md" && git -c user.name=t -c user.email=t@t commit -q -m "add café" 2>/dev/null) \
        && bad "hook: a tell in a non-ASCII-named file must block" || ok "hook: a non-ASCII-named staged tell blocks"
    # a staged submodule gitlink (mode 160000) carries no blob to lint; the hook must
    # skip it and commit rather than fail closed on `git show`'s exit 128 (host-lint#18)
    grepo="$tmpdir/hook-gitlink"
    git init -q "$grepo"
    cp "$HOOK_SCRIPT" "$grepo/.git/hooks/pre-commit"; cp "$BINARY_ABS" "$grepo/.git/hooks/host-lint"
    chmod +x "$grepo/.git/hooks/pre-commit" "$grepo/.git/hooks/host-lint"
    (cd "$grepo" && git update-index --add --cacheinfo 160000,1111111111111111111111111111111111111111,sub \
        && git -c user.name=t -c user.email=t@t commit -q -m "add submodule") \
        && ok "hook: a staged gitlink commits, not fail-closed" || bad "hook: a staged gitlink must not fail the commit closed"
    # fail-closed preserved: a real tell staged alongside a gitlink still blocks
    gtrepo="$tmpdir/hook-gitlink-tell"
    git init -q "$gtrepo"
    cp "$HOOK_SCRIPT" "$gtrepo/.git/hooks/pre-commit"; cp "$BINARY_ABS" "$gtrepo/.git/hooks/host-lint"
    chmod +x "$gtrepo/.git/hooks/pre-commit" "$gtrepo/.git/hooks/host-lint"
    printf '## Phase 1: setup\n' > "$gtrepo/plan.md"
    (cd "$gtrepo" && git update-index --add --cacheinfo 160000,1111111111111111111111111111111111111111,sub \
        && git add plan.md \
        && git -c user.name=t -c user.email=t@t commit -q -m "gitlink plus tell" 2>/dev/null) \
        && bad "hook: a tell alongside a gitlink must still block" || ok "hook: a tell alongside a gitlink still blocks"
else
    bad "hook: pre-commit script not found at $HOOK_SCRIPT"
fi

# --- pack dispatch: the reserved verb (host-lint#22 revised sequence) ---
echo ""
echo "--- pack dispatch ---"
packdir="$tmpdir/packs"
mkdir -p "$packdir"
cat > "$packdir/host-lint-fake" <<'EOF'
#!/bin/sh
echo "fake-pack args=$* HOST_LINT_VERSION=$HOST_LINT_VERSION"
exit "${FAKE_RC:-0}"
EOF
chmod +x "$packdir/host-lint-fake"
out=$(PATH="$packdir:$PATH" "$BINARY_ABS" pack fake alpha beta 2>&1) && rc=0 || rc=$?
{ [ "$rc" -eq 0 ] && echo "$out" | grep -q 'args=alpha beta'; } && ok "pack: args pass through" || bad "pack: args pass through (rc=$rc: $out)"
echo "$out" | grep -qE 'HOST_LINT_VERSION=[0-9]+\.[0-9]+\.[0-9]+' && ok "pack: HOST_LINT_VERSION exported" || bad "pack: HOST_LINT_VERSION exported ($out)"
for want in 1 3; do
    FAKE_RC=$want PATH="$packdir:$PATH" "$BINARY_ABS" pack fake >/dev/null 2>&1 && rc=0 || rc=$?
    [ "$rc" -eq "$want" ] && ok "pack: exit $want passes through" || bad "pack: exit $want passes through (rc=$rc)"
done
out=$(PATH="$packdir:$PATH" "$BINARY_ABS" pack nosuchpack 2>&1) && rc=0 || rc=$?
{ [ "$rc" -eq 2 ] && echo "$out" | grep -q 'host-lint-nosuchpack'; } && ok "pack: missing pack exits 2 with install hint" || bad "pack: missing pack (rc=$rc: $out)"
"$BINARY_ABS" pack >/dev/null 2>&1 && rc=0 || rc=$?
[ "$rc" -eq 2 ] && ok "pack: no name is a usage error" || bad "pack: no name usage error (rc=$rc)"
# The collision the reserved verb exists for (host-lint#23): a name like `fake`
# is a pack to the verb and a plain file argument to the bare CLI.
printf 'plain text\n' > "$packdir/fake"
(cd "$packdir" && PATH="$packdir:$PATH" "$BINARY_ABS" pack fake 2>&1 | grep -q 'fake-pack') && ok "pack: verb dispatches despite a same-named file" || bad "pack: verb dispatches despite a same-named file"
(cd "$packdir" && "$BINARY_ABS" fake >/dev/null 2>&1) && ok "pack: bare name stays a file argument" || bad "pack: bare name stays a file argument"
PATH="$packdir:$PATH" "$BINARY_ABS" packs 2>/dev/null | grep -qx 'fake' && ok "packs: lists installed packs" || bad "packs: lists installed packs"

# --- ffmpeg pack skeleton: dispatch + engine handshake (host-lint#22) ---
echo ""
echo "--- ffmpeg pack skeleton ---"
# The pack binary sits beside the core locally (target/release) and under a
# platform-suffixed asset name in CI's download dir; take either.
PACK_SRC=$(ls "$(dirname "$BINARY_ABS")"/host-lint-ffmpeg* 2>/dev/null | head -1)
if [ -n "$PACK_SRC" ]; then
    pdir="$tmpdir/ffmpeg-pack"
    mkdir -p "$pdir"
    cp "$BINARY_ABS" "$pdir/host-lint"
    cp "$PACK_SRC" "$pdir/host-lint-ffmpeg"
    chmod +x "$pdir/host-lint" "$pdir/host-lint-ffmpeg"
    # Dispatched through the core the versions match, so the skeleton reaches
    # its usage error (exit 2, no lanes yet), never a hollow clean exit.
    out=$("$pdir/host-lint" pack ffmpeg 2>&1) && rc=0 || rc=$?
    { [ "$rc" -eq 2 ] && echo "$out" | grep -q 'no lanes are implemented'; } && ok "pack ffmpeg: dispatch reaches the skeleton" || bad "pack ffmpeg: dispatch (rc=$rc: $out)"
    # A skewed core version refuses to run (host-lint#23, strict handshake).
    out=$(HOST_LINT_VERSION=999.0.0 "$pdir/host-lint-ffmpeg" 2>&1) && rc=0 || rc=$?
    { [ "$rc" -eq 2 ] && echo "$out" | grep -q 'skew'; } && ok "pack ffmpeg: version skew refuses" || bad "pack ffmpeg: skew (rc=$rc: $out)"
    "$pdir/host-lint" packs 2>/dev/null | grep -qx 'ffmpeg' && ok "packs: lists the sibling ffmpeg pack" || bad "packs: lists the sibling ffmpeg pack"
else
    bad "ffmpeg pack binary not found beside $BINARY_ABS"
fi

# --- repo_root under a linked-worktree GIT_DIR (host-lint#25) ---
echo ""
echo "--- repo_root under a linked-worktree GIT_DIR ---"
# hooks in a linked worktree run with GIT_DIR=<store>/worktrees/<name>; the root
# (and with it .host-lintignore and the LEXICON) must resolve to the worktree,
# not to the store
wtbase="$tmpdir/wt-base"
git init -q "$wtbase"
(cd "$wtbase" && printf 'seed\n' > seed.txt && git add seed.txt \
    && git -c user.name=t -c user.email=t@t commit -q -m "seed")
git -C "$wtbase" worktree add -q "$tmpdir/wt-linked" >/dev/null 2>&1
printf 'fixtures.md\n' > "$tmpdir/wt-linked/.host-lintignore"
printf '## Phase 1: setup\n' > "$tmpdir/wt-linked/fixtures.md"
wt_gitdir=$(git -C "$tmpdir/wt-linked" rev-parse --absolute-git-dir)
(cd "$tmpdir/wt-linked" && GIT_DIR="$wt_gitdir" "$BINARY_ABS" --stdin-as fixtures.md < fixtures.md >/dev/null 2>&1) \
    && ok "worktree GIT_DIR: ignored fixture stays clean" || bad "worktree GIT_DIR: ignore list must load from the worktree root (rc=$?)"
# the same tell in a non-ignored file still flags under the worktree GIT_DIR
printf '## Phase 1: setup\n' > "$tmpdir/wt-linked/plan.md"
(cd "$tmpdir/wt-linked" && GIT_DIR="$wt_gitdir" "$BINARY_ABS" --stdin-as plan.md < plan.md >/dev/null 2>&1) && rc=0 || rc=$?
[ "$rc" -eq 1 ] && ok "worktree GIT_DIR: a real tell still flags" || bad "worktree GIT_DIR: a real tell should flag (rc=$rc)"
# the normal-repo hook environment keeps its existing resolution
(cd "$wtbase" && printf 'seed.txt\n' > .host-lintignore && printf '## Phase 1: setup\n' > seed.txt \
    && GIT_DIR="$wtbase/.git" "$BINARY_ABS" --stdin-as seed.txt < seed.txt >/dev/null 2>&1) \
    && ok "plain GIT_DIR: ignore still honored" || bad "plain GIT_DIR: ignore should still be honored (rc=$?)"

# --- Summary ---
echo ""
echo "=== Results ==="
echo "Passed: $PASS / $TOTAL"
echo "Failed: $FAIL / $TOTAL"

if [ $FAIL -gt 0 ]; then
    exit 1
fi
