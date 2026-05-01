# Mod Repository Migration Design

> For agentic workers: this design must be implemented through the Superpowers implementation-planning flow before code or repository changes are made.

## Goal

Move the actively maintained SoupRune example content away from the single `souprune_example_mods` repository-with-branches model. Keep the old repository as an archived historical reference, publish the maintained mods as independent GitHub repositories, and make the main `souprune` repository consume them through standard Git submodules.

## Current State

The main repository currently uses `mods.toml` plus `scripts/setup_mods.sh` and `scripts/setup_mods.ps1` to clone `https://github.com/Bli-AIk/souprune_example_mods.git` as a bare repository under `.mod-repo/`, then creates worktrees under `projects/*`.

The old remote currently has these active branches:

- `mod/undertale_preset`
- `mod/example_mod`
- `mod/example_battle_mod`
- `mod/example_am_mod`

The main repository already uses Git submodules for maintained crates under `crates/*`, so using submodules for maintained project content follows an existing repository management pattern.

## Target Repositories

Create these GitHub repositories under `Bli-AIk` if they do not already exist:

- `souprune_undertale_preset`
  - Public repository.
  - Source content comes from `souprune_example_mods` branch `mod/undertale_preset`.
  - Main repository path: `projects/undertale_preset`.
  - `mod.toml` name stays `undertale_preset`.

- `souprune_mad_dummy_example`
  - Public repository.
  - Source content comes from `souprune_example_mods` branch `mod/example_mod`.
  - Main repository path: `projects/mad_dummy_example`.
  - `mod.toml` name changes from `example_mod` to `mad_dummy_example`.
  - Description should make clear that this is the Mad Dummy example project for SoupRune, not a reusable preset.

Do not migrate these branches into new repositories:

- `mod/example_battle_mod`
- `mod/example_am_mod`

They remain in `souprune_example_mods` as legacy historical branches.

## Main Repository Design

The `souprune` repository should add two submodules:

- `projects/undertale_preset` -> `https://github.com/Bli-AIk/souprune_undertale_preset.git`
- `projects/mad_dummy_example` -> `https://github.com/Bli-AIk/souprune_mad_dummy_example.git`

The `.gitignore` rule `projects/*/` should remain as the default for local user projects, but it must explicitly allow the two maintained submodule paths. This preserves the existing policy that arbitrary local mods are not committed while allowing the curated examples to be versioned as gitlinks.

`projects/config.toml` should use:

```toml
[project]
mod_name = "mad_dummy_example"
language = "en-US"
```

The old `mods.toml` and setup scripts should be removed from the default path. If keeping a pointer to the old workflow is useful, document it as archived legacy behavior rather than supporting it as the current install mechanism.

## Documentation Design

Update user-facing setup instructions in both English and Simplified Chinese README files:

- Quick Start should use `git submodule update --init --recursive`.
- Remove the `Setup Example Mods` step based on `scripts/setup_mods.*`.
- Explain that maintained example content is now shipped as submodules.
- Point users to:
  - `souprune_undertale_preset`
  - `souprune_mad_dummy_example`
- Mention that `souprune_example_mods` is archived and only preserves old branch-based examples.

Update contribution documentation where it references example mods so that it no longer presents `souprune_example_mods` as the active example-mod home.

## Test and Tooling Design

Update tests and packaging code that currently use `example_mod` as the active example project:

- Rename default project references from `example_mod` to `mad_dummy_example`.
- Keep `undertale_preset` as a named project.
- Remove `example_am_mod` from default smoke-test candidates.
- Preserve skip behavior when optional project submodules are absent.
- Update fixture path names only if the fixture directories are part of this repository and need to track the project rename.

Update `scripts/pack.sh` so it no longer depends on `mods.toml` for the maintained examples. It should package the maintained submodule projects by an explicit allowlist:

- `undertale_preset`
- `mad_dummy_example`

## Old Repository Design

After the new repositories are created, populated, and referenced by the main repository:

1. Clone or update `Bli-AIk/souprune_example_mods`.
2. Update its `main` branch README to state:
   - the repository is deprecated and archived,
   - active maintained examples moved to `souprune_undertale_preset` and `souprune_mad_dummy_example`,
   - old branches remain only for historical reference.
3. Push the README change.
4. Archive the GitHub repository with `gh repo archive Bli-AIk/souprune_example_mods --yes`.

Archiving happens only after the replacement repositories are reachable and the main `souprune` repository has working submodule pointers.

## Migration Order

1. Create the two new GitHub repositories if absent.
2. Populate each repository from the matching old branch.
3. Rename `example_mod` content to `mad_dummy_example` inside the new example repository.
4. Add both repositories as submodules in the main repository.
5. Update main repository config, docs, tests, packaging, and references.
6. Run formatting and quality checks required by the main repository.
7. Update and archive `souprune_example_mods`.

## Verification

Minimum verification after implementation:

- `git submodule update --init --recursive` succeeds from the main repository.
- `projects/undertale_preset/mod.toml` exists.
- `projects/mad_dummy_example/mod.toml` exists and declares `name = "mad_dummy_example"`.
- `projects/config.toml` points to `mad_dummy_example`.
- `cargo fmt --all` succeeds.
- `cargo clippy --workspace --all-targets -D warnings` succeeds.
- Relevant smoke tests either pass or skip cleanly when optional project content is absent.
- `gh repo view Bli-AIk/souprune_example_mods --json isArchived` reports `true` after the final archive step.

## Decisions

- Use submodules for maintained example content.
- Keep `undertale_preset` as a stable reusable library mod.
- Rename `example_mod` to `mad_dummy_example` because the content centers on the Mad Dummy battle/example and the new name is specific without implying it is a general preset.
- Do not migrate `example_battle_mod` or `example_am_mod`.
- Do not keep the custom worktree setup scripts as the current install path.
