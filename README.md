# host-lint

Detect phase-synonym agentic tells in commit messages, markdown headers, and code comments.

An anti-slop linter that flags LLM-generated plan language bleeding into source code. Implements the detection rules from [VOCABULARY.md](VOCABULARY.md).

## What It Detects

Flags numbered phase-synonym patterns:

- `## Phase 1: Setup` — markdown header
- `// Pass 1: tokenize` — code comment
- `Step 3 of 5` — prose
- `feat: phase 2 of auth refactor` — conventional commit with phase tell
- `Stage II, data migration` — Roman numeral
- `entry point (Phase 5.0)` — decimal numeral, any position
- `5.5: exec/pty tools` — bare numeral used as a label prefix

Also flags internal tracking codes used as names (a sibling tell):

- `ci: fix the guard regex (review B1)` — review label as a name
- `addresses finding #7` — triage code instead of a description

Warns (advisory, exit 3) on the bare-numeral degenerate form — the noun elided, a bare numeral floating free where it is harder to tell from ordinary use:

- `as decided in 2.1` — bare dotted code used as a name
- `exec tools (5.5)` — parenthetical numeral label
- `work-item 5.3` — filing-code noun with a numbered label

A warning asks the author (or agent) to reconsider; it does not block a commit hook.

Does NOT flag:

- `feat: add parser` — conventional commit, no phase tell
- `TODO: handle null input` — code tag
- `increment the retry counter` — verb, no numeral
- `the first pass over the array` — ordinal, not numeral
- `fixes #18` / `closes #35` — GitHub issue refs, the idiomatic durable reference
- `review 3 files` — bare numeral after the noun, not a code
- `bump to v2.1` / `Python 3.11` / `5.5 seconds` — version strings and quantities

Known limitation: the matcher is offline and cannot validate `#N` against the repository's real issues. An internal tracker ID dressed as a GitHub ref (a `#N` that resolves to nothing) passes by design — catching it is review discipline, not a linter gate. See VOCABULARY.md.

## Usage

### CLI

```bash
# Scan commit message from stdin
echo "Phase 1: setup" | ./host-lint --stdin

# Scan specific files
./host-lint README.md src/main.rs

# JSON output
./host-lint --json --stdin

# Scan all tracked files
./host-lint --all

# Scan every commit message in the repository's history
./host-lint --log
```

### Adopting or upgrading

Hooks only gate new commits, and detection rules grow over time, so a project that adopts the skill late — or upgrades to a newer binary — may already contain tells the current rules would flag. Run a one-shot audit after install or upgrade:

```bash
./host-lint --all   # live files: fix what it flags
./host-lint --log   # commit history: informational by default
```

`--log` reports findings as `<short-sha>:<line>: <text> (<term>)`, one record per offending commit message line.

By default, treat history findings as informational and leave history alone. If you do choose to clean them, archive the original history first, then rewrite:

```bash
git branch archive/pre-host-lint-audit         # preserve the original history
git push origin archive/pre-host-lint-audit
git commit --amend                            # tell in the tip commit only
# deeper history: git rebase -i <base>, rewording flagged commits,
# or git filter-repo --message-callback for bulk rewrites
git push --force-with-lease

# link each replaced commit to its replacement, so the archive stays coherent
git notes add -m "Superseded-by: <new-sha>" <old-sha>
git push origin refs/notes/commits
```

Each replaced commit gets a `Superseded-by:` trailer via `git notes` — the archived sha then points at its rewrite in `git log --notes` without itself being rewritten.

Rewriting changes every descendant sha: collaborators must re-clone or hard-reset, and external references to old shas go stale. Only do this on branches you control.

### Sanctioned vocabulary (`.host-lint-allow`)

Some numbered tokens are legitimate vocabulary in a given repo and not tells: a version string (`NT 3.1`, `DOS 6.22`), a release identity, or a cross-repo filename you are forbidden to rename. List them, one phrase per line, in a `.host-lint-allow` file at the repo root:

```
# sanctioned vocabulary — never flagged anywhere in this repo
NT 3.1
DOS 6.22
Winsock 1.1
section 1
```

Each listed phrase is masked out of every line before detection, case-insensitively and at word boundaries, so an occurrence of that exact phrase never flags — anywhere in the repo, in any mode (`--stdin`, files, `--all`, `--log`). The boundary requirement keeps an entry specific: allow-listing `phase 1` clears `phase 1` but not the longer tell `phase 12`, and a *different* tell on the same line still flags (`section 1 covers phase 4` still reports `phase 4`). Lines beginning with `#` are comments; blank lines are ignored. No file means no allow-list — the feature is opt-in per repo.

This is for acknowledging legitimate vocabulary, not for silencing real tells: prefer renaming work after its content (see VOCABULARY.md) and reserve the allow-list for tokens that genuinely are not slop.

### Excluding paths (`.host-lintignore`)

`--all` is a whole-tree audit. To exclude paths from it — an append-only record, a
vendored tree, generated files — list them in a `.host-lintignore` at the repo root
(gitignore-lite: one pattern per line, `#` comments and blanks ignored):

```
# append-only history is acknowledged, not re-audited
MEMORY.md
plan/*/README.md
archive/
```

A pattern is an exact repo-relative path (`MEMORY.md`), a `*` glob that matches
within a single path segment (`plan/*/README.md`), or a trailing-slash directory
prefix (`archive/`) that excludes everything beneath it. Only `--all` honours the
file; explicit file arguments and `--stdin` are always scanned. This is a path
filter, not a token exemption — for sanctioned *vocabulary* use `.host-lint-allow`.

### Pre-commit Hook

```bash
cp pre-commit .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
```

### Agent Skill

The [SKILL.md](SKILL.md) frontmatter makes this callable as an agent skill. Drop the directory into your agent's skills folder.

## Building

```bash
cargo build --release
```

For a static binary (Linux amd64):

```bash
rustup target add x86_64-unknown-linux-musl
cargo build --release --target x86_64-unknown-linux-musl
```

## Testing

```bash
# Property-based tests (proptest)
cargo test

# Integration tests (VOCABULARY.md cases)
./test-integration.sh ./target/release/host-lint

# Conformance gates (G1-G8 for SKILL.md)
./lint-skill.sh
```

## Specification

The detection behavior is formally specified in [host-lint.allium](host-lint.allium) (Allium format).

## License

Unlicense
