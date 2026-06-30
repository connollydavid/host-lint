use std::fs;
use std::path::Path;
use std::process::Command;

// Tier 1 (flag): the high-centrality words for a unit of iterative project work,
// the agentic ordinal-naming tell the gate exists to block. The set is grounded in
// measured corpus data (plan/0055): each either has proven in-project tells
// (`stage` named six work units in this repo's own history) or near-zero
// false-positive exposure in real code. Domain-heavy words (section, round, step,
// epoch, ...) live in the advisory tier below, not here.
pub const FLAG_TERMS: &[&str] = &[
    "phase", "stage", "iteration", "sprint", "cycle", "increment", "wave",
    "episode", "instalment", "leg", "lap",
    // Positional references to a milestone checklist item (host#16): the
    // "box N" / "boxes N-M" / "steps N-M" shape, the same ordinal-by-position
    // tell aimed at the "[ ]"/"[x]" marks. Plurals are listed explicitly
    // because the scan matches a whole whitespace token.
    "box", "boxes", "steps",
];

// Tier 3 (warn): words whose ordinal use is overwhelmingly domain vocabulary, not
// the naming of a work unit. Measured against ~35.5k real .rs files (plan/0055,
// call/0037): `round` (cipher rounds), `level` (log/DTD levels), `step` (tutorial
// steps), `pass` (compiler passes), `part`, `section` (RFC/doc sections — 2785
// hits, the largest source), `chapter` (book chapters), `epoch` (ML training),
// `batch` (jobs), `era`/`period` (time) all collide with ordinary code even at
// immediate adjacency, and each is a complete flag the LEXICON cannot escape. They
// warn rather than block; strict still escalates an undeclared occurrence to a
// flag, and the gather lane still surfaces it.
pub const WARN_ORDINAL_TERMS: &[&str] = &[
    "pass", "round", "step", "level", "part",
    "section", "chapter", "epoch", "batch", "era", "period",
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

// `gather` (discovery): common words that legitimately precede a numeral and are
// not position labels ("in 2024", "see 3", "line 42"). Kept small; the gather is
// recall-biased and the operator triages the residue.
const GATHER_STOP: &[&str] = &[
    "the", "a", "an", "of", "in", "on", "at", "to", "for", "by", "from", "with",
    "and", "or", "is", "are", "was", "were", "be", "as", "it", "this", "that",
    "about", "over", "under", "up", "all", "see", "line", "lines", "item",
    "items", "issue", "issues", "commit", "rev", "port", "row", "col", "len",
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
    /// A non-locatable, whole-document prose diagnosis (density, anaphora, bullet
    /// patterns): informational only, never gates (exit 0). There is no single span to
    /// edit, so it sits outside the clean-to-zero bar.
    Note,
}

pub struct Match {
    pub file: String,
    pub line: usize,
    /// 1-based character column of the tell on its line; 0 when the tell is line-level
    /// (a naming tell) or non-locatable (an advisory whole-document prose diagnosis).
    pub col: usize,
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

// A numeric range ("4-8"): two non-empty all-digit parts, each at most three
// digits, joined by a single hyphen (ASCII or a typographic en/em-dash). A
// positional reference often spans a contiguous span of checklist items (a range
// like "4-8"), which `is_numeral` does not accept. The three-digit bound keeps a
// four-digit side out: a date or a year range ("2024-01", "1999-2024") reads as
// a year, not a checklist range.
fn is_num_range(word: &str) -> bool {
    let normalized = word.replace(['–', '—'], "-");
    match normalized.split_once('-') {
        Some((a, b)) => {
            !a.is_empty()
                && !b.is_empty()
                && a.len() <= 3
                && b.len() <= 3
                && a.bytes().all(|c| c.is_ascii_digit())
                && b.bytes().all(|c| c.is_ascii_digit())
        }
        None => false,
    }
}

// Whether the token immediately after a tell-noun reads as a *blocking* positional
// numeral. Accepts an arabic integer or single decimal ("2", "5.5"), a checklist
// range ("4-8"), or a multi-letter Roman numeral written in uppercase in the
// source ("IV", "VIII"). `is_numeral` also accepts a single-letter Roman
// (I, V, X, L, C, D, M), but those collide with the English pronoun "I" and with
// language/identifier letters ("port the pass to C"), and a lowercase token that
// merely parses as Roman ("mix", "vi") is an ordinary word — so neither blocks
// here. A genuine "Phase 1" is written with a digit; "Phase i" is too ambiguous
// to block (plan/0055).
fn is_blocking_numeral(lower: &str, orig: &str) -> bool {
    if lower.is_empty() {
        return false;
    }
    if is_num_range(lower) {
        return true;
    }
    // Arabic integer or single decimal (mirrors host-grammar's is_numeral arabic
    // branch): at most two non-empty all-digit parts.
    let parts: Vec<&str> = lower.split('.').collect();
    if parts.len() <= 2 && parts.iter().all(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit())) {
        return true;
    }
    // Multi-letter Roman numeral, uppercase in the source.
    is_numeral(lower)
        && lower.chars().count() >= 2
        && orig.chars().any(|c| c.is_alphabetic())
        && orig.chars().filter(|c| c.is_alphabetic()).all(|c| c.is_ascii_uppercase())
}

pub fn check_line(line: &str) -> Option<String> {
    let lower = line.to_lowercase();
    let words: Vec<&str> = lower.split_whitespace().collect();
    let orig_words: Vec<&str> = line.split_whitespace().collect();

    for (i, word) in words.iter().enumerate() {
        let clean = word.trim_matches(|c: char| !c.is_alphanumeric() && c != '-');
        if FLAG_TERMS.contains(&clean) {
            // A tell-noun immediately followed by a blocking positional numeral.
            // Only the immediately following token counts: a numeral two words
            // away ("step into 3", "port the pass to C") is ordinary English, not
            // a positional reference (plan/0055 dropped the two-word window). The
            // glued form (the noun joined to a numeral by a hyphen) is out of
            // scope: a legitimate glued term has no numeral-free LEXICON prefix to
            // declare, so it could not be escaped, and it is the same class as a
            // noun-glued numeral.
            if let Some(next) = words.get(i + 1) {
                let next_clean = next.trim_matches(|c: char| !c.is_alphanumeric() && c != '-');
                let next_orig = orig_words
                    .get(i + 1)
                    .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric() && c != '-'))
                    .unwrap_or(next_clean);
                if is_blocking_numeral(next_clean, next_orig) {
                    return Some(clean.to_string());
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
    // A bare integer of three or more digits reads as a status code or numeric
    // key ("200: OK", "404: not found"), not a milestone label. The dotted form
    // ("5.5:") and short ordinals ("3:") still flag (plan/0055).
    if !code.contains('.') && code.len() >= 3 {
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
        // W3: a demoted verb/measurement ordinal noun immediately followed by a
        // blocking positional numeral ("pass 2", "round 2", "level 3"). Advisory,
        // because the noun's ordinary verb/measurement use is indistinguishable
        // (plan/0055, call/0037). Immediate adjacency only, so "step into 3" and
        // "port the pass to C" stay clean.
        if WARN_ORDINAL_TERMS.contains(&clean) {
            if let Some(next) = words.get(i + 1) {
                let nc = next.trim_matches(|c: char| !c.is_alphanumeric() && c != '-');
                let no = orig
                    .get(i + 1)
                    .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric() && c != '-'))
                    .unwrap_or(nc);
                if is_blocking_numeral(nc, no) {
                    return Some(clean.to_string());
                }
            }
        }
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
        // A four-digit (or longer) component reads as a year, not a bare ordinal:
        // a changelog "## 2024" or "## 2024.01" heading is not a position label.
        // Mirrors the gather lane's year skip (plan/0055).
        if parts.iter().any(|p| p.len() >= 4) {
            return None;
        }
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

/// A jira-key project key (`PROJ`, `TEAM2`): an uppercase letter then uppercase
/// letters or digits. A LEXICON opts a key into citation-gating via the directive
/// `# host-lint: jira-key <KEY>`.
fn is_jira_key(s: &str) -> bool {
    let mut chars = s.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_uppercase())
        && chars.all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
}

/// Parse a `# host-lint: jira-key <KEY> [<KEY>...]` directive into its declared
/// project keys, or `None` for any other line. Comment-shaped, so the phrase
/// parser ignores it — the same idiom as the strict directive.
pub fn parse_jira_keys(line: &str) -> Option<Vec<String>> {
    let body = line.trim().strip_prefix('#')?.trim();
    let rest = body.strip_prefix("host-lint: jira-key")?;
    if !rest.is_empty() && !rest.starts_with(|c: char| c.is_whitespace()) {
        return None;
    }
    let keys: Vec<String> = rest.split_whitespace().filter(|k| is_jira_key(k)).map(String::from).collect();
    if keys.is_empty() { None } else { Some(keys) }
}

/// A bare tracker reference whose only provenance is a URL: `#N`, `owner/repo#N`,
/// or an opted-in jira-key `PROJ-NNNN`. The offline matcher cannot tell a real
/// `#7` from a phantom `#999`, so an entry of this shape must carry a URL.
/// `PROJ-NNNN` is gated ONLY for a project key the LEXICON declares (`# host-lint:
/// jira-key PROJ`) — opt-in, because the shape is identical to standards tokens the
/// host writes (`RFC-2119`, `UTF-8`), which must stay plain vocabulary by default.
fn is_tracker_ref(phrase: &str, jira_keys: &[String]) -> bool {
    if let Some(d) = phrase.strip_prefix('#') {
        return !d.is_empty() && d.bytes().all(|b| b.is_ascii_digit());
    }
    if let Some((path, num)) = phrase.split_once('#') {
        let segs: Vec<&str> = path.split('/').collect();
        if segs.len() == 2
            && segs.iter().all(|s| !s.is_empty())
            && !num.is_empty()
            && num.bytes().all(|b| b.is_ascii_digit())
        {
            return true;
        }
    }
    if let Some((key, num)) = phrase.split_once('-') {
        if jira_keys.iter().any(|k| k == key)
            && !num.is_empty()
            && num.bytes().all(|b| b.is_ascii_digit())
        {
            return true;
        }
    }
    false
}

/// Validate one entry for registration. `Ok(())` means it may be trusted to mask;
/// `Err(reason)` is a human-actionable rejection. Three guards, all reusing the
/// detection engine rather than inventing new tell logic:
///   - **citation gate** — a bare tracker ref (`#N`, `owner/repo#N`, or an opted-in
///     jira-key `PROJ-NNNN`) must carry a URL. `jira_keys` are the project keys the
///     LEXICON declared; empty = no jira-key gating, so `RFC-2119` stays vocabulary.
///   - **G1 master-key** — a non-reference phrase must hold at least one letter, so a
///     bare `5.5` (which would silently clear every occurrence tree-wide) is refused.
///   - **G2 no-laundering** — a phrase that is *itself* a flag-tier tell (a phase-synonym
///     label, say) is refused: you rename a real tell, you do not allow-list it. A phrase
///     that merely *carries* a position noun as a standalone word (`phase`, `step`, `review`)
///     is refused for the same reason: masking it would blank that noun out of a real
///     `<noun> N` tell, silencing the whole class and defeating strict (plan/0055). A mere
///     warn-tier phrase with no such noun (`Windows 3.1`, `Decision 2.1`) is the legitimate
///     case, accepted.
pub fn validate_lexicon_entry(e: &LexiconEntry, jira_keys: &[String]) -> Result<(), String> {
    if e.phrase.is_empty() {
        return Err("empty phrase".to_string());
    }
    if is_tracker_ref(&e.phrase, jira_keys) {
        let url = match &e.url {
            None => {
                return Err(format!(
                    "'{}' is a tracker reference with no URL — register it as '{} <url>' so the link is provenance, not a phantom",
                    e.phrase, e.phrase
                ))
            }
            Some(u) => u,
        };
        // Offline provenance: the cited URL must actually reference the same number,
        // so a phantom '#999' cited to an unrelated link cannot mask a real
        // 'review #999' (plan/0055, L2). Liveness (the link resolves) still needs the
        // explicit `lexicon --check-urls` lane; a network fetch does not gate by default.
        let number: String = e.phrase.chars().filter(|c| c.is_ascii_digit()).collect();
        if !number.is_empty() && !url.contains(&number) {
            return Err(format!(
                "'{}' cites '{}', which does not reference {} — cite the URL that actually points to the tracker item",
                e.phrase, url, number
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
    // A phrase that carries a position noun as a standalone word would, when
    // masked, blank that noun out of a real "<noun> N" tell — silencing the whole
    // class repo-wide and defeating strict (the masked line never produces the
    // warn strict escalates). Refuse it (plan/0055, L1). Over-strict by design: a
    // legitimate multiword phrase that happens to contain a bare position noun is
    // rephrased; safety beats permissiveness here.
    if let Some(noun) = e.phrase.split_whitespace().find_map(|w| {
        let t = w
            .trim_matches(|c: char| !c.is_alphanumeric() && c != '-')
            .to_ascii_lowercase();
        (FLAG_TERMS.contains(&t.as_str())
            || WARN_ORDINAL_TERMS.contains(&t.as_str())
            || REVIEW_CODE_TERMS.contains(&t.as_str())
            || WARN_NOUNS.contains(&t.as_str()))
        .then_some(t)
    }) {
        return Err(format!(
            "'{}' carries the position noun '{}' as a word — masking it would blank that noun out of a real '{} N' tell; rename the work after its content rather than allow-list the tell shape",
            e.phrase, noun, noun
        ));
    }
    Ok(())
}

/// The repo's LEXICON (issue #13): the validated allowlist phrases (lowercased for
/// case-insensitive masking), the committed `strict` flag, the declared tracker keys,
/// and the parsed entries (for the `lexicon` subcommand). Lives in the shared engine so
/// host-lint's binary and an in-process embedder (host-lifecycle) load and mask the same
/// declared phrases identically (host-lifecycle#2).
pub struct Lexicon {
    pub phrases_lc: Vec<String>,
    pub strict: bool,
    pub jira_keys: Vec<String>,
    pub entries: Vec<LexiconEntry>,
}

/// Read and validate the repo's `LEXICON` file (at `root`). An invalid entry — a master
/// key, a tracker ref with no URL, a laundered tell — is reported to stderr and dropped:
/// it never masks, so soundness does not depend on the file being hand-edited correctly.
/// A missing file yields an empty lexicon (the feature is opt-in per repo). The single
/// loader both the CLI and an embedder call, so the prose/`--docs` lane masks the same
/// declared phrases everywhere.
pub fn load_lexicon(root: &Path) -> Lexicon {
    let mut lex = Lexicon { phrases_lc: Vec::new(), strict: false, jira_keys: Vec::new(), entries: Vec::new() };
    if root.as_os_str().is_empty() {
        return lex;
    }
    let content = match fs::read_to_string(root.join("LEXICON")) {
        Ok(c) => c,
        Err(_) => return lex,
    };
    // Directives first (strict, jira-key), collected before any entry so an
    // entry's validation sees every declared key regardless of line order.
    for line in content.lines() {
        if is_strict_directive(line) {
            lex.strict = true;
        } else if let Some(keys) = parse_jira_keys(line) {
            lex.jira_keys.extend(keys);
        }
    }
    // Then the entries, validated against the collected directives.
    for line in content.lines() {
        if is_strict_directive(line) || parse_jira_keys(line).is_some() {
            continue;
        }
        let Some(entry) = parse_lexicon_line(line) else { continue };
        if let Err(reason) = validate_lexicon_entry(&entry, &lex.jira_keys) {
            eprintln!("host-lint: LEXICON entry ignored ({reason})");
            continue;
        }
        lex.phrases_lc.push(entry.phrase.to_ascii_lowercase());
        lex.entries.push(entry);
    }
    lex
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

// The info string of a markdown fence line (the text after ``` / ~~~), or `None`
// if `line` is not a fence. A fence is opened by 3+ backticks or tildes with at
// most 3 leading spaces; 4+ spaces is an indented code block, not a fence. A bare
// fence (empty info) closes a block; `host-lint:ignore` as the info opens a region
// the naming scan skips (call/0019). Used only for markdown sources.
fn fence_info(line: &str) -> Option<(char, usize, &str)> {
    if line.chars().take_while(|c| *c == ' ').count() >= 4 {
        return None;
    }
    let t = line.trim_start();
    let marker = t.chars().next().filter(|c| *c == '`' || *c == '~')?;
    let run = t.chars().take_while(|c| *c == marker).count();
    if run < 3 {
        return None;
    }
    Some((marker, run, t[run..].trim()))
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
    // A `host-lint:ignore` fenced block quarantines literal reference content (the
    // retired-ordinal dictionary, archived citations) — its lines are skipped, fences
    // included (call/0019). Markdown only; a regular code block and inline backticks
    // stay linted, so a tell cannot be laundered by inline-quoting it.
    // The open ignore fence's marker char and run length, or None when outside a
    // block. Closing requires a bare fence of the *same* marker at least as long
    // (CommonMark), so an inner code sample with a shorter fence does not leak the
    // quarantine (plan/0055, P4), and a longer outer fence can wrap it.
    let mut ignore_fence: Option<(char, usize)> = None;
    let mut last_line = 0usize;
    for (i, line) in input.lines().enumerate() {
        last_line = i + 1;
        if markdown {
            if let Some((mch, mlen)) = ignore_fence {
                if let Some((c, len, info)) = fence_info(line) {
                    if info.is_empty() && c == mch && len >= mlen {
                        ignore_fence = None;
                    }
                }
                continue;
            }
            if let Some((c, len, info)) = fence_info(line) {
                if info == "host-lint:ignore" {
                    ignore_fence = Some((c, len));
                    continue;
                }
            }
        }
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
                col: 0,
                text: line.trim().to_string(),
                term,
                severity,
                cite,
            });
        }
    }
    // An ignore fence left open at end of file silently skipped every line after it
    // (the fail-open the loop's `continue` produced). Fail loud: report it as a flag
    // so the file is never reported clean over content it never scanned (plan/0055, P2).
    if ignore_fence.is_some() {
        matches.push(Match {
            file: source.to_string(),
            line: last_line,
            col: 0,
            text: "unclosed host-lint:ignore fence".to_string(),
            term: "unclosed-ignore-fence".to_string(),
            severity: Severity::Flag,
            cite: "close the ```host-lint:ignore block with a bare fence; an unclosed block skips the rest of the file".to_string(),
        });
    }
}

// Push a prose tell as a Match — a free helper so the occurrence-mapping loop reads
// cleanly.
#[allow(clippy::too_many_arguments)]
fn push_tell(
    matches: &mut Vec<Match>,
    source: &str,
    line: usize,
    col: usize,
    text: &str,
    term: &str,
    severity: Severity,
    cite: &str,
) {
    matches.push(Match {
        file: source.to_string(),
        line,
        col,
        text: text.to_string(),
        term: term.to_string(),
        severity,
        cite: cite.to_string(),
    });
}

// The 1-based (line, column) of byte offset `off` in `input`; the column counts
// characters from the line start.
fn line_col(input: &str, off: usize) -> (usize, usize) {
    let mut line = 1usize;
    let mut col = 1usize;
    for (i, ch) in input.char_indices() {
        if i >= off {
            break;
        }
        if ch == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }
    }
    (line, col)
}

// Best-effort line of a multi-word excerpt the markdown extractor may have normalised
// out of the raw source: the 1-based line containing the excerpt's first few words, or
// None when even that is absent (a synthetic whole-document diagnosis). Both arguments
// are already ascii-lowercased, so the match is case-insensitive.
fn probe_line(input_lc: &str, needle_lc: &str) -> Option<usize> {
    let probe = needle_lc.split_whitespace().take(4).collect::<Vec<_>>().join(" ");
    if probe.is_empty() {
        return None;
    }
    input_lc.lines().position(|l| l.contains(&probe)).map(|i| i + 1)
}

// Find `needle` in `haystack` at or after byte offset `from`, returning its
// absolute offset. When the needle is a single alphanumeric word it must sit on
// word boundaries, so a short tell ("delve") does not map onto a longer word that
// merely contains it ("delved"); a multi-word or punctuation excerpt matches as-is
// (plan/0055, P5). Both arguments are already ascii-lowercased.
fn find_tell(haystack: &str, needle: &str, from: usize) -> Option<usize> {
    let single_word = !needle.is_empty()
        && !needle.contains(char::is_whitespace)
        && needle.chars().all(|c| c.is_alphanumeric());
    let mut search = from;
    while let Some(rel) = haystack.get(search..).and_then(|s| s.find(needle)) {
        let off = search + rel;
        let end = off + needle.len();
        if !single_word {
            return Some(off);
        }
        let left_ok = haystack[..off].chars().next_back().is_none_or(|c| !c.is_alphanumeric());
        let right_ok = haystack[end..].chars().next().is_none_or(|c| !c.is_alphanumeric());
        if left_ok && right_ok {
            return Some(off);
        }
        search = end.max(off + 1);
    }
    None
}

/// Scan `input` as prose for agentic tells (the host-grammar engine), pushing
/// each as an advisory `Warn` match, plus one document-level match when the tell
/// density crosses the threshold. Used for titles, comments, and `--prose` docs;
/// never blocks (Warn = exit 3), so a flagged draft still lands.
pub fn scan_prose_text(input: &str, source: &str, allow_lc: &[String], matches: &mut Vec<Match>) {
    // A markdown source is scanned structurally (code blocks excluded, headings
    // not counted as paragraphs); anything else as plain prose.
    let markdown = source.to_lowercase().ends_with(".md");
    // Mask the sanctioned LEXICON phrases before the density engine sees them, the
    // same pre-detection blank-out the naming lane performs (issue #16): a declared
    // domain phrase like `rehost harness` clears the trope on `harness` within that
    // phrase, while a standalone occurrence still flags. `mask_allowed` is
    // byte-length-preserving, so an offset into `masked` indexes `input` unchanged
    // (the reported excerpt stays the author's own text), and the masked text feeds
    // both the per-tell scan and the document density score, so a declared phrase
    // contributes to neither.
    let masked = mask_allowed(input, allow_lc);
    let tells = if markdown {
        host_grammar::scan_prose_markdown(&masked)
    } else {
        host_grammar::scan_prose_parallel(&masked)
    };
    // Locate each tell. The engine emits one tell per occurrence, but in the markdown
    // path tells of the same (id, excerpt) arrive grouped, and a first-occurrence line
    // lookup collapses them all onto one line (ten em-dashes → ten records at line 12).
    // Occurrence-map instead: assign the k-th tell of an (id, excerpt) to the k-th
    // literal occurrence of its excerpt, yielding a precise line:col. A multi-word
    // excerpt the markdown extractor normalised away falls back to a probe line (no
    // column); one that never appears at all is a non-locatable whole-document
    // diagnosis — advisory, emitted once.
    let input_lc = masked.to_ascii_lowercase();
    let mut cursor: std::collections::HashMap<(&str, &str), usize> =
        std::collections::HashMap::new();
    for t in &tells {
        // Match case-insensitively: the engine returns lowercased lexeme phrases, but a
        // sentence-initial tell ("Let's unpack") is capitalised in the source. Ascii
        // lowercasing is byte-length-preserving, so an offset in `input_lc` is valid in
        // `input`, and the original-case substring is what the author actually wrote.
        let needle = t.excerpt.to_ascii_lowercase();
        let key = (t.id, t.excerpt.as_str());
        let from = *cursor.get(&key).unwrap_or(&0);
        // A key whose cursor reached the sentinel already fell back to a probe/Note
        // once; drop its repeats rather than re-map them.
        if from == usize::MAX {
            continue;
        }
        if let Some(off) = find_tell(&input_lc, &needle, from) {
            let end = off + needle.len();
            cursor.insert(key, end.max(off + 1));
            let (line, col) = line_col(input, off);
            // `mask_allowed` blanks each byte of a multibyte char with a space, so an
            // offset valid in `input_lc` can land mid-char in `input`; guard the slice
            // and fall back to the engine's excerpt rather than panic (plan/0055, P6).
            let text = if input.is_char_boundary(off) && input.is_char_boundary(end) {
                &input[off..end]
            } else {
                t.excerpt.as_str()
            };
            push_tell(matches, source, line, col, text, t.id, Severity::Warn, t.cite);
        } else {
            // No literal occurrence at or after the cursor. Probe the region past the
            // cursor for the excerpt's first words: a hit is a real occurrence the
            // markdown extractor normalised (a soft line wrap), which the literal find
            // misses — emit it rather than drop it (plan/0055, P3). Nothing past the
            // cursor is a phantom surplus or an exhausted repeat. A first miss with no
            // probe hit is a synthetic whole-document diagnosis (advisory Note). After
            // any fallback, drop further repeats of this key.
            let region = input_lc.get(from..).unwrap_or("");
            let base = input_lc[..from].matches('\n').count();
            cursor.insert(key, usize::MAX);
            match probe_line(region, &needle) {
                Some(rel_line) => {
                    push_tell(matches, source, base + rel_line, 0, &t.excerpt, t.id, Severity::Warn, t.cite)
                }
                None if from == 0 => {
                    push_tell(matches, source, 1, 0, &t.excerpt, t.id, Severity::Note, t.cite)
                }
                None => {}
            }
        }
    }
    let score = if markdown {
        host_grammar::tell_score_markdown(&masked)
    } else {
        host_grammar::tell_score(&masked)
    };
    if score.over_threshold {
        matches.push(Match {
            file: source.to_string(),
            line: 1,
            col: 0,
            text: format!(
                "agentic-tell density {:.2} across {} sentences ({} tells)",
                score.density, score.sentences, score.tells
            ),
            term: "tell-density".to_string(),
            severity: Severity::Note,
            cite: "tropes.fyi: many devices together".to_string(),
        });
    }
}

/// `--docs` is the repo-wide prose lane — the counterpart to the naming `--all`.
/// Scope determines type: naming tells hide in any file, but prose tropes are a
/// property of authored narrative, so `--docs` walks `.md` only and never runs the
/// prose engine over `.rs`/`.toml`/`.sh` (which would flag decoration in code
/// comments and string literals, with a meaningless clean-to-zero bar over source).
/// It walks the **authored working tree**: `git ls-files` (tracked and staged) plus
/// `git ls-files --others --exclude-standard` (untracked files git would offer to add),
/// so a brand-new authored doc is audited before it is even staged, and a pre-commit
/// run is never silently clean over a file it skipped (host-lint#17). `--exclude-standard`
/// keeps gitignored output, vendored deps, untracked worktrees, and submodules out, just
/// as the bare `git ls-files` walk did; `.host-lintignore` filters the rest (e.g. the
/// append-only `MEMORY.md`). Prose tells are advisory (warn, exit 3), as elsewhere; the
/// `verify` gate's recheck treats that non-zero as a regression. Returns the matches or an
/// error string — the binary prints it and exits 2; an in-process embedder surfaces it as
/// it chooses. The shared walk, so host-lint and host-lifecycle audit docs through one
/// engine (host-lifecycle#2).
pub fn run_docs(root: &Path, allow: &[String], ignore: &[String]) -> Result<Vec<Match>, String> {
    let mut matches = Vec::new();
    if root.as_os_str().is_empty() {
        // A clean return here would be a fail-open docs audit over nothing. Fail closed.
        return Err("--docs needs a repository root (none resolved)".to_string());
    }
    let root_str = root.to_string_lossy();
    // The authored working tree: tracked and staged, then untracked-but-not-ignored. The
    // two sets are disjoint (an entry is either in the index or not), so no dedup is needed.
    let tracked = git_paths(root_str.as_ref(), &["ls-files", "-z"])?;
    let untracked = git_paths(root_str.as_ref(), &["ls-files", "--others", "--exclude-standard", "-z"])?;
    for rel in tracked.iter().chain(untracked.iter()) {
        if !rel.to_ascii_lowercase().ends_with(".md") {
            continue;
        }
        if path_ignored(rel, ignore) {
            continue;
        }
        let path = root.join(rel);
        if fs::symlink_metadata(&path).map(|m| m.file_type().is_symlink()).unwrap_or(false) {
            continue;
        }
        if !path.is_file() {
            continue;
        }
        if let Ok(content) = fs::read_to_string(&path) {
            scan_prose_text(&content, rel, allow, &mut matches);
        }
    }
    Ok(matches)
}

/// Run a `git ls-files`-family command under `-C <root>` and split its NUL-delimited
/// output into repo-relative paths. A non-zero exit (not a git repo) becomes the `--docs`
/// diagnostic the caller surfaces.
fn git_paths(root: &str, extra: &[&str]) -> Result<Vec<String>, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(extra.iter().copied())
        .output()
        .map_err(|e| format!("--docs needs git on PATH: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "--docs needs a git repository (git {} failed: {})",
            extra.first().copied().unwrap_or("ls-files"),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .split('\0')
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect())
}

/// Escalate decoration tells on the commit subject to blocking `Flag`. The first
/// line of a commit message — or a gh issue/PR title piped on stdin — becomes the
/// squash-merge subject / front-door text, so it is held to the same no-decoration
/// bar as the front-door docs: an em-dash, arrow, or smart quote there blocks
/// rather than warns. Body prose and other tells keep their advisory `Warn`. A
/// A `decoration` match on the first line is the subject's. The match carries its
/// line, so escalate by location, not by substring: a body decoration keeps its
/// advisory `Warn` even when the same character also appears in the subject
/// (plan/0055, P1 — the old substring test escalated every body occurrence of a
/// character the subject happened to use).
pub fn escalate_subject_decoration(_subject: &str, matches: &mut [Match]) {
    for m in matches.iter_mut() {
        if m.term == "decoration" && m.line == 1 && m.col > 0 {
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

/// A candidate emergent tell surfaced by `gather`: a word recurring in the tell
/// shape (a word then a numeral) that the lane does not yet catch.
pub struct Candidate {
    pub word: String,
    pub count: usize,
    pub examples: Vec<String>,
}

/// Scan a corpus (commit subjects, markdown headers) for candidate emergent
/// tells. A candidate is a word in the word-then-numeral shape that is not
/// already a flag term, a warn noun, a known-legitimate context, or a stop
/// word, and whose numeral is neither a four-digit year nor a unit-bearing
/// quantity. Returns candidates recurring at least `min_count` times, ranked by
/// count then name. This is the inverse of the flag scan: the residue the
/// grammar misses, for the operator to triage (propose, declare, or leave).
pub fn gather_candidates(lines: &[String], min_count: usize) -> Vec<Candidate> {
    use std::collections::HashMap;
    let mut seen: HashMap<String, (usize, Vec<String>)> = HashMap::new();
    for line in lines {
        let lower = line.to_lowercase();
        let words: Vec<&str> = lower.split_whitespace().collect();
        for (i, word) in words.iter().enumerate() {
            let clean = word.trim_matches(|c: char| !c.is_alphanumeric() && c != '-');
            if clean.len() < 3 || !clean.bytes().all(|b| b.is_ascii_alphabetic()) {
                continue;
            }
            if FLAG_TERMS.contains(&clean)
                || WARN_NOUNS.contains(&clean)
                || PREV_SKIP.contains(&clean)
                || GATHER_STOP.contains(&clean)
            {
                continue;
            }
            let Some(next) = words.get(i + 1) else { continue };
            // a "#7" issue or PR reference is not an ordinal label
            if next.starts_with('#') {
                continue;
            }
            let nc = next.trim_matches(|c: char| !c.is_alphanumeric() && c != '-');
            if !(is_numeral(nc) || is_num_range(nc)) {
                continue;
            }
            // four or more digits read as a year, a hash, or a quantity, not the
            // small ordinal a positional label uses
            if nc.len() >= 4 && nc.bytes().all(|b| b.is_ascii_digit()) {
                continue;
            }
            // a word then a number then a unit is a quantity, not a position
            if let Some(after) = words.get(i + 2) {
                let ac = after.trim_matches(|c: char| !c.is_alphanumeric());
                if UNITS.contains(&ac) {
                    continue;
                }
            }
            let entry = seen.entry(clean.to_string()).or_insert((0, Vec::new()));
            entry.0 += 1;
            if entry.1.len() < 3 {
                entry.1.push(line.trim().to_string());
            }
        }
    }
    let mut out: Vec<Candidate> = seen
        .into_iter()
        .filter(|(_, (count, _))| *count >= min_count)
        .map(|(word, (count, examples))| Candidate { word, count, examples })
        .collect();
    out.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.word.cmp(&b.word)));
    out
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
