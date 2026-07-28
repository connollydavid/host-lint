//! The commit-message lane (host-lint#22, msg-lane).
//!
//! Every check here was measured against 500 accepted upstream subjects before it
//! was encoded, and the measurement changed the design twice: it named an exemption
//! shape the plan had missed, and it showed the area rule cannot block.
//!
//! The corpus is `fixtures/upstream/accepted-subjects.txt`. Accepted subjects are
//! the only honest basis for a tier, because they are what upstream's own reviewers
//! let through. A rule that flags them is wrong about upstream, not about the commit.

use crate::rules::Tier;

/// One thing the lane found in one message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub rule: &'static str,
    pub tier: Tier,
    pub detail: String,
}

/// Subjects that are exempt from the area-prefix rule because git itself, or
/// upstream practice, produces them in that shape.
///
/// `Reapply` is here because the measurement put it here: it appears in accepted
/// history and the design's exemption list did not name it. `fixup!` and `squash!`
/// never appear in accepted history at all, since they are squashed before merge —
/// they are exempt so the lane can run on a branch before submission, which is the
/// only time they exist.
const AREA_EXEMPT_PREFIXES: &[&str] = &["Revert \"", "Reapply \"", "fixup! ", "squash! "];

/// The vague subjects upstream names verbatim as unacceptable in
/// doc/developer.texi, Patches/Committing, "Commit messages". Enumerated exact
/// strings rather than a cleverness heuristic: upstream named these, and anything
/// beyond them would be this checker's opinion wearing upstream's authority.
const VAGUE_EXACT: &[&str] = &["fixed!", "fixed", "changed it.", "changed it", "fix", "update"];

/// Whether the subject carries an `area: description` prefix.
///
/// The grammar is wider than one identifier because upstream's real subjects are:
/// `avcodec/h264: ...`, `Makefile, ffbuild/{common,library}: ...`,
/// `.forgejo/actions/rebase-pr: ...`. Measured at 489 of 500 bare, and 495 of 500
/// once the exemptions above are applied.
pub fn has_area_prefix(subject: &str) -> bool {
    if AREA_EXEMPT_PREFIXES.iter().any(|p| subject.starts_with(p)) {
        return true;
    }
    let Some((head, rest)) = subject.split_once(':') else {
        return false;
    };
    if head.is_empty() || !rest.starts_with(' ') || rest.trim().is_empty() {
        return false;
    }
    // The area is a path-ish token list. A colon inside prose ("note: see below")
    // is excluded by requiring every character to be path-shaped.
    head.chars().all(|c| {
        c.is_ascii_alphanumeric() || "_./{},*+- ".contains(c)
    })
}

/// Check one commit message. `signoff_required` is the project's mode rather than
/// upstream's rule: FFmpeg does not require a sign-off, but a project consuming
/// this pack may.
/// `tracker_required` is a project mode, not upstream's rule. Upstream says a
/// change addressing a known bug must cite it, and nothing can tell from a message
/// alone whether a change addresses one. A project that knows its own answer (every
/// commit must trace to an issue) can ask for the check; this pack will not guess.
pub fn check_with(message: &str, signoff_required: bool, tracker_required: bool) -> Vec<Finding> {
    let mut out = Vec::new();
    let mut lines = message.lines();
    let subject = lines.next().unwrap_or("").trim_end();

    if subject.is_empty() {
        out.push(Finding {
            rule: "commit-msg-format",
            tier: Tier::Mechanical,
            detail: "the message has no subject line".to_string(),
        });
        return out;
    }

    // Heuristic, and the measurement is why: 5 of 500 accepted subjects carry no
    // area prefix and are not exempt. A mechanical tier here would reject work
    // upstream itself accepted, roughly once every hundred commits.
    if !has_area_prefix(subject) {
        out.push(Finding {
            rule: "commit-msg-format",
            tier: Tier::Heuristic,
            detail: format!("subject has no `area: description` prefix: {subject:?}"),
        });
    }

    // Mechanical: measured at zero occurrences in 500 accepted subjects, so a
    // non-ascii subject is not something upstream is quietly doing.
    if let Some(c) = subject.chars().find(|c| !c.is_ascii()) {
        out.push(Finding {
            rule: "commit-msg-ascii",
            tier: Tier::Mechanical,
            detail: format!("subject carries a non-ascii character {c:?}"),
        });
    }

    let described = subject
        .split_once(':')
        .map(|(_, r)| r.trim())
        .unwrap_or(subject)
        .trim_end_matches('.')
        .to_ascii_lowercase();
    if VAGUE_EXACT.contains(&described.as_str()) {
        out.push(Finding {
            rule: "commit-msg-has-body",
            tier: Tier::Mechanical,
            detail: format!("upstream names this subject as unacceptable: {described:?}"),
        });
    }

    let body: Vec<&str> = lines.collect();
    let has_body = body.iter().any(|l| !l.trim().is_empty());
    if has_body && !body.first().map(|l| l.trim().is_empty()).unwrap_or(true) {
        out.push(Finding {
            rule: "commit-msg-format",
            tier: Tier::Mechanical,
            detail: "the subject and body are not separated by a blank line".to_string(),
        });
    }

    if signoff_required
        && !body
            .iter()
            .any(|l| l.trim_start().starts_with("Signed-off-by:"))
    {
        out.push(Finding {
            rule: "commit-msg-signoff",
            tier: Tier::Mechanical,
            detail: "the project requires a Signed-off-by line and there is none".to_string(),
        });
    }

    if tracker_required && !cites_tracker(message) {
        out.push(Finding {
            rule: "commit-msg-cites-tracker",
            tier: Tier::Mechanical,
            detail: "the project requires a tracker or CVE reference and there is none".to_string(),
        });
    }

    out
}

