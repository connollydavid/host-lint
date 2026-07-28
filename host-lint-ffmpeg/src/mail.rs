//! The mail lane (host-lint#22, mail-lane).
//!
//! Checks a `git format-patch` directory before it is sent. Everything here is about
//! mail that arrives WRONG rather than mail that arrives late, because the failure
//! mode upstream cares about is silent: an oversize message stalls in an unwatched
//! moderation queue, an HTML message is unreviewable, a missing Cc means the
//! maintainer of the touched file never learns the patch exists.
//!
//! Two things are deliberately NOT here. Whether the sender is subscribed, and
//! whether replies are interleaved rather than top-posted, are attested: no artefact
//! carries the answer.

use crate::maintainers;
use crate::rules::Tier;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub rule: &'static str,
    pub tier: Tier,
    pub file: String,
    pub detail: String,
}

/// One message in a series.
#[derive(Debug, Clone)]
pub struct Message {
    pub file: String,
    pub subject: String,
    pub from: String,
    pub to: Vec<String>,
    pub cc: Vec<String>,
    /// `Content-Type`, when the message declares one. `format-patch` output declares
    /// none, which is plain text by default and correct.
    pub content_type: Option<String>,
    /// The composed prose: the part an author wrote, above the diff. The wrap
    /// advisory applies to this and never to the payload, because rewrapping a diff
    /// corrupts it.
    pub prose: Vec<String>,
    pub bytes: usize,
    /// `n/m` from a `[PATCH n/m]` subject, when present.
    pub numbering: Option<(usize, usize)>,
    pub in_reply_to: Option<String>,
}

/// Upstream's per-message ceiling. Above this a message stalls in moderation and the
/// author is not told, which is why this is a flag rather than an advisory.
pub const MAX_MESSAGE_BYTES: usize = 1000 * 1024;

/// The prose wrap advisory.
pub const WRAP_COLUMNS: usize = 70;

/// Parse one `format-patch` file.
pub fn parse_message(file: &str, text: &str) -> Result<Message, String> {
    let mut m = Message {
        file: file.to_string(),
        subject: String::new(),
        from: String::new(),
        to: Vec::new(),
        cc: Vec::new(),
        content_type: None,
        prose: Vec::new(),
        bytes: text.len(),
        numbering: None,
        in_reply_to: None,
    };

    let mut lines = text.lines().peekable();
    // `format-patch` opens with a `From <sha> Mon Sep 17 ...` line. Its absence means
    // this is not a patch file, which is a parse error rather than a clean message.
    match lines.peek() {
        Some(l) if l.starts_with("From ") => {}
        _ => return Err(format!("{file}: not a format-patch message (no `From <sha>` line)")),
    }
    lines.next();

    let mut in_headers = true;
    let mut last_header = String::new();
    for line in lines {
        if in_headers {
            if line.trim().is_empty() {
                in_headers = false;
                continue;
            }
            // A continuation line begins with whitespace and extends the last header.
            if line.starts_with(char::is_whitespace) && !last_header.is_empty() {
                append_header(&mut m, &last_header, line.trim());
                continue;
            }
            if let Some((k, v)) = line.split_once(':') {
                last_header = k.trim().to_ascii_lowercase();
                append_header(&mut m, &last_header, v.trim());
            }
            continue;
        }
        // The payload begins at the diffstat separator or the first `diff --git`.
        if line == "---" || line.starts_with("diff --git") {
            break;
        }
        m.prose.push(line.to_string());
    }

    if m.subject.is_empty() {
        return Err(format!("{file}: no Subject header"));
    }
    m.numbering = parse_numbering(&m.subject);
    Ok(m)
}

fn append_header(m: &mut Message, key: &str, value: &str) {
    match key {
        "subject" => {
            if m.subject.is_empty() {
                m.subject = value.to_string();
            } else {
                m.subject.push(' ');
                m.subject.push_str(value);
            }
        }
        "from" => m.from = value.to_string(),
        "to" => m.to.extend(split_addresses(value)),
        "cc" => m.cc.extend(split_addresses(value)),
        "content-type" => m.content_type = Some(value.to_string()),
        "in-reply-to" => m.in_reply_to = Some(value.to_string()),
        _ => {}
    }
}

