#!/usr/bin/env bash
# Regenerate RULES.md from src/rules.rs, so the human form and the table the checker
# reads cannot drift apart. Run after any change to the registry and commit the diff.
#
# Regenerating and finding no diff is also how RULES.md is verified: CI runs this and
# fails if the committed file is not what the registry produces.
set -euo pipefail
cd "$(dirname "$0")"

commit=$(grep 'UPSTREAM_COMMIT: &str' src/rules.rs | sed 's/.*"\(.*\)".*/\1/')

{
cat <<'HEAD'
# The FFmpeg commit rules, as this pack encodes them

This is the human form of the corpus in `src/rules.rs`, generated from the same table
the checker reads so the two cannot disagree. Regenerate with `./make-rules-doc.sh`
after any change to the registry.

Read the tier column first, because it is the only thing that says how much a finding
is worth.

- **mechanical**: decidable from the artefact with no judgement. Blocks, in a project
  that asked for `enforce`.
- **heuristic**: decidable with a false-positive rate. Reports; never blocks. The
  measured rate is the fraction of accepted upstream history the rule leaves alone.
- **attested**: no artefact answers it. A human does, and the checklist can never
  render it as checked.

Every rule below traces to a named section of a pinned upstream source. Conventions
this pack enforces that upstream does *not* document are listed separately at the end,
because mixing them into the corpus would put this pack's opinions behind FFmpeg's
authority.

## The corpus
HEAD

echo
echo "Pinned at FFmpeg master \`$commit\`."
echo
echo "| Rule | Tier | Lane | Measured | Upstream section |"
echo "|---|---|---|---|---|"
python3 - <<'PY'
import re, pathlib
t = pathlib.Path("src/rules.rs").read_text()
body = t[t.index("pub const RULES: &[Rule] = &["):]
body = body[:body.index("\n];")]
pat = (r'Rule \{ id: "([^"]+)", section: "([^"]+)", subheading: "([^"]*)",\s*'
       r'tier: Tier::(\w+), lane: Lane::(\w+), measured_rate: (None|Some\(([\d.]+)\)),')
for m in re.finditer(pat, body):
    rid, sec, sub, tier, lane, _mr, rate = m.groups()
    rate_s = f"{float(rate):.3f}" if rate else "not measured"
    sub_s = "" if sub == "(prose)" else f" / {sub}"
    print(f"| `{rid}` | {tier.lower()} | {lane.lower()} | {rate_s} | {sec}{sub_s} |")
PY

echo
echo "## Project conventions, which upstream does not document"
echo
echo "| Rule | Why it is not upstream's |"
echo "|---|---|"
python3 - <<'PY'
import re, pathlib
t = pathlib.Path("src/rules.rs").read_text()
body = t[t.index("pub const PROJECT_RULES"):]
body = body[:body.index("\n];")]
for rid, why in re.findall(r'\("([^"]+)", "([^"]+)"\)', body):
    print(f"| `{rid}` | {why} |")
PY

cat <<'TAIL'

## What the tiers cost

Three of these tiers were set by measurement and not by intent, and the record is in
[CALIBRATION.md](CALIBRATION.md). The short version:

- The area-prefix rule flags five of every five hundred accepted subjects, so it
  cannot block. A mechanical tier there would have rejected upstream's own work about
  once in a hundred commits.
- The ascii rule is scoped to code, not comments, because comments are exactly where
  upstream uses non-ascii. A comment-scoped rule would have flagged ten of the eleven
  non-ascii lines in the sample.
- The cosmetic/functional classifier is judged per hunk rather than per diff. Judged
  per diff it reported 108 of 300 accepted commits as mixed.

A rate of 1.000 on a heuristic rule means the corpus did not exercise it. It is not a
promotion to mechanical.
TAIL
} > RULES.md

echo "wrote RULES.md ($(grep -c '^| `' RULES.md) rule rows)"
