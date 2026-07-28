//! The forge lane (host-lint#22, forge-lane).
//!
//! `code.ffmpeg.org` relays a pull request to the mailing list, and **the PR title
//! becomes the relayed subject**. That single fact is why this lane exists: a title
//! that would be fine on a forge arrives on ffmpeg-devel as a commit subject, and the
//! area-prefix grammar applies to it exactly as it applies to a commit.
//!
//! The lane adds only the forge-specific surface. The message, diff and series lanes
//! already apply per commit, because landings are rebase or fast-forward with
//! verbatim messages, so nothing here re-checks a commit.
//!
//! Empirical grounding, and what a re-check reads. These endpoints are anonymous:
//!
//!   - `GET /api/v1/repos/FFmpeg/FFmpeg` — the repository settings, including whether
//!     merge commits are permitted (they are not, which is what makes the
//!     verbatim-message assumption above safe).
//!   - `GET /api/v1/repos/FFmpeg/FFmpeg/pulls?state=closed` — observed landings, for
//!     the title shapes that were actually accepted.
//!
//! The lane itself performs no network access: it reads metadata it is given, so it
//! runs offline and in a hook.

use crate::msg::has_area_prefix;
use crate::rules::Tier;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub rule: &'static str,
    pub tier: Tier,
    pub detail: String,
}

/// The forge metadata this lane judges.
pub struct Pr<'a> {
    pub title: &'a str,
    pub body: &'a str,
    /// Whether the forge marks it a draft, independently of the title.
    pub draft: bool,
}

/// The draft marker upstream uses in a title.
const DRAFT_MARKER: &str = "WIP:";

/// Check one pull request's metadata.
pub fn check(pr: &Pr) -> Vec<Finding> {
    let mut out = Vec::new();

    let title = pr.title.trim();
    let bare = title.strip_prefix(DRAFT_MARKER).unwrap_or(title).trim();

    if title.starts_with(DRAFT_MARKER) || pr.draft {
        out.push(Finding {
            rule: "forge-draft",
            tier: Tier::Heuristic,
            detail: "marked a draft; it will relay to the list as a draft and is not expected to land as is".to_string(),
        });
    }

    // A `[PATCH v2]` style title is mail-series syntax. On the forge the same PR is
    // reused per revision, so a version in the title means the author is treating the
    // forge like the mailing list, and the relayed subject carries the artefact.
    if versioned_title(bare) {
        out.push(Finding {
            rule: "forge-versioned-title",
            tier: Tier::Mechanical,
            detail: "the title carries a patch version; a forge revision reuses the same pull request, so the version does not belong in the relayed subject".to_string(),
        });
    }

    // The title becomes the relayed list subject, so the commit-subject grammar
    // applies. Heuristic for the same measured reason it is heuristic on a commit.
    if !has_area_prefix(bare) {
        // Name the observed shape where it applies: a pasted branch name is the
        // commonest title that had to be rewritten before landing, and saying so is
        // more use than repeating the grammar.
        let detail = if looks_like_branch_name(bare) {
            format!("the title looks like a pasted branch name, and it relays as the list subject: {bare:?}")
        } else {
            format!("the title relays as the list subject and has no `area: description` prefix: {bare:?}")
        };
        out.push(Finding { rule: "forge-title-grammar", tier: Tier::Heuristic, detail });
    }

    if pr.body.trim().is_empty() {
        out.push(Finding {
            rule: "forge-description-cover",
            tier: Tier::Heuristic,
            detail: "no description; it serves as the cover letter for the relayed series".to_string(),
        });
    }

    out
}

/// Whether the title looks like a branch name pasted in rather than a subject. A
/// branch name has no spaces and usually carries separators; it is the shape observed
/// most often on titles that had to be rewritten before landing.
pub fn looks_like_branch_name(title: &str) -> bool {
    let t = title.trim();
    !t.contains(' ') && (t.contains('-') || t.contains('_') || t.contains('/')) && !t.contains(':')
}

