//! The MAINTAINERS parser (host-lint#22, mail-lane).
//!
//! Its whole risk is the observed syntax. `MAINTAINERS` documents its own convention
//! in one line — "A (CC <address>) after the name means that the maintainer prefers
//! to be CC-ed on" — and then uses three different forms for it, alongside two
//! parenthesized forms that are NOT a CC request at all. Getting that distinction
//! wrong in either direction is a real cost: miss a CC and a maintainer never sees
//! the patch, invent one and mail goes to somebody who did not ask for it.
//!
//! The fixture is `fixtures/upstream/maintainers-live-syntax.txt`, copied verbatim,
//! because a synthesized file exercises only the shapes I already thought of.

/// One maintainer entry: the paths it covers and who to tell.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// The section it appeared under. Entries are section-scoped: the same glob can
    /// appear under two sections meaning two different things, and a matcher that
    /// ignored the section would merge them.
    pub section: String,
    pub globs: Vec<String>,
    pub names: Vec<String>,
    /// Addresses that asked to be CC-ed. Empty when nobody did, which is the common
    /// case and must not be confused with an address appearing for another reason.
    pub cc: Vec<String>,
}

/// Parse a MAINTAINERS file.
pub fn parse(text: &str) -> Vec<Entry> {
    let mut out = Vec::new();
    let mut section = String::new();
    for raw in text.lines() {
        let line = raw.trim_end();
        if line.trim().is_empty() {
            continue;
        }
        // A section header sits at column zero and ends in a colon.
        if !line.starts_with(char::is_whitespace) && line.ends_with(':') {
            section = line.trim_end_matches(':').to_string();
            continue;
        }
        // An entry is indented, or (for the architecture table) sits at column zero
        // with the name column separated by run-of-spaces. Either way it needs the
        // two-column split below to be an entry at all.
        let Some(entry) = parse_entry(&section, line) else { continue };
        out.push(entry);
    }
    out
}

fn parse_entry(section: &str, line: &str) -> Option<Entry> {
    // The columns are separated by two or more spaces. One space cannot be the
    // separator: `Linux / PowerPC` and `aacenc*, aaccoder.c` both contain single
    // spaces inside the left column.
    let trimmed = line.trim_start();
    let idx = find_column_break(trimmed)?;
    let (left, right) = trimmed.split_at(idx);
    let paths = left.trim();
    let mut people = right.trim().to_string();

    if paths.is_empty() || people.is_empty() {
        return None;
    }

    // A `[2]` status marker sits between the columns.
    while people.starts_with('[') {
        let Some(close) = people.find(']') else { break };
        people = people[close + 1..].trim_start().to_string();
    }

    let globs: Vec<String> = paths
        .split(',')
        .map(|g| g.trim().to_string())
        .filter(|g| !g.is_empty())
        .collect();

    let mut names = Vec::new();
    let mut cc = Vec::new();
    for part in split_people(&people) {
        let (name, addr) = split_cc(&part);
        if !name.is_empty() {
            names.push(name);
        }
        if let Some(a) = addr {
            cc.push(a);
        }
    }
    if names.is_empty() && cc.is_empty() {
        return None;
    }
    Some(Entry { section: section.to_string(), globs, names, cc })
}

/// The first run of two or more spaces, which is the column break.
fn find_column_break(s: &str) -> Option<usize> {
    let b = s.as_bytes();
    (0..b.len().saturating_sub(1)).find(|&i| b[i] == b' ' && b[i + 1] == b' ')
}

/// Split the people column on commas that are not inside parentheses, so
/// `Sean McGovern (CC <a@b>), Lauri Kasanen` yields two people rather than three.
fn split_people(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut depth = 0i32;
    let mut cur = String::new();
    for c in s.chars() {
        match c {
            '(' => {
                depth += 1;
                cur.push(c);
            }
            ')' => {
                depth -= 1;
                cur.push(c);
            }
            ',' if depth == 0 => {
                out.push(cur.trim().to_string());
                cur.clear();
            }
            _ => cur.push(c),
        }
    }
    if !cur.trim().is_empty() {
        out.push(cur.trim().to_string());
    }
    out
}

