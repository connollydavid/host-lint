const FLAG_TERMS: &[&str] = &[
    "phase", "stage", "step", "part", "pass", "round", "iteration",
    "sprint", "cycle", "increment", "wave", "batch", "section",
    "period", "era", "epoch", "chapter", "episode", "instalment",
    "leg", "lap", "level",
];

const REVIEW_CODE_TERMS: &[&str] = &["review", "finding", "blocker"];

// Tier 3 (warn): filing-system code nouns whose numbered label is a milestone
// code used as a name. Warned, not flagged, because the same nouns have
// ordinary uses ("see item 5 in the list").
const WARN_NOUNS: &[&str] = &["work-item", "workitem", "wi"];

// Tier 3 (warn): a bare "N.N" code immediately preceded by one of these is a
// version string or a cross-reference, not a milestone code — skip it.
const PREV_SKIP: &[&str] = &[
    "v", "version", "ver", "python", "node", "rust", "go", "java", "ruby",
    "php", "gcc", "clang", "llvm", "figure", "fig", "table", "eq", "equation",
    "page", "chapter", "ch", "appendix",
];

// Tier 3 (warn): a bare "N.N" code immediately followed by one of these units
// is a quantity, not a milestone code — skip it.
const UNITS: &[&str] = &[
    "s", "sec", "secs", "second", "seconds", "ms", "min", "mins", "minute",
    "minutes", "h", "hr", "hrs", "hour", "hours", "day", "days",
    "gb", "mb", "kb", "tb",
];

const CI_PATTERNS: &[&str] = &[
    ".github/workflows",
    ".gitlab-ci",
    "jenkinsfile",
    "dockerfile",
    "docker-compose",
];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Severity {
    /// A confirmed tell: blocks (exit 1).
    Flag,
    /// A bare-numeral degenerate form: advisory, asks the author to reconsider (exit 3).
    Warn,
}

pub struct Match {
    pub file: String,
    pub line: usize,
    pub text: String,
    pub term: String,
    pub severity: Severity,
}

pub fn is_ci_file(path: &str) -> bool {
    let lower = path.to_lowercase();
    CI_PATTERNS.iter().any(|p| lower.contains(p))
}

pub fn is_numeral(word: &str) -> bool {
    if word.is_empty() {
        return false;
    }
    // Arabic integer ("5") or single-decimal numeral ("5.5"); a version-like
    // form with two or more dots ("1.2.3") is not a numeral.
    let parts: Vec<&str> = word.split('.').collect();
    if parts.len() <= 2 && parts.iter().all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit())) {
        return true;
    }
    let upper = word.to_uppercase();
    upper.len() <= 4 && upper.chars().all(|c| matches!(c, 'I' | 'V' | 'X' | 'L' | 'C' | 'D' | 'M'))
}

// A bare dotted code: exactly one decimal point, digits on both sides ("5.5").
fn is_dotted_code(word: &str) -> bool {
    let parts: Vec<&str> = word.split('.').collect();
    parts.len() == 2 && parts.iter().all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()))
}

pub fn is_review_code(word: &str) -> bool {
    // "#7" (issue-style number) or "b1" (a severity letter + number). A bare
    // numeral ("3") is NOT a code, so "review 3 files" stays clean.
    if let Some(digits) = word.strip_prefix('#') {
        return !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit());
    }
    let mut chars = word.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() => {
            let rest = chars.as_str();
            !rest.is_empty() && rest.chars().all(|c| c.is_ascii_digit())
        }
        _ => false,
    }
}

pub fn check_line(line: &str) -> Option<String> {
    let lower = line.to_lowercase();
    let words: Vec<&str> = lower.split_whitespace().collect();

    for (i, word) in words.iter().enumerate() {
        let clean = word.trim_matches(|c: char| !c.is_alphanumeric() && c != '-');
        if FLAG_TERMS.contains(&clean) {
            if let Some(next) = words.get(i + 1) {
                let next_clean = next.trim_matches(|c: char| !c.is_alphanumeric() && c != '-');
                if is_numeral(next_clean) {
                    let orig_word = words[i].trim_matches(|c: char| !c.is_alphanumeric() && c != '-');
                    return Some(orig_word.to_string());
                }
            }
            if let Some(next2) = words.get(i + 2) {
                let next_clean = next2.trim_matches(|c: char| !c.is_alphanumeric() && c != '-');
                if is_numeral(next_clean) {
                    let orig_word = words[i].trim_matches(|c: char| !c.is_alphanumeric() && c != '-');
                    return Some(orig_word.to_string());
                }
            }
        }
        // Sibling tell: an internal tracking code used as a name (a noun from
        // REVIEW_CODE_TERMS immediately followed by a hash+digits or
        // letter+digits label). The trim keeps a leading "#" so issue-style
        // codes survive; GitHub refs never match because closes/fixes are
        // not in the noun set.
        if REVIEW_CODE_TERMS.contains(&clean) {
            if let Some(next) = words.get(i + 1) {
                let code = next.trim_matches(|c: char| !c.is_alphanumeric() && c != '#');
                if is_review_code(code) {
                    return Some(clean.to_string());
                }
            }
        }
    }

    None
}

