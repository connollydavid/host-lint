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

fn main() {
    refuse_engine_skew();
    // The lanes land by the build sequence on host-lint#22 (msg, commit,
    // series, mail, build, checklist, rules). Until a lane lands, every
    // invocation is a usage error: the skeleton never exits 0, so it cannot
    // report a clean verdict it did not earn (no-hollow-green).
    eprintln!("host-lint-ffmpeg: no lanes are implemented yet; see the build sequence on host-lint#22");
    process::exit(2);
}
