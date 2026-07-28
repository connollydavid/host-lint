//! The encoded FFmpeg rule corpus (host-lint#22, rule-registry).
//!
//! Every rule here traces to a named subheading in a pinned upstream source, and
//! every rule-bearing section of that source is required to carry at least one
//! rule. The completeness test is what makes the corpus honest: upstream adding a
//! rule-bearing section reddens it rather than being silently absent, which is the
//! failure a hand-maintained checklist always reaches eventually.
//!
//! Three things are deliberately separated:
//!
//!   - the SOURCES, pinned by upstream commit and whole-file digest, so a fetch can
//!     be checked against what was encoded;
//!   - the SECTIONS, each with its own digest, so drift can be localised to the
//!     section that moved rather than reported as "the file changed";
//!   - the RULES, each naming the section and subheading it comes from.
//!
//! `measured_rate` is None on every mechanical rule and stays that way until the
//! calibration node measures it against accepted upstream history. A tier is a
//! claim about detectability, and an unmeasured claim is not evidence.

/// How reliably a rule can be judged from the artifact alone.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier {
    /// Decidable from the artifact with no judgement: a shape, a count, a presence.
    Mechanical,
    /// Decidable with a false-positive rate that has to be measured before the tier
    /// can be trusted. Reported, never blocking, until calibration says otherwise.
    Heuristic,
    /// Not mechanically decidable at all. The pack asks; a human attests. These can
    /// never render as checked by the tool, which the checklist reporter enforces.
    Attested,
}

impl Tier {
    pub fn as_str(self) -> &'static str {
        match self {
            Tier::Mechanical => "mechanical",
            Tier::Heuristic => "heuristic",
            Tier::Attested => "attested",
        }
    }
}

/// Which lane reads the rule. A rule with no lane yet is not encoded at all, so
/// this cannot silently become a bucket for rules nothing checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lane {
    /// The commit message alone.
    Msg,
    /// The diff of one commit.
    Diff,
    /// A series of commits together.
    Series,
    /// A mail-formatted submission.
    Mail,
    /// Requires building or running something.
    Build,
}

impl Lane {
    pub fn as_str(self) -> &'static str {
        match self {
            Lane::Msg => "msg",
            Lane::Diff => "diff",
            Lane::Series => "series",
            Lane::Mail => "mail",
            Lane::Build => "build",
        }
    }
}

/// One pinned upstream file.
#[derive(Debug, Clone, Copy)]
pub struct Source {
    pub path: &'static str,
    pub sha256: &'static str,
}

/// One heading in a pinned source, with its own digest.
#[derive(Debug, Clone, Copy)]
pub struct Section {
    pub source: &'static str,
    pub title: &'static str,
    pub sha256: &'static str,
    /// Carries at least one `@subheading` stating a rule. The completeness test
    /// requires every one of these to be mapped.
    pub rule_bearing: bool,
}

/// One encoded rule.
#[derive(Debug, Clone, Copy)]
pub struct Rule {
    pub id: &'static str,
    pub section: &'static str,
    pub subheading: &'static str,
    pub tier: Tier,
    pub lane: Lane,
    /// Measured true-positive rate over accepted upstream history. None until the
    /// calibration node runs; a tier without one is a declaration, not a finding.
    pub measured_rate: Option<f64>,
    pub summary: &'static str,
}

/// The upstream commit every pin below was taken from.
pub const UPSTREAM_COMMIT: &str = "c6309b5c63add7ad0ec221fafefc32bdcd6f8b91";

pub const SOURCES: &[Source] = &[
    Source { path: "doc/developer.texi", sha256: "26549522babfb7af744059d68077440064de6693ee6ff8c41b53ea3c36534e4c" },
    Source { path: "doc/mailing-list-faq.texi", sha256: "7ba38b8b16e6b94d6054d64da6dfa17738c874b693d85fe8ee9f62b36b7ac0f6" },
    Source { path: "MAINTAINERS", sha256: "55e05c3c17d7909886cbdd563154ff2d7a43bc5d0866bddb63df2adbc5aa0f58" },
    Source { path: "doc/fate.texi", sha256: "07b49cf2c33b20d04b828166ce569f6d7d57e4b2791b113bcd6b134e7960ecba" },
];

