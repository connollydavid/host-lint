const FLAG_TERMS: &[&str] = &[
    "phase", "stage", "step", "part", "pass", "round", "iteration",
    "sprint", "cycle", "increment", "wave", "batch", "section",
    "period", "era", "epoch", "chapter", "episode", "instalment",
    "leg", "lap", "level",
];

const REVIEW_CODE_TERMS: &[&str] = &["review", "finding", "blocker"];

// Tier 3 (warn): filing-system code nouns whose numbered label is a milestone
// code used as a name. Warned, not flagged, because the same nouns have
// ordinary uses ("see item 5 in the list"). `pub` so property tests can exclude
// these from the "safe designator" generator (a warn-noun like "WI" is not safe).
pub const WARN_NOUNS: &[&str] = &["work-item", "workitem", "wi"];

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
    /// Citation for a prose tell (the tropes.fyi catalog name + rhetoric term);
    /// empty for a naming tell, which is self-explanatory.
    pub cite: String,
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
// allow entry specific: a sanctioned phrase masks only its exact occurrence, not
// a longer tell that merely shares its prefix, so allow-listing one occurrence
// cannot silently clear another. `allow_lc` entries are pre-lowercased (ASCII) by
// the caller; ASCII-only folding keeps byte indices aligned between the search
// copy and the original.
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

// === LEXICON: the provenance-enforced contextual allowlist (issue #13) ===
//
// A LEXICON file is the sole source of truth for tell-shaped tokens that are
// legitimate vocabulary in a project (`Windows 3.1`, `COM1`) or cited tracker
// references (`#7 https://…`). Each entry is the *full contextual phrase* that is
// masked before detection; a bare numeral is never an entry. Because a sound,
// declarable escape now exists, the naming-warn tier can escalate WARN -> ERROR
// under the committed `strict` directive. The guards below keep a weak agent (or
// a careless hand-edit) from abusing the escape.

/// One parsed LEXICON entry: the contextual `phrase` masked before detection, and
/// the optional cited `url` recorded as provenance (a tracker reference must carry
/// one). The URL is metadata only — it is never masked, only the phrase is.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LexiconEntry {
    pub phrase: String,
    pub url: Option<String>,
}

/// The `# host-lint: strict` directive that turns on warn->error escalation. A
/// comment-shaped line so it is invisible to the phrase parser, explicit so it is
/// auditable in the committed file.
pub fn is_strict_directive(line: &str) -> bool {
    line.trim()
        .strip_prefix('#')
        .is_some_and(|r| r.trim() == "host-lint: strict")
}

/// Parse one LEXICON line into an entry, or `None` for a blank, comment, or
/// directive line. A comment is `#` followed by a non-digit (so `# note` and
/// `## heading` are comments, but `#7 …` is a hash-number entry — this is what
/// keeps the comment marker from colliding with the `#N` reference shape). A
/// trailing `http(s)://…` whitespace token is split off as the cited URL.
pub fn parse_lexicon_line(line: &str) -> Option<LexiconEntry> {
    let t = line.trim();
    if t.is_empty() {
        return None;
    }
    if let Some(rest) = t.strip_prefix('#') {
        // `#` then a non-digit (or nothing) is a comment/directive, not an entry.
        if !rest.chars().next().is_some_and(|c| c.is_ascii_digit()) {
            return None;
        }
    }
    if let Some((head, last)) = t.rsplit_once(char::is_whitespace) {
        if last.starts_with("http://") || last.starts_with("https://") {
            return Some(LexiconEntry {
                phrase: head.trim_end().to_string(),
                url: Some(last.to_string()),
            });
        }
    }
    Some(LexiconEntry { phrase: t.to_string(), url: None })
}

/// A bare tracker reference: `#N` or `owner/repo#N`. These are citation-gated —
/// an entry of this shape must carry a URL, because the offline matcher cannot
/// tell a real `#7` from a phantom `#999`, so the URL is its only provenance.
/// `PROJ-NNNN` is deliberately NOT gated here: it is syntactically identical to
/// standards tokens the host writes (`RFC-2119`, `UTF-8`, `UAX-29`), so gating it
/// would demand a phantom URL for legitimate vocabulary — it is treated as a
/// plain phrase (carries a legitimizing word, governed by the guards below).
fn is_tracker_ref(phrase: &str) -> bool {
    if let Some(d) = phrase.strip_prefix('#') {
        return !d.is_empty() && d.bytes().all(|b| b.is_ascii_digit());
    }
    if let Some((path, num)) = phrase.split_once('#') {
        let segs: Vec<&str> = path.split('/').collect();
        return segs.len() == 2
            && segs.iter().all(|s| !s.is_empty())
            && !num.is_empty()
            && num.bytes().all(|b| b.is_ascii_digit());
    }
    false
}

