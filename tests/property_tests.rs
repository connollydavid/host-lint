use host_lint::{
    check_bare_numeral_header, check_code_label_prefix, check_label_prefix, check_line,
    check_warn, classify_line, gather_candidates,
    escalate_subject_decoration, is_numeral, is_strict_directive, parse_jira_keys,
    parse_lexicon_line, path_ignored, scan_prose_text, scan_text, scan_text_with_allow,
    scan_text_with_allow_strict, validate_lexicon_entry, LexiconEntry, Severity, WARN_NOUNS,
};
use proptest::prelude::*;
use std::fs;
use std::path::Path;
use std::process::Command;

// host#16: a positional reference to a milestone checklist item (box/boxes/steps
// + a numeral, a range, or a glued hyphen-digit form) is the ordinal-by-position
// tell and flags.
#[test]
fn positional_checklist_references_flag() {
    for line in [
        "plan/0001: box 7 [x]",
        "boxes 4-8 blocked",
        "box 3 root cause localized",
        "plan steps 3-5 updated",
        "step 3-5 closed",
    ] {
        assert!(check_line(line).is_some(), "should flag: {line}");
    }
}

// host#16 boundaries: the literal checklist mark, the disposition verb, and a
// content-named reference carry no noun-plus-numeral, so they stay clean.
#[test]
fn checklist_mark_verb_and_content_name_stay_clean() {
    for line in [
        "- [x] deploy path landed",
        "1. [x] native MSVC build verified",
        "box an irreducible citation in a fence",
        "the deploy-path box landed",
        "what is in the box",
    ] {
        assert!(check_line(line).is_none(), "should be clean: {line}");
    }
}

