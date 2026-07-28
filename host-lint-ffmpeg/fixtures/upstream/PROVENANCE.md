# Provenance for upstream excerpts

Every file in this directory is real upstream material rather than something
synthesized for this repository, so each one is named here with its source, the
commit or fetch it came from, and its license. The fixture-licensing CI job
requires an entry for every file beside this one.

## accepted-subjects.txt

- **Source:** `FFmpeg/FFmpeg`, branch `master`, via the GitHub commits API.
- **Fetched:** 2026-07-28, the five most recent pages of 100 commits, deduplicated.
- **Content:** the first line of each commit message. Subjects only; no message
  bodies, no diffs, no code.
- **License:** FFmpeg is LGPL-2.1-or-later with GPL parts. These are commit
  subject lines, used here as measured ground truth for calibrating a checker
  against what upstream actually accepts, not as a copy of the work.
- Synthesis cannot reach this, because the whole point is that these subjects were
  *accepted upstream*. A synthesized subject proves nothing about what the
  project's reviewers let through, and the rate this corpus measures is the only
  honest basis for a rule's tier.

## maintainers-live-syntax.txt

- **Source:** `FFmpeg/FFmpeg`, `MAINTAINERS`, at commit
  `c6309b5c63add7ad0ec221fafefc32bdcd6f8b91`.
- **Fetched:** 2026-07-28.
- **Content:** eighteen lines copied verbatim, chosen to cover every entry syntax
  the parser must handle, regrouped under their own section headers so the fixture
  is coherent. No line was edited.
- **License:** FFmpeg is LGPL-2.1-or-later with GPL parts. These are maintainer
  attribution lines, used as a parser fixture.
- Synthesis cannot reach this either: the parser's whole risk is the *observed*
  syntax, and a synthesized file would only exercise the shapes I already thought
  of. These lines carry the three CC forms actually in use (angle-bracketed, bare,
  and obfuscated `at`/`dot`), the two parenthesized forms that are NOT a CC request
  (an obfuscated address with no marker, and a plain name), a glob with a
  parenthesis in it (`vf_(t)interlace`), comma-separated globs with and without a
  space, and a `[2]` status marker.
