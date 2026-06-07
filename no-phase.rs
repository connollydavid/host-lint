use std::env;
use std::fs;
use std::io::{self, Read};
use std::path::Path;
use std::process;

const FLAG_TERMS: &[&str] = &[
    "phase", "stage", "step", "part", "pass", "round", "iteration",
    "sprint", "cycle", "increment", "wave", "batch", "section",
    "period", "era", "epoch", "chapter", "episode", "instalment",
    "leg", "lap", "level",
];

const ALLOWLIST_PREFIXES: &[&str] = &[
    "feat", "fix", "docs", "style", "refactor", "perf", "test",
    "build", "ci", "chore", "revert", "improvement",
    "praise", "nitpick", "nit", "suggestion", "issue", "todo",
    "question", "thought",
    "wip", "lgtm", "ptal", "tbd", "tl;dr", "iirc", "afaict", "rfc",
];

const CODE_TAGS: &[&str] = &[
    "todo", "fixme", "xxx", "hack", "bug", "note", "nb",
    "optimize", "review", "wontfix", "nobug",
];

const CI_PATTERNS: &[&str] = &[
    ".github/workflows",
    ".gitlab-ci",
    "jenkinsfile",
    "dockerfile",
    "docker-compose",
];

struct Match {
    file: String,
    line: usize,
    text: String,
    term: String,
}

fn is_ci_file(path: &str) -> bool {
    let lower = path.to_lowercase();
    CI_PATTERNS.iter().any(|p| lower.contains(p))
}

fn is_allowlist(line: &str) -> bool {
    let trimmed = line.trim().to_lowercase();
    let words: Vec<&str> = trimmed.split_whitespace().take(1).collect();
    let first = words.first().copied().unwrap_or("");
    let first = first.trim_matches(|c: char| !c.is_alphanumeric());

    for prefix in ALLOWLIST_PREFIXES {
        if first == *prefix || first.starts_with(&format!("{}(", prefix)) || first.starts_with(&format!("{}:", prefix)) {
            return true;
        }
    }

    for tag in CODE_TAGS {
        if first == *tag || first.starts_with(&format!("{}(", tag)) || first.starts_with(&format!("{}:", tag)) || first.starts_with(&format!("{} ", tag)) {
            return true;
        }
    }

    false
}

fn is_numeral(word: &str) -> bool {
    let cleaned: String = word.chars().filter(|c| c.is_ascii_digit() || *c == 'I' || *c == 'i' || *c == 'V' || *c == 'v' || *c == 'X' || *c == 'x' || *c == 'L' || *c == 'l').collect();
    if cleaned.is_empty() {
        return false;
    }
    if cleaned.chars().all(|c| c.is_ascii_digit()) {
        return true;
    }
    let upper = cleaned.to_uppercase();
    upper.chars().all(|c| matches!(c, 'I' | 'V' | 'X' | 'L' | 'C' | 'D' | 'M')) && upper.len() <= 4
}

fn check_line(line: &str) -> Option<String> {
    if is_allowlist(line) {
        return None;
    }

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

fn scan_text(input: &str, source: &str, matches: &mut Vec<Match>) {
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

fn scan_file(path: &Path, matches: &mut Vec<Match>) {
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
    scan_text(&content, path.to_string_lossy().as_ref(), matches);
}

fn is_scannable(ext: &str) -> bool {
    matches!(ext, "" | "md" | "txt" | "rst" | "py" | "rs" | "js" | "ts" | "jsx" | "tsx" | "go" | "java" | "c" | "cpp" | "h" | "hpp" | "rb" | "sh" | "yaml" | "yml" | "toml" | "json" | "xml" | "html" | "css" | "sql" | "r" | "lua" | "swift" | "kt" | "scala" | "ex" | "exs" | "clj" | "hs" | "ml" | "vim" | "ps1" | "bat" | "cmake" | "makefile")
}

fn output_text(matches: &[Match]) {
    for m in matches {
        eprintln!("{}:{}: {} ({})", m.file, m.line, m.text, m.term);
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
        out.push_str(&format!("\"text\": \"{}\", ", escape_json(&m.text)));
        out.push_str(&format!("\"term\": \"{}\"", escape_json(&m.term)));
        out.push_str("}");
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

fn run_all_files(matches: &mut Vec<Match>) {
    let root = env::var("GIT_DIR").ok().and_then(|d| {
        Path::new(&d).parent().and_then(|p| p.to_str()).map(String::from)
    }).unwrap_or_else(|| env::current_dir().ok().and_then(|p| p.to_str().map(String::from)).unwrap_or_default());

    if root.is_empty() {
        return;
    }

    for entry in walkdir_simple(&root) {
        if entry.starts_with(".git") || entry.starts_with("node_modules") || entry.starts_with("target") || entry.starts_with("vendor") {
            continue;
        }
        scan_file(Path::new(&entry), matches);
    }
}

fn walkdir_simple(dir: &str) -> Vec<String> {
    let mut files = Vec::new();
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return files,
    };
    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let path = entry.path();
        let path_str = path.to_string_lossy().to_string();
        if path.is_dir() {
            files.extend(walkdir_simple(&path_str));
        } else if path.is_file() {
            files.push(path_str);
        }
    }
    files
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut stdin_flag = false;
    let mut json_flag = false;
    let mut all_flag = false;
    let mut files: Vec<String> = Vec::new();

    for arg in &args[1..] {
        match arg.as_str() {
            "--stdin" => stdin_flag = true,
            "--json" => json_flag = true,
            "--all" => all_flag = true,
            _ => files.push(arg.clone()),
        }
    }

    let mut matches = Vec::new();

    if stdin_flag {
        let mut input = String::new();
        io::stdin().read_to_string(&mut input).unwrap_or_default();
        scan_text(&input, "stdin", &mut matches);
    } else if all_flag {
        run_all_files(&mut matches);
    } else if files.is_empty() {
        eprintln!("Usage: no-phase [--stdin] [--json] [--all] [files...]");
        process::exit(2);
    } else {
        for f in &files {
            scan_file(Path::new(f), &mut matches);
        }
    }

    if json_flag {
        output_json(&matches);
    } else {
        output_text(&matches);
    }

    if !matches.is_empty() {
        process::exit(1);
    }
}
