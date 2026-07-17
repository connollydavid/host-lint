# Fixture licensing policy

This repository is released under the Unlicense; FFmpeg source is LGPL
(GPL for some files). Test fixtures for the pack therefore follow the
fixture-licensing rule from the design review (host-lint#23):

- A fixture is synthesized by default: written for this repository and
  shaped like the case it exercises, with no upstream code in it.
- A real upstream excerpt is used only where synthesis cannot reproduce
  the case. Such an excerpt lives under `fixtures/upstream/`, and that
  directory carries a `PROVENANCE.md` naming, for every file, the source
  commit, the upstream path, and the license of the excerpted code.
  Nothing under `fixtures/upstream/` is covered by this repository's
  Unlicense grant.

CI refuses an `upstream/` directory with no `PROVENANCE.md`. The rest of
`fixtures/` is ordinary repository content under the Unlicense.
