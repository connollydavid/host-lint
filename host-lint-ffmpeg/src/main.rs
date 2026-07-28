mod rules;
mod sha256;

use std::env;
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
        "-- {} rule(s) over {} rule-bearing section(s); every tier is declared and none is measured yet",
        rules::RULES.len(),
        rules::SECTIONS.iter().filter(|s| s.rule_bearing).count()
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
    if args.first().map(String::as_str) == Some("rules") {
        run_rules(&args[1..]);
    }
    // The lanes land by the build sequence on host-lint#22 (msg, commit,
    // series, mail, build, checklist, rules). Until a lane lands, every
    // invocation is a usage error: the skeleton never exits 0, so it cannot
    // report a clean verdict it did not earn (no-hollow-green).
    eprintln!("host-lint-ffmpeg: only `rules` is implemented; the lanes land by the build sequence on host-lint#22");
    process::exit(2);
}