pub const SECTIONS: &[Section] = &[
    Section { source: "doc/developer.texi", title: "Introduction", sha256: "577941bfa97b46b860ebdfd4b3ba28a6741957b644480ee796558a1e60178c00", rule_bearing: false },
    Section { source: "doc/developer.texi", title: "Coding Rules", sha256: "fbc1bbb3997a096754d4b8fd3905374db462a2c617c8ea2cb10c603fc5f1eac8", rule_bearing: false },
    Section { source: "doc/developer.texi", title: "Language", sha256: "c72d2f2973ff3b7e54b76732a8f663a21959b29b05425542a54665195311fb3d", rule_bearing: false },
    Section { source: "doc/developer.texi", title: "SIMD/DSP", sha256: "33fb4c409ac0f78bb2d4c45a5839b3542bf2be861dbe7445f85d3c7bf0eae3f4", rule_bearing: false },
    Section { source: "doc/developer.texi", title: "Other languages", sha256: "8e3b20619d8af73fd344f707451820cfa5ac37336514ffa1997a4d0b852d733b", rule_bearing: false },
    Section { source: "doc/developer.texi", title: "Code formatting conventions", sha256: "9842e28dc568a6a5145160c4fac9c968f205782cc269504830bbf9093436dc4b", rule_bearing: false },
    Section { source: "doc/developer.texi", title: "Examples", sha256: "24347c31ec9c4eb18c80feb92790784d1f07f2d752cf927256705d6ce880807d", rule_bearing: false },
    Section { source: "doc/developer.texi", title: "Vim configuration", sha256: "d3630f81d24c6a5bf2cda21a91a7590f5243b4b271ff475b8bc8482ad34a68f9", rule_bearing: false },
    Section { source: "doc/developer.texi", title: "Emacs configuration", sha256: "19a7ed022f8118e74d0a1392d9d18360136ef7e2435946101379807fe9d65fca", rule_bearing: false },
    Section { source: "doc/developer.texi", title: "Comments", sha256: "3e25315b539bc37f14c76044c5977b78eb4052670d8a996dbbcb517c265a2308", rule_bearing: false },
    Section { source: "doc/developer.texi", title: "Naming conventions", sha256: "26a4bfd4289b10119ddc355fc773e6f81a6bc09ea4ac57a0dc028bc4bfa9465c", rule_bearing: false },
    Section { source: "doc/developer.texi", title: "Miscellaneous conventions", sha256: "f0a4db743feca4c08b8e2d9a3967916cdbdf5a7f41c738d1beae0bfb48cf7241", rule_bearing: false },
    Section { source: "doc/developer.texi", title: "Development Policy", sha256: "ab7cace5acde8e8514797b044fb683dae08acfb39477bda56a675793dd871b2a", rule_bearing: false },
    Section { source: "doc/developer.texi", title: "Code behaviour", sha256: "2881a255e63dbaa3a3f7bb86e0b8895ea5f4bb80dc9d0ea8a93e8b3977c1793e", rule_bearing: true },
    Section { source: "doc/developer.texi", title: "Patches/Committing", sha256: "653644de600130e555e01d86d8e130199d5f8abc954a9e98801b6c4ed4fb83e8", rule_bearing: true },
    Section { source: "doc/developer.texi", title: "Code", sha256: "8daf608e26e16e0a0e0212e0412ae237e255ce896020b962e52d3eff46969152", rule_bearing: true },
    Section { source: "doc/developer.texi", title: "Library public interfaces", sha256: "1d9ca664716d471ac943a516b69ee4792b22527565be8a6ca6974a57ab9fd4ca", rule_bearing: false },
    Section { source: "doc/developer.texi", title: "Adding new interfaces", sha256: "dc2425bdee62543d65a4339a2e4d427800fb07a3705419f2cf25a36b8a3aa05a", rule_bearing: false },
    Section { source: "doc/developer.texi", title: "Removing interfaces", sha256: "679712e49f1f997a71cd03de58c512525ab3d3652138a8a150793de3f8a444cd", rule_bearing: false },
    Section { source: "doc/developer.texi", title: "Major version bumps", sha256: "cc1f0821a4379c4b5dd5618a5d70896a50817237fb35be8a5859cdd35aa8aa08", rule_bearing: false },
    Section { source: "doc/developer.texi", title: "Documentation/Other", sha256: "cab3fc613c90f4e0e15c82d8e5b16111ad561597d52920888dba59ab8c6b6b70", rule_bearing: true },
    Section { source: "doc/developer.texi", title: "Submitting patches", sha256: "fa6c9023f38c1a5e4b3947f56f4b7b7fe841d09df955a7b6ca01c325f4caa34f", rule_bearing: true },
    Section { source: "doc/developer.texi", title: "New codecs or formats checklist", sha256: "0ceed207797567b9cffa34bb34ac06e200b16c4366f7f79cb0ba5998f996135c", rule_bearing: false },
    Section { source: "doc/developer.texi", title: "Patch submission checklist", sha256: "7b7a97f3c047254014b1eb8d96dd66ad515749134925e86336207ad69d47318d", rule_bearing: false },
    Section { source: "doc/developer.texi", title: "Patch review process", sha256: "dd46ba3ca97d3b5e63707e2ef13ca02670ce718b13724dbf2b3ff54e2972f49e", rule_bearing: false },
    Section { source: "doc/developer.texi", title: "Regression tests", sha256: "c181c869988bba4597b6e520edbf6ca736f0fd64c32a75b04a4e42842fab750f", rule_bearing: false },
    Section { source: "doc/developer.texi", title: "Adding files to the fate-suite dataset", sha256: "6b78ca24ae5c8294310cba3076e16b43b7442d65cced182986c6ddef38632644", rule_bearing: false },
    Section { source: "doc/developer.texi", title: "Visualizing Test Coverage", sha256: "6a00170dfbf143f1d5829bb016aaeb8508d16e1756a8906b88492550eba500a1", rule_bearing: false },
    Section { source: "doc/developer.texi", title: "Using Valgrind", sha256: "ff69f8e68d7128a64fb482fce6437daadbd9ac87456857ed5974e085573f1817", rule_bearing: false },
    Section { source: "doc/developer.texi", title: "Maintenance process", sha256: "667ea52eeb61b617354033f4717e83d7659ca67ada9c0305f340e2dd1debe252", rule_bearing: false },
    Section { source: "doc/developer.texi", title: "MAINTAINERS", sha256: "06baaf4fa8f3eff412a1ce541441571e9582f6917b91c18cd54282b0801aa913", rule_bearing: false },
    Section { source: "doc/developer.texi", title: "Becoming a maintainer", sha256: "56bce68a494dc0bbc0053158e1079d1d9456d022b82c856875e05997483662b0", rule_bearing: false },
    Section { source: "doc/developer.texi", title: "Release process", sha256: "5e0349bef871a5805dc628535dcd527033137c2f4c431cb2b9db87ecd849173c", rule_bearing: false },
    Section { source: "doc/developer.texi", title: "Criteria for Point Releases", sha256: "c8ee4a9afc59ec819cead0090442c334a006faf7f47359b5235ab1b4a6b0903f", rule_bearing: false },
    Section { source: "doc/developer.texi", title: "Release Checklist", sha256: "19da7c46882cd3c99a5ae9ae9253c1a97d165b5d52bcacf3d497d3eaa82e0ace", rule_bearing: false },
];