// plan/0035: `gather` surfaces a recurring word-then-numeral shape the lane does
// not catch, and skips flag terms, legitimate contexts, quantities, years, and
// one-offs.
#[test]
fn gather_surfaces_recurring_novel_shape_and_skips_noise() {
    let lines: Vec<String> = [
        "plan/0001: widget 7 done",
        "widget 3 root cause localized",
        "phase 2 of auth refactor",
        "see figure 3 for details",
        "wait 5 seconds for the retry",
        "released in 2024 at last",
        "gizmo 9 one-off only",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    let words: Vec<String> = gather_candidates(&lines, 2)
        .into_iter()
        .map(|c| c.word)
        .collect();
    assert!(words.iter().any(|w| w == "widget"), "widget recurs + is novel: {words:?}");
    assert!(!words.iter().any(|w| w == "phase"), "phase is already a flag term");
    assert!(!words.iter().any(|w| w == "figure"), "figure is a legitimate context");
    assert!(!words.iter().any(|w| w == "wait"), "a unit-bearing quantity is not a position");
    assert!(!words.iter().any(|w| w == "gizmo"), "gizmo is a one-off below the threshold");
}

proptest! {
    #[test]
    fn flag_term_followed_by_arabic_numeral_is_detected(
        term in "phase|stage|step|part|pass|round|iteration|sprint|cycle|increment|wave|batch|section|period|era|epoch|chapter|episode|instalment|leg|lap|level",
        numeral in "[0-9]+"
    ) {
        let line = format!("{} {}", term, numeral);
        prop_assert!(check_line(&line).is_some(), "line: {}", line);
    }

    #[test]
    fn flag_term_followed_by_roman_numeral_is_detected(
        term in "phase|stage|step|part|pass|round",
        roman in "[IVXLCDM]{1,4}"
    ) {
        let line = format!("{} {}", term, roman);
        prop_assert!(check_line(&line).is_some(), "line: {}", line);
    }

    #[test]
    fn flag_term_without_numeral_is_not_detected(
        term in "phase|stage|step|part|pass|round",
        word in "[a-z]{3,10}"
    ) {
        let line = format!("{} {}", term, word);
        if !is_numeral(&word) {
            prop_assert!(check_line(&line).is_none(), "line: {}", line);
        }
    }

    #[test]
    fn ordinal_words_are_not_numerals(
        ordinal in "first|second|third|fourth|fifth|sixth|seventh|eighth|ninth|tenth"
    ) {
        prop_assert!(!is_numeral(&ordinal), "ordinal: {}", ordinal);
    }

    #[test]
    fn descriptive_prose_with_flag_term_is_not_detected(
        prefix in "the|this|that|my|your",
        term in "phase|stage|step|pass",
        suffix in "over|through|across|into"
    ) {
        let line = format!("{} {} {} the array", prefix, term, suffix);
        prop_assert!(check_line(&line).is_none(), "line: {}", line);
    }

    #[test]
    fn flag_term_with_intermediate_word_is_detected(
        term in "phase|stage|step",
        intermediate in "of|in|to",
        numeral in "[0-9]+"
    ) {
        let line = format!("{} {} {}", term, intermediate, numeral);
        prop_assert!(check_line(&line).is_some(), "line: {}", line);
    }

    #[test]
    fn conventional_commit_with_phase_synonym_is_detected(
        prefix in "feat|fix|docs|style|refactor|perf|test|build|ci|chore|revert",
        term in "phase|stage|step",
        numeral in "[0-9]+"
    ) {
        let line = format!("{}: {} {} of refactor", prefix, term, numeral);
        prop_assert!(check_line(&line).is_some(), "line: {}", line);
    }

    #[test]
    fn case_insensitive_detection(
        term in "Phase|PHASE|phase|PhAsE|sTaGe|STEP",
        numeral in "[0-9]+"
    ) {
        let line = format!("## {}: {}", term, numeral);
        prop_assert!(check_line(&line).is_some(), "line: {}", line);
    }

    #[test]
    fn markdown_headers_are_detected(
        level in 1..7usize,
        term in "phase|stage|step|part",
        numeral in "[0-9]+"
    ) {
        let hashes = "#".repeat(level);
        let line = format!("{}{} {}: setup", hashes, term, numeral);
        prop_assert!(check_line(&line).is_some(), "line: {}", line);
    }

    #[test]
    fn code_comments_are_detected(
        prefix in r"//|#|--|\*",
        term in "pass|phase|step|round",
        numeral in "[0-9]+"
    ) {
        let line = format!("{} {} {}: tokenize", prefix, term, numeral);
        prop_assert!(check_line(&line).is_some(), "line: {}", line);
    }

    #[test]
    fn bare_numeral_headers_are_detected(
        level in 1..7usize,
        major in 0..1000u32,
        minor in proptest::option::of(0..1000u32)
    ) {
        let rest = match minor {
            Some(m) => format!("{}.{}", major, m),
            None => format!("{}", major),
        };
        let line = format!("{} {}", "#".repeat(level), rest);
        prop_assert!(check_bare_numeral_header(&line).is_some(), "line: {}", line);
    }

    #[test]
    fn version_headings_are_not_bare_numeral_headers(
        level in 1..7usize,
        a in 0..100u32, b in 0..100u32, c in 0..100u32
    ) {
        let line = format!("{} {}.{}.{}", "#".repeat(level), a, b, c);
        prop_assert!(check_bare_numeral_header(&line).is_none(), "line: {}", line);
    }

    #[test]
    fn named_headers_are_not_bare_numeral_headers(
        level in 1..7usize,
        word in "[A-Za-z][a-z]{2,10}"
    ) {
        let line = format!("{} {}", "#".repeat(level), word);
        prop_assert!(check_bare_numeral_header(&line).is_none(), "line: {}", line);
    }

    #[test]
    fn bare_numeral_headers_only_flagged_in_markdown_sources(
        major in 0..1000u32
    ) {
        let input = format!("# {}", major);
        let mut md_matches = Vec::new();
        scan_text(&input, "PLAN.md", &mut md_matches);
        prop_assert!(!md_matches.is_empty(), "input: {}", input);
        let mut rs_matches = Vec::new();
        scan_text(&input, "main.rs", &mut rs_matches);
        prop_assert!(rs_matches.is_empty(), "input: {}", input);
    }

    #[test]
    fn review_noun_followed_by_letter_digit_code_is_detected(
        term in "review|finding|blocker",
        letter in "[a-z]",
        digits in "[0-9]{1,3}"
    ) {
        let line = format!("fix the guard regex ({} {}{})", term, letter, digits);
        prop_assert!(check_line(&line).is_some(), "line: {}", line);
    }

    #[test]
    fn review_noun_followed_by_issue_code_is_detected(
        term in "review|finding|blocker",
        digits in "[0-9]{1,4}"
    ) {
        let line = format!("addresses {} #{}", term, digits);
        prop_assert!(check_line(&line).is_some(), "line: {}", line);
    }

    #[test]
    fn review_noun_followed_by_bare_numeral_is_clean(
        term in "review|finding|blocker",
        digits in "[0-9]{1,3}"
    ) {
        let line = format!("{} {} files", term, digits);
        prop_assert!(check_line(&line).is_none(), "line: {}", line);
    }

    #[test]
    fn github_issue_refs_are_clean(
        verb in "closes|fixes",
        digits in "[0-9]{1,4}"
    ) {
        let line = format!("{} #{}", verb, digits);
        prop_assert!(check_line(&line).is_none(), "line: {}", line);
    }

    // --- Tier 1: decimal numerals after a flag noun ---

    #[test]
    fn flag_term_followed_by_decimal_numeral_is_detected(
        term in "phase|stage|step|part|section",
        major in 0..100u32,
        minor in 0..100u32
    ) {
        let line = format!("entry point ({} {}.{})", term, major, minor);
        prop_assert!(check_line(&line).is_some(), "line: {}", line);
    }

    #[test]
    fn single_decimal_is_a_numeral_but_version_is_not(
        major in 0..100u32, minor in 0..100u32, patch in 0..100u32
    ) {
        let dec = format!("{}.{}", major, minor);
        let ver = format!("{}.{}.{}", major, minor, patch);
        prop_assert!(is_numeral(&dec), "dec: {}", dec);
        prop_assert!(!is_numeral(&ver), "ver: {}", ver);
    }

    // --- Tier 2: leading label prefix (flag) ---

    #[test]
    fn leading_numeral_label_prefix_is_flagged(
        marker in r"|// |/// |//! |# |## |-- |\* ",
        major in 0..100u32,
        minor in proptest::option::of(0..100u32)
    ) {
        let code = match minor {
            Some(m) => format!("{}.{}", major, m),
            None => format!("{}", major),
        };
        let line = format!("{}{}: exec tools", marker, code);
        prop_assert!(check_label_prefix(&line).is_some(), "line: {}", line);
    }

    #[test]
    fn clock_time_is_not_a_label_prefix(
        h in 0..24u32, m in 10..60u32
    ) {
        // colon followed by a digit, not whitespace -> a time, not a label
        let line = format!("{}:{} standup notes", h, m);
        prop_assert!(check_label_prefix(&line).is_none(), "line: {}", line);
    }

    // --- Tier 3: warn ---

    #[test]
    fn bare_dotted_code_in_prose_warns(
        major in 0..100u32, minor in 0..100u32
    ) {
        let line = format!("as decided in {}.{}", major, minor);
        prop_assert!(check_warn(&line).is_some(), "line: {}", line);
    }

    #[test]
    fn version_with_letter_does_not_warn(
        major in 0..100u32, minor in 0..100u32
    ) {
        let line = format!("bump to v{}.{}", major, minor);
        prop_assert!(check_warn(&line).is_none(), "line: {}", line);
    }

    #[test]
    fn decimal_quantity_does_not_warn(
        major in 0..100u32, minor in 0..100u32,
        unit in "seconds|ms|hours|gb|mb"
    ) {
        let line = format!("elapsed {}.{} {}", major, minor, unit);
        prop_assert!(check_warn(&line).is_none(), "line: {}", line);
    }

    #[test]
    fn allcaps_designator_before_decimal_does_not_warn(
        designator in "[A-Z]{2,5}",
        major in 0..100u32, minor in 0..100u32
    ) {
        // "NT 3.1", "SDK 2.1": an all-caps product/version designator, not a code.
        // Exclude the filing-system warn-nouns (e.g. "WI" == "wi"): those are
        // codes, not designators, and warn by design (see filing_noun_with_numeral_warns).
        prop_assume!(!WARN_NOUNS.contains(&designator.to_ascii_lowercase().as_str()));
        let line = format!("runs on {} {}.{}", designator, major, minor);
        prop_assert!(check_warn(&line).is_none(), "line: {}", line);
    }

    #[test]
    fn titlecase_noun_before_decimal_still_warns(
        noun in "Decision|Milestone|Item|Round",
        major in 0..100u32, minor in 0..100u32
    ) {
        // A Title-case noun is not a designator — the milestone-code register warns.
        let line = format!("see {} {}.{}", noun, major, minor);
        prop_assert!(check_warn(&line).is_some(), "line: {}", line);
    }

    #[test]
    fn filing_noun_with_numeral_warns(
        noun in "work-item|workitem|wi",
        major in 0..100u32,
        minor in proptest::option::of(0..100u32)
    ) {
        let code = match minor {
            Some(m) => format!("{}.{}", major, m),
            None => format!("{}", major),
        };
        let line = format!("implements {} {}", noun, code);
        prop_assert!(check_warn(&line).is_some(), "line: {}", line);
    }

    #[test]
    fn leading_code_label_warns(
        bullet in r"|- |\* |// |# |-- ",
        letter in "[A-Za-z]",
        digits in 1..1000u32,
        dash in "—|–|-"
    ) {
        // "F1 — …", "- **B2** – …": a one-letter+digits code as a leading label.
        let line = format!("{}{}{} {} the fix", bullet, letter, digits, dash);
        prop_assert!(check_code_label_prefix(&line).is_some(), "line: {}", line);
    }

    #[test]
    fn multi_letter_device_noun_is_not_a_code_label(
        digits in 1..1000u32
    ) {
        // "COM1 open — …": a three-letter device noun is not a one-letter code,
        // and is followed by a word, not a delimiter.
        let line = format!("COM{} open — seed the DCB", digits);
        prop_assert!(check_code_label_prefix(&line).is_none(), "line: {}", line);
    }

    #[test]
    fn code_without_a_delimiter_is_not_a_label(
        letter in "[A-Za-z]", digits in 1..1000u32
    ) {
        // "F1 key handler": a code followed by an ordinary word, no label dash.
        let line = format!("{}{} key handler", letter, digits);
        prop_assert!(check_code_label_prefix(&line).is_none(), "line: {}", line);
    }

    #[test]
    fn flag_wins_over_warn_on_the_same_line(
        term in "phase|stage|step",
        a in 0..100u32, b in 0..100u32
    ) {
        // a flag noun plus a stray dotted code -> classified as a flag
        let line = format!("{} {} touched the {}.{} surface", term, a, a, b);
        prop_assert_eq!(classify_line(&line, false).map(|(s, _)| s), Some(Severity::Flag), "line: {}", line);
    }
}

// --- Deterministic cases from issue #10 ---

#[test]
fn issue_10_flag_cases() {
    // worded noun + decimal, mid-line parenthetical
    assert!(check_line("entry point (Phase 5.0).").is_some());
    // leading bare-numeral label prefix
    assert!(check_label_prefix("5.5: exec/pty tools").is_some());
    assert!(check_label_prefix("// 5.5: the pty exec tool advertises").is_some());
    assert!(check_label_prefix("## 5.5: error handling").is_some());
}

#[test]
fn issue_10_warn_cases() {
    for line in [
        "as decided in 2.1",
        "exec tools (5.5)",
        "the peek/poke tools arrive in 5.3",
        "* mem_ops.h - work-item 5.3",
    ] {
        assert_eq!(
            classify_line(line, false).map(|(s, _)| s),
            Some(Severity::Warn),
            "expected warn for: {}",
            line
        );
    }
}

#[test]
fn leading_code_label_warn_cases() {
    // PR #22 structured its fixes as bare "F1 — …" leading labels: the
    // section-5 code-as-name tell with no preceding review/finding/blocker noun.
    for line in [
        "F1 — PE version stamp 3.10",
        "- **F2** — SetHandleInformation routed through a feat.c probe",
        "* B3 – lstrcpynA swapped at 50 sites",
        "F4: the durable name follows the colon",
    ] {
        assert_eq!(
            classify_line(line, false).map(|(s, _)| s),
            Some(Severity::Warn),
            "expected warn for: {}",
            line
        );
    }
}

#[test]
fn leading_code_label_clean_cases() {
    // A multi-letter device noun, a code with no label delimiter, and a real
    // GitHub ref must not be read as a leading code label.
    for line in [
        "COM1 open — GetCommState-first DCB seeding",
        "the F1 key opens help",
        "fixes #18",
        "review 3 files",
    ] {
        assert!(
            check_code_label_prefix(line).is_none(),
            "expected no code-label match for: {}",
            line
        );
    }
}

#[test]
fn allcaps_designator_clean_cases() {
    // An all-caps product/version designator before a decimal is a version
    // string, not a milestone code — it must not even warn.
    for line in [
        "Make the device load and wire-respond on the Windows NT 3.1 floor",
        "targets NT 3.1",
        "ships the SDK 2.1 headers",
        "DOS 6.2 compatibility",
    ] {
        assert_eq!(classify_line(line, false), None, "expected clean for: {}", line);
    }
    // …but a Title-case milestone noun before a decimal still warns.
    assert_eq!(
        classify_line("see Decision 2.1", false).map(|(s, _)| s),
        Some(Severity::Warn),
        "Title-case noun should still warn",
    );
}

#[test]
fn issue_10_clean_cases() {
    // version strings and quantities must not even warn
    for line in [
        "bump to v2.1",
        "requires Python 3.11",
        "5.5 seconds elapsed",
        "increased by 2.1%",
        "review 3 files",
    ] {
        assert_eq!(classify_line(line, false), None, "expected clean for: {}", line);
    }
}

// --- Sanctioned-token allowlist (the LEXICON, masked before detection) ---

// A helper: scan one line under an allow-list, return the matches.
fn scan_one(line: &str, source: &str, allow: &[&str]) -> Vec<host_lint::Match> {
    let allow_lc: Vec<String> = allow.iter().map(|s| s.to_ascii_lowercase()).collect();
    let mut m = Vec::new();
    scan_text_with_allow(line, source, &allow_lc, &mut m);
    m
}

#[test]
fn allowed_phrase_suppresses_its_own_flag() {
    // "section 1" is a hard flag (section is a flag noun); allow-listing it clears it.
    assert!(!scan_one("see section 1 of the spec", "doc.md", &[]).is_empty());
    assert!(scan_one("see section 1 of the spec", "doc.md", &["section 1"]).is_empty());
}

#[test]
fn allowed_phrase_suppresses_a_dotted_code_warn() {
    // A milestone-style dotted code ("Decision 2.1") trips the advisory warn; an
    // allow entry clears it. (An all-caps designator like "NT 3.1" no longer
    // warns at all — it is recognised as a version string by the engine.)
    assert!(!scan_one("see Decision 2.1 here", "README.md", &[]).is_empty());
    assert!(scan_one("see Decision 2.1 here", "README.md", &["Decision 2.1"]).is_empty());
}

#[test]
fn allow_is_case_insensitive() {
    assert!(scan_one("Built for DOS 6.22 hosts", "doc.md", &["dos 6.22"]).is_empty());
}

#[test]
fn allow_does_not_clear_a_different_tell_on_the_same_line() {
    // Allow-listing "section 1" must not mask the separate "phase 4" tell.
    let m = scan_one("section 1 covers phase 4 work", "doc.md", &["section 1"]);
    assert_eq!(m.len(), 1, "phase 4 must still flag");
    assert_eq!(m[0].term, "phase");
}

#[test]
fn allow_respects_word_boundaries() {
    // "phase 1" allow-listed must NOT clear the longer tell "phase 12".
    assert!(!scan_one("phase 12 begins", "doc.md", &["phase 1"]).is_empty());
}

#[test]
fn empty_allow_list_is_unchanged_behaviour() {
    let with_empty = scan_one("## Phase 2: setup", "PLAN.md", &[]);
    let mut baseline = Vec::new();
    scan_text("## Phase 2: setup", "PLAN.md", &mut baseline);
    assert_eq!(with_empty.len(), baseline.len());
    assert_eq!(with_empty.len(), 1);
}

// --- Prose lane consults the LEXICON (issue #16) ---

// A helper: scan text as prose under an allow-list, return the matches.
fn prose_one(input: &str, source: &str, allow: &[&str]) -> Vec<host_lint::Match> {
    let allow_lc: Vec<String> = allow.iter().map(|s| s.to_ascii_lowercase()).collect();
    let mut m = Vec::new();
    scan_prose_text(input, source, &allow_lc, &mut m);
    m
}

#[test]
fn prose_lexicon_masks_a_trope_within_a_declared_phrase() {
    // `harness` is an ai-diction prose trope; a project that legitimately runs a
    // `rehost harness` declares the phrase, and the prose lane masks the trope
    // within it, the same pre-detection blank-out the naming lane already performs.
    let line = "The rehost harness logs to disk; a second harness runs nightly.";
    let undeclared = prose_one(line, "doc.md", &[]);
    assert_eq!(undeclared.len(), 2, "both harness occurrences flag with no LEXICON");
    let masked = prose_one(line, "doc.md", &["rehost harness"]);
    assert_eq!(masked.len(), 1, "the occurrence inside the declared phrase is cleared");
    // The surviving flag is the standalone `harness`, not the one inside the phrase:
    // it sits at the second occurrence's column (surgical at the word boundary).
    assert_eq!(masked[0].col, undeclared[1].col);
}

#[test]
fn prose_empty_or_irrelevant_allow_leaves_all_flags() {
    let line = "The rehost harness logs to disk; a second harness runs nightly.";
    assert_eq!(prose_one(line, "doc.md", &[]).len(), 2, "no LEXICON masks nothing");
    // An entry that does not occur in the text masks nothing (unchanged behaviour).
    assert_eq!(prose_one(line, "doc.md", &["windows 3.1"]).len(), 2);
}

// --- Path ignore (.host-lintignore) ---

#[test]
fn path_ignore_exact_glob_and_dir() {
    let pats = vec![
        "MEMORY.md".to_string(),
        "plan/*/README.md".to_string(),
        "archive/".to_string(),
    ];
    // exact root-level file
    assert!(path_ignored("MEMORY.md", &pats));
    // single-segment glob
    assert!(path_ignored("plan/0004-command-execution/README.md", &pats));
    // directory prefix ignores everything beneath
    assert!(path_ignored("archive/old/notes.md", &pats));
    assert!(path_ignored("archive", &pats));
    // not matched: live index, wrong depth, non-root MEMORY
    assert!(!path_ignored("plan/PLAN.md", &pats));
    assert!(!path_ignored("plan/0004/extra/README.md", &pats));
    assert!(!path_ignored("docs/MEMORY.md", &pats));
}

#[test]
fn empty_ignore_matches_nothing() {
    assert!(!path_ignored("MEMORY.md", &[]));
}

#[test]
fn coauthored_by_trailers_are_exempt() {
    // A discretionary attribution trailer is skipped entirely, neither flagged
    // nor warned — even a phase-like co-author name. Dealer's choice field.
    for line in [
        "Co-Authored-By: Claude Opus 4.8 <noreply@anthropic.com>",
        "co-authored-by: someone 2.1",
        "  Co-Authored-By: Phase 2 Bot <bot@example.com>",
    ] {
        assert_eq!(classify_line(line, false), None, "expected exempt for: {}", line);
        assert_eq!(classify_line(line, true), None, "expected exempt (md) for: {}", line);
    }
}

#[test]
fn prose_tells_are_advisory_warns() {
    // Prose tells are advisory — never Flag — so a title or comment with a tell warns
    // (locatable, exit 3) or notes (a non-locatable whole-document diagnosis, exit 0)
    // but never blocks a commit.
    let mut m = Vec::new();
    scan_prose_text("Let's unpack this. We delve into the rich tapestry.", "stdin", &[], &mut m);
    assert!(!m.is_empty(), "expected prose tells");
    assert!(m.iter().all(|x| x.severity != Severity::Flag), "prose never blocks");
    assert!(m.iter().any(|x| x.term == "pedagogical-hook"));
    assert!(m.iter().all(|x| !x.cite.is_empty()), "prose tells carry a citation");
}

#[test]
fn decoration_occurrences_map_to_distinct_lines_and_columns() {
    // The headline plan/0031 fix: em-dashes on different lines report at their own
    // line:col, not all collapsed onto the first occurrence's line (the call/0019 defect
    // where ten dashes all reported at line 12).
    let input = "Alpha — one.\nBeta line here all.\nGamma — two — three.\n";
    let mut m = Vec::new();
    scan_prose_text(input, "doc.md", &[], &mut m);
    let dec: Vec<_> = m.iter().filter(|x| x.term == "decoration").collect();
    assert_eq!(dec.len(), 3, "three em-dashes → three records");
    let lines: Vec<usize> = dec.iter().map(|x| x.line).collect();
    assert!(lines.contains(&1) && lines.contains(&3), "on their real lines, got {lines:?}");
    assert!(dec.iter().all(|x| x.col > 0), "each decoration carries a column");
    let mut seen = std::collections::HashSet::new();
    assert!(
        dec.iter().all(|x| seen.insert((x.line, x.col))),
        "distinct line:col per occurrence"
    );
}

#[test]
fn structural_diagnoses_are_advisory_notes() {
    // A whole-document diagnosis (density, anaphora, a self-answered question) has no
    // single editable span, so it is advisory (Note, exit 0) — outside the clean-to-zero
    // gate — while locatable tropes stay Warn.
    let input = "Let's unpack this. It's not a tweak, it's a revolution. We delve. \
                 We leverage. We harness. The result? Pure synergy. Fast, clean, and robust.";
    let mut m = Vec::new();
    scan_prose_text(input, "stdin", &[], &mut m);
    assert!(
        m.iter().any(|x| x.severity == Severity::Note),
        "a structural diagnosis is advisory"
    );
    assert!(
        m.iter().filter(|x| x.severity == Severity::Note).all(|x| x.col == 0),
        "advisory diagnoses have no column"
    );
    assert!(
        m.iter().any(|x| x.severity == Severity::Warn),
        "locatable tropes still warn"
    );
}

#[test]
fn subject_decoration_escalates_to_flag() {
    // A decoration tell on the commit subject (first line) / a gh title blocks.
    let input = "classify: refuse adoption — print the case";
    let mut m = Vec::new();
    scan_prose_text(input, "stdin", &[], &mut m);
    escalate_subject_decoration(input.lines().next().unwrap(), &mut m);
    assert!(m.iter().any(|x| x.term == "decoration" && x.severity == Severity::Flag));
}

#[test]
fn body_decoration_stays_advisory() {
    // Decoration confined to the body keeps its Warn — only the subject blocks.
    let input = "classify: refuse adoption to print the case\n\nIt prints the case — unless the target is software.";
    let mut m = Vec::new();
    scan_prose_text(input, "stdin", &[], &mut m);
    escalate_subject_decoration(input.lines().next().unwrap(), &mut m);
    assert!(m.iter().any(|x| x.term == "decoration"), "expected a body decoration tell");
    assert!(m.iter().filter(|x| x.term == "decoration").all(|x| x.severity == Severity::Warn));
}

#[test]
fn clean_prose_emits_no_tells() {
    // Ordinary technical prose stays silent — no Warn, no density gate.
    let mut m = Vec::new();
    scan_prose_text(
        "The parser reads each line and reports the first tell it finds. \
         A missing allow-list file means no phrases are masked.",
        "stdin",
        &[],
        &mut m,
    );
    assert!(m.is_empty(), "clean prose tripped: {:?}", m.iter().map(|x| &x.term).collect::<Vec<_>>());
}

#[test]
fn dense_prose_crosses_the_density_gate() {
    let mut m = Vec::new();
    scan_prose_text(
        "Let's unpack this. It's not a tweak, it's a revolution. We delve. \
         We leverage. We harness. The result? Pure synergy. Fast, clean, and robust.",
        "stdin",
        &[],
        &mut m,
    );
    assert!(m.iter().any(|x| x.term == "tell-density"), "expected the density summary");
}

// --- LEXICON: parsing, the strict directive, and the three guards (issue #13) ---

fn entry(phrase: &str, url: Option<&str>) -> LexiconEntry {
    LexiconEntry { phrase: phrase.to_string(), url: url.map(String::from) }
}

#[test]
fn lexicon_parse_skips_blanks_and_comments() {
    assert_eq!(parse_lexicon_line(""), None);
    assert_eq!(parse_lexicon_line("   "), None);
    assert_eq!(parse_lexicon_line("# a note"), None);
    // A markdown-style "## heading" is a comment (hash + non-digit), not an entry.
    assert_eq!(parse_lexicon_line("## heading"), None);
    // The strict directive is comment-shaped, so the phrase parser ignores it.
    assert_eq!(parse_lexicon_line("# host-lint: strict"), None);
}

#[test]
fn lexicon_parse_keeps_hash_number_entries() {
    // "#7" is an entry (hash + digit), NOT a comment — this is the carve-out that
    // stops the comment marker colliding with the hash-number reference shape.
    assert_eq!(parse_lexicon_line("#7"), Some(entry("#7", None)));
}

#[test]
fn lexicon_parse_splits_a_trailing_url() {
    let e = parse_lexicon_line("#7 https://github.com/o/r/issues/7").unwrap();
    assert_eq!(e.phrase, "#7");
    assert_eq!(e.url.as_deref(), Some("https://github.com/o/r/issues/7"));
    // A phrase with no URL keeps every token (including internal spaces).
    assert_eq!(parse_lexicon_line("Windows 3.1"), Some(entry("Windows 3.1", None)));
    // A trailing non-URL token is part of the phrase, not a URL.
    assert_eq!(parse_lexicon_line("Windows 3.1 beta"), Some(entry("Windows 3.1 beta", None)));
}

#[test]
fn lexicon_strict_directive_recognised() {
    assert!(is_strict_directive("# host-lint: strict"));
    assert!(is_strict_directive("#host-lint: strict"));
    assert!(!is_strict_directive("# some other comment"));
    assert!(!is_strict_directive("Windows 3.1"));
}

#[test]
fn lexicon_guard_accepts_legitimate_vocabulary() {
    // A warn-tier phrase (a dotted code with a legitimizing word) is the intended
    // case: it carries a letter and is not a flag-tier tell.
    assert!(validate_lexicon_entry(&entry("Windows 3.1", None), &[]).is_ok());
    assert!(validate_lexicon_entry(&entry("Decision 2.1", None), &[]).is_ok());
    assert!(validate_lexicon_entry(&entry("COM1", None), &[]).is_ok());
    // PROJ-NNNN-shaped standards tokens are vocabulary unless a jira-key is declared.
    assert!(validate_lexicon_entry(&entry("RFC-2119", None), &[]).is_ok());
}

#[test]
fn lexicon_guard_g1_rejects_a_bare_master_key() {
    // A bare numeral/dotted code with no legitimizing word would clear every
    // occurrence tree-wide — refused.
    assert!(validate_lexicon_entry(&entry("5.5", None), &[]).is_err());
    assert!(validate_lexicon_entry(&entry("2.1", None), &[]).is_err());
}

#[test]
fn lexicon_guard_g2_rejects_laundering_a_real_tell() {
    // The phrase is itself a flag-tier tell — you rename it, you do not allow-list
    // it. (The 4B test tried exactly this: `lexicon add "Phase 5.5"`.)
    assert!(validate_lexicon_entry(&entry("Phase 5.5", None), &[]).is_err());
    assert!(validate_lexicon_entry(&entry("Step 3", None), &[]).is_err());
}

#[test]
fn lexicon_guard_citation_gates_tracker_refs() {
    // A bare tracker ref must carry a URL (provenance), else it is a phantom.
    assert!(validate_lexicon_entry(&entry("#7", None), &[]).is_err());
    assert!(validate_lexicon_entry(&entry("#7", Some("https://github.com/o/r/issues/7")), &[]).is_ok());
    assert!(validate_lexicon_entry(&entry("connollydavid/host#7", None), &[]).is_err());
    assert!(validate_lexicon_entry(
        &entry("connollydavid/host#7", Some("https://github.com/connollydavid/host/issues/7")),
        &[]
    )
    .is_ok());
}

#[test]
fn lexicon_jira_key_gating_is_opt_in() {
    let proj = vec!["PROJ".to_string()];
    // Without a declared key, PROJ-NNNN is plain vocabulary (no URL required) —
    // this is what keeps standards tokens (RFC-2119, UTF-8) from being forced to cite.
    assert!(validate_lexicon_entry(&entry("PROJ-1234", None), &[]).is_ok());
    // With PROJ declared, PROJ-1234 is a citation-gated tracker ref: URL required.
    assert!(validate_lexicon_entry(&entry("PROJ-1234", None), &proj).is_err());
    assert!(validate_lexicon_entry(&entry("PROJ-1234", Some("https://jira.example/PROJ-1234")), &proj).is_ok());
    // A different key is unaffected: declaring PROJ does NOT gate RFC-2119.
    assert!(validate_lexicon_entry(&entry("RFC-2119", None), &proj).is_ok());
}

#[test]
fn lexicon_jira_directive_parses_declared_keys() {
    assert_eq!(parse_jira_keys("# host-lint: jira-key PROJ"), Some(vec!["PROJ".to_string()]));
    assert_eq!(
        parse_jira_keys("# host-lint: jira-key PROJ TEAM2"),
        Some(vec!["PROJ".to_string(), "TEAM2".to_string()])
    );
    // Lowercase / non-key tokens are dropped; an empty declaration is None.
    assert_eq!(parse_jira_keys("# host-lint: jira-key"), None);
    assert_eq!(parse_jira_keys("# host-lint: strict"), None);
    assert_eq!(parse_jira_keys("PROJ-1"), None);
}

#[test]
fn lexicon_strict_escalates_an_undeclared_warn_to_a_flag() {
    // A bare dotted code warns by default, blocks under strict, and is silenced by
    // a LEXICON entry that masks the full phrase.
    let scan = |strict: bool, allow: &[&str]| {
        let allow_lc: Vec<String> = allow.iter().map(|s| s.to_ascii_lowercase()).collect();
        let mut m = Vec::new();
        scan_text_with_allow_strict("see Decision 2.1 here", "README.md", &allow_lc, strict, &mut m);
        m
    };
    assert_eq!(scan(false, &[]).first().map(|m| m.severity), Some(Severity::Warn));
    assert_eq!(scan(true, &[]).first().map(|m| m.severity), Some(Severity::Flag));
    assert!(scan(true, &["Decision 2.1"]).is_empty(), "an allowed phrase is not escalated");
}

#[test]
fn lexicon_masking_clears_a_cited_tracker_ref() {
    // "finding #7" is a flag (review noun + code); masking the cited "#7" phrase
    // leaves "finding " with no code, so the line is clean.
    let allow = vec!["#7".to_string()];
    let mut m = Vec::new();
    scan_text_with_allow_strict("see finding #7 in the log", "doc.md", &allow, true, &mut m);
    assert!(m.is_empty(), "cited #7 should mask the review-code flag: {:?}", m.iter().map(|x| &x.term).collect::<Vec<_>>());
}

// --- host-lint:ignore fenced blocks (call/0019, plan/0027) ---

#[test]
fn host_lint_ignore_block_skips_naming_tells_in_markdown() {
    let scan = |text: &str, src: &str| {
        let mut m = Vec::new();
        scan_text_with_allow_strict(text, src, &[], false, &mut m);
        m
    };
    // Inside a host-lint:ignore block: skipped (the literal-citation quarantine).
    assert!(scan("```host-lint:ignore\nPhase 1 was the bootstrap\n```", "doc.md").is_empty());
    // The same tell in prose: still flags.
    assert!(!scan("Phase 1 was the bootstrap", "doc.md").is_empty());
    // In a REGULAR fenced block: still flags — only host-lint:ignore is skipped.
    assert!(!scan("```\nPhase 1 was the bootstrap\n```", "doc.md").is_empty());
    // A different language tag is not the ignore tag: still flags.
    assert!(!scan("```text\nPhase 1 was the bootstrap\n```", "doc.md").is_empty());
    // Inline backtick: still flags — only blocks are skipped, never inline.
    assert!(!scan("see `Phase 1` here", "doc.md").is_empty());
    // Non-markdown (a commit message): the fence is literal text, the tell flags.
    assert!(!scan("```host-lint:ignore\nPhase 1\n```", "stdin").is_empty());
    // Detection resumes after the block closes.
    assert!(!scan("```host-lint:ignore\nold Phase 1\n```\nnow Phase 2 here", "doc.md").is_empty());
    // An indented code block is not a fence — its `Phase 1` line is scanned and flags.
    assert!(!scan("    ```host-lint:ignore\n    Phase 1\n    ```", "doc.md").is_empty());
}

// Regression for the plan/0022 design-review boxing: a host-lint:ignore region,
// then a REGULAR fenced code block that must still be scanned, then a second
// host-lint:ignore region. The bare fence closing the code block must not be read
// as an ignore boundary, and only a bare fence (never an info-string fence) closes
// an ignore region.
#[test]
fn host_lint_ignore_regions_flank_a_still_scanned_code_block() {
    let scan = |text: &str, src: &str| {
        let mut m = Vec::new();
        scan_text_with_allow_strict(text, src, &[], false, &mut m);
        m
    };
    // ignore(Phase 1) | regular rust(Phase 2) | ignore(Phase 3).
    let doc = "```host-lint:ignore\ncite Phase 1 here\n```\n\n```rust\n// Phase 2 in real code\n```\n\n```host-lint:ignore\ncite Phase 3 here\n```";
    let m = scan(doc, "doc.md");
    assert_eq!(m.len(), 1, "only the regular code block's tell flags: {:?}",
        m.iter().map(|x| (x.line, x.text.clone())).collect::<Vec<_>>());
    assert!(m[0].text.contains("Phase 2"), "flagged line is the code-block tell: {:?}", m[0].text);
    // An info-string fence (```rust) inside an ignore region does NOT close it —
    // the whole region stays quarantined, fences and all.
    assert!(scan("```host-lint:ignore\nPhase 1\n```rust\nPhase 2\n```", "doc.md").is_empty(),
        "an info-string fence must not close an ignore region");
}

// host-lifecycle#2: the LEXICON loader and the `--docs` walk live in the shared engine,
// so an in-process embedder (host-lifecycle's prose recheck) masks exactly the phrases
// the CLI does. These two tests pin that contract through the public API.

fn git(dir: &Path, args: &[&str]) {
    let ok = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    assert!(ok, "git {args:?} failed");
}

#[test]
fn load_lexicon_returns_validated_lowercased_phrases() {
    let dir = std::env::temp_dir().join(format!("hl-lex-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    // Two declared phrases (mixed case), plus a master-key entry that must be dropped.
    fs::write(dir.join("LEXICON"), "Wdm-Harness\nthe harness\n*\n").unwrap();
    let lex = host_lint::load_lexicon(&dir);
    assert!(lex.phrases_lc.contains(&"wdm-harness".to_string()), "phrases are ASCII-lowercased");
    assert!(lex.phrases_lc.contains(&"the harness".to_string()));
    assert!(
        !lex.phrases_lc.iter().any(|p| p == "*"),
        "a master-key entry is dropped — it never masks"
    );
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn run_docs_masks_a_lexicon_declared_prose_tell() {
    let dir = std::env::temp_dir().join(format!("hl-docs-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["config", "user.email", "t@t"]);
    git(&dir, &["config", "user.name", "t"]);
    // "harness" is an ai-diction term; two occurrences in one doc trip the density warn.
    fs::write(
        dir.join("doc.md"),
        "# Title\n\nThe wdm-harness drives the lane. The harness emits a verdict.\n",
    )
    .unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-qm", "doc"]);
    // Undeclared: the prose tell fires.
    let bare = host_lint::run_docs(&dir, &[], &[]).unwrap();
    assert!(
        bare.iter().any(|m| m.severity == Severity::Warn),
        "undeclared, the ai-diction term warns in the --docs walk"
    );
    // Declared: the same phrases are masked before detection, so the warn clears —
    // the in-process embedder gets the identical verdict to standalone `host-lint --docs`.
    let allow = vec!["wdm-harness".to_string(), "the harness".to_string()];
    let masked = host_lint::run_docs(&dir, &allow, &[]).unwrap();
    assert!(
        !masked.iter().any(|m| m.severity == Severity::Warn),
        "a LEXICON-declared phrase clears the prose tell in the shared --docs walk"
    );
    let _ = fs::remove_dir_all(&dir);
}
