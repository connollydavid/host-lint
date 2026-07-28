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

Pinned at FFmpeg master `c6309b5c63add7ad0ec221fafefc32bdcd6f8b91`.

| Rule | Tier | Lane | Measured | Upstream section |
|---|---|---|---|---|
| `commit-msg-format` | heuristic | msg | 0.990 | Patches/Committing / Commit messages |
| `commit-msg-has-body` | heuristic | msg | 1.000 | Patches/Committing / Commit messages |
| `commit-msg-cites-tracker` | mechanical | msg | not measured | Patches/Committing / Commit messages |
| `patch-license-compatible` | heuristic | diff | not measured | Patches/Committing / Licenses for patches must be compatible with FFmpeg. |
| `no-broken-build` | attested | build | not measured | Patches/Committing / You must not commit code which breaks FFmpeg! |
| `testing-proportionate` | attested | build | not measured | Patches/Committing / Testing must be adequate but not excessive. |
| `one-change-per-commit` | heuristic | diff | not measured | Patches/Committing / Do not commit unrelated changes together. |
| `backport-stays-focused` | heuristic | diff | not measured | Patches/Committing / Bug fixes intended for backporting should stay focused. |
| `cosmetic-separate` | heuristic | diff | not measured | Patches/Committing / Cosmetic changes should be kept in separate patches. |
| `credit-the-author` | mechanical | msg | not measured | Patches/Committing / Credit the author of the patch. |
| `credit-researchers` | mechanical | msg | not measured | Patches/Committing / Credit any researchers |
| `wait-before-push` | attested | series | not measured | Patches/Committing / Always wait long enough before pushing changes |
| `correctness` | attested | build | not measured | Code behaviour / Correctness |
| `thread-and-library-safety` | attested | build | not measured | Code behaviour / Thread- and library-safety |
| `robustness` | attested | build | not measured | Code behaviour / Robustness |
| `memory-allocation` | heuristic | diff | not measured | Code behaviour / Memory allocation |
| `no-stdio` | mechanical | diff | not measured | Code behaviour / stdio |
| `warning-suppression-last-resort` | heuristic | diff | not measured | Code / Warnings for correct code may be disabled if there is no other option. |
| `subscribe-devel` | attested | mail | not measured | Documentation/Other / Subscribe to the ffmpeg-devel mailing list. |
| `subscribe-cvslog` | attested | mail | not measured | Documentation/Other / Subscribe to the ffmpeg-cvslog mailing list. |
| `docs-current` | heuristic | diff | not measured | Documentation/Other / Keep the documentation up to date. |
| `discussion-in-public` | attested | mail | not measured | Documentation/Other / Important discussions should be accessible to all. |
| `maintainers-entry-current` | mechanical | diff | not measured | Documentation/Other / Check your entries in MAINTAINERS. |
| `send-email-setup` | attested | mail | not measured | Submitting patches / How to setup git send-email? |
| `no-client-mangling` | mechanical | mail | not measured | Submitting patches / Sending patches from email clients |
| `review-replies-addressed` | attested | series | not measured | Submitting patches / Reviews |
| `naming-lowercase-functions` | mechanical | diff | not measured | Naming conventions |
| `naming-camelcase-types` | mechanical | diff | not measured | Naming conventions |
| `naming-uppercase-constants` | mechanical | diff | not measured | Naming conventions |
| `naming-namespace-prefix` | heuristic | diff | 1.000 | Naming conventions |
| `comment-nontrivial-functions` | heuristic | diff | not measured | Comments |
| `language-c11-headers-c99` | attested | build | not measured | Language |
| `misc-conventions` | attested | series | not measured | Miscellaneous conventions |
| `api-interface-discipline` | attested | series | not measured | Library public interfaces |
| `api-adding-interfaces` | attested | series | not measured | Adding new interfaces |
| `api-removing-interfaces` | attested | series | not measured | Removing interfaces |
| `submit-for-review` | attested | series | not measured | Introduction |
| `submission-checklist` | attested | series | not measured | Patch submission checklist |
| `regression-tests-run` | attested | build | not measured | Regression tests |
| `diff-trailing-whitespace` | mechanical | diff | 1.000 | Code formatting conventions |
| `diff-tab-indent` | mechanical | diff | 1.000 | Code formatting conventions |
| `format-indent-four` | heuristic | diff | not measured | Code formatting conventions |
| `format-line-length` | heuristic | diff | not measured | Code formatting conventions |
| `api-major-bump-scope` | attested | series | not measured | Major version bumps |
| `review-process-followed` | attested | series | not measured | Patch review process |

## Project conventions, which upstream does not document

| Rule | Why it is not upstream's |
|---|---|
| `commit-msg-signoff` | FFmpeg requires no sign-off; a project may |
| `commit-msg-ascii` | no upstream rule covers subject encoding; measured at zero non-ascii in 500 accepted subjects |
| `diff-ascii-code` | no upstream rule covers source encoding; comments and names legitimately carry non-ascii |
| `diff-narrow-scope` | a reviewer preference expressed in review, not documented |
| `diff-avoption-self-describing` | named by a reviewer, not by upstream documentation |
| `forge-title-grammar` | code.ffmpeg.org relays the title as the list subject; the forge is not upstream doctrine |
| `forge-versioned-title` | forge revision discipline reuses one pull request; a mail-series version does not apply |
| `forge-draft` | the WIP marker is a forge convention |
| `forge-description-cover` | the description is the cover letter on the forge, and only there |
| `forge-rationale-in-commits` | a forge description never enters git history; upstream documents no forge |

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
