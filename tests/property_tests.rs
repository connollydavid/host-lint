use no_phase::{
    check_bare_numeral_header, check_label_prefix, check_line, check_warn, classify_line,
    is_numeral, scan_text, Severity,
};
use proptest::prelude::*;

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