fn versioned_title(title: &str) -> bool {
    let t = title.trim_start();
    // `[PATCH v2] ...`, `[PATCH 3/5] ...`, or a bare `v2:` lead.
    if t.starts_with('[') {
        if let Some(close) = t.find(']') {
            let inside = &t[1..close];
            let lower = inside.to_ascii_lowercase();
            if lower.contains("patch") || lower.starts_with('v') {
                return true;
            }
        }
    }
    false
}

/// The rationale a reviewer needs must be in the commits, because a forge description
/// never enters git history. Reported when the description carries substantially more
/// than the commit messages do.
pub fn rationale_lands_in_commits(pr: &Pr, commit_bodies: &[&str]) -> Option<Finding> {
    let described = pr.body.split_whitespace().count();
    let committed: usize = commit_bodies.iter().map(|b| b.split_whitespace().count()).sum();
    if described > 40 && committed * 3 < described {
        return Some(Finding {
            rule: "forge-rationale-in-commits",
            tier: Tier::Heuristic,
            detail: format!(
                "the description carries {described} words and the commit messages {committed}; a forge description never enters git history"
            ),
        });
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pr<'a>(title: &'a str, body: &'a str) -> Pr<'a> {
        Pr { title, body, draft: false }
    }

    #[test]
    fn an_area_prefixed_title_passes() {
        let f = check(&pr("avcodec/h264: fix a leak", "why it leaked, at length"));
        assert!(f.is_empty(), "{f:?}");
    }

    #[test]
    fn a_bare_branch_name_title_flags() {
        // The shape observed on titles that had to be rewritten before landing.
        let f = check(&pr("fix-h264-leak", "why"));
        assert!(f.iter().any(|x| x.rule == "forge-title-grammar"), "{f:?}");
        assert!(looks_like_branch_name("fix-h264-leak"));
        assert!(!looks_like_branch_name("avcodec/h264: fix a leak"));
    }

    #[test]
    fn a_wip_title_notes_without_blocking() {
        let f = check(&pr("WIP: avcodec/h264: fix a leak", "why"));
        let draft = f.iter().find(|x| x.rule == "forge-draft");
        assert!(draft.is_some(), "{f:?}");
        assert_eq!(draft.unwrap().tier, Tier::Heuristic);
        // The marker must not make the grammar check misread the title.
        assert!(f.iter().all(|x| x.rule != "forge-title-grammar"), "{f:?}");
    }

    #[test]
    fn the_forge_draft_flag_counts_even_without_the_marker() {
        let p = Pr { title: "avcodec/h264: fix a leak", body: "why", draft: true };
        assert!(check(&p).iter().any(|x| x.rule == "forge-draft"));
    }

    #[test]
    fn a_versioned_title_flags() {
        for t in ["[PATCH v2] avcodec/h264: fix a leak", "[PATCH 3/5] avcodec/h264: fix", "[v3] avcodec/h264: fix"] {
            let f = check(&pr(t, "why"));
            assert!(
                f.iter().any(|x| x.rule == "forge-versioned-title"),
                "{t:?} should flag: {f:?}"
            );
        }
        assert!(check(&pr("avcodec/h264: fix a leak", "why"))
            .iter()
            .all(|x| x.rule != "forge-versioned-title"));
    }

    #[test]
    fn an_empty_description_advises_because_it_is_the_cover_letter() {
        let f = check(&pr("avcodec/h264: fix a leak", "   "));
        let hit = f.iter().find(|x| x.rule == "forge-description-cover");
        assert!(hit.is_some(), "{f:?}");
        assert_eq!(hit.unwrap().tier, Tier::Heuristic);
    }

    #[test]
    fn rationale_only_in_the_description_advises() {
        let long = "word ".repeat(60);
        let p = pr("avcodec/h264: fix a leak", &long);
        assert!(rationale_lands_in_commits(&p, &["fix"]).is_some());
        // Once the commits carry the reasoning, the advisory goes quiet.
        assert!(rationale_lands_in_commits(&p, &[&long]).is_none());
        // A short description is not the case this is about.
        let short = pr("avcodec/h264: fix", "brief");
        assert!(rationale_lands_in_commits(&short, &[""]).is_none());
    }
}
