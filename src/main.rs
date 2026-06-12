use std::env;
use std::fs;
use std::io::{self, Read};
use std::path::Path;
use std::process;

use host_lint::{Match, scan_text, is_ci_file, is_scannable};

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

fn run_log(matches: &mut Vec<Match>) {
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
        scan_text(message, label, matches);
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
    let mut log_flag = false;
    let mut files: Vec<String> = Vec::new();

    for arg in &args[1..] {
        match arg.as_str() {
            "--stdin" => stdin_flag = true,
            "--json" => json_flag = true,
            "--all" => all_flag = true,
            "--log" => log_flag = true,
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
    } else if log_flag {
        run_log(&mut matches);
    } else if files.is_empty() {
        eprintln!("Usage: host-lint [--stdin] [--json] [--all] [--log] [files...]");
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