/// Whether the message cites a tracker item or a CVE. Forgejo issue references
/// join the legacy Trac shapes because the primary tracker moved; a checker that
/// knew only the old form would report every new citation as absent.
pub fn cites_tracker(message: &str) -> bool {
    let m = message.to_ascii_lowercase();
    m.contains("cve-")
        || m.contains("trac.ffmpeg.org")
        || m.contains("ticket #")
        || m.contains("fixes #")
        || m.contains("closes #")
        || m.contains("code.ffmpeg.org")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The must-pass corpus. Every subject here was accepted upstream, so any
    /// MECHANICAL finding against one of them is this checker being wrong about
    /// FFmpeg rather than FFmpeg being wrong.
    const ACCEPTED: &str = include_str!("../fixtures/upstream/accepted-subjects.txt");

    #[test]
    fn no_mechanical_rule_fires_on_accepted_upstream_history() {
        let mut wrong = Vec::new();
        for s in ACCEPTED.lines().filter(|l| !l.trim().is_empty()) {
            for f in check_with(s, false, false) {
                if f.tier == Tier::Mechanical {
                    wrong.push(format!("{s:?} -> {} ({})", f.rule, f.detail));
                }
            }
        }
        assert!(
            wrong.is_empty(),
            "{} accepted subject(s) hit a blocking rule:\n  {}",
            wrong.len(),
            wrong.join("\n  ")
        );
    }

    /// The heuristic tier is allowed to fire on accepted history, and the rate is
    /// the thing being asserted. If upstream practice shifts far from this, the
    /// test fails and the tier gets re-examined rather than quietly drifting.
    #[test]
    fn the_area_rule_matches_the_measured_rate() {
        let subs: Vec<&str> = ACCEPTED.lines().filter(|l| !l.trim().is_empty()).collect();
        let total = subs.len();
        let flagged = subs.iter().filter(|s| !has_area_prefix(s)).count();
        let rate = (total - flagged) as f64 / total as f64;
        assert!(
            rate > 0.98,
            "area prefix held for {}/{} ({rate:.3}); measured 0.990 on 2026-07-28",
            total - flagged,
            total
        );
        assert!(
            flagged > 0,
            "no accepted subject is flagged, which would mean the rule cannot fire at all"
        );
    }

    #[test]
    fn the_exemptions_are_the_measured_ones() {
        assert!(has_area_prefix("Revert \"avformat/concatdec: Check recursion depth\""));
        // Named by the measurement rather than by the design: it appears in
        // accepted history and the plan's exemption list did not have it.
        assert!(has_area_prefix("Reapply \"avfilter/avfiltergraph: always retry\""));
        assert!(has_area_prefix("fixup! avcodec/h264: fix it"));
        assert!(has_area_prefix("squash! avcodec/h264: fix it"));
    }

    #[test]
    fn real_upstream_area_shapes_are_accepted() {
        for s in [
            "avcodec/h264dec: fix a leak",
            "Makefile, ffbuild/{common,library}: Allow to build DEVTOOLS",
            ".forgejo/actions/rebase-pr: workaround stale value",
            "avfilter/vf_scale, swscale: share the context",
        ] {
            assert!(has_area_prefix(s), "should accept {s:?}");
        }
    }

    #[test]
    fn prose_subjects_are_not_read_as_an_area() {
        assert!(!has_area_prefix("Add FATE tests for stale metadata"));
        assert!(!has_area_prefix("avcodec/h264:no space after the colon"));
        assert!(!has_area_prefix("avcodec/h264: "));
    }

    /// The limit of a shape-only rule, asserted so it is a known property rather
    /// than a surprise. `note:` is indistinguishable from an area prefix without a
    /// tree to check the area against, and this checker has no tree. Tightening it
    /// means resolving the area against real paths and MAINTAINERS, which is a
    /// different check with its own false-negative cost; it is not done here, and
    /// pretending otherwise would be worse than saying so.
    #[test]
    fn a_prose_word_before_a_colon_is_indistinguishable_from_an_area() {
        assert!(has_area_prefix("note: this is prose, and the rule cannot tell"));
    }

    #[test]
    fn each_synthetic_violation_fires_its_rule_exactly_once() {
        let cases: &[(&str, &str)] = &[
            ("Add a thing with no area", "commit-msg-format"),
            ("avcodec/h264: fixed!", "commit-msg-has-body"),
            ("avcodec/h264: café encoding", "commit-msg-ascii"),
        ];
        for (msg, rule) in cases {
            let hits: Vec<_> = check_with(msg, false, false).into_iter().filter(|f| f.rule == *rule).collect();
            assert_eq!(hits.len(), 1, "{msg:?} should fire {rule} once, got {hits:?}");
        }
    }

    #[test]
    fn a_body_must_be_separated_from_the_subject() {
        let bad = "avcodec/h264: fix a leak\nthe body starts immediately";
        assert!(check_with(bad, false, false).iter().any(|f| f.detail.contains("blank line")));
        let good = "avcodec/h264: fix a leak\n\nthe body is separated";
        assert!(!check_with(good, false, false).iter().any(|f| f.detail.contains("blank line")));
    }

    #[test]
    fn signoff_is_the_projects_rule_not_upstreams() {
        let m = "avcodec/h264: fix a leak\n\nwhy it leaked";
        assert!(check_with(m, false, false).is_empty(), "FFmpeg itself requires no sign-off");
        assert!(check_with(m, true, false).iter().any(|f| f.rule == "commit-msg-signoff"));
        let signed = "avcodec/h264: fix a leak\n\nwhy\n\nSigned-off-by: A <a@b>";
        assert!(!check_with(signed, true, false).iter().any(|f| f.rule == "commit-msg-signoff"));
    }

    #[test]
    fn the_tracker_requirement_is_the_projects_rule_not_upstreams() {
        let m = "avcodec/h264: fix a leak\n\nwhy it leaked";
        assert!(check_with(m, false, false).is_empty(), "upstream requires no citation here");
        assert!(check_with(m, false, true).iter().any(|f| f.rule == "commit-msg-cites-tracker"));
        let cited = "avcodec/h264: fix a leak\n\nFixes #42";
        assert!(!check_with(cited, false, true).iter().any(|f| f.rule == "commit-msg-cites-tracker"));
    }

    #[test]
    fn tracker_citations_cover_the_moved_tracker() {
        assert!(cites_tracker("fixes CVE-2024-1234"));
        assert!(cites_tracker("see https://trac.ffmpeg.org/ticket/1234"));
        assert!(cites_tracker("see https://code.ffmpeg.org/FFmpeg/FFmpeg/issues/42"));
        assert!(cites_tracker("Fixes #42"));
        assert!(!cites_tracker("no reference at all"));
    }

    #[test]
    fn an_empty_subject_is_reported_and_stops_there() {
        let f = check_with("", false, false);
        assert_eq!(f.len(), 1);
        assert!(f[0].detail.contains("no subject"));
    }
}
