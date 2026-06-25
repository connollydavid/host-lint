use std::env;
use std::fs;
use std::io::{self, Read};
use std::path::Path;
use std::process;

use host_lint::{Match, Severity, LexiconEntry, scan_text_with_allow_strict, scan_prose_text, escalate_subject_decoration, is_ci_file, is_scannable, path_ignored, parse_lexicon_line, is_strict_directive, parse_jira_keys, validate_lexicon_entry};

const LEXICON_FILE: &str = "LEXICON";
const IGNORE_FILE: &str = ".host-lintignore";

// The repo root: the parent of GIT_DIR when set (so hooks resolve correctly),
// else the current directory. Mirrors `run_all_files`.
fn repo_root() -> String {
    env::var("GIT_DIR")
        .ok()
        .and_then(|d| Path::new(&d).parent().and_then(|p| p.to_str()).map(String::from))
        .or_else(|| env::current_dir().ok().and_then(|p| p.to_str().map(String::from)))
        .unwrap_or_default()
}

// The repo's LEXICON (issue #13): the validated allowlist phrases (lowercased for
// case-insensitive masking), the committed `strict` flag, and the parsed entries
// (for the `lexicon` subcommand). An invalid entry — a master key, a tracker ref
// with no URL, a laundered tell — is reported to stderr and then dropped: it never
// masks, so soundness does not depend on the file being hand-edited correctly.
// A missing file yields an empty lexicon (the feature is opt-in per repo).
struct Lexicon {
    phrases_lc: Vec<String>,
    strict: bool,
    jira_keys: Vec<String>,
    entries: Vec<LexiconEntry>,
}

