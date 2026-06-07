#!/bin/bash
# lint-skill.sh — mechanical conformance gates G1-G8 for SKILL.md
# Exit 0 if all pass, 1 if any fail.

set -e

SKILL_MD="${1:-SKILL.md}"
FAIL=0

pass() { echo "  PASS: $1"; }
fail() { echo "  FAIL: $1"; FAIL=1; }

echo "=== Conformance Gates ==="
echo "Target: $SKILL_MD"
echo ""

# G1 — Frontmatter present and parseable
echo "G1 — Frontmatter present and parseable"
if head -1 "$SKILL_MD" | grep -q '^---$'; then
    # Extract frontmatter block
    fm=$(awk '/^---$/{n++; if(n==2){exit}} n==1{print}' "$SKILL_MD")
    if echo "$fm" | python3 -c "import sys,yaml; yaml.safe_load(sys.stdin)" 2>/dev/null; then
        pass "G1"
    else
        fail "G1: frontmatter does not parse as YAML"
    fi
else
    fail "G1: file does not begin with ---"
fi

# G2 — Required keys
echo "G2 — Required keys (name, description)"
name=$(echo "$fm" | python3 -c "import sys,yaml; d=yaml.safe_load(sys.stdin); print(d.get('name',''))" 2>/dev/null)
desc=$(echo "$fm" | python3 -c "import sys,yaml; d=yaml.safe_load(sys.stdin); print(d.get('description',''))" 2>/dev/null)
if [ -n "$name" ] && [ -n "$desc" ]; then
    pass "G2"
else
    fail "G2: missing or empty name/description"
fi

# G3 — Name format
echo "G3 — Name format"
dirname=$(basename "$(dirname "$SKILL_MD")")
if [ "$dirname" = "." ]; then
    dirname=$(basename "$(git rev-parse --show-toplevel 2>/dev/null || pwd)")
fi
if echo "$name" | grep -qE '^[a-z0-9]+(-[a-z0-9]+)*$'; then
    if [ ${#name} -le 64 ]; then
        if [ "$name" = "$dirname" ]; then
            pass "G3"
        else
            fail "G3: name '$name' != dirname '$dirname'"
        fi
    else
        fail "G3: name exceeds 64 chars"
    fi
else
    fail "G3: name does not match ^[a-z0-9]+(-[a-z0-9]+)*$"
fi

# G4 — Description length
echo "G4 — Description length"
if [ ${#desc} -le 1024 ]; then
    pass "G4"
else
    fail "G4: description exceeds 1024 chars"
fi

# G5 — Portable frontmatter only
echo "G5 — Portable frontmatter only"
keys=$(echo "$fm" | python3 -c "import sys,yaml; d=yaml.safe_load(sys.stdin); print(' '.join(d.keys()))" 2>/dev/null)
extra=""
for k in $keys; do
    if [ "$k" != "name" ] && [ "$k" != "description" ]; then
        extra="$extra $k"
    fi
done
if [ -z "$extra" ]; then
    pass "G5"
else
    if grep -q '# Portability notes' "$SKILL_MD"; then
        pass "G5 (extra keys $extra documented in Portability notes)"
    else
        fail "G5: non-portable keys:$extra (no Portability notes section)"
    fi
fi

# G6 — Body length
echo "G6 — Body length"
body_lines=$(awk '/^---$/{n++; if(n==2){found=1; next}} found{print}' "$SKILL_MD" | wc -l)
if [ "$body_lines" -le 500 ]; then
    pass "G6 ($body_lines lines)"
else
    fail "G6: body exceeds 500 lines ($body_lines)"
fi

# G7 — References resolve
echo "G7 — References resolve"
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
                fail "G7: dangling reference '$ref'"
                dangling=1
            fi
        fi
    fi
done
if [ $dangling -eq 0 ]; then
    pass "G7"
fi

# G8 — Imperative density
echo "G8 — Imperative density"
imperatives=$(grep -oiE '\b(MUST|ALWAYS|NEVER)\b' "$SKILL_MD" 2>/dev/null || true)
if [ -z "$imperatives" ]; then
    pass "G8 (no imperatives)"
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
        pass "G8"
    else
        fail "G8: imperative without causal reason"
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
