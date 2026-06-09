---
name: host-lint
description: Lints phase-synonym agentic tells in commit messages, markdown headers, and code comments. Use when checking commits, reviewing code for LLM-slop, running pre-commit hooks, or when the user mentions linting, phase detection, commit hygiene, or slop.
---

Detects numbered phase-synonym patterns that signal LLM-generated plan language bleeding into commits, headers, and comments. Implements the rules in VOCABULARY.md.

## Install

Get the `host-lint` binary (statically linked linux-amd64) from the latest GitHub release, or build it with `cargo build --release`:

```
gh release download -R connollydavid/host-lint -p host-lint -O host-lint
chmod +x host-lint
```

To use as an agent skill in Claude Code, copy or symlink this directory into `.claude/skills/` of the consuming repo.

## Usage

### As a git hook

Copy the binary and `pre-commit` into `.git/hooks/` of the target repo. The script dispatches on its installed name, so install it as `pre-commit` (scans staged files), `commit-msg` (scans the commit message), or both:

```
cp host-lint .git/hooks/host-lint
cp pre-commit .git/hooks/pre-commit
cp pre-commit .git/hooks/commit-msg
chmod +x .git/hooks/host-lint .git/hooks/pre-commit .git/hooks/commit-msg
```

### As a CLI

```
# Scan commit message from stdin
echo "Phase 1: setup" | ./host-lint --stdin

# Scan specific files
./host-lint README.md src/main.rs

# JSON output
./host-lint --json --stdin

# Scan all tracked files in a repo
./host-lint --all
```

### As an agent skill

Run the binary against the target and act on results:

1. Identify what to check (commit message, staged files, or specific files)
2. Run `host-lint` with appropriate flags
3. If exits 0, clean. If exits 1, flag found — report matches to user.

## Exit codes

- `0` — no phase-synonym tells found
- `1` — one or more tells detected

## Portability notes

The released `host-lint` binary is a statically linked linux-amd64 executable with no runtime dependencies. For other platforms, build from source with `cargo build --release`.
