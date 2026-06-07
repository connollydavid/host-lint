# no-phase-skill

Detect phase-synonym agentic tells in commit messages, markdown headers, and code comments.

An anti-slop linter that flags LLM-generated plan language bleeding into source code. Implements the detection rules from [VOCABULARY.md](VOCABULARY.md).

## What It Detects

Flags numbered phase-synonym patterns:

- `## Phase 1: Setup` — markdown header
- `// Pass 1: tokenize` — code comment
- `Step 3 of 5` — prose
- `feat: phase 2 of auth refactor` — conventional commit with phase tell
- `Stage II, data migration` — Roman numeral

Does NOT flag:

- `feat: add parser` — conventional commit, no phase tell
- `TODO: handle null input` — code tag
- `increment the retry counter` — verb, no numeral
- `the first pass over the array` — ordinal, not numeral

## Usage

### CLI

```bash
# Scan commit message from stdin
echo "Phase 1: setup" | ./no-phase --stdin

# Scan specific files
./no-phase README.md src/main.rs

# JSON output
./no-phase --json --stdin

# Scan all tracked files
./no-phase --all
```

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
./test-integration.sh ./target/release/no-phase

# Conformance gates (G1-G8 for SKILL.md)
./lint-skill.sh
```

## Specification

The detection behavior is formally specified in [no-phase.allium](no-phase.allium) (Allium format).

## License

Unlicense