fn split_addresses(v: &str) -> Vec<String> {
    v.split(',')
        .map(|a| {
            let a = a.trim();
            match (a.find('<'), a.find('>')) {
                (Some(i), Some(j)) if j > i => a[i + 1..j].to_string(),
                _ => a.to_string(),
            }
        })
        .filter(|a| !a.is_empty())
        .collect()
}

/// `[PATCH 2/5] ...` yields `(2, 5)`. A bare `[PATCH]` yields None, which is a single
/// patch rather than a broken series.
fn parse_numbering(subject: &str) -> Option<(usize, usize)> {
    let open = subject.find('[')?;
    let close = subject.find(']')?;
    let inside = subject.get(open + 1..close)?;
    let frac = inside.split_whitespace().find(|t| t.contains('/'))?;
    let (a, b) = frac.split_once('/')?;
    Some((a.parse().ok()?, b.parse().ok()?))
}

/// Check one message on its own.
pub fn check_message(m: &Message, touched: &[&str], maint: &[maintainers::Entry]) -> Vec<Finding> {
    let mut out = Vec::new();

    if m.bytes > MAX_MESSAGE_BYTES {
        out.push(Finding {
            rule: "mail-oversize",
            tier: Tier::Mechanical,
            file: m.file.clone(),
            detail: format!(
                "{} bytes exceeds the {} kB ceiling; an oversize message stalls in an unwatched moderation queue and the sender is not told",
                m.bytes,
                MAX_MESSAGE_BYTES / 1024
            ),
        });
    }

    if let Some(ct) = &m.content_type {
        let lower = ct.to_ascii_lowercase();
        if !lower.starts_with("text/plain") {
            out.push(Finding {
                rule: "mail-not-plain-text",
                tier: Tier::Mechanical,
                file: m.file.clone(),
                detail: format!("Content-Type is {ct:?}; a patch that is not text/plain cannot be applied or reviewed inline"),
            });
        }
    }

    for (i, line) in m.prose.iter().enumerate() {
        if line.chars().count() > WRAP_COLUMNS {
            out.push(Finding {
                rule: "mail-prose-wrap",
                tier: Tier::Heuristic,
                file: m.file.clone(),
                detail: format!("composed line {} is {} characters; wrap prose at {WRAP_COLUMNS}", i + 1, line.chars().count()),
            });
            break;
        }
    }

    // Single-list addressing: exactly one FFmpeg list across To and Cc. Two lists
    // means a thread that forks and reviewers who see half of it.
    let lists: Vec<&String> = m
        .to
        .iter()
        .chain(m.cc.iter())
        .filter(|a| a.contains("@ffmpeg.org"))
        .collect();
    if lists.len() > 1 {
        out.push(Finding {
            rule: "mail-multiple-lists",
            tier: Tier::Mechanical,
            file: m.file.clone(),
            detail: format!("addressed to {} FFmpeg lists; a series sent to two lists forks its own thread", lists.len()),
        });
    }

    let required = maintainers::required_cc(maint, touched);
    for addr in required {
        if !m.cc.iter().any(|c| c.eq_ignore_ascii_case(&addr)) {
            out.push(Finding {
                rule: "mail-missing-maintainer-cc",
                tier: Tier::Mechanical,
                file: m.file.clone(),
                detail: format!("{addr} maintains a touched path and asked to be CC-ed, and is not in the Cc list"),
            });
        }
    }

    // A From: domain publishing a strict DMARC policy has its mail bounced off the
    // list. Advisory because the policy is a DNS fact this lane does not read.
    if let Some(domain) = from_domain(&m.from) {
        const STRICT: &[&str] = &["yahoo.com", "aol.com"];
        if STRICT.iter().any(|d| domain.eq_ignore_ascii_case(d)) {
            out.push(Finding {
                rule: "mail-dmarc-risk",
                tier: Tier::Heuristic,
                file: m.file.clone(),
                detail: format!("{domain} publishes a strict DMARC policy, and mail from it bounces off ffmpeg-devel"),
            });
        }
    }

    out
}

