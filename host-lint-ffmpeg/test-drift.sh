#!/usr/bin/env bash
# The upstream-drift lane's own test (host-lint#22, upstream-drift-lane).
#
# The lane's contract has three parts and each is exercised here: a doctored source
# fails, acknowledging the new digest passes, and drift in a section that states no
# rules never gates. The acknowledgement step is deliberately manual in the lane
# itself (it is a corpus edit), so this test simulates it by pointing the checker at
# a tree it already acknowledges.

set -uo pipefail
PACK=${1:?usage: test-drift.sh <host-lint-ffmpeg binary> <clean-ffmpeg-tree>}
TREE=${2:?usage: test-drift.sh <host-lint-ffmpeg binary> <clean-ffmpeg-tree>}
pass=0; fail=0
ok()  { pass=$((pass+1)); echo "  PASS: $1"; }
bad() { fail=$((fail+1)); echo "  FAIL: $1"; }
# `A && ok || bad` reads as if-then-else and is not: a non-zero ok would run bad too.
want() { if [ "$1" -eq "$2" ]; then ok "$3"; else bad "$3 (rc=$1)"; fi; }

work=$(mktemp -d); trap 'rm -rf "$work"' EXIT
cp -r "$TREE"/. "$work/"

echo "--- upstream drift lane ---"

"$PACK" rules --verify-source "$work" >/dev/null 2>&1 && rc=0 || rc=$?
want "$rc" 0 "an acknowledged tree passes"

# Doctor each rule-bearing source in turn.
for f in doc/mailing-list-faq.texi doc/fate.texi MAINTAINERS; do
    cp "$TREE/$f" "$work/$f"
    printf '\n# doctored\n' >> "$work/$f"
    "$PACK" rules --verify-source "$work" >/dev/null 2>&1 && rc=0 || rc=$?
    want "$rc" 1 "a doctored $f fails the lane"
    cp "$TREE/$f" "$work/$f"
done

# developer.texi is split, so doctoring must be localised to a rule-bearing section.
python3 - "$work/doc/developer.texi" <<'PY'
import sys, pathlib
p = pathlib.Path(sys.argv[1]); t = p.read_text()
p.write_text(t.replace("@subheading Commit messages", "@subheading Commit messages (doctored)"))
PY
"$PACK" rules --verify-source "$work" >/dev/null 2>&1 && rc=0 || rc=$?
want "$rc" 1 "a doctored rule-bearing section fails the lane"

# Restoring it is what acknowledgement looks like from the checker's side: the
# digest it compares against once again matches the bytes.
cp "$TREE/doc/developer.texi" "$work/doc/developer.texi"
"$PACK" rules --verify-source "$work" >/dev/null 2>&1 && rc=0 || rc=$?
want "$rc" 0 "acknowledging the digest passes the lane again"

# Drift outside a rule-bearing section is reported and must not gate, or the lane
# cries wolf on every unrelated upstream commit until nobody reads it.
python3 - "$work/doc/developer.texi" <<'PY'
import sys, pathlib
p = pathlib.Path(sys.argv[1]); t = p.read_text()
i = t.find("@subsection Vim configuration"); j = t.find("@subsection Emacs configuration")
p.write_text(t[:i] + t[i:j].replace("set ", "set  ", 1) + t[j:])
PY
out=$("$PACK" rules --verify-source "$work" 2>&1); rc=$?
want "$rc" 0 "drift in a section that states no rules does not gate"
case $out in
    *"states no rules"*) ok "the non-gating drift is still reported" ;;
    *) bad "non-gating drift not reported" ;;
esac

echo ""
echo "$pass passed, $fail failed"
[ "$fail" -eq 0 ]