/// Split one person into a name and, if they asked for it, a CC address.
///
/// The three observed CC forms are all `(CC ...)`. A parenthesized part WITHOUT the
/// `CC` marker is not a CC request, whatever it contains: `aadec.c` carries an
/// obfuscated address that way and `evc*` carries a plain name.
fn split_cc(person: &str) -> (String, Option<String>) {
    let Some(open) = person.find('(') else {
        return (person.trim().to_string(), None);
    };
    let name = person[..open].trim().to_string();
    let inside = person[open + 1..].trim_end_matches(')').trim();

    let Some(rest) = inside.strip_prefix("CC").map(str::trim) else {
        // Not a CC request. The parenthesized content is dropped rather than
        // guessed at: an address here belongs to a maintainer who did not ask.
        return (name, None);
    };
    let addr = deobfuscate(rest.trim_start_matches('<').trim_end_matches('>').trim());
    if addr.is_empty() {
        (name, None)
    } else {
        (name, Some(addr))
    }
}

/// `t.rapp at noa-archive dot com` is an address written to defeat scrapers, and it
/// is one of the three forms in use. Turning it back into an address is required to
/// put it in a Cc list.
fn deobfuscate(s: &str) -> String {
    if !s.contains(" at ") && !s.contains(" dot ") {
        return s.to_string();
    }
    s.replace(" at ", "@").replace(" dot ", ".")
}

/// Source extensions an extensionless entry may name. Upstream writes `vf_bwdif`
/// meaning that filter's source, not a file literally called `vf_bwdif`.
const SOURCE_EXTENSIONS: &[&str] = &[".c", ".h", ".cpp", ".m", ".S", ".asm", ".mak", ".texi"];

/// Whether a glob matches a path. Only `*` is supported, because that is all
/// MAINTAINERS uses; a `(` in a glob is a literal, as in `vf_(t)interlace`.
///
/// An entry with no `*` and no extension is matched against the basename with a
/// source extension appended, because that is how upstream writes them. Deliberately
/// NOT treated as a prefix: `vf_bwdif` would then claim `vf_bwdif_cuda.c`, whose
/// maintainer may be someone else, and putting mail in front of the wrong person is
/// the failure that stops people trusting a Cc check.
pub fn glob_matches(glob: &str, path: &str) -> bool {
    let base = path.rsplit('/').next().unwrap_or(path);
    if wildcard(glob, path) || wildcard(glob, base) {
        return true;
    }
    if !glob.contains('*') && !glob.contains('.') {
        return SOURCE_EXTENSIONS
            .iter()
            .any(|e| base == format!("{glob}{e}"));
    }
    false
}

fn wildcard(pat: &str, s: &str) -> bool {
    match pat.find('*') {
        None => pat == s,
        Some(i) => {
            let (head, tail) = (&pat[..i], &pat[i + 1..]);
            if !s.starts_with(head) {
                return false;
            }
            let rest = &s[head.len()..];
            if tail.is_empty() {
                return true;
            }
            // One `*` is all MAINTAINERS uses; recurse anyway so two cannot silently
            // match the wrong thing.
            (0..=rest.len()).any(|k| wildcard(tail, &rest[k..]))
        }
    }
}