fn load_lexicon(root: &str) -> Lexicon {
    let mut lex = Lexicon { phrases_lc: Vec::new(), strict: false, jira_keys: Vec::new(), entries: Vec::new() };
    if root.is_empty() {
        return lex;
    }
    let content = match fs::read_to_string(Path::new(root).join(LEXICON_FILE)) {
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

// `host-lint lexicon <list|add|rm|--check>`: the CRUD that owns every LEXICON
// decision so a weak agent never hand-authors the file (issue #13). `add` runs the
// three guards and refuses a master key, a laundered tell, or an un-cited tracker
// ref — the tool, not the prompt, is the gate. Always exits.
fn run_lexicon(root: &str, args: &[String]) -> ! {
    let path = Path::new(root).join(LEXICON_FILE);
    match args.first().map(String::as_str) {
        Some("list") => {
            let lex = load_lexicon(root);
            println!("strict: {}", if lex.strict { "on" } else { "off" });
            for e in &lex.entries {
                match &e.url {
                    Some(u) => println!("{}  ({})", e.phrase, u),
                    None => println!("{}", e.phrase),
                }
            }
            process::exit(0);
        }
        Some("add") => {
            let Some(phrase) = args.get(1).filter(|p| !p.is_empty()) else {
                eprintln!("usage: host-lint lexicon add \"<phrase>\" [--url <url>]");
                process::exit(2);
            };
            let url = parse_url_flag(&args[2..]);
            let entry = LexiconEntry { phrase: phrase.clone(), url };
            let existing = load_lexicon(root);
            if let Err(reason) = validate_lexicon_entry(&entry, &existing.jira_keys) {
                eprintln!("host-lint: refused ({reason})");
                process::exit(1);
            }
            // Idempotent: a phrase already present is a no-op, not an error.
            if existing.entries.iter().any(|e| e.phrase.eq_ignore_ascii_case(&entry.phrase)) {
                println!("already present: {}", entry.phrase);
                process::exit(0);
            }
            let line = match &entry.url {
                Some(u) => format!("{} {}\n", entry.phrase, u),
                None => format!("{}\n", entry.phrase),
            };
            let mut content = fs::read_to_string(&path).unwrap_or_default();
            if !content.is_empty() && !content.ends_with('\n') {
                content.push('\n');
            }
            content.push_str(&line);
            if let Err(e) = fs::write(&path, content) {
                eprintln!("host-lint: cannot write {}: {e}", path.display());
                process::exit(2);
            }
            println!("added: {}", entry.phrase);
            process::exit(0);
        }
        Some("rm") => {
            let Some(phrase) = args.get(1) else {
                eprintln!("usage: host-lint lexicon rm \"<phrase>\"");
                process::exit(2);
            };
            let content = match fs::read_to_string(&path) {
                Ok(c) => c,
                Err(_) => {
                    eprintln!("host-lint: no LEXICON at {}", path.display());
                    process::exit(1);
                }
            };
            let mut removed = 0;
            let kept: Vec<&str> = content
                .lines()
                .filter(|line| {
                    let drop = parse_lexicon_line(line)
                        .is_some_and(|e| e.phrase.eq_ignore_ascii_case(phrase));
                    if drop {
                        removed += 1;
                    }
                    !drop
                })
                .collect();
            if removed == 0 {
                eprintln!("host-lint: not in LEXICON: {phrase}");
                process::exit(1);
            }
            let mut out = kept.join("\n");
            out.push('\n');
            if let Err(e) = fs::write(&path, out) {
                eprintln!("host-lint: cannot write {}: {e}", path.display());
                process::exit(2);
            }
            println!("removed: {phrase}");
            process::exit(0);
        }
        Some("--check") => {
            let content = fs::read_to_string(&path).unwrap_or_default();
            let mut jira_keys: Vec<String> = Vec::new();
            for line in content.lines() {
                if let Some(keys) = parse_jira_keys(line) {
                    jira_keys.extend(keys);
                }
            }
            let (mut total, mut errs) = (0, 0);
            for (i, line) in content.lines().enumerate() {
                if is_strict_directive(line) || parse_jira_keys(line).is_some() {
                    continue;
                }
                let Some(e) = parse_lexicon_line(line) else { continue };
                total += 1;
                if let Err(reason) = validate_lexicon_entry(&e, &jira_keys) {
                    eprintln!("{}:{}: invalid entry ({reason})", LEXICON_FILE, i + 1);
                    errs += 1;
                }
            }
            if errs > 0 {
                eprintln!("host-lint: {errs} invalid of {total} LEXICON entries");
                process::exit(1);
            }
            println!("LEXICON OK ({total} entries)");
            process::exit(0);
        }
        // The network lane (issue #13 guard 3): the offline format-check cannot
        // tell a real `#7` from a phantom `#999`, and a weak agent fabricates URLs,
        // so a network-having lane (CI / opt-in) re-derives liveness. Off the commit
        // hook by design — it needs the network the hook must not.
        Some("--check-urls") => {
            let cited: Vec<LexiconEntry> = load_lexicon(root)
                .entries
                .into_iter()
                .filter(|e| e.url.is_some())
                .collect();
            if cited.is_empty() {
                println!("LEXICON: no cited references to check");
                process::exit(0);
            }
            let mut dead = 0;
            for e in &cited {
                let url = e.url.as_deref().unwrap_or_default();
                match url_status(url) {
                    Ok(code) if (200..400).contains(&code) => {
                        println!("ok   {code}  {}  ({url})", e.phrase)
                    }
                    Ok(code) => {
                        eprintln!("DEAD {code}  {}  ({url})", e.phrase);
                        dead += 1;
                    }
                    Err(msg) => {
                        eprintln!("ERR  {}  ({url}): {msg}", e.phrase);
                        dead += 1;
                    }
                }
            }
            if dead > 0 {
                eprintln!("host-lint: {dead} dead/unreachable LEXICON reference(s)");
                process::exit(1);
            }
            println!("LEXICON URLs OK ({} checked)", cited.len());
            process::exit(0);
        }
        _ => {
            eprintln!("usage: host-lint lexicon <list | add \"<phrase>\" [--url <url>] | rm \"<phrase>\" | --check | --check-urls>");
            process::exit(2);
        }
    }
}

// Pull the value of a `--url <value>` flag from a lexicon-subcommand argument tail.
fn parse_url_flag(args: &[String]) -> Option<String> {
    args.iter()
        .position(|a| a == "--url")
        .and_then(|i| args.get(i + 1))
        .cloned()
}

// Resolve a URL to its final HTTP status by shelling `curl` (following redirects,
// body discarded, 10s cap) and parsing the code in-process — one tool, one parse,
// per the weak-agent thesis. A transport failure (DNS, timeout, no curl) is `Err`.
fn url_status(url: &str) -> Result<u32, String> {
    let out = process::Command::new("curl")
        .args(["-sSL", "-o", "/dev/null", "-w", "%{http_code}", "--max-time", "10", url])
        .output()
        .map_err(|e| format!("curl unavailable: {e}"))?;
    if !out.status.success() {
        return Err(format!("unreachable ({})", String::from_utf8_lossy(&out.stderr).trim()));
    }
    let code = String::from_utf8_lossy(&out.stdout);
    code.trim().parse::<u32>().map_err(|_| format!("bad status '{}'", code.trim()))
}

fn scan_file(path: &Path, allow: &[String], strict: bool, matches: &mut Vec<Match>) {
    if !path.is_file() {
        return;
    }
    if is_ci_file(path.to_string_lossy().as_ref()) {
        return;
    }
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    if !is_scannable(ext) {
        return;
    }
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return,
    };
    scan_text_with_allow_strict(&content, path.to_string_lossy().as_ref(), allow, strict, matches);
}

fn output_text(matches: &[Match]) {
    for m in matches {
        let loc = if m.col > 0 {
            format!("{}:{}:{}", m.file, m.line, m.col)
        } else {
            format!("{}:{}", m.file, m.line)
        };
        let tag = if m.cite.is_empty() {
            m.term.clone()
        } else {
            format!("{} — {}", m.term, m.cite)
        };
        let fix = fix_hint(&m.term, &m.text)
            .map(|f| format!(" [fix: {}]", f))
            .unwrap_or_default();
        match m.severity {
            Severity::Warn => eprintln!("{}: warning: {} ({}){}", loc, m.text, tag, fix),
            Severity::Flag => eprintln!("{}: {} ({}){}", loc, m.text, tag, fix),
            Severity::Note => eprintln!("{}: note: {} ({})", loc, m.text, tag),
        }
    }
}

// A mechanical rewrite hint for the tropes a weak agent can fix by a known edit, keyed
// on the tell id (and the matched character for decoration). Judgement tropes — reword
// a sentence — carry no hint, so only mechanically-fixable tells get one.
fn fix_hint(term: &str, text: &str) -> Option<&'static str> {
    match term {
        "decoration" => Some(match text {
            "—" | "–" => "replace with a comma, period, or parentheses",
            "“" | "”" | "‘" | "’" => "use a straight quote",
            "→" => "replace with a word (to, then, leads to)",
            _ => "rewrite as plain punctuation",
        }),
        _ => None,
    }
}