/// Check a whole series: numbering coherence, and the cover letter.
pub fn check_series(msgs: &[Message]) -> Vec<Finding> {
    let mut out = Vec::new();
    if msgs.is_empty() {
        return out;
    }

    let numbered: Vec<&Message> = msgs.iter().filter(|m| m.numbering.is_some()).collect();
    if let Some(first) = numbered.first() {
        let (_, total) = first.numbering.unwrap();
        // Every message must agree on the total, or a reviewer cannot tell when the
        // series is complete.
        for m in &numbered {
            let (_, t) = m.numbering.unwrap();
            if t != total {
                out.push(Finding {
                    rule: "mail-numbering",
                    tier: Tier::Mechanical,
                    file: m.file.clone(),
                    detail: format!("claims a series of {t} where the first message claims {total}"),
                });
            }
        }
        // And the indices must be exactly 1..=total, with the cover letter at 0.
        let mut seen: Vec<usize> = numbered.iter().map(|m| m.numbering.unwrap().0).collect();
        seen.sort_unstable();
        let expected: Vec<usize> = (0..=total).collect();
        let expected_no_cover: Vec<usize> = (1..=total).collect();
        if seen != expected && seen != expected_no_cover {
            out.push(Finding {
                rule: "mail-numbering",
                tier: Tier::Mechanical,
                file: numbered[0].file.clone(),
                detail: format!("the series numbers are {seen:?}, which is not a complete 1..{total}"),
            });
        }
    }

    let has_cover = msgs
        .iter()
        .any(|m| m.numbering.map(|(n, _)| n == 0).unwrap_or(false));
    if msgs.len() > 2 && !has_cover {
        out.push(Finding {
            rule: "mail-no-cover-letter",
            tier: Tier::Heuristic,
            file: msgs[0].file.clone(),
            detail: format!("{} messages with no cover letter; a reviewer has nowhere to read what the series is for", msgs.len()),
        });
    }

    out
}

/// The domain of a `From:` header. `A <a@b.com>` yields `b.com`: taking everything
/// after the last `@` keeps the closing angle bracket, and the comparison then never
/// matches, which would make this check silently useless.
fn from_domain(from: &str) -> Option<&str> {
    let after = from.rsplit('@').next()?;
    Some(after.trim_end_matches('>').trim())
}

/// Whether an `--in-reply-to` target belongs to this series' own prior thread.
///
/// Threading a v2 onto an unrelated message hijacks somebody else's thread, and the
/// reviewers of the v1 never see it. The check needs the prior thread's message ids;
/// with none supplied it cannot judge and says so by returning None rather than
/// passing.
pub fn thread_target_ok(m: &Message, prior_thread_ids: &[&str]) -> Option<bool> {
    let target = m.in_reply_to.as_ref()?;
    if prior_thread_ids.is_empty() {
        return None;
    }
    Some(prior_thread_ids.iter().any(|id| target.contains(id)))
}

#[cfg(test)]
mod tests {
    use super::*;

    const MAINT: &str = include_str!("../fixtures/upstream/maintainers-live-syntax.txt");

    fn msg(subject: &str) -> String {
        format!(
            "From 8dbf7b5e Mon Sep 17 00:00:00 2001\nFrom: A <a@b.com>\nDate: Tue, 28 Jul 2026 03:04:53 +0100\nSubject: {subject}\n\nsome prose\n---\n libavfilter/vf_bwdif.c | 1 +\n\ndiff --git a/libavfilter/vf_bwdif.c b/libavfilter/vf_bwdif.c\n+int x;\n"
        )
    }

