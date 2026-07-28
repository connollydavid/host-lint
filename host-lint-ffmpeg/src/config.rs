//! The project pack config (host-lint#22, project-pack-config).
//!
//! A consuming project tells the pack what it is checking against: which upstream
//! ref, how strict to be, and what its branch and tag names must look like. None of
//! that is upstream's rule, so none of it is encoded in the registry.
//!
//! **Worktree-first resolution** is the whole point of the loader. A clone with
//! several worktrees can hold one worktree on a frozen series and another on live
//! work, and those need different modes at the same time. So the worktree's own
//! config wins over the repository's, and a check that read only the repository root
//! would apply one worktree's mode to the other.
//!
//! The parser is written out rather than pulling in a TOML crate: this crate carries
//! no crates.io dependencies, which is what the vendored bundle and the offline
//! reproducible build rest on. The schema is a flat set of `key = "value"` lines
//! under one optional `[ffmpeg]` table, which is all the schema needs to be.

use std::path::Path;

/// How strict the pack is in this tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Everything reports; nothing blocks. The mode a project adopts on day one,
    /// before it has cleaned up its history.
    Advise,
    /// Mechanical rules block, heuristics advise. The ordinary working mode.
    Enforce,
    /// A frozen series: the history is a fixed reference and must not be rewritten,
    /// so findings report and never block however severe.
    Frozen,
}

impl Mode {
    fn parse(s: &str) -> Option<Mode> {
        match s {
            "advise" => Some(Mode::Advise),
            "enforce" => Some(Mode::Enforce),
            "frozen" => Some(Mode::Frozen),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Mode::Advise => "advise",
            Mode::Enforce => "enforce",
            Mode::Frozen => "frozen",
        }
    }

    /// Whether a mechanical finding blocks in this mode. Frozen never blocks: the
    /// history is a reference, and a gate demanding it be rewritten would be asking
    /// for the one thing the mode exists to forbid.
    pub fn blocks(self) -> bool {
        self == Mode::Enforce
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    /// The upstream ref this project checks against.
    pub upstream_ref: String,
    pub mode: Mode,
    /// A prefix every branch name must start with, if the project sets one.
    pub branch_prefix: Option<String>,
    /// A prefix every tag must start with, if the project sets one.
    pub tag_prefix: Option<String>,
    /// Where this config was read from, so a surprising mode can be traced.
    pub source: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            upstream_ref: "master".to_string(),
            mode: Mode::Advise,
            branch_prefix: None,
            tag_prefix: None,
            source: "(defaults)".to_string(),
        }
    }
}

/// Parse the flat schema. Unknown keys are an error rather than ignored: a typo'd
/// `mode` silently leaving the tree in advise mode is exactly the quiet failure this
/// config exists to make loud.
pub fn parse(text: &str, source: &str) -> Result<Config, String> {
    let mut c = Config { source: source.to_string(), ..Default::default() };
    let mut saw_mode = false;
    for (n, raw) in text.lines().enumerate() {
        let line = raw.split('#').next().unwrap_or("").trim();
        if line.is_empty() || line.starts_with('[') {
            continue;
        }
        let Some((k, v)) = line.split_once('=') else {
            return Err(format!("{source}:{}: not a key = value line: {raw:?}", n + 1));
        };
        let key = k.trim();
        let val = v.trim().trim_matches('"').to_string();
        match key {
            "upstream_ref" => c.upstream_ref = val,
            "mode" => {
                c.mode = Mode::parse(&val)
                    .ok_or_else(|| format!("{source}:{}: unknown mode {val:?}; expected advise, enforce or frozen", n + 1))?;
                saw_mode = true;
            }
            "branch_prefix" => c.branch_prefix = Some(val),
            "tag_prefix" => c.tag_prefix = Some(val),
            _ => return Err(format!("{source}:{}: unknown key {key:?}", n + 1)),
        }
    }
    let _ = saw_mode;
    Ok(c)
}

/// Resolve the config for a working directory, worktree first.
///
/// The order is the worktree's own file, then the repository's common directory, then
/// defaults. `git rev-parse --git-common-dir` names the shared directory even from a
/// linked worktree, which is how the two levels stay distinguishable.
pub fn load(dir: &Path) -> Result<Config, String> {
    let worktree = dir.join(".host-lint-ffmpeg.toml");
    if worktree.is_file() {
        let text = std::fs::read_to_string(&worktree)
            .map_err(|e| format!("cannot read {}: {e}", worktree.display()))?;
        return parse(&text, &worktree.display().to_string());
    }

    if let Some(common) = git_common_dir(dir) {
        let shared = common.join("host-lint-ffmpeg.toml");
        if shared.is_file() {
            let text = std::fs::read_to_string(&shared)
                .map_err(|e| format!("cannot read {}: {e}", shared.display()))?;
            return parse(&text, &shared.display().to_string());
        }
    }

    Ok(Config::default())
}

