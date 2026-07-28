//! The mixed cosmetic/functional classifier (host-lint#22, cosmetic-separation).
//!
//! Upstream's rule is that cosmetic changes belong in their own commit, so the check
//! is not "is this cosmetic" but "does this commit mix the two". The distinction is
//! drawn the way git-howto draws it: a hunk that vanishes under `diff -w -b`
//! normalisation carried no functional change.
//!
//! Three allowances are encoded because upstream grants them, and each is a fixture:
//!
//!   - **Braces.** Adding or removing braces around a single statement changes no
//!     behaviour, and upstream treats it as cosmetic rather than as a mixed commit.
//!   - **Whitespace-only.** A commit that is nothing but whitespace is cosmetic in
//!     full, and clean rather than mixed.
//!   - **Blank lines.** Adding or removing an empty line is whitespace, and a commit
//!     that only does that is not mixed.
//!
//! The failure this classifier exists to prevent is the reverse of the obvious one.
//! Reporting a pure re-indentation as "mixed" would push authors to bundle the
//! re-indent INTO the functional commit to silence the tool, which is the opposite of
//! the rule.

use crate::diff::is_golden_output;
use crate::rules::Tier;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// Nothing but whitespace, braces or blank lines. Cosmetic in full.
    Cosmetic,
    /// Changes behaviour, and carries no cosmetic noise worth separating.
    Functional,
    /// Both, in one commit. This is the reportable state.
    Mixed,
    /// No changed lines at all.
    Empty,
}

#[derive(Debug, Clone)]
pub struct Finding {
    pub rule: &'static str,
    pub tier: Tier,
    pub detail: String,
}

/// Normalise a line the way `diff -w -b` compares them: all whitespace collapsed
/// away entirely, so indentation and inter-token spacing cannot register.
fn normalize(line: &str) -> String {
    line.chars().filter(|c| !c.is_whitespace()).collect()
}

/// Whether a normalised line is only brace or delimiter punctuation.
fn is_delimiter_only(norm: &str) -> bool {
    !norm.is_empty() && norm.chars().all(|c| matches!(c, '{' | '}' | ';'))
}

/// The brace allowance needs a second, looser comparison. Wrapping one statement in
/// braces both ADDS a `}` line and changes `if (a)` into `if (a) {`, so the first
/// normalisation leaves the modified line looking functional. Stripping braces as
/// well as whitespace lets the pair cancel, which is what makes the allowance an
/// allowance rather than a special case for the standalone `}`.
/// Strips braces ONLY. Non-golden lines reached this already whitespace-normalised,
/// so touching whitespace here would strip it a second time — and that would defeat
/// the golden-output exemption, whose lines arrive raw precisely because their
/// whitespace is the data under test.
fn normalize_braces(line: &str) -> String {
    line.chars().filter(|c| !matches!(c, '{' | '}')).collect()
}

