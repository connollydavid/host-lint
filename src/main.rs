use std::env;
use std::fs;
use std::io::{self, Read};
use std::path::Path;
use std::process;

use host_lint::{Match, Severity, LexiconEntry, load_lexicon, run_docs, scan_text_with_allow_strict, scan_prose_text, escalate_subject_decoration, is_ci_file, is_scannable, path_ignored, parse_lexicon_line, is_strict_directive, parse_jira_keys, validate_lexicon_entry};

const LEXICON_FILE: &str = "LEXICON";
const IGNORE_FILE: &str = ".host-lintignore";

// The repo root: the parent of GIT_DIR when set (so hooks resolve correctly),
// else the current directory. Mirrors `run_all_files`.
fn repo_root() -> String {
    env::var("GIT_DIR")
        .ok()
        .and_then(|d| Path::new(&d).parent().and_then(|p| p.to_str()).map(String::from))
        // A relative GIT_DIR (".git", which `git --git-dir=.git commit` exports)
        // has an empty parent; an empty root would drop the LEXICON and downgrade
        // strict to advisory, so fall through to the working directory instead.
        .filter(|p| !p.is_empty())
        .or_else(|| env::current_dir().ok().and_then(|p| p.to_str().map(String::from)))
        .unwrap_or_default()
}

