const FLAG_TERMS: &[&str] = &[
    "phase", "stage", "step", "part", "pass", "round", "iteration",
    "sprint", "cycle", "increment", "wave", "batch", "section",
    "period", "era", "epoch", "chapter", "episode", "instalment",
    "leg", "lap", "level",
];

const REVIEW_CODE_TERMS: &[&str] = &["review", "finding", "blocker"];

const CI_PATTERNS: &[&str] = &[
    ".github/workflows",
    ".gitlab-ci",
    "jenkinsfile",
    "dockerfile",
    "docker-compose",
];

pub struct Match {
    pub file: String,
    pub line: usize,
    pub text: String,
    pub term: String,
}

pub fn is_ci_file(path: &str) -> bool {
    let lower = path.to_lowercase();
    CI_PATTERNS.iter().any(|p| lower.contains(p))
}

pub fn is_numeral(word: &str) -> bool {
    if word.is_empty() {
        return false;
    }
    if word.chars().all(|c| c.is_ascii_digit()) {
        return true;
    }
    let upper = word.to_uppercase();
    upper.len() <= 4 && upper.chars().all(|c| matches!(c, 'I' | 'V' | 'X' | 'L' | 'C' | 'D' | 'M'))
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

pub fn scan_text(input: &str, source: &str, matches: &mut Vec<Match>) {
    let markdown = source.to_lowercase().ends_with(".md");
    for (i, line) in input.lines().enumerate() {
        let found = check_line(line)
            .or_else(|| if markdown { check_bare_numeral_header(line) } else { None });
        if let Some(term) = found {
            matches.push(Match {
                file: source.to_string(),
                line: i + 1,
                text: line.trim().to_string(),
                term,
            });
        }
    }
}

pub fn is_scannable(ext: &str) -> bool {
    matches!(ext, "" | "md" | "txt" | "rst" | "py" | "rs" | "js" | "ts" | "jsx" | "tsx" | "go" | "java" | "c" | "cpp" | "h" | "hpp" | "rb" | "sh" | "yaml" | "yml" | "toml" | "json" | "xml" | "html" | "css" | "sql" | "r" | "lua" | "swift" | "kt" | "scala" | "ex" | "exs" | "clj" | "hs" | "ml" | "vim" | "ps1" | "bat" | "cmake" | "makefile")
}
