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

The signature of the flag list is a blocking noun from section 1 followed by a numeral (`Phase 1`, `Sprint 3`, `Stage II`, `Wave 2`). The numeral is the actual tell; the noun alone is not. The framing is cross-model: GPT, Gemini, Claude, Cursor, and Copilot all produce numbered `Phase`/`Step`/`Part` headers, so this is not specific to one assistant (the domain-heavy nouns like `Step` warn rather than block, per §1b).

The boundary is register, not the words themselves. These nouns in ordinary descriptive use ("the first pass over the array", "another round of review") are normal English, and the numeral gate allowlists exactly that. The tell is the filing-system register: numbered codes used as names in prose, headers, and commit subjects. People organise sequence with names and ordinary connectives; project-management segmentation as a universal organising principle is a register transplant, and that transplant is why it reads as machine output. The degenerate form — a bare numeral as a name with the noun elided (`## 3`, `## 5.5`, "as decided in 2.1") — is the same tell, not a fix for it.

## 1. Flag list: agentic phase-synonyms (blocking)

True synonyms for "phase" as a unit of project work named by its ordinal position. A
term blocks (Flag, exit 1) when it appears in a heading, a leading comment, or a commit
subject immediately followed by a blocking numeral (see §4). The blocking set is the
high-centrality work-unit words only: each either carries proven ordinal-naming tells in
real history (`stage` named six work units in this project's own commit log) or has
near-zero false-positive exposure in real code. The disposition is grounded in a corpus
measurement, not intuition (see §4 and `call/0037`).

| Term | Register | Note |
|---|---|---|
| Phase | SDLC, PM | canonical; the word usually being substituted |
| Stage | SDLC, PM | a unit of staged work; proven in-project tells. CI keyword caveat: excluded in pipeline YAML and Dockerfiles (see §4) |
| Sprint | scrum | timeboxed iteration; the agile work unit |
| Iteration | agile | a sprint by another name; generic "iteration N" |
| Cycle | agile | release or development cycle |
| Increment | agile, SAFe | shippable output of a sprint |
| Wave | rollout | "wave 1/2" of a staged rollout |
| Episode / Instalment | narrative | thesaurus synonyms, rare in code, near-zero collision |
| Leg / Lap | general | a leg or lap of a longer effort; rare in code |

### 1b. Advisory ordinal nouns (warn, not blocking)

These nouns name a unit of work by ordinal far less often than they carry ordinary domain
meaning. Measured across roughly 35,500 real `.rs` files (`call/0037`), each is
overwhelmingly domain usage, and because the noun-plus-numeral form is itself a flag it
could not be escaped through the `LEXICON`, so a false flag would force `--no-verify`.
They warn (exit 3, advisory); under `# host-lint: strict` an undeclared occurrence still
escalates to a flag, and the gather lane still surfaces a recurring shape for triage.

| Term | Domain reading (why advisory) |
|---|---|
| Section | document, specification, and legal sections — the single largest source |
| Round | cipher rounds, tournament rounds |
| Level | log levels, nesting levels, game levels |
| Step | tutorial and algorithm steps |
| Part | a part of a whole |
| Pass | compiler passes |
| Chapter | book chapters |
| Epoch | machine-learning training epochs |
| Batch | batch jobs |
| Era / Period | historical eras, time periods |

### 1a. Positional references to a milestone checklist item (host#16)

A milestone's state marks (the checklist boxes) get cited by their list position, the
same ordinal-by-position tell as a numbered name, aimed at a checklist rather than a
milestone title. The position is load-bearing in the reference, so it rots when the plan
is re-cut. The matched shapes, with the legitimate quantities that need declaring rather
than flagging:

```host-lint:ignore
box 7        boxes 4-8        steps 3-5
```

`box`, `boxes`, and `steps` (plural) join the flag list, and a numeral or a numeric range
(`N-M`) after the noun flags. The singular `step` is an advisory noun (§1b), so `step 3-5`
warns rather than blocks. These boundaries stay clean: the literal checklist mark (`- [ ]`,
`1. [x]`) carries no noun-plus-numeral; a content-named reference ("the deploy-path box")
names the item by its content; and the disposition verb "box" ("box an irreducible
citation") has no trailing numeral.

When a same-shaped token is a genuine quantity rather than a position, declare it in the
`LEXICON`. Declare the numeral-free contextual prefix (for the decode example, the phrase
without its number): the numbered phrase is itself a flag-tier tell, so the allowlist
guard refuses it by design, because the allowlist legitimizes vocabulary and does not
silence a real tell. The glued hyphen-digit form (the number joined to the noun inside
one token) stays out of scope: a legitimate glued term has no numeral-free prefix to
declare, so it could not be escaped through the allowlist.

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

Scope the linter to commit subjects, plan and design markdown headers, and comments in application source. Exclude CI/CD pipeline definitions and Dockerfiles, where `stage` and `steps` are reserved keywords.

The blocking match is case-insensitive and word-boundaried: a blocking noun immediately
followed by a blocking numeral. The numeral is an Arabic integer, a single-decimal
milestone numeral (`5.5`), a short checklist range (`4-8`), or a **multi-letter Roman
numeral written uppercase** (`IV`, `VIII`). A version-like form with two or more dots
(`1.2.3`) is not a numeral; a **single-letter Roman** (`I`, `C`, `V`) never blocks,
because it collides with the pronoun "I" and with language and identifier letters; and
only the immediately following token counts, so a numeral two words away is not a
positional reference. A four-digit (year-shaped) side is not a checklist range.

Blocking nouns (the work-unit words of §1 and the host#16 checklist terms of §1a):

```
\b(phase|stage|sprint|iteration|cycle|increment|wave|episode|instalment|leg|lap|box|boxes|steps)\b
```

The advisory nouns of §1b (`section`, `round`, `step`, `level`, `part`, `pass`, `chapter`,
`epoch`, `batch`, `era`, `period`) match the same numeral shape but warn, never block.

The canonical term lists, kept in sync with `FLAG_TERMS` and `WARN_ORDINAL_TERMS` in
`src/lib.rs` by a test (`vocabulary_term_lists_match_the_code`), so this document cannot
silently drift from what ships:

```host-lint:ignore
flag: phase stage sprint iteration cycle increment wave episode instalment leg lap box boxes steps
warn: pass round step level part section chapter epoch batch era period
```

Should block:

- `## Phase 1: Setup`
- `feat: phase 2 of auth refactor`
- `Sprint 3 backlog`
- `Phase IV, data migration` (multi-letter uppercase Roman)
- `boxes 4-8 blocked` (host#16 checklist range)

Should warn, not block (the §1b advisory nouns):

- `// Pass 1: tokenize`
- `see section 3 of the spec`
- `train for epoch 0`

Must not match (clean):

- `feat: add parser`
- `in this pass I fixed the bug` (the pronoun "I" is not a Roman numeral)
- `port the lexer to C` (a single language letter is not a Roman numeral)
- `step into 3 dimensions` (a numeral two words away)
- `increment the retry counter` (verb, no numeral)
- `the first pass over the array` (descriptive prose, no numeral)

The blocking set holds only the high-centrality work-unit words, so the verb and
descriptive-noun collisions (`round`, `level`, `step`, `pass`, `section`, `epoch`, ...)
warn rather than block; the disposition was measured against a real code corpus
(`call/0037`). The CI-keyword scoping above still excludes `stage`/`steps` in pipeline
YAML and Dockerfiles.

Bare-numeral headers (`^#{1,6}\s*\d+(\.\d+)?\s*$`) are the noun-elided form of the same
tell and are flagged in plan and design markdown; ordinary numbered-list items, changelog
version headings, and a four-digit year heading (`## 2024`) are excluded.

### Two severities: flag and warn

The matcher reports at two severities. A **flag** (exit 1) is a confirmed tell and blocks a commit hook. A **warn** (exit 3) is the bare-numeral degenerate form caught where it is harder to tell from ordinary use; it is advisory — a hook prints it and lets the commit through, and an agent treats it as a prompt to reconsider, not a hard stop. A line is classified at its most severe outcome (flag wins over warn).

Flag, in addition to the noun-gated and bare-numeral-header patterns above:

- **Leading label prefix** — a bare numeral used as a name at the start of a subject line, markdown header, or comment, followed by a colon and whitespace: `5.5: exec/pty tools`, `// 5.5: the pty exec tool`, `## 5.5: error handling`. The colon-then-space requirement separates a label from a clock time (`5:30 standup` does not match).

Warn — advisory forms that collide too readily with ordinary use to block:

- **Advisory ordinal noun + numeral** — one of the §1b domain-heavy nouns (`section`, `round`, `step`, `level`, `part`, `pass`, `chapter`, `epoch`, `batch`, `era`, `period`) immediately followed by a blocking numeral (`section 3`, `round 2`, `epoch 0`). Measured as overwhelmingly domain usage in real code (`call/0037`), so the noun warns rather than blocks; under `# host-lint: strict` an undeclared one escalates to a flag.
- **Filing-code noun + numeral** — `work-item`, `workitem`, or `wi` followed by a numeral (`work-item 5.3`). These nouns are not phase-synonyms; the warn catches the milestone-code register without hard-flagging ordinary use.
- **Bare dotted code** — a standalone `N.N` token used as a name (`as decided in 2.1`, `exec tools (5.5)`, `the tools arrive in 5.3`). A token carrying a letter (`v2.1`) is a version and is skipped; a trailing `%` or a following unit (`5.5 seconds`) marks a quantity and is skipped; a preceding version or cross-reference word (`python 3.11`, `figure 2.1`) is skipped. A preceding **all-caps designator** of two or more letters (`NT 3.1`, `SDK 2.1`, `DOS 6.2`) reads as a product or version name and is skipped, while a Title-case or lowercase noun before the decimal (`Decision 2.1`, `in 2.1`) still warns. The warn is deliberately recall-biased — it will still warn on some version or figure references (`upgrade to 2.1`), which is acceptable because it only asks the author to reconsider.

**Exempt — discretionary attribution trailers.** A `Co-Authored-By:` line is skipped entirely, neither flagged nor warned. Its contents are a co-author's name or a tool's version string (`Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>`), which the author sets at will and the linter has no business policing — without this the trailing `4.8` would read as a bare dotted code and warn on every AI-assisted commit.

## 5. Sibling tell: internal tracking codes used as names

A second tell class, distinct from the phase-synonym: an internal review or triage label (`B1`, `N2`, `finding #7`) cited in a commit subject or comment as if it were a name. The label is a working-memory handle scoped to one review run; it carries no meaning once that review scrolls off. The positive rule, as in the rewrite dictionary above: promote every finding to a durable identity — a descriptive technical name, or a filed issue — then reference that identity. The review label is the handle used while triaging, never the handle that ships.

| Working-memory label (never emit) | Durable identity (emit this) |
|---|---|
| `ci: fix the guard (review B1)` | `ci: fix the no-OS-comm guard: nm regex fails open on __imp__ thunks` |
| `ci: fix the guard (review B1)` | `ci: fix the no-OS-comm guard regex (fixes #NN)` — after filing the finding as an issue |
| `addresses blocker 1` | name the defect, or `closes #NN` |

Traceability and idiom are not in tension: `fixes #NN` is already allowlisted, so filing the finding as an issue yields a clean, durable, citeable handle. The linter flag below is only the backstop that catches a slip — it fires when a finding was never promoted to a real identity.

The escape hatch carries an obligation: cite issue numbers that exist. A bare `#N` taken from a private task tracker is the same code-as-name tell dressed as a GitHub reference — and worse, it renders as a resolvable link that resolves to nothing. The offline matcher cannot tell a real `#N` from a fake one (that would require resolving the number against the repository's live issue set), so a fake reference passes the linter by design. `#N` is not auto-vetted; verifying that the number resolves is review discipline, not a gate the *offline hook* provides.

The `LEXICON` (issue #13) supplies the gate that the offline hook cannot. A project declares its legitimate tell-shaped tokens there — each a full contextual phrase, masked before detection; a tracker reference carries its backing URL as required provenance. Because a sound, declarable escape now exists, the `# host-lint: strict` directive escalates the identifier/reference warn tier to a blocking **flag**: an undeclared tell-shaped token is no longer merely advisory. Three guards keep the escape honest — a bare numeral master key is refused, a phrase that is *itself* a flag-tier tell is refused (you rename it, you do not declare it), and a tracker reference with no URL is refused: a bare `#N`/`owner/repo#N`, or a `PROJ-NNNN` whose project key the project opted into gating with `# host-lint: jira-key PROJ`. That opt-in is why `PROJ-NNNN` is not gated by default — the shape is identical to standards tokens the host writes (`RFC-2119`, `UTF-8`), which must stay plain vocabulary unless a project deliberately declares the key. URL liveness is re-derived by a network-having lane (`host-lint lexicon --check-urls`), not the offline hook — the offline caveat above still stands for the hook itself, which is why the lexicon, not the matcher, carries the provenance.

A third escape, for markdown that must reproduce a tell-shaped token **literally** (a retired-ordinal dictionary, an archived citation): a fenced code block whose info string is `host-lint:ignore` is skipped by the naming scan (the prose scan already skips every fenced block). It is the idiomatic markdown info-string pattern (like ` ```mermaid `) — a deliberate, source-visible exemption, never a blanket file mute and never `--no-verify`. Three boundaries keep it honest and are part of the spec: it applies to **markdown only** (in a commit message or code file the fence is literal text and the tell flags); to **`host-lint:ignore`-tagged blocks only** (a regular ` ``` ` block, or one tagged `rust`/`text`, is still scanned); and to **blocks, not inline** (an inline `` `Phase 1` `` still flags, so a tell cannot be laundered by inline-quoting). A tell-shaped token in linted content is legitimate only as a real tracker reference, a `LEXICON` entry, or inside a `host-lint:ignore` block; anything else is reworded out (host `call/0019`).

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

### Bare leading code label (warn)

The flag above needs a `review`/`finding`/`blocker` noun in front of the code. The same code-as-name tell also appears with **no** noun, as a leading label that structures a write-up — a PR body whose fixes read `F1 — PE version stamp`, `F2 — handle isolation`, `F3 — bounded copy`. The diagnostic is that the label could be dropped and nothing would be lost: the durable name sits right after the dash. Promote the label away (`fix(loader): stamp PE version 3.10`) — the `F1` handle is scoped to one write-up and carries no meaning once it scrolls off.

This is **warned, not flagged**, because the bare one-letter+digits shape collides with legitimate single-letter identifiers — most importantly hardware **reference designators** used exactly this way (`R1 — 10kΩ resistor`, `U2 — microcontroller`, `Q3 — transistor`), which you genuinely cannot drop. The matcher cannot apply the drop-it test, so it asks the author to reconsider rather than blocking.

Match a code of the section-5 shape (one ASCII letter + digits) as the **first non-bullet token** of a line — leading bullets and emphasis (`-`, `*`, `**`, `//`, `#`) are skipped, so a markdown list item reads like a bare line — immediately followed by a label delimiter: an em/en-dash token (`—`/`–`), a spaced hyphen (a standalone `-` token), or a trailing colon on the code itself (`F4:`).

Should match (warn):

- `F1 — PE version stamp 3.10`
- `- **F2** — the handle-isolation fix` (markdown bullet + emphasis)
- `B3: the durable name follows the colon`

Must not match:

- `COM1 open — DCB seeding` (multi-letter device noun, not a one-letter code; also followed by a word, not a delimiter)
- `the F1 key opens help` (code not in leading position, no delimiter)
- `fixes #18` (a GitHub ref, not a leading letter+digit code)

## 6. Prose agentic tells (advisory): a token-free trope adaptation

Sections 1–5 catch *naming* tells. A second family is *prose* tells — stylistic
devices that, piled up, read as machine-generated. This is a token-free English
adaptation of the trope catalog at tropes.fyi (Ossama): no model, no licensed
lexicon, just public-domain phrases and explicit equations. It lives in
`host-grammar`'s `tells` module and host-lint calls it. **Every prose tell is
warn-tier (exit 3) — advisory, never blocking** — because any one device is
legitimate rhetoric; the signal is *density*, not any single use. The engine
runs on titles and drafts (`--stdin`, i.e. commit subjects and gh issue/PR
titles before filing) and on documents on demand (`--prose`); the staged-file
pre-commit path does not prose-scan, so ordinary file commits stay quiet.

**One exception — the subject line.** On a `--stdin` scan, a `decoration` tell
(em/en-dash, smart quote, arrow) on the *first line* — the commit subject, or a
gh title, which becomes the squash-merge subject and front-door text — escalates
to **flag (exit 1)**, the same no-decoration bar the front-door docs hold. The
density argument does not apply: a single em-dash in a one-line subject is not
rhetoric, it is the tell. Body prose and `--prose` documents are unaffected;
their decoration stays advisory.

### Lexical layer (phrase rules)

Word-boundaried, case-folded phrases, each with a low weight (one word is never
a verdict):

| Tell id | Signals | Cite |
|---|---|---|
| `ai-diction` | delve, utilize, leverage, robust, streamline, harness, tapestry, landscape, realm, paradigm, synergy, ecosystem, underscore, showcase, intricate, nuanced, multifaceted | tropes.fyi: AI vocabulary |
| `magic-adverb` | deeply, fundamentally, remarkably, profoundly, crucially | tropes.fyi: intensifier inflation |
| `serves-as` | serves as, stands as, represents a, acts as a | tropes.fyi: copula dodge |
| `filler-transition` | it's worth noting, it bears mentioning, importantly, notably, needless to say | tropes.fyi: empty signpost |
| `signposted-conclusion` | in conclusion, to sum up, in summary, all in all | tropes.fyi: signposted conclusion |
| `pedagogical-hook` | let's unpack, let's dive in, let's break this down, here's the kicker, here's the thing, buckle up | tropes.fyi: false suspense |
| `decoration` | em/en-dash `—` `–`, smart quotes `“ ” ‘ ’`, arrow `→` | tropes.fyi: typographic polish |

### Structural layer (equations)

Each is token-free and windowed over sentences (split with
`unicode-segmentation`). `s` is a sentence, `W` a token window.

- **negative-parallelism** = `|{(i,j): word_i ∈ NEG, word_j ∈ PIVOT, 0 < j−i ≤ 6}|` — "it's not X, it's Y". (antithesis)
- **tricolon**: `is_triad(s)` = a short comma triad `A, B, and C` where each span is ≤ 5 words with no internal terminal punctuation. (classical rhetoric)
- **anaphora** = `Σ_runs max(0, L_r − 2)²` over runs of consecutive sentences sharing an opener (first content word past a leading stopword); superlinear, so a pair is free. (classical rhetoric)
- **countdown** = `max(0, run_len)` on a run of ≥2 sentence-initial `Not …` closed by `Just/Only …`. (triadic close)
- **self-answered-question** = count of `?`-terminated sentences immediately followed by a ≤5-word fragment. (hypophora)
- **listicle** = anaphora over ordinals (first/second/next/finally …). (listicle-in-prose)
- **ing-tail**: a trailing `, verbing …` participial clause at a sentence end. (participial tail)
- **false-range** = density of `from X … to Y` (count only; spectrum validity needs semantics). (false range)
- **punchy-fragments** = `single_sentence_paragraphs / paragraphs` when high. (staccato paragraphs)
- **bold-first-bullets** = `bullets_opening_** / bullets` when high. (bold-lead bullets)

### Composite density

`tell_score(text)` sums the weights and divides by sentence count; a document is
**over threshold** only when the weighted mass is high in absolute terms *and*
dense relative to length (conservative gates, tuned by fixtures). This is the
"the problem is many together" rule — individual tells stay advisory; the
density is what escalates. Out of scope (needs semantics, not token-free, and so
documented but not detected): one-point dilution, content duplication, invented
concept labels, grandiose stakes, false vulnerability, dead-metaphor repetition.

## Sources

- tropes.fyi — token-free trope catalog (Ossama, ossama.is); itself AI-assisted, cited as the catalog, not as primary linguistics: https://tropes.fyi/tropes-md
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