pub const RULES: &[Rule] = &[
    // --- Patches/Committing: the ten rules the message and diff lanes turn on ---
    Rule { id: "commit-msg-format", section: "Patches/Committing", subheading: "Commit messages",
           tier: Tier::Mechanical, lane: Lane::Msg, measured_rate: None,
           summary: "A commit message is `area changed: short 1 line description`, then a blank line, then detail saying what and why" },
    Rule { id: "commit-msg-has-body", section: "Patches/Committing", subheading: "Commit messages",
           tier: Tier::Heuristic, lane: Lane::Msg, measured_rate: None,
           summary: "The body explains what and why rather than restating the subject; `fixed!` and `Changed it.` are named upstream as unacceptable" },
    Rule { id: "commit-msg-cites-tracker", section: "Patches/Committing", subheading: "Commit messages",
           tier: Tier::Mechanical, lane: Lane::Msg, measured_rate: None,
           summary: "A change addressing a known bug or CVE carries the identifier, in addition to the explanation and never instead of it" },
    Rule { id: "patch-license-compatible", section: "Patches/Committing", subheading: "Licenses for patches must be compatible with FFmpeg.",
           tier: Tier::Heuristic, lane: Lane::Diff, measured_rate: None,
           summary: "A new file carries a proper license header, LGPL 2.1 or later preferred, copied from an existing file rather than from a random place" },
    Rule { id: "no-broken-build", section: "Patches/Committing", subheading: "You must not commit code which breaks FFmpeg!",
           tier: Tier::Attested, lane: Lane::Build, measured_rate: None,
           summary: "The tree builds and FATE passes before pushing; unfinished code may only land disabled" },
    Rule { id: "testing-proportionate", section: "Patches/Committing", subheading: "Testing must be adequate but not excessive.",
           tier: Tier::Attested, lane: Lane::Build, measured_rate: None,
           summary: "Tests cover the change without inflating FATE beyond what the change warrants" },
    Rule { id: "one-change-per-commit", section: "Patches/Committing", subheading: "Do not commit unrelated changes together.",
           tier: Tier::Heuristic, lane: Lane::Diff, measured_rate: None,
           summary: "One commit carries one logical change; unrelated edits are split" },
    Rule { id: "backport-stays-focused", section: "Patches/Committing", subheading: "Bug fixes intended for backporting should stay focused.",
           tier: Tier::Heuristic, lane: Lane::Diff, measured_rate: None,
           summary: "A fix meant for a release branch carries the fix and nothing beside it, which is narrower than the unrelated-changes rule" },
    Rule { id: "cosmetic-separate", section: "Patches/Committing", subheading: "Cosmetic changes should be kept in separate patches.",
           tier: Tier::Heuristic, lane: Lane::Diff, measured_rate: None,
           summary: "Re-indentation and whitespace live in their own commit, so a functional diff stays reviewable" },
    Rule { id: "credit-the-author", section: "Patches/Committing", subheading: "Credit the author of the patch.",
           tier: Tier::Mechanical, lane: Lane::Msg, measured_rate: None,
           summary: "A patch applied on someone's behalf records them as author rather than the committer" },
    Rule { id: "credit-researchers", section: "Patches/Committing", subheading: "Credit any researchers",
           tier: Tier::Mechanical, lane: Lane::Msg, measured_rate: None,
           summary: "A fix for a reported issue credits the reporter or researcher by name" },
    Rule { id: "wait-before-push", section: "Patches/Committing", subheading: "Always wait long enough before pushing changes",
           tier: Tier::Attested, lane: Lane::Series, measured_rate: None,
           summary: "Review time is allowed to elapse before a push; only the author knows when it did" },

    // --- Code behaviour ---
    Rule { id: "correctness", section: "Code behaviour", subheading: "Correctness",
           tier: Tier::Attested, lane: Lane::Build, measured_rate: None,
           summary: "The change is correct, which no checker decides" },
    Rule { id: "thread-and-library-safety", section: "Code behaviour", subheading: "Thread- and library-safety",
           tier: Tier::Attested, lane: Lane::Build, measured_rate: None,
           summary: "Code is safe to use from a library and under threads" },
    Rule { id: "robustness", section: "Code behaviour", subheading: "Robustness",
           tier: Tier::Attested, lane: Lane::Build, measured_rate: None,
           summary: "Input handling survives malformed and hostile data" },
    Rule { id: "memory-allocation", section: "Code behaviour", subheading: "Memory allocation",
           tier: Tier::Heuristic, lane: Lane::Diff, measured_rate: None,
           summary: "Allocations are checked and freed on every path" },
    Rule { id: "no-stdio", section: "Code behaviour", subheading: "stdio",
           tier: Tier::Mechanical, lane: Lane::Diff, measured_rate: None,
           summary: "Library code does not use stdio directly" },

    // --- Code ---
    Rule { id: "warning-suppression-last-resort", section: "Code", subheading: "Warnings for correct code may be disabled if there is no other option.",
           tier: Tier::Heuristic, lane: Lane::Diff, measured_rate: None,
           summary: "A warning is silenced only where correct code cannot be written otherwise" },

    // --- Documentation/Other ---
    Rule { id: "subscribe-devel", section: "Documentation/Other", subheading: "Subscribe to the ffmpeg-devel mailing list.",
           tier: Tier::Attested, lane: Lane::Mail, measured_rate: None,
           summary: "A committer is subscribed to ffmpeg-devel" },
    Rule { id: "subscribe-cvslog", section: "Documentation/Other", subheading: "Subscribe to the ffmpeg-cvslog mailing list.",
           tier: Tier::Attested, lane: Lane::Mail, measured_rate: None,
           summary: "A committer is subscribed to ffmpeg-cvslog and reads the replies to their commits" },
    Rule { id: "docs-current", section: "Documentation/Other", subheading: "Keep the documentation up to date.",
           tier: Tier::Heuristic, lane: Lane::Diff, measured_rate: None,
           summary: "A change to a documented behaviour updates its documentation in the same commit" },
    Rule { id: "discussion-in-public", section: "Documentation/Other", subheading: "Important discussions should be accessible to all.",
           tier: Tier::Attested, lane: Lane::Mail, measured_rate: None,
           summary: "Decisions are made where everyone can read them rather than in private mail" },
    Rule { id: "maintainers-entry-current", section: "Documentation/Other", subheading: "Check your entries in MAINTAINERS.",
           tier: Tier::Mechanical, lane: Lane::Diff, measured_rate: None,
           summary: "A maintainer's own MAINTAINERS entries are accurate" },

    // --- Submitting patches ---
    Rule { id: "send-email-setup", section: "Submitting patches", subheading: "How to setup git send-email?",
           tier: Tier::Attested, lane: Lane::Mail, measured_rate: None,
           summary: "Patches are sent with git send-email, configured as upstream documents" },
    Rule { id: "no-client-mangling", section: "Submitting patches", subheading: "Sending patches from email clients",
           tier: Tier::Mechanical, lane: Lane::Mail, measured_rate: None,
           summary: "A mailed patch is not mangled by the client: no flowed wrapping, no base64 body, no altered whitespace" },
    Rule { id: "review-replies-addressed", section: "Submitting patches", subheading: "Reviews",
           tier: Tier::Attested, lane: Lane::Series, measured_rate: None,
           summary: "Review comments are answered before a push" },
];

