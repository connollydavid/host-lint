# Phase-synonym agentic tells vs idiomatic git vocabulary

## Write it this way: the rewrite dictionary

Before the flag lists, the positive rule. Name work after its content, and encode sequence with document order and ordinary connectives ("after the parser lands", "once CI is green"). If you plan with internal codes, keep a dictionary in working memory that maps each plan code to a descriptive name, and always emit the descriptive side; the code never reaches prose, headers, or commit subjects.

| Internal plan code (never emit) | Descriptive text (emit this) |
|---|---|
| `## Phase 1: Setup` | `## Setup` |
| `// Pass 1: tokenize` | `// tokenization pass` |
| `Step 3 of 5: configure CI` | `Configure CI (after tests pass, before release)` |
| `feat: phase 2 of auth refactor` | `feat(auth): add session storage` |
| `M2 delivered the projection model` | `the projection model is verified against three reference kernels` |
| `## 5.5` | `## Error handling` |

The last pair of rows carry the two hardest lessons: artifacts state claims about content, not diary entries about plan position; and eliding the noun to leave a bare numeral is the same tell, not a fix.

Reference for an anti-slop linter or commit hook. Two lists that must not be confused: the numbered-segment vocabulary an LLM coding agent stamps on plans, comments, and commit subjects (flag), and the established git, code-review, and source-annotation vocabulary that is normal human practice (allow). A third gray-zone list covers terms that are neither, to stop the linter over-matching.

The signature of the flag list is a noun from section 1 followed by a numeral (`Phase 1`, `Step 2`, `Stage II`, `Pass 1 of 3`). The numeral and any "of N" total are the actual tell; the noun alone is not. The framing is cross-model: GPT, Gemini, Claude, Cursor, and Copilot all produce numbered `Phase`/`Step`/`Part` headers, so this is not specific to one assistant.

The boundary is register, not the words themselves. These nouns in ordinary descriptive use ("the first pass over the array", "another round of review") are normal English, and the numeral gate allowlists exactly that. The tell is the filing-system register: numbered codes used as names in prose, headers, and commit subjects. People organise sequence with names and ordinary connectives; project-management segmentation as a universal organising principle is a register transplant, and that transplant is why it reads as machine output. The degenerate form — a bare numeral as a name with the noun elided (`## 3`, `## 5.5`, "as decided in 2.1") — is the same tell, not a fix for it.

## 1. Flag list: agentic phase-synonyms

True synonyms for "phase" as an ordered, numberable span. Flag when one appears in a heading, a leading comment, or a commit subject and is followed by a numeral.

| Term | Register | Note and false-positive risk |
|---|---|---|
| Phase | SDLC, PM | canonical; the word usually being substituted |
| Stage | SDLC, PM, CI/CD | high FP risk: `stage` is a first-class keyword in CI/CD pipelines and Docker multi-stage builds. Gate on numeral, exclude pipeline YAML and Dockerfiles |
| Step | SDLC, CI/CD | high FP risk: `steps:` is a GitHub Actions and pipeline keyword. Same scoping caveat |
| Part | general | "Part 1/2"; common plan and commit header |
| Section | general | code or doc region; weak temporal sense |
| Pass | compilers | "first/second pass"; legitimate compiler term, gate on numeral |
| Round | review, general | "round 1/2" of edits or review |
| Iteration | agile | a sprint by another name; also generic "iteration N" |
| Sprint | scrum | timeboxed iteration; same slot |
| Cycle | agile, hardware | release or dev cycle; FP risk with "clock cycle" |
| Increment | agile, SAFe | shippable output of a sprint; FP risk with the verb "increment" |
| Wave | rollout | "wave 1/2" of a staged rollout |
| Batch | data | "batch 1/2" of work; FP risk with data-processing batches |
| Period / Era / Epoch | general, ML | time-spans; `epoch` is an ML training term, restrict to numbered headers |
| Chapter / Episode / Instalment | narrative | thesaurus synonyms, rare in code, occasional in long plans |
| Leg / Lap | general | a leg or lap of a longer effort; rare in code |
| Level | games, general | sometimes a stage ("level 1"), often a hierarchy degree instead |

## 2. Allowlist: idiomatic git and review vocabulary (do not flag)

These carry defined meaning in established conventions. None denotes a phase. They classify a change, a review remark, or an in-source annotation.

### Conventional Commits types

The prefix states the kind of change, per the Conventional Commits spec.

