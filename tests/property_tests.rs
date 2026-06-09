use no_phase::{check_bare_numeral_header, check_line, is_numeral, scan_text};
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
}
