#!/bin/bash
# lint-skill.sh — mechanical conformance gates for SKILL.md, each named after what
# it checks: a bare letter-and-number gate label is itself the ordinal tell host-lint
# detects, so the gates are named by content instead.
# Exit 0 when every gate succeeds; non-zero on any failure.

set -e

SKILL_MD="${1:-SKILL.md}"
FAIL=0

pass() { printf '  PASS %s\n' "$1"; }
fail() { echo "  FAIL: $1"; FAIL=1; }

echo "=== Conformance Gates ==="
echo "Target: $SKILL_MD"
echo ""

# frontmatter-parseable — present and valid YAML
echo "frontmatter-parseable"
if head -1 "$SKILL_MD" | grep -q '^---$'; then
    # Extract frontmatter block
    fm=$(awk '/^---$/{n++; if(n==2){exit}} n==1{print}' "$SKILL_MD")
    if echo "$fm" | python3 -c "import sys,yaml; yaml.safe_load(sys.stdin)" 2>/dev/null; then
        pass "frontmatter-parseable"
    else
        fail "frontmatter-parseable: frontmatter does not parse as YAML"
    fi
else
    fail "frontmatter-parseable: file does not begin with ---"
fi

# required-keys — name and description present
echo "required-keys (name, description)"
name=$(echo "$fm" | python3 -c "import sys,yaml; d=yaml.safe_load(sys.stdin); print(d.get('name',''))" 2>/dev/null)
desc=$(echo "$fm" | python3 -c "import sys,yaml; d=yaml.safe_load(sys.stdin); print(d.get('description',''))" 2>/dev/null)
if [ -n "$name" ] && [ -n "$desc" ]; then
    pass "required-keys"
else
    fail "required-keys: missing or empty name/description"
fi

# name-format — a slug that matches the skill's directory
echo "name-format"
skill_parent=$(cd "$(dirname "$SKILL_MD")" && pwd)
dirname=$(basename "$skill_parent")
# A materialized worktree lives at software/<name>/main/ (the reproducibility-anchor
# layout), so the directory holding SKILL.md is the worktree line "main", not the
# skill name. Use the parent directory's name in that case, so the gate passes in
# the worktree exactly as it does on an installed skill (.claude/skills/<name>/) or
# a repo-named CI checkout. A bare "main" elsewhere is not a skill home, so this only
# affects the materialized layout.
if [ "$dirname" = "main" ]; then
    dirname=$(basename "$(dirname "$skill_parent")")
fi
if echo "$name" | grep -qE '^[a-z0-9]+(-[a-z0-9]+)*$'; then
    if [ ${#name} -le 64 ]; then
        if [ "$name" = "$dirname" ]; then
            pass "name-format"
        else
            fail "name-format: name '$name' != dirname '$dirname'"
        fi
    else
        fail "name-format: name exceeds 64 chars"
    fi
else
    fail "name-format: name does not match ^[a-z0-9]+(-[a-z0-9]+)*$"
fi

# description-length — within the portable limit
echo "description-length"
if [ ${#desc} -le 1024 ]; then
    pass "description-length"
else
    fail "description-length: description exceeds 1024 chars"
fi

# portable-frontmatter — only the portable keys, or extras documented
echo "portable-frontmatter"
keys=$(echo "$fm" | python3 -c "import sys,yaml; d=yaml.safe_load(sys.stdin); print(' '.join(d.keys()))" 2>/dev/null)
extra=""
for k in $keys; do
    if [ "$k" != "name" ] && [ "$k" != "description" ]; then
        extra="$extra $k"
    fi
done
if [ -z "$extra" ]; then
    pass "portable-frontmatter"
else
    if grep -q '# Portability notes' "$SKILL_MD"; then
        pass "portable-frontmatter (extra keys $extra documented in Portability notes)"
    else
        fail "portable-frontmatter: non-portable keys:$extra (no Portability notes section)"
    fi
fi

# body-length — within the readable limit
echo "body-length"
body_lines=$(awk '/^---$/{n++; if(n==2){found=1; next}} found{print}' "$SKILL_MD" | wc -l)
if [ "$body_lines" -le 500 ]; then
    pass "body-length ($body_lines lines)"
else
    fail "body-length: body exceeds 500 lines ($body_lines)"
fi

# references-resolve — every local file reference exists
echo "references-resolve"
dangling=0
skill_dir=$(dirname "$SKILL_MD")
body=$(awk '/^---$/{n++; if(n==2){found=1; next}} found{print}' "$SKILL_MD")
body_no_code=$(awk 'BEGIN{in_code=0} /^`{3,}/{in_code=!in_code; next} !in_code' <<< "$body")
refs=$(grep -oE '\b[a-zA-Z_][a-zA-Z0-9_.-]+/[a-zA-Z0-9_.-]+' <<< "$body_no_code" 2>/dev/null || true)
refs2=$(grep -oE '\b[a-zA-Z_][a-zA-Z0-9_.-]+\.(md|yaml|yml|toml|py|sh|rs)$' <<< "$body_no_code" 2>/dev/null || true)
all_refs="$refs $refs2"
for ref in $all_refs; do
    # Skip URLs
    if echo "$ref" | grep -qE '^https?://'; then
        continue
    fi
    if [ ! -e "$skill_dir/$ref" ] && [ ! -e "$ref" ]; then
        # Check if it's in the same directory
        found=0
        for f in "$skill_dir"/*; do
            if [ "$(basename "$f")" = "$ref" ]; then
                found=1
                break
            fi
        done
        if [ $found -eq 0 ]; then
            # Only flag if it looks like a local file reference (contains .)
            if echo "$ref" | grep -q '\.'; then
                fail "references-resolve: dangling reference '$ref'"
                dangling=1
            fi
        fi
    fi
done
if [ $dangling -eq 0 ]; then
    pass "references-resolve"
fi

# imperative-density — every MUST/ALWAYS/NEVER carries a causal reason
echo "imperative-density"
imperatives=$(grep -oiE '\b(MUST|ALWAYS|NEVER)\b' "$SKILL_MD" 2>/dev/null || true)
if [ -z "$imperatives" ]; then
    pass "imperative-density (no imperatives)"
else
    # Check each imperative has a causal marker in the same paragraph
    bad=0
    for imp in $imperatives; do
        # Find the line containing this imperative
        line=$(grep -i "$imp" "$SKILL_MD" | head -1)
        if echo "$line" | grep -qiE 'because|so|since|therefore|thus|otherwise|or'; then
            : # has causal marker
        else
            bad=1
        fi
    done
    if [ $bad -eq 0 ]; then
        pass "imperative-density"
    else
        fail "imperative-density: imperative without causal reason"
    fi
fi

echo ""
echo "=== Result ==="
if [ $FAIL -eq 0 ]; then
    echo "ALL GATES PASSED"
    exit 0
else
    echo "GATES FAILED"
    exit 1
fi