    #[test]
    fn a_real_format_patch_message_parses() {
        let m = parse_message("0001.patch", &msg("[PATCH 1/2] avfilter/vf_bwdif: add x")).unwrap();
        assert_eq!(m.subject, "[PATCH 1/2] avfilter/vf_bwdif: add x");
        assert_eq!(m.from, "A <a@b.com>");
        assert_eq!(m.numbering, Some((1, 2)));
        // The prose stops at the diffstat separator: the payload is not prose.
        assert_eq!(m.prose, vec!["some prose"]);
    }

    #[test]
    fn something_that_is_not_a_patch_is_a_parse_error() {
        // Not a clean verdict: a directory holding a stray file must not be reported
        // as a well-formed series.
        assert!(parse_message("x", "Subject: hello\n\nbody\n").is_err());
        assert!(parse_message("x", "").is_err());
    }

    #[test]
    fn a_message_with_no_subject_is_an_error() {
        let t = "From abc Mon Sep 17 00:00:00 2001\nFrom: A <a@b>\n\nbody\n";
        assert!(parse_message("x", t).is_err());
    }

    #[test]
    fn broken_numbering_is_reported() {
        let a = parse_message("1", &msg("[PATCH 1/3] a: x")).unwrap();
        let b = parse_message("2", &msg("[PATCH 2/2] a: y")).unwrap();
        let f = check_series(&[a, b]);
        assert!(f.iter().any(|x| x.rule == "mail-numbering"), "{f:?}");
    }

    #[test]
    fn a_complete_series_with_a_cover_letter_is_clean() {
        let c = parse_message("0", &msg("[PATCH 0/2] a: cover")).unwrap();
        let a = parse_message("1", &msg("[PATCH 1/2] a: x")).unwrap();
        let b = parse_message("2", &msg("[PATCH 2/2] a: y")).unwrap();
        let f = check_series(&[c, a, b]);
        assert!(f.is_empty(), "{f:?}");
    }

    #[test]
    fn a_missing_cover_letter_advises_on_a_long_series() {
        let ms: Vec<Message> = (1..=4)
            .map(|n| parse_message(&n.to_string(), &msg(&format!("[PATCH {n}/4] a: x"))).unwrap())
            .collect();
        let f = check_series(&ms);
        let hit = f.iter().find(|x| x.rule == "mail-no-cover-letter");
        assert!(hit.is_some(), "{f:?}");
        assert_eq!(hit.unwrap().tier, Tier::Heuristic);
    }

    #[test]
    fn an_oversize_message_flags() {
        let mut m = parse_message("1", &msg("[PATCH] a: x")).unwrap();
        m.bytes = MAX_MESSAGE_BYTES + 1;
        let f = check_message(&m, &[], &[]);
        assert!(f.iter().any(|x| x.rule == "mail-oversize"), "{f:?}");
    }

    #[test]
    fn an_html_message_flags_and_plain_text_does_not() {
        let mut m = parse_message("1", &msg("[PATCH] a: x")).unwrap();
        m.content_type = Some("text/html; charset=utf-8".to_string());
        assert!(check_message(&m, &[], &[]).iter().any(|x| x.rule == "mail-not-plain-text"));
        m.content_type = Some("text/plain; charset=UTF-8".to_string());
        assert!(check_message(&m, &[], &[]).iter().all(|x| x.rule != "mail-not-plain-text"));
        // format-patch declares no Content-Type, which is plain text and correct.
        m.content_type = None;
        assert!(check_message(&m, &[], &[]).iter().all(|x| x.rule != "mail-not-plain-text"));
    }

    #[test]
    fn two_lists_flag_and_one_does_not() {
        let mut m = parse_message("1", &msg("[PATCH] a: x")).unwrap();
        m.to = vec!["ffmpeg-devel@ffmpeg.org".to_string()];
        assert!(check_message(&m, &[], &[]).iter().all(|x| x.rule != "mail-multiple-lists"));
        m.cc = vec!["ffmpeg-security@ffmpeg.org".to_string()];
        assert!(check_message(&m, &[], &[]).iter().any(|x| x.rule == "mail-multiple-lists"));
    }