/// Classify one unified diff.
pub fn classify(diff: &str) -> Kind {
    let mut removed: Vec<String> = Vec::new();
    let mut added: Vec<String> = Vec::new();
    let mut path = "";
    let mut any = false;

    for line in diff.lines() {
        if let Some(p) = line.strip_prefix("+++ b/") {
            path = p;
            continue;
        }
        if line.starts_with("---") || line.starts_with("@@") || path.is_empty() {
            continue;
        }
        // Golden output is data: a changed expectation is functional, and its
        // whitespace must never be normalised away.
        let golden = is_golden_output(path);
        if let Some(b) = line.strip_prefix('+') {
            if line.starts_with("+++") {
                continue;
            }
            any = true;
            added.push(if golden { b.to_string() } else { normalize(b) });
        } else if let Some(b) = line.strip_prefix('-') {
            if line.starts_with("---") {
                continue;
            }
            any = true;
            removed.push(if golden { b.to_string() } else { normalize(b) });
        }
    }

    if !any {
        return Kind::Empty;
    }

    // Cancel in two passes. First on whitespace alone, which catches re-indentation
    // and spacing. Then on whitespace-and-braces, which catches the brace allowance.
    // A line surviving both passes changed something that is not layout.
    let mut left = removed.clone();
    let mut func_added: Vec<String> = Vec::new();
    for a in &added {
        if let Some(i) = left.iter().position(|r| r == a) {
            left.remove(i);
        } else {
            func_added.push(a.clone());
        }
    }

    // Every cancellation is a line that was MODIFIED and whose modification was
    // layout only. That count is what "cosmetic change present" means; the first
    // version of this treated any brace or blank line on the functional side as
    // cosmetic, and reported 213 of 300 accepted commits as mixed, because adding a
    // new function adds braces and blank lines. A rule firing on 71% of accepted
    // work would be muted the day it shipped.
    let mut cancelled = removed.len() - left.len();

    let mut left2: Vec<String> = left.iter().map(|l| normalize_braces(l)).collect();
    let mut func2: Vec<String> = Vec::new();
    for a in &func_added {
        let ab = normalize_braces(a);
        if let Some(i) = left2.iter().position(|r| *r == ab) {
            left2.remove(i);
            cancelled += 1;
        } else {
            func2.push(ab);
        }
    }
    let (func_added, left) = (func2, left2);

    // A blank line or a bare delimiter left over is structure belonging to whatever
    // else changed, not a functional change in its own right.
    let functional: Vec<&String> = func_added
        .iter()
        .chain(left.iter())
        .filter(|l| !l.trim().is_empty() && !is_delimiter_only(l))
        .collect();

    // Only a layout-only modification of existing code counts. Pure additions carry
    // no cosmetic component: there is no previous formatting to have changed.
    let cosmetic_present = cancelled > 0;

    match (functional.is_empty(), cosmetic_present) {
        (true, _) => Kind::Cosmetic,
        (false, true) => Kind::Mixed,
        (false, false) => Kind::Functional,
    }
}

/// Split a unified diff into hunks, each carrying the `+++ b/` path in scope.
fn hunks(diff: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut path = String::new();
    let mut cur: Option<String> = None;
    for line in diff.lines() {
        if let Some(p) = line.strip_prefix("+++ b/") {
            if let Some(h) = cur.take() {
                out.push(h);
            }
            path = p.to_string();
            continue;
        }
        if line.starts_with("@@") {
            if let Some(h) = cur.take() {
                out.push(h);
            }
            cur = Some(format!("--- a/{path}\n+++ b/{path}\n{line}\n"));
            continue;
        }
        if let Some(h) = cur.as_mut() {
            h.push_str(line);
            h.push('\n');
        }
    }
    if let Some(h) = cur.take() {
        out.push(h);
    }
    out
}

