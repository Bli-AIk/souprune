# Third-party forks

This directory contains vendored third-party repositories that SoupRune needs to
pin, patch, or test against directly.

Code under `third_party/` is different from code under `crates/`:

- `crates/` contains first-party SoupRune crates or actively maintained support
  crates owned as part of this workspace.
- `third_party/` contains forks of upstream projects. These forks are kept only
  when a local patch, pinned branch, or upstream integration gap makes a normal
  crates.io dependency insufficient.
- SoupRune CI may build and test against these forks, but their own lint policy,
  release process, and long-term maintenance remain upstream-oriented.
- We do not actively maintain these forks as independent products. Keep local
  changes minimal, document why they exist, and prefer upstream issues or pull
  requests once the patch is understood.

Use this directory for temporary or narrowly scoped upstream forks, especially
when keeping them separate from first-party crates makes review, updates, and
eventual upstreaming clearer.