fn output_json(matches: &[Match]) {
    let json = serde_json_like(matches);
    println!("{}", json);
}

fn serde_json_like(matches: &[Match]) -> String {
    let mut out = String::from("[\n");
    for (i, m) in matches.iter().enumerate() {
        out.push_str("  {");
        out.push_str(&format!("\"file\": \"{}\", ", escape_json(&m.file)));
        out.push_str(&format!("\"line\": {}, ", m.line));
        out.push_str(&format!("\"col\": {}, ", m.col));
        out.push_str(&format!("\"text\": \"{}\", ", escape_json(&m.text)));
        out.push_str(&format!("\"term\": \"{}\", ", escape_json(&m.term)));
        let severity = match m.severity {
            Severity::Warn => "warn",
            Severity::Flag => "flag",
            Severity::Note => "note",
        };
        out.push_str(&format!("\"severity\": \"{}\"", severity));
        if let Some(f) = fix_hint(&m.term, &m.text) {
            out.push_str(&format!(", \"fix\": \"{}\"", escape_json(f)));
        }
        if !m.cite.is_empty() {
            out.push_str(&format!(", \"cite\": \"{}\"", escape_json(&m.cite)));
        }
        out.push('}');
        if i < matches.len() - 1 {
            out.push(',');
        }
        out.push('\n');
    }
    out.push(']');
    out
}

fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n").replace('\r', "\\r").replace('\t', "\\t")
}

fn run_all_files(root: &str, allow: &[String], strict: bool, ignore: &[String], matches: &mut Vec<Match>) {
    if root.is_empty() {
        return;
    }
    // `--all` audits the repo's tracked files (as the README states). Listing them
    // via `git ls-files` respects `.gitignore` by construction: gitignored build
    // output, vendored dependencies, and `.git/` are untracked and never appear, so
    // a naive tree walk's slowness and noise are gone without a hardcoded skip list.
    // `-z` is robust to paths containing spaces or newlines; paths are root-relative
    // and `/`-separated. `.host-lintignore` still filters tracked-but-sanctioned paths.
    let output = match process::Command::new("git")
        .args(["-C", root, "ls-files", "-z"])
        .output()
    {
        Ok(o) if o.status.success() => o,
        Ok(o) => {
            eprintln!(
                "host-lint: --all needs a git repository (git ls-files failed: {})",
                String::from_utf8_lossy(&o.stderr).trim()
            );
            process::exit(2);
        }
        Err(e) => {
            eprintln!("host-lint: --all needs git on PATH: {e}");
            process::exit(2);
        }
    };
    let text = String::from_utf8_lossy(&output.stdout);
    for rel in text.split('\0').filter(|s| !s.is_empty()) {
        if path_ignored(rel, ignore) {
            continue;
        }
        let path = Path::new(root).join(rel);
        // git tracks symlinks as symlinks; skip them — following a file symlink would
        // scan its target twice (the target, if tracked, is listed and scanned
        // directly), and a dir symlink (e.g. a cycle) is not a file to scan anyway.
        if fs::symlink_metadata(&path).map(|m| m.file_type().is_symlink()).unwrap_or(false) {
            continue;
        }
        // scan_file additionally skips non-files (tracked-but-deleted), CI files,
        // and unscannable extensions.
        scan_file(&path, allow, strict, matches);
    }
}

// Repo-relative paths to exclude from `--all` (`.host-lintignore`, gitignore-lite:
// one pattern per line, `#` comments and blanks ignored). A migration writes this
// to exclude the append-only record; absent file → no exclusions.
fn load_ignore(root: &str) -> Vec<String> {
    if root.is_empty() {
        return Vec::new();
    }
    match fs::read_to_string(Path::new(root).join(IGNORE_FILE)) {
        Ok(content) => content
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .map(String::from)
            .collect(),
        Err(_) => Vec::new(),
    }
}