/// Report a commit that mixes the two, judged PER HUNK.
///
/// The unit matters more than the test. Judged over the whole diff, any functional
/// change that reformats the lines it touches reads as mixed, and 108 of 300 accepted
/// commits were reported that way. Upstream's rule is about bundling an unrelated
/// re-indent with a fix, so the reportable shape is one hunk that is purely cosmetic
/// sitting beside another that is functional. Reformatting inside the hunk you are
/// already changing is not what the rule prohibits.
///
/// Heuristic regardless: a separate hunk can still be formatting the functional
/// change required, and no diff carries that intent.
pub fn check(diff: &str) -> Vec<Finding> {
    let kinds: Vec<Kind> = hunks(diff).iter().map(|h| classify(h)).collect();
    let cosmetic_hunks = kinds.iter().filter(|k| **k == Kind::Cosmetic).count();
    let functional_hunks = kinds
        .iter()
        .filter(|k| matches!(k, Kind::Functional | Kind::Mixed))
        .count();

    if cosmetic_hunks > 0 && functional_hunks > 0 {
        return vec![Finding {
            rule: "cosmetic-separate",
            tier: Tier::Heuristic,
            detail: format!(
                "{cosmetic_hunks} hunk(s) change only formatting while {functional_hunks} change behaviour; upstream asks for them in separate patches"
            ),
        }];
    }
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn diff(path: &str, body: &str) -> String {
        format!("--- a/{path}\n+++ b/{path}\n@@ -1,8 +1,8 @@\n{body}")
    }

    /// Fixture one: a pure re-indentation. Cosmetic, and reporting it as mixed
    /// would push the author to bundle it into the functional commit.
    #[test]
    fn pure_reindent_is_cosmetic() {
        let d = diff("libavcodec/h264.c", "-  int x = compute(a, b);\n+    int x = compute(a, b);\n");
        assert_eq!(classify(&d), Kind::Cosmetic);
        assert!(check(&d).is_empty());
    }

    /// Fixture two: a real change and nothing else.
    #[test]
    fn pure_functional_is_functional() {
        let d = diff("libavcodec/h264.c", "-    int x = compute(a, b);\n+    int x = compute(b, a);\n");
        assert_eq!(classify(&d), Kind::Functional);
        assert!(check(&d).is_empty());
    }

    /// Fixture three: the reportable state is a cosmetic-only hunk sitting beside a
    /// functional one, which is the bundling upstream's rule is about.
    #[test]
    fn a_cosmetic_hunk_beside_a_functional_one_is_reported() {
        let d = concat!(
            "--- a/libavcodec/h264.c\n+++ b/libavcodec/h264.c\n",
            "@@ -1,2 +1,2 @@\n-  int x = compute(a, b);\n+    int x = compute(a, b);\n",
            "@@ -40,2 +40,2 @@\n-    int y = 1;\n+    int y = 2;\n"
        );
        let f = check(d);
        assert_eq!(f.len(), 1, "{f:?}");
        assert_eq!(f[0].tier, Tier::Heuristic);
    }

    /// The counterpart decision, and the reason the unit is the hunk. Reformatting
    /// the lines you are already changing is not what the rule prohibits, and judging
    /// the whole diff at once reported 108 of 300 accepted commits as mixed.
    #[test]
    fn reformatting_inside_the_hunk_you_are_changing_is_not_reported() {
        let d = diff(
            "libavcodec/h264.c",
            "-  int x = compute(a, b);\n-    int y = 1;\n+    int x = compute(a, b);\n+    int y = 2;\n",
        );
        assert_eq!(classify(&d), Kind::Mixed, "the hunk itself does mix");
        assert!(check(&d).is_empty(), "but one hunk is not the bundling the rule forbids");
    }

    /// Fixture four: the brace allowance.
    #[test]
    fn adding_braces_around_one_statement_is_cosmetic() {
        let d = diff(
            "libavcodec/h264.c",
            "-    if (a)\n-        b();\n+    if (a) {\n+        b();\n+    }\n",
        );
        assert_eq!(classify(&d), Kind::Cosmetic);
    }

    /// Fixture five: the whitespace-only relaxation.
    #[test]
    fn a_whitespace_only_commit_is_cosmetic_in_full() {
        let d = diff("libavcodec/h264.c", "-int x=1;\n+int x = 1;\n");
        assert_eq!(classify(&d), Kind::Cosmetic);
    }

    /// Fixture six: blank lines alone.
    #[test]
    fn a_blank_line_change_alone_is_cosmetic() {
        let d = diff("libavcodec/h264.c", "+\n");
        assert_eq!(classify(&d), Kind::Cosmetic);
    }

    /// Golden output is data, so its whitespace is never normalised away: changing
    /// an expected value IS the functional change, and treating it as cosmetic would
    /// hide the one edit a reviewer most needs to see.
    #[test]
    fn golden_output_whitespace_is_functional_not_cosmetic() {
        let d = diff("tests/ref/fate/x", "-TAG:brand=qt\n+TAG:brand=qt  \n");
        assert_eq!(classify(&d), Kind::Functional);
    }

    #[test]
    fn an_empty_diff_is_empty_rather_than_cosmetic() {
        assert_eq!(classify("--- a/x\n+++ b/x\n@@ -1,0 +1,0 @@\n"), Kind::Empty);
        assert_eq!(classify(""), Kind::Empty);
    }
}
