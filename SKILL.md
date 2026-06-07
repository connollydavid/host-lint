---
name: host-lint
description: Lints phase-synonym agentic tells in commit messages, markdown headers, and code comments. Use when checking commits, reviewing code for LLM-slop, running pre-commit hooks, or when the user mentions linting, phase detection, commit hygiene, or slop.
---

Detects numbered phase-synonym patterns that signal LLM-generated plan language bleeding into commits, headers, and comments. Implements the rules in VOCABULARY.md.

## Usage

### As a pre-commit hook

Copy `pre-commit` into `.git/hooks/` of the target repo:

```
cp host-lint/pre-commit .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
```

The hook runs the `host-lint` binary against the commit message (commit-msg stage) and staged files (pre-commit stage).

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

The `host-lint` binary is a statically linked Linux amd64 executable committed to this directory. It requires no runtime dependencies. For other platforms, compile from `host-lint.rs` with `rustc`.
