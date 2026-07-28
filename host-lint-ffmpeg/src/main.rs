mod config;
mod cosmetic;
mod diff;
mod msg;
mod rules;
mod sha256;

use std::env;
use std::fs;
use std::io;
use std::process;

// The engine handshake (host-lint#23, version-handshake-fails-open): the
// dispatching core exports HOST_LINT_VERSION, and a pack built against a
// different major/minor refuses to run rather than lint with mismatched
// semantics. A may-warn check would fail open the same way a stale
// hook-copied binary does, so the refusal is strict. A direct invocation
// with no HOST_LINT_VERSION set has nothing to skew against and proceeds.
fn refuse_engine_skew() {
    let Ok(core) = env::var("HOST_LINT_VERSION") else { return };
    let major_minor = |v: &str| {
        let mut parts = v.split('.');
        (
            parts.next().unwrap_or("").to_string(),
            parts.next().unwrap_or("").to_string(),
        )
    };
    if major_minor(&core) != major_minor(host_lint::ENGINE_VERSION) {
        eprintln!(
            "host-lint-ffmpeg: engine version skew: core {core}, pack built against {}; reinstall the pair together",
            host_lint::ENGINE_VERSION
        );
        process::exit(2);
    }
}

/// The message lane: `msg [--signoff] [<file>]`, or stdin. Exits 1 on a mechanical
/// finding, 3 on heuristic findings alone, 0 clean, 2 on a usage or IO error, which
/// is the core's own verdict split so a hook can treat both the same way.
/// `config` shows what the pack resolved and where from, so a surprising mode can be
/// traced to the file that set it rather than guessed at.
fn run_config(args: &[String]) -> ! {
    let dir = args
        .iter()
        .find(|a| !a.starts_with("--"))
        .cloned()
        .unwrap_or_else(|| ".".to_string());
    match config::load(std::path::Path::new(&dir)) {
        Err(e) => {
            eprintln!("host-lint-ffmpeg: {e}");
            process::exit(2);
        }
        Ok(c) => {
            println!("source        {}", c.source);
            println!("upstream_ref  {}", c.upstream_ref);
            println!("mode          {} ({})", c.mode.as_str(),
                if c.mode.blocks() { "mechanical findings block" } else { "nothing blocks" });
            println!("branch_prefix {}", c.branch_prefix.as_deref().unwrap_or("(unset)"));
            println!("tag_prefix    {}", c.tag_prefix.as_deref().unwrap_or("(unset)"));
            process::exit(0);
        }
    }
}

/// The verdict a lane exits with, filtered through the project's mode. In advise and
/// frozen modes a mechanical finding still PRINTS as a flag — the finding is what it
/// is — and the exit code drops to the advisory one, because the mode governs
/// consequences rather than truth.
fn verdict(blocking: bool, mode: config::Mode) -> i32 {
    match (blocking, mode.blocks()) {
        (true, true) => 1,
        (true, false) => 3,
        (false, _) => 3,
    }
}

fn run_msg(args: &[String]) -> ! {
    let signoff = args.iter().any(|a| a == "--signoff");
    let tracker = args.iter().any(|a| a == "--require-tracker");
    let file = args.iter().find(|a| !a.starts_with("--"));

    let text = match file {
        Some(f) => match fs::read_to_string(f) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("host-lint-ffmpeg: cannot read {f}: {e}");
                process::exit(2);
            }
        },
        None => {
            let mut s = String::new();
            if let Err(e) = io::Read::read_to_string(&mut io::stdin(), &mut s) {
                eprintln!("host-lint-ffmpeg: cannot read stdin: {e}");
                process::exit(2);
            }
            s
        }
    };

    let mode = config::load(std::path::Path::new("."))
        .map(|c| c.mode)
        .unwrap_or(config::Mode::Advise);
    let findings = msg::check_with(&text, signoff, tracker);
    if findings.is_empty() {
        process::exit(0);
    }
    let mut blocking = false;
    for f in &findings {
        let label = match f.tier {
            rules::Tier::Mechanical => {
                blocking = true;
                "flag"
            }
            _ => "warn",
        };
        println!("{label}: {} — {}", f.rule, f.detail);
    }
    process::exit(verdict(blocking, mode));
}