- `feat` new feature
- `fix` bug fix
- `docs` documentation only
- `style` formatting, whitespace, no behaviour change
- `refactor` behaviour-preserving restructure
- `perf` performance change
- `test` tests
- `build` build system or dependencies
- `ci` CI configuration
- `chore` maintenance with no production code change
- `revert` undo of a previous commit

Some configs (Angular-derived) also recommend `improvement`. A scope in parentheses (`feat(parser):`) and a `BREAKING CHANGE:` footer belong to the same spec.

### Conventional Comments labels

The prefix states the intent of a review remark, per the Conventional Comments spec. Eight core labels:

- `praise`
- `nitpick` (commonly `nit`) trivial, preference-based, author free to ignore
- `suggestion`
- `issue` a user-facing problem
- `todo`
- `question`
- `thought`
- `chore`

Decorations attach in parentheses: `(blocking)`, `(non-blocking)`, `(if-minor)`. Example: `nit (non-blocking): rename uc to userCount`.

### Code-tag comment markers

In-source annotations with a lineage going back to PEP 350 and earlier Sun Java conventions. Scanned by tools such as the `fixme` family and editor TODO panels.

- `TODO` deferred work
- `FIXME` known-broken, deferred
- `XXX` flag for something bogus, or a value placeholder; higher severity than TODO by Sun's reading
- `HACK` deliberate ugly workaround
- `BUG` known defect
- `NOTE` / `NB` explanatory note
- `OPTIMIZE` performance opportunity
- `REVIEW` needs a second look
- `WONTFIX` / `NOBUG` acknowledged, will not change

### Status and review shorthand

Human idiom used by people and agents alike, so not a reliable AI signal on its own.

- `WIP` work in progress (draft PRs, scratch commits)
- `LGTM` looks good to me
- `PTAL` please take another look
- `TBD` to be decided
- `TL;DR`
- `IIRC`, `AFAICT`
- `RFC` request for comments

Note: `WIP` is long-standing human practice, not an LLM tell. Do not flag it.

## 3. Gray zone: context-dependent (neither flag nor allowlist)

These fill nearby slots but are not phase-synonyms. Letting the linter treat them as flags produces false positives on normal roadmap and architecture writing.

| Term | What it actually is |
|---|---|
| Milestone | a point or marker, not a span |
| Gate / Stage-gate / Phase-gate | a go/no-go decision point between phases |
| Checkpoint | a save or verify point |
| Workstream | a parallel track, runs concurrently |
| Initiative | a strategic container above epics |
| Epic | a work container spanning several sprints |
| Story / User story | a single work item inside an epic |
| Task / Sub-task / Ticket | a unit of work |
| Deliverable | an output produced within a phase |
| Module / Component / Feature | a part of the system, not the timeline |
| Tier / Grade / Notch / Rung | degree or hierarchy senses |

## 4. Detection notes

Scope the linter to commit subjects, plan and design markdown headers, and comments in application source. Exclude CI/CD pipeline definitions and Dockerfiles, where `stage` and `step` are reserved keywords.

Core pattern, case-insensitive, word-boundary, numeral-gated:

```
\b(phase|stage|step|part|pass|round|iteration|sprint|cycle|increment|wave|batch|section)\s+(\d+|[ivxlcdm]+)\b
```

Heading and leading-comment variants:

```
^#{1,6}\s*(phase|stage|step|part)\s+\d                  # markdown header
^\s*(//|#|--|/\*|\*)\s*(phase|stage|step|part)\s+\d     # leading code comment
\b(phase|stage|step)\s+\d+\s+of\s+\d+\b                 # explicit "N of M"
```

Should match:

- `## Phase 1: Setup`
- `// Pass 1: tokenize`
- `Step 3 of 5`
- `feat: phase 2 of auth refactor`
- `Stage II, data migration`

Must not match:

- `feat: add parser`
- `nit: rename uc to userCount`
- `fix(api): correct fee calculation`
- `TODO: handle null input`
- `chore: bump deps`
- `WIP: draft, do not merge`
- `// FIXME: race condition on shutdown`
- `increment the retry counter` (verb, no numeral)
- `the first pass over the array` (descriptive prose, no numeral)

The numeral gate removes most verb and descriptive-noun collisions (`increment`, `cycle`, `pass`, `level`). Residual risk sits with `stage`/`step` in infra config and `epoch` in ML code; the scoping rule above handles those.

Bare-numeral headers (`^#{1,6}\s*\d+(\.\d+)?\s*$`) are the noun-elided form of the same tell and can be flagged at low severity in plan and design markdown; exclude ordinary numbered-list items and changelog version headings.

## 5. Sibling tell: internal tracking codes used as names