/// Validate one entry for registration. `Ok(())` means it may be trusted to mask;
/// `Err(reason)` is a human-actionable rejection. Three guards, all reusing the
/// detection engine rather than inventing new tell logic:
///   - **citation gate** — a bare tracker ref (`#N`, `owner/repo#N`) must carry a URL.
///   - **G1 master-key** — a non-reference phrase must hold at least one letter, so a
///     bare `5.5` (which would silently clear every occurrence tree-wide) is refused.
///   - **G2 no-laundering** — a phrase that is *itself* a flag-tier tell (a phase-synonym
///     label, say) is refused: you rename a real tell, you do not allow-list it. A mere
///     warn-tier phrase (`Windows 3.1`, `Decision 2.1`) is the legitimate case, accepted.
pub fn validate_lexicon_entry(e: &LexiconEntry) -> Result<(), String> {
    if e.phrase.is_empty() {
        return Err("empty phrase".to_string());
    }
    if is_tracker_ref(&e.phrase) {
        if e.url.is_none() {
            return Err(format!(
                "'{}' is a tracker reference with no URL — register it as '{} <url>' so the link is provenance, not a phantom",
                e.phrase, e.phrase
            ));
        }
        return Ok(());
    }
    if !e.phrase.chars().any(|c| c.is_ascii_alphabetic()) {
        return Err(format!(
            "'{}' is a bare numeral/code — a master key that would clear every occurrence; add the legitimizing word (e.g. 'Windows {}') or rename the work",
            e.phrase, e.phrase
        ));
    }
    if let Some((Severity::Flag, term)) = classify_line(&e.phrase, false) {
        return Err(format!(
            "'{}' is itself a tell ({}) — rename the work after its content; the lexicon legitimizes vocabulary, it does not silence real tells",
            e.phrase, term
        ));
    }
    Ok(())
}

pub fn scan_text(input: &str, source: &str, matches: &mut Vec<Match>) {
    scan_text_with_allow(input, source, &[], matches);
}

// As `scan_text`, but a repo's sanctioned phrases (the LEXICON, ASCII-lowercased
// by the caller) are masked out of each line before classification. A line still
// flags on any tell the mask leaves behind, and the reported `text` is the
// original line so the author sees real context. Non-strict: the warn tier stays
// advisory (this is the entry point external callers keep).
pub fn scan_text_with_allow(
    input: &str,
    source: &str,
    allow_lc: &[String],
    matches: &mut Vec<Match>,
) {
    scan_text_with_allow_strict(input, source, allow_lc, false, matches);
}

// The full scan: under `strict`, a naming-warn the mask did not clear escalates to
// a blocking flag (issue #13 — the LEXICON makes a sound escape declarable, so an
// *un*-declared tell-shaped token is now a hard signal, not merely advisory). The
// escalated match carries a remedy in `cite` so the audience can act. Prose tells
// (host-grammar) are a different tier and are not escalated here.
pub fn scan_text_with_allow_strict(
    input: &str,
    source: &str,
    allow_lc: &[String],
    strict: bool,
    matches: &mut Vec<Match>,
) {
    let markdown = source.to_lowercase().ends_with(".md");
    for (i, line) in input.lines().enumerate() {
        let scanned = mask_allowed(line, allow_lc);
        if let Some((mut severity, term)) = classify_line(&scanned, markdown) {
            let mut cite = String::new();
            if strict && severity == Severity::Warn {
                severity = Severity::Flag;
                cite = "not in LEXICON; rename or run: host-lint lexicon add".to_string();
            }
            matches.push(Match {
                file: source.to_string(),
                line: i + 1,
                text: line.trim().to_string(),
                term,
                severity,
                cite,
            });
        }
    }
}

// Best-effort line lookup: the 1-based line of `source` whose text contains the
// start of `needle`; 1 when not found (titles and synthetic excerpts).
fn locate_line(input: &str, needle: &str) -> usize {
    let probe = needle.split_whitespace().take(4).collect::<Vec<_>>().join(" ");
    if probe.is_empty() {
        return 1;
    }
    input
        .lines()
        .position(|l| l.contains(&probe))
        .map(|i| i + 1)
        .unwrap_or(1)
}