/// Every CC address whose entry covers one of these paths, section-scoped.
pub fn required_cc(entries: &[Entry], paths: &[&str]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for e in entries {
        if e.cc.is_empty() {
            continue;
        }
        if paths
            .iter()
            .any(|p| e.globs.iter().any(|g| glob_matches(g, p)))
        {
            for a in &e.cc {
                if !out.contains(a) {
                    out.push(a.clone());
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const LIVE: &str = include_str!("../fixtures/upstream/maintainers-live-syntax.txt");

    fn live() -> Vec<Entry> {
        parse(LIVE)
    }

    #[test]
    fn the_three_cc_forms_all_parse() {
        let e = live();
        let cc = |glob: &str| {
            e.iter()
                .find(|x| x.globs.iter().any(|g| g == glob))
                .unwrap_or_else(|| panic!("no entry for {glob}"))
                .cc
                .clone()
        };
        // Angle-bracketed.
        assert_eq!(cc("vf_bwdif"), vec!["thomas.mundt@hr.de"]);
        // Bare.
        assert_eq!(cc("dshow.c"), vec!["rogerdpack@gmail.com"]);
        // Obfuscated, which has to be turned back into an address to be usable.
        assert_eq!(cc("vf_readvitc.c"), vec!["t.rapp@noa-archive.com"]);
    }

    #[test]
    fn a_parenthesized_part_without_the_cc_marker_is_not_a_cc() {
        let e = live();
        // An obfuscated address, but nobody asked to be CC-ed.
        let aadec = e.iter().find(|x| x.globs.iter().any(|g| g == "aadec.c")).unwrap();
        assert!(aadec.cc.is_empty(), "{:?}", aadec.cc);
        assert_eq!(aadec.names, vec!["Vesselin Bontchev"]);
        // A plain name in parentheses.
        let evc = e.iter().find(|x| x.globs.iter().any(|g| g == "evc*")).unwrap();
        assert!(evc.cc.is_empty());
        assert_eq!(evc.names, vec!["Samsung"]);
    }

    #[test]
    fn entries_are_section_scoped() {
        let e = live();
        let f = e.iter().find(|x| x.globs.iter().any(|g| g == "vf_bwdif")).unwrap();
        assert_eq!(f.section, "Filters");
        let c = e.iter().find(|x| x.globs.iter().any(|g| g == "asv*")).unwrap();
        assert_eq!(c.section, "Codecs");
    }

    #[test]
    fn comma_separated_globs_parse_with_and_without_a_space() {
        let e = live();
        let with = e.iter().find(|x| x.globs.contains(&"aacenc*".to_string())).unwrap();
        assert_eq!(with.globs, vec!["aacenc*", "aaccoder.c"]);
        let without = e.iter().find(|x| x.globs.contains(&"amfdec*".to_string())).unwrap();
        assert_eq!(without.globs, vec!["amfdec*", "amfenc*"]);
    }

    #[test]
    fn a_status_marker_is_not_part_of_the_name() {
        let e = live();
        let amf = e.iter().find(|x| x.globs.contains(&"amfdec*".to_string())).unwrap();
        assert_eq!(amf.names, vec!["Dmitrii Ovchinnikov", "Araz Iusubov"]);
    }

    #[test]
    fn several_maintainers_split_on_commas_outside_parentheses() {
        let e = live();
        let arch = e.iter().find(|x| x.globs.iter().any(|g| g == "Linux / PowerPC")).unwrap();
        assert_eq!(arch.names, vec!["Sean McGovern", "Lauri Kasanen"]);
        assert_eq!(arch.cc, vec!["gseanmcg@gmail.com"]);
    }

    #[test]
    fn a_glob_containing_a_parenthesis_is_a_literal() {
        let e = live();
        let g = e.iter().find(|x| x.globs.iter().any(|g| g == "vf_(t)interlace"));
        assert!(g.is_some(), "vf_(t)interlace should parse as a glob");
    }

    #[test]
    fn globs_match_full_paths_and_basenames() {
        assert!(glob_matches("asv*", "libavcodec/asvdec.c"));
        assert!(glob_matches("asv*", "asvdec.c"));
        assert!(glob_matches("aadec.c", "libavformat/aadec.c"));
        assert!(!glob_matches("asv*", "libavcodec/h264.c"));
        assert!(!glob_matches("aadec.c", "libavformat/aadec.h"));

        // An extensionless entry names that filter's source.
        assert!(glob_matches("vf_bwdif", "libavfilter/vf_bwdif.c"));
        assert!(glob_matches("vf_bwdif", "libavfilter/vf_bwdif.h"));
        // And is NOT a prefix: a sibling file may have a different maintainer, and
        // mailing the wrong person is what stops a Cc check being trusted.
        assert!(!glob_matches("vf_bwdif", "libavfilter/vf_bwdif_cuda.c"));
    }

    #[test]
    fn required_cc_covers_the_touched_paths_and_nothing_else() {
        let e = live();
        assert_eq!(
            required_cc(&e, &["libavfilter/vf_bwdif.c"]),
            vec!["thomas.mundt@hr.de"]
        );
        // A path whose maintainer never asked yields nothing, rather than their
        // address scraped from somewhere else in the line.
        assert!(required_cc(&e, &["libavformat/aadec.c"]).is_empty());
        // Nothing touched, nothing required.
        assert!(required_cc(&e, &[]).is_empty());
    }

    #[test]
    fn the_convention_line_is_not_read_as_an_entry() {
        // "A (CC <address>) after the name means..." is prose describing the syntax.
        // Parsing it as an entry would put a literal `<address>` in a Cc list.
        let e = live();
        assert!(
            e.iter().all(|x| !x.cc.iter().any(|a| a.contains("address"))),
            "the documentation line was parsed as an entry: {:?}",
            e.iter().filter(|x| x.cc.iter().any(|a| a.contains("address"))).collect::<Vec<_>>()
        );
    }
}