/// The added-line lane: `diff [<file>]`, or a unified diff on stdin.
fn run_diff(args: &[String]) -> ! {
    let file = args.iter().find(|a| !a.starts_with("--"));
    let text = match file {
        Some(f) => match fs::read_to_string(f) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("host-lint-ffmpeg: cannot read {f}: {e}");
                process::exit(2);
            }
        },
        None => {
            let mut s = String::new();
            if let Err(e) = io::Read::read_to_string(&mut io::stdin(), &mut s) {
                eprintln!("host-lint-ffmpeg: cannot read stdin: {e}");
                process::exit(2);
            }
            s
        }
    };

    let mode = config::load(std::path::Path::new("."))
        .map(|c| c.mode)
        .unwrap_or(config::Mode::Advise);
    let mut findings = diff::check_diff(&text);
    // The mixed cosmetic/functional check reads the whole diff rather than one line,
    // so it joins here rather than in the per-line pass.
    for c in cosmetic::check(&text) {
        findings.push(diff::Finding {
            rule: c.rule,
            tier: c.tier,
            path: String::new(),
            line: 0,
            detail: c.detail,
        });
    }
    if findings.is_empty() {
        process::exit(0);
    }
    let mut blocking = false;
    for f in &findings {
        let label = match f.tier {
            rules::Tier::Mechanical => {
                blocking = true;
                "flag"
            }
            _ => "warn",
        };
        if f.path.is_empty() {
            println!("{label}: {} — {}", f.rule, f.detail);
        } else {
            println!("{label}: {}:{}: {} — {}", f.path, f.line, f.rule, f.detail);
        }
    }
    process::exit(verdict(blocking, mode));
}

/// `branch` checks the current branch and its tags against the project's grammar,
/// and reports whether the series is frozen. Frozen is derived from tags rather than
/// declared, because a declaration goes stale the moment somebody tags.
fn run_branch(args: &[String]) -> ! {
    let dir = args
        .iter()
        .find(|a| !a.starts_with("--"))
        .cloned()
        .unwrap_or_else(|| ".".to_string());
    let path = std::path::Path::new(&dir);
    let cfg = match config::load(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("host-lint-ffmpeg: {e}");
            process::exit(2);
        }
    };
    let out = process::Command::new("git")
        .arg("-C").arg(&dir).args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output();
    let branch = match out {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).trim().to_string(),
        _ => {
            eprintln!("host-lint-ffmpeg: cannot read the current branch of {dir}");
            process::exit(2);
        }
    };

    let mut blocking = false;
    if !config::branch_ok(&cfg, &branch) {
        blocking = true;
        println!(
            "flag: branch-grammar — {branch:?} does not start with the configured prefix {:?}",
            cfg.branch_prefix.as_deref().unwrap_or("")
        );
    }
    let tags = process::Command::new("git")
        .arg("-C").arg(&dir).args(["tag", "--merged", &branch])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();
    for tag in tags.lines().map(str::trim).filter(|t| !t.is_empty()) {
        if !config::tag_ok(&cfg, tag) {
            blocking = true;
            println!(
                "flag: tag-grammar — {tag:?} does not start with the configured prefix {:?}",
                cfg.tag_prefix.as_deref().unwrap_or("")
            );
        }
    }
    if config::is_frozen_branch(path, &cfg, &branch) {
        println!("note: {branch} is a frozen series; findings report and never block");
    }
    if !blocking {
        println!("branch: {branch} satisfies the project grammar");
        process::exit(0);
    }
    process::exit(verdict(blocking, cfg.mode));
}

