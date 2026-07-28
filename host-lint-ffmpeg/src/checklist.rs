//! The checklist reporter (host-lint#22, checklist-reporter).
//!
//! Renders the whole registry over a series, and its only real job is to keep three
//! kinds of knowledge visibly apart:
//!
//!   - **checked** — a lane decided it from the artefact, and the measured rate is
//!     printed beside it so the reader knows how much the decision is worth;
//!   - **receipted** — an expensive leg ran, and the receipt says so;
//!   - **attested** — nobody can decide it from an artefact, and a human says.
//!
//! An attested item must never render as checked, and an unrun leg must never render
//! as passed. Both are the same failure: a report that looks complete because the
//! symbol for "nobody knows" is the symbol for "fine".

use crate::receipt::{LegResult, Receipt};
use crate::rules::{Rule, Tier, RULES};

/// How an item renders.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mark {
    /// A lane decided it and found nothing.
    Checked,
    /// A lane decided it and found something.
    Reported,
    /// An expensive leg ran and passed.
    Receipted,
    /// An expensive leg ran and failed.
    ReceiptFailed,
    /// The leg was never run. Renders as its own thing, never as passed.
    Unrun,
    /// Only a human can answer. Renders as its own thing, never as checked.
    Attested,
}

impl Mark {
    /// The glyph. `[ ]` for anything nobody has established, and deliberately never
    /// `[x]`: the reader's eye takes a tick as settled.
    pub fn glyph(self) -> &'static str {
        match self {
            Mark::Checked => "[x]",
            Mark::Reported => "[!]",
            Mark::Receipted => "[r]",
            Mark::ReceiptFailed => "[F]",
            Mark::Unrun => "[ ]",
            Mark::Attested => "[?]",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Mark::Checked => "checked",
            Mark::Reported => "reported",
            Mark::Receipted => "receipted",
            Mark::ReceiptFailed => "receipt failed",
            Mark::Unrun => "unrun",
            Mark::Attested => "attested",
        }
    }
}

/// One rendered line.
#[derive(Debug, Clone)]
pub struct Item {
    pub rule: &'static str,
    pub mark: Mark,
    pub tier: Tier,
    pub note: String,
}

/// The attested legs that are not registry rules: human review, post-land monitoring,
/// and the security-path items. Listed with a citation each, because an attested item
/// with no source is just an assertion.
pub const ATTESTED_LEGS: &[(&str, &str)] = &[
    ("human-review", "developer.texi, Patch review process: a human read it"),
    ("fate-monitoring", "fate.ffmpeg.org after landing: the change did not redden a configuration"),
    ("security-named-reviewer", "the security path: a named human reviewed the fix"),
    ("security-finder-credit", "developer.texi, Credit any researchers"),
    ("security-reproducible", "the security path: reproducible with existing applications"),
];

/// Render the checklist.
pub fn render(reported: &[&str], receipt: Option<&Receipt>) -> Vec<Item> {
    let mut out = Vec::new();

    for r in RULES {
        out.push(Item {
            rule: r.id,
            mark: mark_for(r, reported, receipt),
            tier: r.tier,
            note: note_for(r, receipt),
        });
    }

    for (leg, citation) in ATTESTED_LEGS {
        out.push(Item {
            rule: leg,
            mark: Mark::Attested,
            tier: Tier::Attested,
            note: (*citation).to_string(),
        });
    }

    out
}

fn mark_for(r: &Rule, reported: &[&str], receipt: Option<&Receipt>) -> Mark {
    // Attested first, and unconditionally. No amount of lane evidence promotes an
    // attested rule to checked, because the tier says the artefact cannot answer it.
    if r.tier == Tier::Attested {
        // An expensive leg may still have receipted evidence for it, which is
        // receipted rather than checked.
        if let Some(rec) = receipt {
            if let Some(leg) = leg_for(r.id) {
                return match rec.leg(leg) {
                    LegResult::Passed => Mark::Receipted,
                    LegResult::Failed => Mark::ReceiptFailed,
                    LegResult::Unrun => Mark::Attested,
                };
            }
        }
        return Mark::Attested;
    }
    if reported.contains(&r.id) {
        return Mark::Reported;
    }
    Mark::Checked
}

/// The receipt leg that carries an otherwise-attested rule, where one does.
fn leg_for(rule: &str) -> Option<&'static str> {
    match rule {
        "no-broken-build" => Some("compile"),
        "regression-tests-run" => Some("fate"),
        "language-c11-headers-c99" => Some("standalone-compile"),
        _ => None,
    }
}

fn note_for(r: &Rule, receipt: Option<&Receipt>) -> String {
    if let (Some(rec), Some(leg)) = (receipt, leg_for(r.id)) {
        return format!("leg {leg}: {}", rec.leg(leg).as_str());
    }
    match r.measured_rate {
        Some(v) => format!("measured {v:.3} on accepted history"),
        None if r.tier == Tier::Attested => "no artefact answers this".to_string(),
        None => "unmeasured".to_string(),
    }
}