/// Scan `input` as prose for agentic tells (the host-grammar engine), pushing
/// each as an advisory `Warn` match, plus one document-level match when the tell
/// density crosses the threshold. Used for titles, comments, and `--prose` docs;
/// never blocks (Warn = exit 3), so a flagged draft still lands.
pub fn scan_prose_text(input: &str, source: &str, matches: &mut Vec<Match>) {
    // A markdown source is scanned structurally (code blocks excluded, headings
    // not counted as paragraphs); anything else as plain prose.
    let markdown = source.to_lowercase().ends_with(".md");
    let tells = if markdown {
        host_grammar::scan_prose_markdown(input)
    } else {
        host_grammar::scan_prose_parallel(input)
    };
    for t in tells {
        matches.push(Match {
            file: source.to_string(),
            line: locate_line(input, &t.excerpt),
            text: t.excerpt,
            term: t.id.to_string(),
            severity: Severity::Warn,
            cite: t.cite.to_string(),
        });
    }
    let score = if markdown {
        host_grammar::tell_score_markdown(input)
    } else {
        host_grammar::tell_score(input)
    };
    if score.over_threshold {
        matches.push(Match {
            file: source.to_string(),
            line: 1,
            text: format!(
                "agentic-tell density {:.2} across {} sentences ({} tells)",
                score.density, score.sentences, score.tells
            ),
            term: "tell-density".to_string(),
            severity: Severity::Warn,
            cite: "tropes.fyi: many devices together".to_string(),
        });
    }
}

/// Escalate decoration tells on the commit subject to blocking `Flag`. The first
/// line of a commit message — or a gh issue/PR title piped on stdin — becomes the
/// squash-merge subject / front-door text, so it is held to the same no-decoration
/// bar as the front-door docs: an em-dash, arrow, or smart quote there blocks
/// rather than warns. Body prose and other tells keep their advisory `Warn`. A
/// `decoration` match whose excerpt occurs in `subject` is the subject's.
pub fn escalate_subject_decoration(subject: &str, matches: &mut [Match]) {
    for m in matches.iter_mut() {
        if m.term == "decoration" && subject.contains(&m.text) {
            m.severity = Severity::Flag;
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

// Kani proof harnesses (the host-prove `kani-conformance` lane). `#[cfg(kani)]`
// keeps them out of `cargo build`/`cargo test`, so the release artifact stays
// byte-identical — they run only under `cargo kani`. Targets are chosen to be
// CBMC-tractable: char- and byte-level predicates, NOT `str::split`/`Vec`/`String`,
// which pull in `memchr` + heap modeling and blow CBMC up. Each proves a contract
// for ALL byte values at a bounded length — a stronger discharge than an example
// test. Dispositioned `kani:<harness>` in host-lint.obligations.
#[cfg(kani)]
mod kani_proofs {
    use super::*;

    // "#<digit>" is an internal code label (rule-success.DetectInternalCodeAsName).
    // is_review_code is char-based (strip_prefix + chars) — no split, no memchr.
    #[kani::proof]
    #[kani::unwind(4)]
    fn verify_review_code_accepts_hash_digit() {
        let d: u8 = kani::any();
        kani::assume(d.is_ascii_digit());
        let bytes = [b'#', d];
        let word = core::str::from_utf8(&bytes).unwrap();
        assert!(is_review_code(word));
    }

    // A two-letter word (e.g. a device noun like "NT") is NOT a code label
    // (rule-failure.DetectInternalCodeAsName.1): a letter prefix needs digits after.
    #[kani::proof]
    #[kani::unwind(4)]
    fn verify_review_code_rejects_two_letters() {
        let a: u8 = kani::any();
        let b: u8 = kani::any();
        kani::assume(a.is_ascii_alphabetic() && b.is_ascii_alphabetic());
        let bytes = [a, b];
        let word = core::str::from_utf8(&bytes).unwrap();
        assert!(!is_review_code(word));
    }

    // The '*' segment-glob wildcard matches any segment — a pure byte-index matcher
    // (no heap, no memchr): the Kani-ideal shape. Proves wildcard semantics and
    // panic-freedom for every 4-byte segment. (Extra code-correctness proof; the
    // .host-lintignore glob matcher has no allium obligation of its own.)
    #[kani::proof]
    #[kani::unwind(6)]
    fn verify_seg_glob_star_matches_any() {
        let s: [u8; 4] = kani::any();
        assert!(seg_glob(b"*", &s));
    }
}