fn run_rules(args: &[String]) -> ! {
    // `--verify-source <tree>` checks a real FFmpeg checkout against the pinned
    // digests. It reports per section, because "the file changed" is not something
    // an operator can act on and an edit outside a rule-bearing section is not
    // rule drift.
    if args.first().map(String::as_str) == Some("--verify-source") {
        let Some(tree) = args.get(1) else {
            eprintln!("usage: host-lint pack ffmpeg rules --verify-source <ffmpeg-tree>");
            process::exit(2);
        };
        match rules::drifted_sections(std::path::Path::new(tree)) {
            Err(e) => {
                eprintln!("host-lint-ffmpeg: {e}");
                process::exit(2);
            }
            Ok(d) if d.is_empty() => {
                println!(
                    "rules: every pinned section matches {tree} at {}",
                    rules::UPSTREAM_COMMIT
                );
                process::exit(0);
            }
            Ok(d) => {
                let gating = d.iter().filter(|x| x.rule_bearing).count();
                for s in d.iter().filter(|x| x.rule_bearing) {
                    println!("DRIFT  {}", s.what);
                }
                for s in d.iter().filter(|x| !x.rule_bearing) {
                    println!("moved  {} (states no rules; reported, not gating)", s.what);
                }
                println!(
                    "-- {gating} rule-bearing section(s) differ from the corpus pinned at {}; {} other section(s) moved",
                    rules::UPSTREAM_COMMIT,
                    d.len() - gating
                );
                process::exit(if gating > 0 { 1 } else { 0 });
            }
        }
    }

    // `--check-freshness <tree>` asks whether the pin is still the newest commit
    // touching the rule source. Answered from a git checkout rather than the network:
    // the pack has no HTTP client, the operator already has a tree for
    // --verify-source, and a check that needs the internet cannot run in the
    // offline build this project verifies in.
    if args.first().map(String::as_str) == Some("--check-freshness") {
        let Some(tree) = args.get(1) else {
            eprintln!("usage: host-lint pack ffmpeg rules --check-freshness <ffmpeg-git-tree>");
            process::exit(2);
        };
        let out = process::Command::new("git")
            .args(["-C", tree, "log", "-1", "--format=%H", "--", "doc/developer.texi"])
            .output();
        let newest = match out {
            Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).trim().to_string(),
            Ok(o) => {
                eprintln!(
                    "host-lint-ffmpeg: git could not read {tree}: {}",
                    String::from_utf8_lossy(&o.stderr).trim()
                );
                process::exit(2);
            }
            Err(e) => {
                eprintln!("host-lint-ffmpeg: cannot run git: {e}");
                process::exit(2);
            }
        };
        if newest.is_empty() {
            eprintln!("host-lint-ffmpeg: {tree} has no history for doc/developer.texi");
            process::exit(2);
        }
        if newest == rules::UPSTREAM_COMMIT {
            println!("rules: the corpus is pinned at the newest doc/developer.texi commit ({newest})");
            process::exit(0);
        }
        println!("STALE  corpus pinned at {}", rules::UPSTREAM_COMMIT);
        println!("       newest doc/developer.texi commit is {newest}");
        println!("-- re-encode against the newer tree before trusting a clean rules verdict");
        process::exit(1);
    }

    let json = args.first().map(String::as_str) == Some("--json");
    if json {
        println!("{{");
        println!("  \"upstream_commit\": \"{}\",", rules::UPSTREAM_COMMIT);
        println!("  \"sources\": [");
        for (i, s) in rules::SOURCES.iter().enumerate() {
            let comma = if i + 1 == rules::SOURCES.len() { "" } else { "," };
            println!("    {{\"path\": \"{}\", \"sha256\": \"{}\"}}{comma}", s.path, s.sha256);
        }
        println!("  ],");
        println!("  \"rules\": [");
        for (i, r) in rules::RULES.iter().enumerate() {
            let comma = if i + 1 == rules::RULES.len() { "" } else { "," };
            let rate = match r.measured_rate {
                Some(v) => format!("{v}"),
                None => "null".to_string(),
            };
            println!(
                "    {{\"id\": \"{}\", \"section\": {}, \"subheading\": {}, \"tier\": \"{}\", \"lane\": \"{}\", \"measured_rate\": {rate}, \"summary\": {}}}{comma}",
                r.id,
                json_str(r.section),
                json_str(r.subheading),
                r.tier.as_str(),
                r.lane.as_str(),
                json_str(r.summary)
            );
        }
        println!("  ]");
        println!("}}");
        process::exit(0);
    }

    // The corpus checks itself before reporting. A registry that listed rules while
    // a rule-bearing section sat unmapped would be presenting an incomplete corpus
    // as the corpus, which is the failure the completeness test exists to catch —
    // and a test only catches it in CI, while this catches it wherever it runs.
    let unmapped = rules::unmapped_sections();
    let orphans = rules::orphan_rules();
    if !unmapped.is_empty() || !orphans.is_empty() {
        for s in &unmapped {
            eprintln!("host-lint-ffmpeg: rule-bearing section with no rule: {s}");
        }
        for r in &orphans {
            eprintln!("host-lint-ffmpeg: rule naming no known section: {r}");
        }
        eprintln!("host-lint-ffmpeg: the corpus is incomplete; refusing to present it as complete");
        process::exit(2);
    }

    println!("FFmpeg rule corpus, pinned at {}", rules::UPSTREAM_COMMIT);
    println!();
    for r in rules::RULES {
        let rate = match r.measured_rate {
            Some(v) => format!("{v:.2}"),
            None => "unmeasured".to_string(),
        };
        println!("{:<32} {:<11} {:<7} {}", r.id, r.tier.as_str(), r.lane.as_str(), rate);
        println!("    {}", r.summary);
        println!("    {} / {}", r.section, r.subheading);
    }
    println!();
    println!(
        "-- {} rule(s) over {} rule-bearing section(s); {} carry a measured rate (CALIBRATION.md)",
        rules::RULES.len(),
        rules::SECTIONS.iter().filter(|s| s.rule_bearing).count(),
        rules::RULES.iter().filter(|r| r.measured_rate.is_some()).count()
    );
    process::exit(0);
}

/// Minimal JSON string escaping: the corpus carries quotes and backslashes in rule
/// text, and emitting them raw would produce output nothing can parse.
fn json_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn main() {
    refuse_engine_skew();
    let args: Vec<String> = env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("rules") => run_rules(&args[1..]),
        Some("msg") => run_msg(&args[1..]),
        Some("diff") => run_diff(&args[1..]),
        Some("config") => run_config(&args[1..]),
        Some("branch") => run_branch(&args[1..]),
        _ => {}
    }
    // The lanes land by the build sequence on host-lint#22 (msg, commit,
    // series, mail, build, checklist, rules). Until a lane lands, every
    // invocation is a usage error: the skeleton never exits 0, so it cannot
    // report a clean verdict it did not earn (no-hollow-green).
    eprintln!("host-lint-ffmpeg: only `rules` is implemented; the lanes land by the build sequence on host-lint#22");
    process::exit(2);
}
