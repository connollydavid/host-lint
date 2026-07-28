# Calibration against accepted upstream history

Every tier in `src/rules.rs` is a claim about how reliably a rule can be judged from
the artifact. This is the measurement behind those claims. It is the reason
`measured_rate` exists and the reason it was empty until now.

## The corpus

| | |
|---|---|
| Repository | `FFmpeg/FFmpeg`, branch `master` |
| Commits | the 300 most recent non-merge commits |
| Newest | `c6309b5c63add7ad0ec221fafefc32bdcd6f8b91`, 2026-07-27 |
| Oldest | `5d6f409cdd65c3943d1b7447524008bba4ac36c9`, 2026-07-05 |
| Added lines | 20,615 |
| Subjects | 500 unique, the five most recent API pages, in `fixtures/upstream/accepted-subjects.txt` |
| Measured | 2026-07-28 |

Accepted commits are the only honest basis for a tier. They passed upstream's own
reviewers, so a rule that flags one is wrong about FFmpeg rather than right about
the commit. A rate below is therefore the fraction of accepted material the rule
leaves alone.

## What the measurement decided

**No mechanical rule fires anywhere in the corpus.** That is the must-pass
condition, asserted by tests in both lanes, and it is what makes a blocking tier
defensible at all.

Three findings changed the design rather than confirming it.

**The area-prefix rule cannot block.** 489 of 500 accepted subjects carry an
`area: description` prefix; 495 do once the exemptions apply. The remaining five are ordinary
accepted commits with prose subjects, such as `Guard against loop underflow` and
`Add new mode to mpdecimate video filter`. A mechanical tier would have rejected
upstream's own work about once in a hundred commits, so the rule is heuristic.

**`Reapply "..."` is an exemption the design did not name.** It appears in accepted
history beside `Revert "..."`. The design's list held Revert, `fixup!` and `squash!`;
a checker without Reapply flags real upstream work. `fixup!` and `squash!` never
appear in accepted history at all, because they are squashed before merge, so they
are exempt for the pre-submission case rather than this one.

**The ascii rule had to be inverted.** The design lists `ascii-comments` among the
diff checks. Comments are exactly where upstream uses non-ascii: of the eleven
non-ascii added lines, three are arrows in a C comment, four are em-dashes in comments
(C and `.mak`), one is a name in a copyright line (`Kacper Michajłow`), and one is
`Schloß` in golden test data. A rule flagging non-ascii in comments would have reported ten of
eleven accepted lines. The rule is scoped to code instead, where the corpus holds
zero occurrences.

The measurement also grounded two exemptions that had been designed rather than observed. All five trailing-whitespace
occurrences are `tests/ref` golden files, where the trailing space is the expected
output. The single tab is in a Makefile-class file. Neither exemption needs to be
wider than the evidence.

## The rates

| Rule | Tier | Denominator | Leaves alone | Rate |
|---|---|---|---|---|
| `commit-msg-format` (area prefix) | heuristic | 500 subjects | 495 | 0.990 |
| `commit-msg-ascii` | mechanical | 500 subjects | 500 | 1.000 |
| `commit-msg-has-body` (vague list) | mechanical | 500 subjects | 500 | 1.000 |
| `diff-trailing-whitespace` | mechanical | 20,615 lines | 20,615 | 1.000 |
| `diff-tab-indent` | mechanical | 20,615 lines | 20,615 | 1.000 |
| `diff-ascii-code` | mechanical | 20,615 lines | 20,615 | 1.000 |
| `naming-namespace-prefix` | heuristic | 300 commits | 300 | 1.000 |
| `diff-narrow-scope` | heuristic | 300 commits | 298 | 0.993 |
| `diff-avoption-self-describing` | heuristic | 300 commits | 298 | 0.993 |
| `cosmetic-separate` | heuristic | 300 commits | 294 | 0.980 |

A rate of 1.000 on a heuristic rule is not a promotion to mechanical. It means the
corpus did not exercise it, and a rule that never fires has not been shown to work.
`naming-namespace-prefix` is the case in point: zero hits across 300 commits is
consistent with a well-scoped rule and equally consistent with one too narrow to
fire, and this corpus cannot distinguish them.

## The classifier the measurement rewrote twice

`cosmetic-separate` is the clearest case for calibrating before shipping, because two
plausible designs both failed and only measurement said so.

The first counted any brace or blank line on the functional side as a cosmetic
change, and reported **213 of 300** accepted commits as mixed (rate 0.290): adding a
new function adds braces and blank lines, and neither is a change to existing
formatting. Corrected so that "cosmetic present" means a removed/added pair
that cancelled under normalisation (an edit to existing code that was layout only), it
reported **108 of 300** (rate 0.640).

Still unusable, and for a reason the rule's own wording hides. A functional change
routinely reformats the lines it touches, and upstream accepts that; what it asks is
that an unrelated re-indent not be bundled with a fix. So the unit is the hunk, not
the diff: the reportable shape is a purely cosmetic hunk beside a functional one.
That reports **6 of 300** (rate 0.980).

The lesson generalises past this rule. A rule firing on 71% of accepted work would be
muted the day it shipped, and nothing in its specification said which unit to judge
it over. Only the corpus did.

## What is not calibrated

The attested rules, which no measurement can reach: they are the ones a human
answers. Their tier states that detectability is not available, which is a different kind of
claim from a rate.

The rules whose lanes are not built yet (`series`, `mail`, `build`) carry no rate,
and a test refuses to let them acquire one until their lane exists and is measured
the same way.

The corpus spans three weeks. One counter-example is
enough to disprove a mechanical tier, so three weeks settles that direction. The
opposite claim needs far more, which is why a 1.000 here is written down as "not
exercised" rather than as confidence.