// `--docs` is the repo-wide prose lane — the counterpart to the naming `--all`.
// Scope determines type: naming tells hide in any file, but prose tropes are a
// property of authored narrative, so `--docs` walks `.md` only and never runs the
// prose engine over `.rs`/`.toml`/`.sh` (which would flag decoration in code
// comments and string literals, with a meaningless clean-to-zero bar over source).
// It reuses the `--all` `git ls-files` walk, so gitignored output, vendored deps,
// untracked worktrees, and submodules never appear; `.host-lintignore` filters the
// rest (e.g. the append-only `MEMORY.md`). Prose tells are advisory (warn, exit 3),
// as elsewhere; the `verify` gate's recheck treats that non-zero as a regression.
fn run_docs(root: &str, allow: &[String], ignore: &[String], matches: &mut Vec<Match>) {
    if root.is_empty() {
        return;
    }
    let output = match process::Command::new("git")
        .args(["-C", root, "ls-files", "-z"])
        .output()
    {
        Ok(o) if o.status.success() => o,
        Ok(o) => {
            eprintln!(
                "host-lint: --docs needs a git repository (git ls-files failed: {})",
                String::from_utf8_lossy(&o.stderr).trim()
            );
            process::exit(2);
        }
        Err(e) => {
            eprintln!("host-lint: --docs needs git on PATH: {e}");
            process::exit(2);
        }
    };
    let text = String::from_utf8_lossy(&output.stdout);
    for rel in text.split('\0').filter(|s| !s.is_empty()) {
        if !rel.to_ascii_lowercase().ends_with(".md") {
            continue;
        }
        if path_ignored(rel, ignore) {
            continue;
        }
        let path = Path::new(root).join(rel);
        if fs::symlink_metadata(&path).map(|m| m.file_type().is_symlink()).unwrap_or(false) {
            continue;
        }
        if !path.is_file() {
            continue;
        }
        if let Ok(content) = fs::read_to_string(&path) {
            scan_prose_text(&content, rel, allow, matches);
        }
    }
}

fn run_log(allow: &[String], strict: bool, matches: &mut Vec<Match>) {
    let output = match process::Command::new("git")
        .args(["log", "-z", "--format=%H%n%B"])
        .output()
    {
        Ok(o) if o.status.success() => o,
        Ok(o) => {
            eprintln!("host-lint: git log failed: {}", String::from_utf8_lossy(&o.stderr).trim());
            process::exit(2);
        }
        Err(e) => {
            eprintln!("host-lint: failed to run git: {}", e);
            process::exit(2);
        }
    };
    let text = String::from_utf8_lossy(&output.stdout);
    for record in text.split('\0') {
        let record = record.trim_end_matches('\n');
        if record.is_empty() {
            continue;
        }
        let (sha, message) = match record.split_once('\n') {
            Some((s, m)) => (s, m),
            None => (record, ""),
        };
        let label = if sha.len() >= 7 { &sha[..7] } else { sha };
        scan_text_with_allow_strict(message, label, allow, strict, matches);
    }
}

// `gather` (the reflective-practice discovery, plan/0035): scan commit subjects
// and markdown headers for a recurring word-then-numeral shape the lane does not
// catch, and report the candidates for the operator to triage. Advisory: it
// surfaces, it never decides, and it exits zero.
fn run_gather(root: &str) -> ! {
    let mut lines: Vec<String> = Vec::new();
    // commit subjects — the richest source of an ordinal-by-position tell
    if let Ok(o) = process::Command::new("git")
        .args(["-C", root, "log", "--format=%s"])
        .output()
    {
        if o.status.success() {
            for l in String::from_utf8_lossy(&o.stdout).lines() {
                lines.push(l.to_string());
            }
        }
    }
    // header lines from tracked markdown docs
    if let Ok(o) = process::Command::new("git")
        .args(["-C", root, "ls-files", "-z"])
        .output()
    {
        if o.status.success() {
            for rel in String::from_utf8_lossy(&o.stdout)
                .split('\0')
                .filter(|s| s.ends_with(".md"))
            {
                if let Ok(content) = fs::read_to_string(Path::new(root).join(rel)) {
                    for l in content.lines() {
                        if l.trim_start().starts_with('#') {
                            lines.push(l.to_string());
                        }
                    }
                }
            }
        }
    }
    let candidates = host_lint::gather_candidates(&lines, 2);
    if candidates.is_empty() {
        println!("gather: no recurring candidate tells — the lane covers this corpus");
        process::exit(0);
    }
    println!(
        "gather: {} candidate tell-shape(s) recurring in commit subjects and headers",
        candidates.len()
    );
    println!("(advisory — triage each: propose it upstream, declare it in the LEXICON, or leave it)");
    for c in &candidates {
        println!("  {:>3}x  {}", c.count, c.word);
        for ex in &c.examples {
            println!("         e.g. {ex}");
        }
    }
    process::exit(0);
}