// `host-lint lexicon <list|add|rm|--check>`: the CRUD that owns every LEXICON
// decision so a weak agent never hand-authors the file (issue #13). `add` runs the
// three guards and refuses a master key, a laundered tell, or an un-cited tracker
// ref — the tool, not the prompt, is the gate. Always exits.
fn run_lexicon(root: &str, args: &[String]) -> ! {
    let path = Path::new(root).join(LEXICON_FILE);
    match args.first().map(String::as_str) {
        Some("list") => {
            let lex = load_lexicon(Path::new(root));
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
            let existing = load_lexicon(Path::new(root));
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
            let cited: Vec<LexiconEntry> = load_lexicon(Path::new(root))
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

fn scan_file(path: &Path, allow: &[String], units: &[String], strict: bool, matches: &mut Vec<Match>) {
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
    scan_text_with_allow_strict(&content, path.to_string_lossy().as_ref(), allow, units, strict, matches);
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
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            // Any other control character (0x00-0x1F) must be escaped, or a line
            // carrying a raw ESC/NUL byte produces invalid JSON.
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

/// Run the tracked-doc prose audit over `root` (host-lint's `run_docs`), extending
/// `matches`. An I/O error walking the tree exits 2 rather than passing it silently.
/// Shared by `--all`, `--docs`, and no-file `--prose` so one repo-wide prose audit
/// backs all three and matches the host-lifecycle prose gate (host-lint#20).
fn audit_tracked_docs(root: &str, allow: &[String], matches: &mut Vec<Match>) {
    match run_docs(Path::new(root), allow, &load_ignore(root)) {
        Ok(m) => matches.extend(m),
        Err(e) => {
            eprintln!("host-lint: {e}");
            process::exit(2);
        }
    }
}

fn run_all_files(root: &str, allow: &[String], units: &[String], strict: bool, ignore: &[String], matches: &mut Vec<Match>) {
    if root.is_empty() {
        // No resolvable repository root: a clean exit here would be a fail-open
        // audit that scanned nothing. Fail closed.
        eprintln!("host-lint: --all needs a repository root (none resolved)");
        process::exit(2);
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
        scan_file(&path, allow, units, strict, matches);
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

fn run_log(allow: &[String], units: &[String], strict: bool, matches: &mut Vec<Match>) {
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
        scan_text_with_allow_strict(message, label, allow, units, strict, matches);
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
    // `--stdin-as <path>` lints content piped on stdin as if it were the file at
    // <path>: the extension picks the naming semantics and the path drives the
    // ignore rules, so the pre-commit hook can lint the *staged* blob
    // (`git show :path`) rather than the working-tree copy.
    let mut stdin_as: Option<String> = None;
    let mut files: Vec<String> = Vec::new();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--stdin" => stdin_flag = true,
            "--json" => json_flag = true,
            "--all" => all_flag = true,
            "--log" => log_flag = true,
            "--prose" => prose_flag = true,
            "--docs" => docs_flag = true,
            "--stdin-as" => {
                i += 1;
                match args.get(i) {
                    Some(p) => stdin_as = Some(p.clone()),
                    None => {
                        eprintln!("host-lint: --stdin-as needs a path");
                        process::exit(2);
                    }
                }
            }
            other => files.push(other.to_string()),
        }
        i += 1;
    }

    let root = repo_root();
    let lex = load_lexicon(Path::new(&root));
    let allow = lex.phrases_lc.as_slice();
    let units = lex.units.as_slice();
    let strict = lex.strict;
    let mut matches = Vec::new();

    if let Some(path) = &stdin_as {
        // Lint piped content (the staged blob) as the file at `path`: the naming
        // lane only, gated exactly as the per-file scan is — CI files and
        // unscannable extensions produce no naming tells, and `.host-lintignore`
        // applies to the real path so a sanctioned file (a test fixture, the
        // append-only record) is not flagged when committed.
        let mut input = String::new();
        // Drain stdin first (even when we will not scan) so the upstream `git show`
        // completes rather than taking SIGPIPE under the hook's `pipefail`.
        let read = io::stdin().read_to_string(&mut input);
        let rel = path.trim_start_matches(['/', '\\']).replace('\\', "/");
        let ext = Path::new(path).extension().and_then(|e| e.to_str()).unwrap_or("");
        if !path_ignored(&rel, &load_ignore(&root)) && !is_ci_file(path) && is_scannable(ext) {
            // A scannable file whose staged content is not valid UTF-8 cannot be
            // scanned: fail closed rather than pass it unseen (plan/0055 cast review).
            if let Err(e) = read {
                eprintln!("host-lint: cannot read staged content of {path} as UTF-8: {e}");
                process::exit(2);
            }
            scan_text_with_allow_strict(&input, path, allow, units, strict, &mut matches);
        }
    } else if stdin_flag {
        let mut input = String::new();
        if let Err(e) = io::stdin().read_to_string(&mut input) {
            // Fail closed rather than scan an empty string over an unreadable input.
            eprintln!("host-lint: cannot read stdin as UTF-8: {e}");
            process::exit(2);
        }
        // A stdin title/draft gets both naming and prose tells.
        scan_text_with_allow_strict(&input, "stdin", allow, units, strict, &mut matches);
        scan_prose_text(&input, "stdin", allow, &mut matches);
        // The subject (first line) becomes a squash-merge subject / gh title; a
        // decoration tell there blocks rather than warns. The body stays advisory.
        escalate_subject_decoration(input.lines().next().unwrap_or(""), &mut matches);
    } else if all_flag {
        // `--all` is the comprehensive repo audit: the naming lane over tracked files
        // plus the prose lane over tracked authored docs, so one repo-wide command
        // matches the host-lifecycle naming + prose gate (host-lint#20). A `--prose`
        // passed alongside `--all` is redundant and folded in here.
        run_all_files(&root, allow, units, strict, &load_ignore(&root), &mut matches);
        audit_tracked_docs(&root, allow, &mut matches);
    } else if prose_flag {
        if files.is_empty() {
            // `--prose` with no files audits the tracked authored docs (the repo-wide
            // prose audit that matches the host-lifecycle prose gate, host-lint#20),
            // rather than scanning nothing and exiting clean (a fail-open) or erroring.
            audit_tracked_docs(&root, allow, &mut matches);
        } else {
            // `--prose <files>` scans exactly those files; an unreadable one is an
            // error, not a silent skip.
            for f in &files {
                match fs::read_to_string(f) {
                    Ok(content) => scan_prose_text(&content, f, allow, &mut matches),
                    Err(e) => {
                        eprintln!("host-lint: cannot read {f}: {e}");
                        process::exit(2);
                    }
                }
            }
        }
    } else if docs_flag {
        audit_tracked_docs(&root, allow, &mut matches);
    } else if log_flag {
        run_log(allow, units, strict, &mut matches);
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
            // An explicit file argument that cannot be scanned fails closed (exit 2)
            // rather than passing silently: a typo'd path reported clean is a
            // fail-open audit (host-lint#23). The deliberate skips stay silent
            // because they are policy, not errors: an ignored path above, and the
            // CI-file and unscannable-extension skips below. `--all` keeps its own
            // silent non-file skip (tracked-but-deleted), which is likewise policy.
            let path = Path::new(f);
            let meta = match fs::metadata(path) {
                Ok(m) => m,
                Err(e) => {
                    eprintln!("host-lint: cannot scan {f}: {e}");
                    process::exit(2);
                }
            };
            if !meta.is_file() {
                eprintln!("host-lint: cannot scan {f}: not a regular file");
                process::exit(2);
            }
            if is_ci_file(f) {
                continue;
            }
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if !is_scannable(ext) {
                continue;
            }
            match fs::read_to_string(path) {
                Ok(content) => {
                    scan_text_with_allow_strict(&content, f, allow, units, strict, &mut matches)
                }
                Err(e) => {
                    eprintln!("host-lint: cannot read {f}: {e}");
                    process::exit(2);
                }
            }
        }
    }

    if json_flag {
        output_json(&matches);
    } else {
        output_text(&matches);
    }

    process::exit(host_lint::verdict_code(&matches));
}