fn git_common_dir(dir: &Path) -> Option<std::path::PathBuf> {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["rev-parse", "--git-common-dir"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let p = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if p.is_empty() {
        return None;
    }
    let path = std::path::PathBuf::from(&p);
    Some(if path.is_absolute() { path } else { dir.join(path) })
}

/// Whether a branch name satisfies the project's grammar.
pub fn branch_ok(c: &Config, branch: &str) -> bool {
    c.branch_prefix.as_ref().is_none_or(|p| branch.starts_with(p))
}

/// Whether a tag satisfies the project's grammar.
pub fn tag_ok(c: &Config, tag: &str) -> bool {
    c.tag_prefix.as_ref().is_none_or(|p| tag.starts_with(p))
}

/// Whether this branch is frozen, derived from history tags rather than declared.
///
/// A series is frozen once it has been tagged: the tag is a promise that those bytes
/// are a fixed reference. Derived rather than configured because a declaration goes
/// stale the moment somebody tags without editing the config, and the tag is the
/// fact.
pub fn is_frozen_branch(dir: &Path, c: &Config, branch: &str) -> bool {
    if c.mode == Mode::Frozen {
        return true;
    }
    let Some(prefix) = c.tag_prefix.as_deref() else {
        return false;
    };
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["tag", "--merged", branch])
        .output();
    match out {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout)
            .lines()
            .any(|t| t.trim().starts_with(prefix)),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn tmp(name: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("hlf-cfg-{}-{name}", std::process::id()));
        fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn the_defaults_are_the_cautious_ones() {
        let c = Config::default();
        assert_eq!(c.mode, Mode::Advise);
        assert!(!c.mode.blocks(), "a project that configured nothing is never blocked");
        assert_eq!(c.upstream_ref, "master");
    }

    #[test]
    fn frozen_never_blocks_however_severe() {
        // The mode exists to protect a fixed history. A gate demanding it be
        // rewritten would ask for the one thing the mode forbids.
        assert!(!Mode::Frozen.blocks());
        assert!(Mode::Enforce.blocks());
        assert!(!Mode::Advise.blocks());
    }

    #[test]
    fn a_typo_is_an_error_rather_than_a_silent_default() {
        assert!(parse("mode = \"enfroce\"\n", "x").is_err());
        assert!(parse("modee = \"enforce\"\n", "x").is_err());
        assert!(parse("not a pair\n", "x").is_err());
        // The error names the file and line so it can be found.
        let e = parse("mode = \"nope\"\n", "proj.toml").unwrap_err();
        assert!(e.contains("proj.toml:1"), "{e}");
    }

    #[test]
    fn comments_the_table_header_and_blank_lines_are_skipped() {
        let c = parse("# a comment\n[ffmpeg]\n\nmode = \"enforce\"  # trailing\n", "x").unwrap();
        assert_eq!(c.mode, Mode::Enforce);
    }

    #[test]
    fn the_worktree_config_wins_over_the_repository() {
        // The precedence that matters: one clone, two worktrees, different modes.
        let root = tmp("prec");
        let wt = root.join("wt");
        fs::create_dir_all(&wt).unwrap();
        fs::write(root.join("host-lint-ffmpeg.toml"), "mode = \"enforce\"\n").unwrap();
        fs::write(wt.join(".host-lint-ffmpeg.toml"), "mode = \"frozen\"\n").unwrap();

        let c = load(&wt).unwrap();
        assert_eq!(c.mode, Mode::Frozen, "the worktree's own file must win");
        assert!(c.source.contains("wt"), "{}", c.source);
        fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn with_no_config_anywhere_the_defaults_are_used_and_said_so() {
        let d = tmp("empty");
        let c = load(&d).unwrap();
        assert_eq!(c.mode, Mode::Advise);
        assert_eq!(c.source, "(defaults)");
        fs::remove_dir_all(&d).ok();
    }

    #[test]
    fn branch_and_tag_grammars_are_checked_only_when_set() {
        let mut c = Config::default();
        assert!(branch_ok(&c, "anything"), "no grammar means no constraint");
        assert!(tag_ok(&c, "anything"));

        c.branch_prefix = Some("pgs".to_string());
        c.tag_prefix = Some("pgs-v".to_string());
        assert!(branch_ok(&c, "pgs9-wip"));
        assert!(!branch_ok(&c, "feature/x"));
        assert!(tag_ok(&c, "pgs-v9"));
        assert!(!tag_ok(&c, "v9"));
    }

    #[test]
    fn a_frozen_branch_is_derived_from_its_tags() {
        let d = tmp("frozen");
        let run = |args: &[&str]| {
            std::process::Command::new("git")
                .arg("-C").arg(&d).args(args)
                .output().expect("git runs");
        };
        run(&["init", "-q", "."]);
        fs::write(d.join("f"), "x\n").unwrap();
        run(&["add", "-A"]);
        run(&["-c", "user.email=t@t", "-c", "user.name=t", "commit", "-qm", "init"]);

        let mut c = Config { tag_prefix: Some("pgs-v".to_string()), ..Default::default() };
        let branch = String::from_utf8_lossy(
            &std::process::Command::new("git").arg("-C").arg(&d)
                .args(["rev-parse", "--abbrev-ref", "HEAD"]).output().unwrap().stdout,
        ).trim().to_string();

        assert!(!is_frozen_branch(&d, &c, &branch), "untagged history is not frozen");
        run(&["tag", "pgs-v9"]);
        assert!(is_frozen_branch(&d, &c, &branch), "a tag is the fact that freezes it");

        // A tag outside the grammar does not freeze anything.
        let d2 = tmp("frozen2");
        std::process::Command::new("git").arg("-C").arg(&d2).args(["init","-q","."]).output().unwrap();
        fs::write(d2.join("f"), "x\n").unwrap();
        for a in [vec!["add","-A"], vec!["-c","user.email=t@t","-c","user.name=t","commit","-qm","i"], vec!["tag","v9"]] {
            std::process::Command::new("git").arg("-C").arg(&d2).args(a).output().unwrap();
        }
        assert!(!is_frozen_branch(&d2, &c, &branch), "a tag outside the grammar is not the project's freeze");

        c.mode = Mode::Frozen;
        assert!(is_frozen_branch(&d, &c, &branch), "an explicit frozen mode is honoured too");
        fs::remove_dir_all(&d).ok();
        fs::remove_dir_all(&d2).ok();
    }
}
