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

/// Re-exported from `host-grammar` so the checker (here) and the generator
/// (`host-lifecycle`) share one definition of a numeral.
pub use host_grammar::is_numeral;

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
                // A version/product designator in all-caps ("NT 3.1", "SDK 2.1",
                // "DOS 6.2") reads as a version string, not a milestone code.
                // Title-case nouns ("Decision 2.1") and ordinary lowercase words
                // ("in 2.1") still warn — only an all-uppercase acronym is skipped.
                if let Some(prev) = orig.get(i - 1) {
                    let po = prev.trim_matches(|c: char| !c.is_alphanumeric());
                    if po.len() >= 2 && po.chars().all(|c| c.is_ascii_uppercase()) {
                        continue;
                    }
                }
            }
            return Some(code.to_string());
        }
    }

    None
}

// Tier 3 (warn): a bare review-code (one letter + digits, e.g. "F1", "B2")
// used as a leading label — the first non-bullet token of a line, immediately
// followed by a label delimiter (an em/en-dash token, or a trailing colon on
// the code itself). This is the section-5 code-as-name tell in its bare leading
// form, where no review/finding/blocker noun precedes the code (a PR body that
// structures its fixes as "F1 — …", "F2 — …"). Warned, not flagged: a
// multi-letter device noun ("COM1") is already excluded by the one-letter code
// shape, but a single-letter hardware reference designator ("R1 — 10kΩ
// resistor") fits the same shape, so the call is advisory rather than blocking.
pub fn check_code_label_prefix(line: &str) -> Option<String> {
    let tokens: Vec<&str> = line.split_whitespace().collect();
    // The first token with an alphanumeric core — skips leading bullets and
    // emphasis markers ("-", "*", "**", "//", "#") so a markdown list item
    // ("- **F1** — …") is read the same as a bare "F1 — …".
    let start = tokens
        .iter()
        .position(|t| t.chars().any(|c| c.is_alphanumeric()))?;
    let raw = tokens[start];
    // "F2:" — the code carries its own label colon, no following dash needed.
    if let Some(stem) = raw.strip_suffix(':') {
        let core = stem.trim_matches(|c: char| !c.is_alphanumeric());
        if is_review_code(core) {
            return Some(core.to_string());
        }
    }
    let core = raw.trim_matches(|c: char| !c.is_alphanumeric());
    if !is_review_code(core) {
        return None;
    }
    // "F1 —" — a bare dash delimiter as the next whitespace token.
    match tokens.get(start + 1).copied() {
        Some("—") | Some("–") | Some("-") => Some(core.to_string()),
        _ => None,
    }
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
/// Co-Authored-By is a discretionary attribution trailer: a co-author's name
/// or a tool's version string (e.g. "Claude Opus 4.8") is the author's to set,
/// not ours to police. Respect it — never flag or warn on this line.
fn is_coauthor_trailer(line: &str) -> bool {
    let key = "co-authored-by:";
    line.trim_start()
        .get(..key.len())
        .is_some_and(|p| p.eq_ignore_ascii_case(key))
}

pub fn classify_line(line: &str, markdown: bool) -> Option<(Severity, String)> {
    if is_coauthor_trailer(line) {
        return None;
    }
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
    if let Some(t) = check_code_label_prefix(line) {
        return Some((Severity::Warn, t));
    }
    None
}

// Blank out every word-boundaried, case-insensitive occurrence of a sanctioned
// phrase so the classifier never sees it. The boundary requirement (a
// non-alphanumeric neighbour or a string edge on each side) is what keeps an
// allow entry specific: `phase 1` masks `phase 1` but NOT the longer tell
// `phase 12`, so allow-listing one occurrence cannot silently clear another.
// `allow_lc` entries are pre-lowercased (ASCII) by the caller; ASCII-only
// folding keeps byte indices aligned between the search copy and the original.
fn mask_allowed(line: &str, allow_lc: &[String]) -> String {
    if allow_lc.is_empty() {
        return line.to_string();
    }
    let lower = line.to_ascii_lowercase();
    let lb = lower.as_bytes();
    let mut out = line.as_bytes().to_vec();
    for p in allow_lc {
        if p.is_empty() {
            continue;
        }
        let mut start = 0;
        while let Some(rel) = lower[start..].find(p.as_str()) {
            let at = start + rel;
            let end = at + p.len();
            let left_ok = at == 0 || !lb[at - 1].is_ascii_alphanumeric();
            let right_ok = end == lb.len() || !lb[end].is_ascii_alphanumeric();
            if left_ok && right_ok {
                for b in &mut out[at..end] {
                    *b = b' ';
                }
            }
            start = end;
        }
    }
    String::from_utf8(out).unwrap_or_else(|_| line.to_string())
}

pub fn scan_text(input: &str, source: &str, matches: &mut Vec<Match>) {
    scan_text_with_allow(input, source, &[], matches);
}

// As `scan_text`, but a repo's sanctioned phrases (`.host-lint-allow`,
// ASCII-lowercased by the caller) are masked out of each line before
// classification. A line still flags on any tell the mask leaves behind, and
// the reported `text` is the original line so the author sees real context.
pub fn scan_text_with_allow(
    input: &str,
    source: &str,
    allow_lc: &[String],
    matches: &mut Vec<Match>,
) {
    let markdown = source.to_lowercase().ends_with(".md");
    for (i, line) in input.lines().enumerate() {
        let scanned = mask_allowed(line, allow_lc);
        if let Some((severity, term)) = classify_line(&scanned, markdown) {
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

/// True if `rel` (a repo-relative, `/`-separated path) matches any ignore
/// pattern. Patterns are gitignore-lite: an exact path (`MEMORY.md`), a `*`
/// glob that matches within a single path segment (`plan/*/README.md`), or a
/// trailing-slash directory prefix (`archive/`) ignoring everything beneath it.
/// `--all` honours these so a migrated project can exclude its append-only
/// record from the audit without the engine learning any methodology policy.
pub fn path_ignored(rel: &str, patterns: &[String]) -> bool {
    patterns.iter().any(|p| {
        if let Some(dir) = p.strip_suffix('/') {
            !dir.is_empty() && (rel == dir || rel.starts_with(&format!("{dir}/")))
        } else {
            glob_path(p, rel)
        }
    })
}

fn glob_path(pat: &str, path: &str) -> bool {
    let pp: Vec<&str> = pat.split('/').collect();
    let tp: Vec<&str> = path.split('/').collect();
    pp.len() == tp.len() && pp.iter().zip(&tp).all(|(p, t)| seg_glob(p.as_bytes(), t.as_bytes()))
}

// Wildcard match within one path segment: `*` matches any run of characters
// (two-pointer glob with backtracking).
fn seg_glob(pat: &[u8], s: &[u8]) -> bool {
    let (mut p, mut t) = (0usize, 0usize);
    let (mut star, mut mark): (Option<usize>, usize) = (None, 0);
    while t < s.len() {
        if p < pat.len() && pat[p] == b'*' {
            star = Some(p);
            mark = t;
            p += 1;
        } else if p < pat.len() && pat[p] == s[t] {
            p += 1;
            t += 1;
        } else if let Some(sp) = star {
            p = sp + 1;
            mark += 1;
            t = mark;
        } else {
            return false;
        }
    }
    while p < pat.len() && pat[p] == b'*' {
        p += 1;
    }
    p == pat.len()
}

pub fn is_scannable(ext: &str) -> bool {
    matches!(ext, "" | "md" | "txt" | "rst" | "py" | "rs" | "js" | "ts" | "jsx" | "tsx" | "go" | "java" | "c" | "cpp" | "h" | "hpp" | "rb" | "sh" | "yaml" | "yml" | "toml" | "json" | "xml" | "html" | "css" | "sql" | "r" | "lua" | "swift" | "kt" | "scala" | "ex" | "exs" | "clj" | "hs" | "ml" | "vim" | "ps1" | "bat" | "cmake" | "makefile")
}