/// Every rule-bearing section, with the rules mapped to it. The completeness test
/// asserts this leaves nothing unmapped, which is what stops the corpus drifting
/// out of date silently when upstream adds a rule.
pub fn unmapped_sections() -> Vec<&'static str> {
    SECTIONS
        .iter()
        .filter(|s| s.rule_bearing)
        .filter(|s| !RULES.iter().any(|r| r.section == s.title))
        .map(|s| s.title)
        .collect()
}

/// Rules naming a section that is not in the table, which would be a rule with no
/// traceable source.
pub fn orphan_rules() -> Vec<&'static str> {
    RULES
        .iter()
        .filter(|r| !SECTIONS.iter().any(|s| s.title == r.section))
        .map(|r| r.id)
        .collect()
}

/// One section whose bytes no longer match the pin.
#[derive(Debug, Clone)]
pub struct Drift {
    pub what: String,
    /// Whether the moved section states rules. Only this kind gates: upstream
    /// re-indenting its Vim recipe is not rule drift, and a check that could not
    /// tell the difference would cry wolf on every unrelated upstream commit until
    /// somebody stopped reading it.
    pub rule_bearing: bool,
}

/// Sections whose digest differs from the pinned one, given a real FFmpeg tree.
/// Localised per section on purpose: "the file changed" is not actionable.
pub fn drifted_sections(tree: &std::path::Path) -> Result<Vec<Drift>, String> {
    let mut drifted = Vec::new();
    for src in SOURCES {
        let p = tree.join(src.path);
        let text = std::fs::read_to_string(&p)
            .map_err(|e| format!("cannot read {}: {e}", p.display()))?;
        if src.path != "doc/developer.texi" {
            if sha256_hex(text.as_bytes()) != src.sha256 {
                // These sources are not split into sections, so any change to them
                // is a change to material the rules rest on.
                drifted.push(Drift { what: format!("{} (whole file)", src.path), rule_bearing: true });
            }
            continue;
        }
        for (title, body) in split_sections(&text) {
            if let Some(pinned) = SECTIONS.iter().find(|s| s.source == src.path && s.title == title) {
                if sha256_hex(body.as_bytes()) != pinned.sha256 {
                    drifted.push(Drift {
                        what: format!("{}: {}", src.path, title),
                        rule_bearing: pinned.rule_bearing,
                    });
                }
            } else {
                // A section upstream added. Rule-bearing until somebody reads it:
                // an unknown section is exactly where a new rule would appear, and
                // treating it as harmless is how a corpus goes quietly out of date.
                drifted.push(Drift {
                    what: format!("{}: {} (new section, unmapped)", src.path, title),
                    rule_bearing: true,
                });
            }
        }
    }
    Ok(drifted)
}

