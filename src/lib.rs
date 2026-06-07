const FLAG_TERMS: &[&str] = &[
    "phase", "stage", "step", "part", "pass", "round", "iteration",
    "sprint", "cycle", "increment", "wave", "batch", "section",
    "period", "era", "epoch", "chapter", "episode", "instalment",
    "leg", "lap", "level",
];

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
    }

    None
}

pub fn scan_text(input: &str, source: &str, matches: &mut Vec<Match>) {
    for (i, line) in input.lines().enumerate() {
        if let Some(term) = check_line(line) {
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