fn main() {
    let args: Vec<String> = env::args().collect();

    // `lexicon` is a subcommand (CRUD over the allowlist), not a scan flag.
    if args.get(1).map(String::as_str) == Some("lexicon") {
        run_lexicon(&repo_root(), &args[2..]);
    }

    // `gather` is a discovery subcommand (plan/0035), not a scan flag.
    if args.get(1).map(String::as_str) == Some("gather") {
        run_gather(&repo_root());
    }

    let mut stdin_flag = false;
    let mut json_flag = false;
    let mut all_flag = false;
    let mut log_flag = false;
    let mut prose_flag = false;
    let mut docs_flag = false;
    let mut files: Vec<String> = Vec::new();

    for arg in &args[1..] {
        match arg.as_str() {
            "--stdin" => stdin_flag = true,
            "--json" => json_flag = true,
            "--all" => all_flag = true,
            "--log" => log_flag = true,
            "--prose" => prose_flag = true,
            "--docs" => docs_flag = true,
            _ => files.push(arg.clone()),
        }
    }

    let root = repo_root();
    let lex = load_lexicon(&root);
    let allow = lex.phrases_lc.as_slice();
    let strict = lex.strict;
    let mut matches = Vec::new();

    if stdin_flag {
        let mut input = String::new();
        io::stdin().read_to_string(&mut input).unwrap_or_default();
        // A stdin title/draft gets both naming and prose tells.
        scan_text_with_allow_strict(&input, "stdin", allow, strict, &mut matches);
        scan_prose_text(&input, "stdin", allow, &mut matches);
        // The subject (first line) becomes a squash-merge subject / gh title; a
        // decoration tell there blocks rather than warns. The body stays advisory.
        escalate_subject_decoration(input.lines().next().unwrap_or(""), &mut matches);
    } else if prose_flag {
        // Treat each file purely as prose for the agentic-tell engine.
        for f in &files {
            if let Ok(content) = fs::read_to_string(f) {
                scan_prose_text(&content, f, allow, &mut matches);
            }
        }
    } else if all_flag {
        run_all_files(&root, allow, strict, &load_ignore(&root), &mut matches);
    } else if docs_flag {
        run_docs(&root, allow, &load_ignore(&root), &mut matches);
    } else if log_flag {
        run_log(allow, strict, &mut matches);
    } else if files.is_empty() {
        eprintln!("Usage: host-lint [--stdin] [--prose] [--docs] [--json] [--all] [--log] [files...]");
        process::exit(2);
    } else {
        // Honor `.host-lintignore` for explicit file args too — the git hook scans
        // per staged file (`host-lint <file>`), so the ignore list must apply here,
        // not only in the `--all` walk. Otherwise a detector's own test fixtures
        // (which must embed the tells they exercise) can never pass the hook, forcing
        // `--no-verify`. Match on the same repo-relative, `/`-separated path.
        let ignore = load_ignore(&root);
        for f in &files {
            let abs = fs::canonicalize(f).unwrap_or_else(|_| Path::new(f).to_path_buf());
            let rel = abs
                .strip_prefix(&root)
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_else(|_| f.clone())
                .trim_start_matches(['/', '\\'])
                .replace('\\', "/");
            if path_ignored(&rel, &ignore) {
                continue;
            }
            scan_file(Path::new(f), allow, strict, &mut matches);
        }
    }

    if json_flag {
        output_json(&matches);
    } else {
        output_text(&matches);
    }

    if matches.iter().any(|m| m.severity == Severity::Flag) {
        process::exit(1);
    }
    if matches.iter().any(|m| m.severity == Severity::Warn) {
        process::exit(3);
    }
}