    #[test]
    fn a_maintainer_who_asked_must_be_cc_ed() {
        let maint = maintainers::parse(MAINT);
        let mut m = parse_message("1", &msg("[PATCH] avfilter/vf_bwdif: x")).unwrap();
        let touched = ["libavfilter/vf_bwdif.c"];

        let f = check_message(&m, &touched, &maint);
        assert!(
            f.iter().any(|x| x.rule == "mail-missing-maintainer-cc" && x.detail.contains("thomas.mundt@hr.de")),
            "{f:?}"
        );

        m.cc = vec!["thomas.mundt@hr.de".to_string()];
        assert!(check_message(&m, &touched, &maint).iter().all(|x| x.rule != "mail-missing-maintainer-cc"));

        // A path whose maintainer never asked requires nothing.
        assert!(check_message(&m, &["libavformat/aadec.c"], &maint)
            .iter()
            .all(|x| x.rule != "mail-missing-maintainer-cc"));
    }

    #[test]
    fn long_prose_advises_and_the_payload_never_does() {
        let mut m = parse_message("1", &msg("[PATCH] a: x")).unwrap();
        m.prose = vec!["x".repeat(WRAP_COLUMNS + 5)];
        assert!(check_message(&m, &[], &[]).iter().any(|x| x.rule == "mail-prose-wrap"));
        // A long line in the DIFF is payload. Rewrapping it would corrupt the patch,
        // so the parser must never have put it in prose in the first place.
        let long_diff = format!(
            "From abc Mon Sep 17 00:00:00 2001\nFrom: A <a@b>\nSubject: [PATCH] a: x\n\nshort\n---\ndiff --git a/x b/x\n+{}\n",
            "y".repeat(200)
        );
        let m2 = parse_message("2", &long_diff).unwrap();
        assert!(check_message(&m2, &[], &[]).iter().all(|x| x.rule != "mail-prose-wrap"));
    }

    #[test]
    fn a_hijacked_thread_is_reported_and_an_unknown_one_is_not_guessed() {
        let mut m = parse_message("1", &msg("[PATCH v2 1/1] a: x")).unwrap();
        m.in_reply_to = Some("<somebody-elses@example.com>".to_string());
        assert_eq!(thread_target_ok(&m, &["our-v1@example.com"]), Some(false));
        m.in_reply_to = Some("<our-v1@example.com>".to_string());
        assert_eq!(thread_target_ok(&m, &["our-v1@example.com"]), Some(true));
        // With no prior thread known, the lane cannot judge and says so rather than
        // passing, which is what stops a missing input reading as a clean result.
        assert_eq!(thread_target_ok(&m, &[]), None);
        // No In-Reply-To at all is not a threading question.
        m.in_reply_to = None;
        assert_eq!(thread_target_ok(&m, &["x"]), None);
    }

    #[test]
    fn a_strict_dmarc_domain_advises() {
        let mut m = parse_message("1", &msg("[PATCH] a: x")).unwrap();
        m.from = "A <a@yahoo.com>".to_string();
        let f = check_message(&m, &[], &[]);
        let hit = f.iter().find(|x| x.rule == "mail-dmarc-risk");
        assert!(hit.is_some(), "{f:?}");
        assert_eq!(hit.unwrap().tier, Tier::Heuristic);
    }

    #[test]
    fn folded_headers_are_joined() {
        let t = "From abc Mon Sep 17 00:00:00 2001\nFrom: A <a@b>\nSubject: [PATCH 1/1] avfilter:\n  a very long subject\nCc: one@x.com,\n  two@y.com\n\nprose\n---\n";
        let m = parse_message("x", t).unwrap();
        assert!(m.subject.contains("a very long subject"), "{:?}", m.subject);
        assert_eq!(m.cc, vec!["one@x.com", "two@y.com"]);
    }
}
