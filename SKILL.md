---
name: no-phase-skill
description: Lints phase-synonym agentic tells in commit messages, markdown headers, and code comments. Use when checking commits, reviewing code for LLM-slop, running pre-commit hooks, or when the user mentions linting, phase detection, commit hygiene, or slop.
---

Detects numbered phase-synonym patterns that signal LLM-generated plan language bleeding into commits, headers, and comments. Implements the rules in VOCABULARY.md.

## Install

Get the `no-phase` binary for your platform from the latest GitHub release, or build it with `cargo build --release`. Release assets cover linux, darwin, and windows on amd64 and arm64 (e.g. `no-phase-linux-amd64`, `no-phase-darwin-arm64`, `no-phase-windows-amd64.exe`):

```
# substitute your platform asset name
gh release download -R connollydavid/no-phase-skill -p no-phase-linux-amd64 -O no-phase
chmod +x no-phase
```

To use as an agent skill in Claude Code, copy or symlink this directory into `.claude/skills/` of the consuming repo.

## Usage

### As a git hook

Copy the binary and `pre-commit` into `.git/hooks/` of the target repo. The script dispatches on its installed name, so install it as `pre-commit` (scans staged files), `commit-msg` (scans the commit message), or both:

```
cp no-phase .git/hooks/no-phase
cp pre-commit .git/hooks/pre-commit
cp pre-commit .git/hooks/commit-msg
chmod +x .git/hooks/no-phase .git/hooks/pre-commit .git/hooks/commit-msg
```

### As a CLI

```
# Scan commit message from stdin
echo "Phase 1: setup" | ./no-phase --stdin

# Scan specific files
./no-phase README.md src/main.rs

# JSON output
./no-phase --json --stdin

# Scan all tracked files in a repo
./no-phase --all

# Scan every commit message in the repo's history
./no-phase --log
```

### Adoption / upgrade audit

Hooks only gate new commits, and rules grow over time, so run a one-shot audit when installing the skill into an existing repo or after upgrading the binary: `./no-phase --all` (fix flagged live files) and `./no-phase --log` (history findings — informational by default). If the user opts to clean history, guide the archive-then-rewrite flow from the README: create and push an archive branch preserving the original history, then `git commit --amend` (tip) or rebase/filter-repo (deeper) and force-push with lease. Never rewrite without archiving first or on branches the user does not control.

### As an agent skill

Run the binary against the target and act on results:

1. Identify what to check (commit message, staged files, or specific files)
2. Run `no-phase` with appropriate flags
3. If exits 0, clean. If exits 1, flag found — report matches to user.

## Exit codes

- `0` — no phase-synonym tells found
- `1` — one or more tells detected

## Portability notes

Released binaries cover linux (static musl), macOS, and windows on amd64 and arm64, with no runtime dependencies. For other platforms, build from source with `cargo build --release`.