A second tell class, distinct from the phase-synonym: an internal review or triage label (`B1`, `N2`, `finding #7`) cited in a commit subject or comment as if it were a name. The label is a working-memory handle scoped to one review run; it carries no meaning once that review scrolls off. The positive rule, as in the rewrite dictionary above: promote every finding to a durable identity — a descriptive technical name, or a filed issue — then reference that identity. The review label is the handle used while triaging, never the handle that ships.

| Working-memory label (never emit) | Durable identity (emit this) |
|---|---|
| `ci: fix the guard (review B1)` | `ci: fix the no-OS-comm guard: nm regex fails open on __imp__ thunks` |
| `ci: fix the guard (review B1)` | `ci: fix the no-OS-comm guard regex (fixes #NN)` — after filing the finding as an issue |
| `addresses blocker 1` | name the defect, or `closes #NN` |

Traceability and idiom are not in tension: `fixes #NN` is already allowlisted, so filing the finding as an issue yields a clean, durable, citeable handle. The linter flag below is only the backstop that catches a slip — it fires when a finding was never promoted to a real identity.

The escape hatch carries an obligation: cite issue numbers that exist. A bare `#N` taken from a private task tracker is the same code-as-name tell dressed as a GitHub reference — and worse, it renders as a resolvable link that resolves to nothing. The offline matcher cannot tell a real `#N` from a fake one (that would require resolving the number against the repository's live issue set), so a fake reference passes the linter by design. `#N` is not auto-vetted; verifying that the number resolves is review discipline, not a gate the engine provides.

Flag `review`, `finding`, or `blocker` immediately followed by a `#N` or a letter+digit code, case-insensitive. Matching is token-based, and the token rule is the spec: split the line on whitespace; trim surrounding punctuation from the noun token (hyphens kept, so `code-review` is not the noun); trim surrounding punctuation from the code token except a leading `#`. The code must then be exactly `#` plus digits, or one letter plus digits — codes with attachments (`b1's`, `#7/8`) do not match.

A shell approximation, looser than the token rule at both boundaries (it misses punctuation-trimmed forms like a parenthesised code, and matches hyphen-joined nouns the token rule does not):

```
(^|[^a-z])(review|finding|blocker)\s+(#[0-9]+|[a-z][0-9]+)([^a-z0-9]|$)
```

The letter/`#` gate separates the code-as-name tell from ordinary use of the same nouns. The noun set deliberately excludes `closes` and `fixes`, so GitHub issue references stay clean — `fixes #NN` is the allowlisted way to carry traceability into a subject, and filing the finding as an issue is the sanctioned alternative to naming it descriptively.

Should match:

- `ci: fix the guard regex (review B1)`
- `addresses finding #7`
- `blocker B2 resolved`
- `addresses review (B1)` (punctuation around the code is trimmed)

Must not match:

- `review 3 files` (bare numeral after the noun, not a code)
- `finding 0 results`
- `fixes #18` / `closes #35` (GitHub refs; the verbs are not in the noun set)
- `Finding #B1` (letter-prefixed code after `#`; plan-document convention, not this tell)

Known gate limits, accepted for parity with the field-tested wrapper rule: version, quarter, and infrastructure tokens (`review v2`, `review q3`, `review s3`) match the letter+digit shape and will flag; the allowlisted `REVIEW` code-tag (section 2) followed by a letter+digit identifier also flags — when those collide, this rule wins.

## Sources

- Conventional Commits v1.0.0: https://www.conventionalcommits.org/en/v1.0.0/
- Conventional Comments: https://conventionalcomments.org/
- PEP 350, Codetags (TODO, FIXME, XXX, HACK): https://peps.python.org/pep-0350/
- "What Does Nit Mean in Code Review", Augment Code: https://www.augmentcode.com/guides/what-does-nit-mean-in-code-review
- `fixme`, in-source tag scanner: https://github.com/JohnPostlethwait/fixme
- SDLC phase/stage/step usage, Atlassian: https://www.atlassian.com/agile/software-development/sdlc
- Sprint, iteration, increment equivalence, Sila: https://www.silasg.com/resources/agile-faq/sprint-iteration-increment
- Agile epics (epic spans multiple sprints), Atlassian: https://www.atlassian.com/agile/project-management/epics
- Phase, gate, tranche definitions, APM glossary: https://www.apm.org.uk/resources/glossary/
- Phase-gate process, Planisware: https://planisware.com/glossary/phase-gate-or-stage-gate
- "Phase" synonyms, Merriam-Webster Thesaurus: https://www.merriam-webster.com/thesaurus/phase