/// Split a texinfo file into (heading, body) at chapter/section/subsection level,
/// the same boundaries the pinned digests were computed over.
pub fn split_sections(text: &str) -> Vec<(String, String)> {
    let mut marks: Vec<(usize, String)> = Vec::new();
    let mut offset = 0usize;
    for line in text.split_inclusive('\n') {
        let t = line.trim_end();
        for kw in ["@chapter ", "@section ", "@subsection "] {
            if let Some(rest) = t.strip_prefix(kw) {
                marks.push((offset, rest.trim().to_string()));
                break;
            }
        }
        offset += line.len();
    }
    let mut out = Vec::new();
    for (k, (start, title)) in marks.iter().enumerate() {
        let end = marks.get(k + 1).map(|(s, _)| *s).unwrap_or(text.len());
        out.push((title.clone(), text[*start..end].to_string()));
    }
    out
}

/// SHA-256, so the pack carries no dependency for one digest.
pub fn sha256_hex(data: &[u8]) -> String {
    crate::sha256::hex(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The completeness rule this registry exists for: upstream adding a
    /// rule-bearing section must redden a test rather than be silently absent. A
    /// hand-maintained checklist always drifts; this is what stops it.
    #[test]
    fn every_rule_bearing_section_is_mapped() {
        let unmapped = unmapped_sections();
        assert!(unmapped.is_empty(), "rule-bearing sections with no rule: {unmapped:?}");
    }

    /// The other direction: a rule must trace to a section that exists, or it is a
    /// claim about upstream with no source behind it.
    #[test]
    fn every_rule_traces_to_a_known_section() {
        let orphans = orphan_rules();
        assert!(orphans.is_empty(), "rules naming no known section: {orphans:?}");
    }

    /// Every rule names a subheading that section actually carries. Without this a
    /// typo'd subheading reads as a mapped rule and the completeness test passes
    /// over a rule pointing at nothing.
    #[test]
    fn rule_ids_are_unique() {
        let mut ids: Vec<&str> = RULES.iter().map(|r| r.id).collect();
        ids.sort_unstable();
        let before = ids.len();
        ids.dedup();
        assert_eq!(before, ids.len(), "duplicate rule id");
    }

    /// A mechanical or heuristic tier is a claim about detectability that the
    /// calibration node has to measure. Until it runs, no rule may carry a rate:
    /// a number nobody measured is worse than an empty field, because it reads as
    /// evidence.
    #[test]
    fn no_rule_claims_an_unmeasured_rate() {
        for r in RULES {
            assert!(
                r.measured_rate.is_none(),
                "{} carries a rate before the calibration node ran",
                r.id
            );
        }
    }

    /// An attested rule can never be decided by the tool. Encoding one into a
    /// mechanical lane would let the checklist reporter render it as checked, which
    /// is the hollow green the reporter is built to refuse.
    #[test]
    fn attested_rules_never_claim_a_mechanical_lane() {
        for r in RULES.iter().filter(|r| r.tier == Tier::Attested) {
            assert!(
                !matches!(r.lane, Lane::Diff),
                "{} is attested but sits in the diff lane, where the tool would decide it",
                r.id
            );
        }
    }

    /// The digests are computed over the same boundaries the splitter produces, so
    /// a real tree can be checked against them. Synthesized rather than shipped:
    /// this repository is Unlicense and FFmpeg is LGPL/GPL, so an upstream excerpt
    /// would need a provenance entry under fixtures/upstream (the licensing gate).
    #[test]
    fn the_splitter_matches_the_pinned_boundaries() {
        let doc = "@chapter One\nbody one\n@section Two\nbody two\n@subsection Three\nbody three\n";
        let parts = split_sections(doc);
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0].0, "One");
        assert_eq!(parts[1].0, "Two");
        assert_eq!(parts[2].0, "Three");
        // Each body runs to the next heading and no further.
        assert!(parts[0].1.contains("body one") && !parts[0].1.contains("body two"));
        assert!(parts[2].1.contains("body three"));
    }

    #[test]
    fn drift_is_reported_per_section_not_per_file() {
        let dir = std::env::temp_dir().join(format!("hlf-drift-{}", std::process::id()));
        let doc = dir.join("doc");
        std::fs::create_dir_all(&doc).unwrap();

        // A tree whose developer.texi holds one section, with a title the registry
        // knows and a body it does not. The report must name that section.
        std::fs::write(doc.join("developer.texi"), "@section Patches/Committing\nnot the pinned body\n").unwrap();
        for (p, _) in [("doc/mailing-list-faq.texi", 0), ("MAINTAINERS", 0), ("doc/fate.texi", 0)] {
            let f = dir.join(p);
            std::fs::create_dir_all(f.parent().unwrap()).unwrap();
            std::fs::write(f, "placeholder\n").unwrap();
        }

        let drifted = drifted_sections(&dir).unwrap();
        assert!(
            drifted.iter().any(|d| d.what.contains("Patches/Committing") && d.rule_bearing),
            "the moved section should be named: {drifted:?}"
        );
        // And the whole-file sources report as whole files, since only
        // developer.texi is split into sections.
        assert!(drifted.iter().any(|d| d.what.contains("MAINTAINERS")), "{drifted:?}");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn an_unknown_section_reports_as_unmapped_rather_than_being_ignored() {
        let dir = std::env::temp_dir().join(format!("hlf-new-{}", std::process::id()));
        let doc = dir.join("doc");
        std::fs::create_dir_all(&doc).unwrap();
        std::fs::write(doc.join("developer.texi"), "@section Brand New Upstream Section\nrules\n").unwrap();
        for p in ["doc/mailing-list-faq.texi", "MAINTAINERS", "doc/fate.texi"] {
            let f = dir.join(p);
            std::fs::create_dir_all(f.parent().unwrap()).unwrap();
            std::fs::write(f, "placeholder\n").unwrap();
        }
        let drifted = drifted_sections(&dir).unwrap();
        assert!(
            drifted.iter().any(|d| d.what.contains("new section, unmapped") && d.rule_bearing),
            "a section upstream added must surface: {drifted:?}"
        );
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn a_missing_source_is_an_error_not_an_empty_report() {
        let dir = std::env::temp_dir().join(format!("hlf-absent-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let e = drifted_sections(&dir).unwrap_err();
        assert!(e.contains("cannot read"), "{e}");
        std::fs::remove_dir_all(&dir).ok();
    }
}
