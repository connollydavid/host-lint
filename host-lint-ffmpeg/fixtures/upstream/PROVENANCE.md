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
- **Why it cannot be synthesized:** the whole point is that these subjects were
  *accepted upstream*. A synthesized subject proves nothing about what the
  project's reviewers let through, and the rate this corpus measures is the only
  honest basis for a rule's tier.
