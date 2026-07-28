//! Build receipts (host-lint#22, build-receipts).
//!
//! Some obligations cannot be judged from a diff: whether the tree compiles, whether
//! `--enable-shared` links, whether `-Wvla` is clean. Running them is expensive, so
//! the result is recorded and re-read.
//!
//! The whole design turns on one rule: **an unrun leg renders as unrun, never as
//! passed.** A receipt is evidence of what was done, and the absence of a leg is
//! evidence of nothing. The second rule follows from the first: a receipt whose head
//! SHA no longer matches is stale, and stale evidence is not evidence either.
//!
//! Receipts live in the git common directory rather than a worktree, so several
//! worktrees of one clone share them, and they survive a worktree being removed.

use crate::sha256;

/// What one leg's run established.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LegResult {
    Passed,
    Failed,
    /// Never run. Distinct from Failed on purpose: "we did not check" and "we checked
    /// and it broke" are different facts, and collapsing them is how an unrun leg
    /// becomes a green tick.
    Unrun,
}

impl LegResult {
    pub fn as_str(self) -> &'static str {
        match self {
            LegResult::Passed => "passed",
            LegResult::Failed => "failed",
            LegResult::Unrun => "unrun",
        }
    }

    fn parse(s: &str) -> LegResult {
        match s {
            "passed" => LegResult::Passed,
            "failed" => LegResult::Failed,
            _ => LegResult::Unrun,
        }
    }
}

/// The legs a receipt can carry.
pub const LEGS: &[&str] = &[
    "compile",
    "enable-shared",
    "wvla",
    "standalone-compile",
    "disable-x86asm",
    "out-of-tree",
    "fate",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Receipt {
    pub base: String,
    pub head: String,
    /// The toolchain the legs ran in. Two receipts for the same SHAs from different
    /// toolchains are different evidence.
    pub toolchain: String,
    /// A digest of the configure arguments, so a receipt from a different
    /// configuration is not read as covering this one.
    pub config_digest: String,
    pub legs: Vec<(String, LegResult)>,
}

impl Receipt {
    pub fn leg(&self, name: &str) -> LegResult {
        self.legs
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, r)| *r)
            .unwrap_or(LegResult::Unrun)
    }

    /// The export form: one `key value` per line, legs last. Chosen so it round-trips
    /// through a pipe and a human can read it in a terminal.
    pub fn export(&self) -> String {
        let mut s = String::new();
        s.push_str(&format!("base {}\n", self.base));
        s.push_str(&format!("head {}\n", self.head));
        s.push_str(&format!("toolchain {}\n", self.toolchain));
        s.push_str(&format!("config-digest {}\n", self.config_digest));
        for (n, r) in &self.legs {
            s.push_str(&format!("leg {} {}\n", n, r.as_str()));
        }
        s
    }

    pub fn parse(text: &str) -> Result<Receipt, String> {
        let mut r = Receipt {
            base: String::new(),
            head: String::new(),
            toolchain: String::new(),
            config_digest: String::new(),
            legs: Vec::new(),
        };
        for (n, line) in text.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let mut it = line.split_whitespace();
            let Some(key) = it.next() else { continue };
            match key {
                "base" => r.base = it.next().unwrap_or_default().to_string(),
                "head" => r.head = it.next().unwrap_or_default().to_string(),
                "toolchain" => r.toolchain = it.collect::<Vec<_>>().join(" "),
                "config-digest" => r.config_digest = it.next().unwrap_or_default().to_string(),
                "leg" => {
                    let name = it.next().unwrap_or_default().to_string();
                    let res = LegResult::parse(it.next().unwrap_or_default());
                    if name.is_empty() {
                        return Err(format!("line {}: a leg with no name", n + 1));
                    }
                    r.legs.push((name, res));
                }
                _ => return Err(format!("line {}: unknown key {key:?}", n + 1)),
            }
        }
        if r.head.is_empty() || r.base.is_empty() {
            return Err("a receipt must name its base and head".to_string());
        }
        Ok(r)
    }
}

/// A digest of the configure arguments.
pub fn config_digest(args: &[&str]) -> String {
    sha256::hex(args.join(" ").as_bytes())[..16].to_string()
}

/// Whether the receipt still describes this head.
pub fn is_stale(r: &Receipt, head_now: &str) -> bool {
    r.head != head_now
}

/// Where a receipt lives: the git common directory, so worktrees of one clone share
/// it and it outlives any single worktree.
pub fn receipt_path(common_dir: &std::path::Path, head: &str) -> std::path::PathBuf {
    common_dir.join("host-lint-ffmpeg-receipts").join(format!("{head}.receipt"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Receipt {
        Receipt {
            base: "aaaa1111".to_string(),
            head: "bbbb2222".to_string(),
            toolchain: "gcc 14.2 x86_64-linux-gnu".to_string(),
            config_digest: config_digest(&["--enable-gpl", "--disable-doc"]),
            legs: vec![
                ("compile".to_string(), LegResult::Passed),
                ("wvla".to_string(), LegResult::Passed),
                ("fate".to_string(), LegResult::Unrun),
            ],
        }
    }

    #[test]
    fn the_export_form_round_trips() {
        let r = sample();
        let back = Receipt::parse(&r.export()).unwrap();
        assert_eq!(r, back);
    }

    #[test]
    fn a_leg_that_was_never_run_reads_unrun_not_passed() {
        let r = sample();
        assert_eq!(r.leg("fate"), LegResult::Unrun);
        // A leg absent from the receipt entirely is also unrun, which is the case
        // that matters: silence must never read as success.
        assert_eq!(r.leg("out-of-tree"), LegResult::Unrun);
        assert_ne!(r.leg("out-of-tree"), LegResult::Passed);
    }

    #[test]
    fn failed_and_unrun_are_different_facts() {
        let mut r = sample();
        r.legs.push(("enable-shared".to_string(), LegResult::Failed));
        assert_eq!(r.leg("enable-shared"), LegResult::Failed);
        assert_ne!(r.leg("enable-shared"), r.leg("fate"));
    }

    #[test]
    fn a_rewritten_head_makes_the_receipt_stale() {
        let r = sample();
        assert!(!is_stale(&r, "bbbb2222"));
        assert!(is_stale(&r, "cccc3333"), "a rewritten head is not covered by old evidence");
    }

    #[test]
    fn a_different_configuration_is_a_different_digest() {
        let a = config_digest(&["--enable-gpl"]);
        let b = config_digest(&["--enable-gpl", "--enable-shared"]);
        assert_ne!(a, b, "a receipt from another configuration must not read as covering this one");
        assert_eq!(a.len(), 16);
    }

    #[test]
    fn a_receipt_without_its_shas_is_an_error() {
        assert!(Receipt::parse("toolchain gcc\n").is_err());
        assert!(Receipt::parse("base a\n").is_err());
        assert!(Receipt::parse("base a\nhead b\nnonsense x\n").is_err());
        assert!(Receipt::parse("base a\nhead b\nleg\n").is_err());
    }

    #[test]
    fn comments_and_blank_lines_parse() {
        let r = Receipt::parse("# written by the build lane\n\nbase a\nhead b\n").unwrap();
        assert_eq!(r.base, "a");
        assert!(r.legs.is_empty());
    }

    #[test]
    fn the_receipt_lives_in_the_common_dir_so_worktrees_share_it() {
        let p = receipt_path(std::path::Path::new("/repo/.git"), "abc123");
        assert!(p.to_string_lossy().contains("/repo/.git/host-lint-ffmpeg-receipts/"));
        assert!(p.to_string_lossy().ends_with("abc123.receipt"));
    }
}