// Tier 2 (flag): a bare numeral used as a label prefix at the start of a
// subject line, header, or comment ("5.5: exec tools", "// 5.5: ..."). The
// colon must be followed by whitespace or end-of-line so a clock time
// ("5:30 standup") does not match.
pub fn check_label_prefix(line: &str) -> Option<String> {
    let mut s = line.trim_start();
    loop {
        let stripped = s
            .strip_prefix("///")
            .or_else(|| s.strip_prefix("//!"))
            .or_else(|| s.strip_prefix("//"))
            .or_else(|| s.strip_prefix("/**"))
            .or_else(|| s.strip_prefix("/*"))
            .or_else(|| s.strip_prefix("--"))
            .or_else(|| s.strip_prefix("*"))
            .or_else(|| s.strip_prefix("#"));
        match stripped {
            Some(rest) => s = rest.trim_start(),
            None => break,
        }
    }
    let code: String = s.chars().take_while(|&c| c.is_ascii_digit() || c == '.').collect();
    if code.is_empty() || !is_numeral(&code) {
        return None;
    }
    let mut after = s[code.len()..].chars();
    if after.next() == Some(':') {
        match after.next() {
            None => return Some(code),
            Some(c) if c.is_whitespace() => return Some(code),
            _ => {}
        }
    }
    None
}

// Tier 3 (warn): the bare-numeral degenerate form with the noun elided — a
// filing-system code noun followed by a numeral, or a bare dotted code used as
// a name outside version/quantity contexts. Advisory only.
pub fn check_warn(line: &str) -> Option<String> {
    let lower = line.to_lowercase();
    let words: Vec<&str> = lower.split_whitespace().collect();
    let orig: Vec<&str> = line.split_whitespace().collect();

    for i in 0..words.len() {
        let word = words[i];
        let clean = word.trim_matches(|c: char| !c.is_alphanumeric() && c != '-');
        // W1: a filing-code noun followed by a numeral (within two words).
        if WARN_NOUNS.contains(&clean) {
            for k in 1..=2 {
                if let Some(next) = words.get(i + k) {
                    let nc = next.trim_matches(|c: char| !c.is_alphanumeric() && c != '-');
                    if is_numeral(nc) {
                        return Some(clean.to_string());
                    }
                }
            }
        }
        // W2: a bare dotted code ("5.5") used as a name. A token carrying a
        // letter ("v2.1") is a version string and is left alone.
        let ow = if i < orig.len() { orig[i] } else { word };
        if ow.chars().any(|c| c.is_ascii_alphabetic()) {
            continue;
        }
        let code = word.trim_matches(|c: char| !c.is_ascii_digit() && c != '.');
        if is_dotted_code(code) {
            if ow.ends_with('%') {
                continue;
            }
            if let Some(next) = words.get(i + 1) {
                let nc = next.trim_matches(|c: char| !c.is_alphanumeric());
                if UNITS.contains(&nc) {
                    continue;
                }
            }
            if i > 0 {
                let pc = words[i - 1].trim_matches(|c: char| !c.is_alphanumeric());
                if PREV_SKIP.contains(&pc) {
                    continue;
                }
            }
            return Some(code.to_string());
        }
    }

    None
}

pub fn check_bare_numeral_header(line: &str) -> Option<String> {
    let t = line.trim();
    let hashes = t.chars().take_while(|&c| c == '#').count();
    if hashes == 0 || hashes > 6 {
        return None;
    }
    let rest = t[hashes..].trim();
    let parts: Vec<&str> = rest.split('.').collect();
    if parts.len() > 2 {
        // version-like heading (1.2.3), not a bare ordinal
        return None;
    }
    if parts.iter().all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit())) {
        return Some(rest.to_string());
    }
    None
}

// Classify a single line, preferring the most severe outcome: a confirmed tell
// (flag) wins over the bare-numeral degenerate form (warn).
pub fn classify_line(line: &str, markdown: bool) -> Option<(Severity, String)> {
    if let Some(t) = check_line(line) {
        return Some((Severity::Flag, t));
    }
    if let Some(t) = check_label_prefix(line) {
        return Some((Severity::Flag, t));
    }
    if markdown {
        if let Some(t) = check_bare_numeral_header(line) {
            return Some((Severity::Flag, t));
        }
    }
    if let Some(t) = check_warn(line) {
        return Some((Severity::Warn, t));
    }
    None
}

pub fn scan_text(input: &str, source: &str, matches: &mut Vec<Match>) {
    let markdown = source.to_lowercase().ends_with(".md");
    for (i, line) in input.lines().enumerate() {
        if let Some((severity, term)) = classify_line(line, markdown) {
            matches.push(Match {
                file: source.to_string(),
                line: i + 1,
                text: line.trim().to_string(),
                term,
                severity,
            });
        }
    }
}

pub fn is_scannable(ext: &str) -> bool {
    matches!(ext, "" | "md" | "txt" | "rst" | "py" | "rs" | "js" | "ts" | "jsx" | "tsx" | "go" | "java" | "c" | "cpp" | "h" | "hpp" | "rb" | "sh" | "yaml" | "yml" | "toml" | "json" | "xml" | "html" | "css" | "sql" | "r" | "lua" | "swift" | "kt" | "scala" | "ex" | "exs" | "clj" | "hs" | "ml" | "vim" | "ps1" | "bat" | "cmake" | "makefile")
}