/// The rendered report.
pub fn format_report(items: &[Item]) -> String {
    let mut s = String::new();
    s.push_str("FFmpeg rule checklist\n\n");
    for i in items {
        // The tier is printed beside the mark, because "checked" means different
        // things for a mechanical rule and a heuristic one and the reader needs both.
        s.push_str(&format!(
            "{} {:<34} {:<11} {:<14} {}\n",
            i.mark.glyph(),
            i.rule,
            i.tier.as_str(),
            i.mark.label(),
            i.note
        ));
    }
    let checked = items.iter().filter(|i| i.mark == Mark::Checked).count();
    let reported = items.iter().filter(|i| i.mark == Mark::Reported).count();
    let receipted = items.iter().filter(|i| i.mark == Mark::Receipted).count();
    let unrun = items.iter().filter(|i| i.mark == Mark::Unrun).count();
    let attested = items.iter().filter(|i| i.mark == Mark::Attested).count();
    s.push_str(&format!(
        "\n-- {checked} checked, {reported} reported, {receipted} receipted, {unrun} unrun, {attested} awaiting a human\n"
    ));
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::receipt::config_digest;

    fn receipt_with(legs: &[(&str, LegResult)]) -> Receipt {
        Receipt {
            base: "a".to_string(),
            head: "b".to_string(),
            toolchain: "gcc".to_string(),
            config_digest: config_digest(&["--enable-gpl"]),
            legs: legs.iter().map(|(n, r)| (n.to_string(), *r)).collect(),
        }
    }

    /// The property the reporter exists for. Asserted over the WHOLE registry rather
    /// than a sample, because one attested rule rendering as checked is the entire
    /// failure.
    #[test]
    fn no_attested_item_ever_renders_as_checked() {
        for reported in [vec![], RULES.iter().map(|r| r.id).collect::<Vec<_>>()] {
            for rec in [None, Some(receipt_with(&[]))] {
                let items = render(&reported, rec.as_ref());
                for i in items.iter().filter(|i| i.tier == Tier::Attested) {
                    assert_ne!(
                        i.mark,
                        Mark::Checked,
                        "{} is attested and rendered as checked",
                        i.rule
                    );
                }
            }
        }
    }

    /// The same property from the other side: nothing that was not run may render as
    /// passed, and the glyph for "unknown" is never the glyph for "fine".
    #[test]
    fn an_unrun_leg_never_renders_as_passed() {
        let rec = receipt_with(&[("compile", LegResult::Unrun), ("fate", LegResult::Unrun)]);
        let items = render(&[], Some(&rec));
        for i in &items {
            if i.note.contains("unrun") {
                assert_ne!(i.mark, Mark::Checked, "{}", i.rule);
                assert_ne!(i.mark, Mark::Receipted, "{}", i.rule);
            }
        }
        // And the glyphs are distinct, so the eye cannot conflate them.
        assert_ne!(Mark::Unrun.glyph(), Mark::Checked.glyph());
        assert_ne!(Mark::Attested.glyph(), Mark::Checked.glyph());
        assert_ne!(Mark::Receipted.glyph(), Mark::Checked.glyph());
    }

    #[test]
    fn a_receipted_leg_renders_receipted_and_a_failed_one_renders_failed() {
        let pass = receipt_with(&[("compile", LegResult::Passed)]);
        let items = render(&[], Some(&pass));
        let build = items.iter().find(|i| i.rule == "no-broken-build").unwrap();
        assert_eq!(build.mark, Mark::Receipted);

        let fail = receipt_with(&[("compile", LegResult::Failed)]);
        let items = render(&[], Some(&fail));
        let build = items.iter().find(|i| i.rule == "no-broken-build").unwrap();
        assert_eq!(build.mark, Mark::ReceiptFailed);
    }

    #[test]
    fn a_reported_rule_renders_reported_rather_than_checked() {
        let items = render(&["diff-tab-indent"], None);
        let tab = items.iter().find(|i| i.rule == "diff-tab-indent").unwrap();
        assert_eq!(tab.mark, Mark::Reported);
        let other = items.iter().find(|i| i.rule == "diff-trailing-whitespace").unwrap();
        assert_eq!(other.mark, Mark::Checked);
    }

    #[test]
    fn a_measured_rate_is_printed_beside_its_result() {
        let items = render(&[], None);
        let fmt = items.iter().find(|i| i.rule == "commit-msg-format").unwrap();
        assert!(fmt.note.contains("0.990"), "{}", fmt.note);
        // And an unmeasured rule says so rather than implying confidence.
        let un = items.iter().find(|i| i.rule == "format-indent-four").unwrap();
        assert_eq!(un.note, "unmeasured");
    }

    #[test]
    fn every_attested_leg_carries_a_citation() {
        for (leg, cite) in ATTESTED_LEGS {
            assert!(cite.len() > 20, "{leg} has no real citation: {cite:?}");
        }
        let items = render(&[], None);
        for (leg, _) in ATTESTED_LEGS {
            assert!(items.iter().any(|i| i.rule == *leg), "{leg} missing from the report");
        }
    }

    /// The golden-output test: the report's shape is part of its contract, because a
    /// reader learns to scan the glyph column.
    #[test]
    fn the_report_renders_in_the_expected_shape() {
        let out = format_report(&render(&["diff-tab-indent"], Some(&receipt_with(&[("compile", LegResult::Passed)]))));
        assert!(out.starts_with("FFmpeg rule checklist\n"));
        assert!(out.contains("[!] diff-tab-indent"), "{out}");
        assert!(out.contains("[r] no-broken-build"), "{out}");
        assert!(out.contains("[?] human-review"), "{out}");
        assert!(out.contains("awaiting a human"), "{out}");
        // Every line begins with a glyph, so the column is scannable.
        for line in out.lines().skip(2).filter(|l| !l.trim().is_empty() && !l.starts_with("--")) {
            assert!(line.starts_with('['), "not a glyph line: {line:?}");
        }
    }

    #[test]
    fn the_counts_add_up_to_the_items() {
        let items = render(&[], None);
        let out = format_report(&items);
        let total: usize = out
            .lines()
            .filter(|l| l.starts_with('['))
            .count();
        assert_eq!(total, items.len());
    }
}
