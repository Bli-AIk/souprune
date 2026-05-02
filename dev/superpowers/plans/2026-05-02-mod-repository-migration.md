# Mod Repository Migration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Publish the maintained SoupRune project mods as independent GitHub repositories, consume them from `souprune` as submodules, and archive the old branch-based example-mod repository.

**Architecture:** The active project content moves from a custom bare-repo/worktree installer to two standard Git submodules under `projects/`. `souprune_undertale_preset` remains the reusable library preset; `souprune_mad_dummy_example` becomes the concrete example project that depends on it. The old `souprune_example_mods` repository is updated with a deprecation README and archived only after the replacement repos and main repository pointers are verified.

**Tech Stack:** Git, GitHub CLI (`gh`), Rust/Cargo, Bash/PowerShell docs cleanup, Git submodules.

---

## File Structure

Remote repositories:

- Create/populate: `https://github.com/Bli-AIk/souprune_undertale_preset`
- Create/populate: `https://github.com/Bli-AIk/souprune_mad_dummy_example`
- Update/archive: `https://github.com/Bli-AIk/souprune_example_mods`

Main repository files:

- Modify: `.gitmodules`
- Modify: `.gitignore`
- Delete: `mods.toml`
- Delete: `scripts/setup_mods.sh`
- Delete: `scripts/setup_mods.ps1`
- Modify: `scripts/pack.sh`
- Modify: `projects/config.toml`
- Modify: `readme.md`
- Modify: `readme_zh-hans.md`
- Modify: `CONTRIBUTING.md`
- Modify: `CONTRIBUTING_zh-hans.md`
- Modify: `crates/souprune/src/config.rs`
- Modify: `crates/souprune/tests/test_support.rs`
- Modify: `crates/souprune/tests/asset_parse_smoke.rs`
- Modify: `crates/souprune_cauld_ron/tests/*`
- Modify: `crates/souprune_lint/readme.md`
- Modify: `crates/souprune_lint/readme_zh-hans.md`
- Add gitlinks: `projects/undertale_preset`, `projects/mad_dummy_example`

## Task 1: Create And Populate New Mod Repositories

- [ ] **Step 1: Confirm GitHub authentication and repository absence**

Run:

```bash
gh auth status
gh repo view Bli-AIk/souprune_undertale_preset --json name,url 2>/dev/null || true
gh repo view Bli-AIk/souprune_mad_dummy_example --json name,url 2>/dev/null || true
```

Expected: authenticated as `Bli-AIk`; the two new repo lookups either fail because they do not exist or show existing repos that can be reused.

- [ ] **Step 2: Create missing repositories**

Run only for repositories that do not exist:

```bash
gh repo create Bli-AIk/souprune_undertale_preset --public --description "Undertale-style reusable preset project for SoupRune"
gh repo create Bli-AIk/souprune_mad_dummy_example --public --description "Mad Dummy example project for SoupRune"
```

Expected: both repositories exist and are public.

- [ ] **Step 3: Populate `souprune_undertale_preset` from the old branch**

Run:

```bash
rm -rf /tmp/souprune_undertale_preset_migrate
git clone --single-branch --branch mod/undertale_preset https://github.com/Bli-AIk/souprune_example_mods.git /tmp/souprune_undertale_preset_migrate
cd /tmp/souprune_undertale_preset_migrate
git branch -M main
git remote set-url origin https://github.com/Bli-AIk/souprune_undertale_preset.git
git push -u origin main
```

Expected: `souprune_undertale_preset` has a `main` branch containing the preset content from `mod/undertale_preset`.

- [ ] **Step 4: Populate `souprune_mad_dummy_example` from the old branch**

Run:

```bash
rm -rf /tmp/souprune_mad_dummy_example_migrate
git clone --single-branch --branch mod/example_mod https://github.com/Bli-AIk/souprune_example_mods.git /tmp/souprune_mad_dummy_example_migrate
cd /tmp/souprune_mad_dummy_example_migrate
git branch -M main
git remote set-url origin https://github.com/Bli-AIk/souprune_mad_dummy_example.git
```

Expected: local migration checkout has `example_mod` content on branch `main`, ready for rename edits.

- [ ] **Step 5: Rename example metadata in the new example repository**

Modify `/tmp/souprune_mad_dummy_example_migrate/mod.toml`:

```toml
name = "mad_dummy_example"
version = "0.1.0"
authors = ["SoupRune Contributors"]
description = "Mad Dummy example project for SoupRune."
```

Modify README files in `/tmp/souprune_mad_dummy_example_migrate` so they describe `souprune_mad_dummy_example`, not the old multi-branch collection. Keep setup instructions focused on placing the repository at `projects/mad_dummy_example` or using the main `souprune` submodule.

- [ ] **Step 6: Commit and push the renamed example repository**

Run:

```bash
cd /tmp/souprune_mad_dummy_example_migrate
git status --short
git add mod.toml readme.md readme_zh-hans.md
git commit -m "chore: rename to mad dummy example"
git push -u origin main
```

Expected: `souprune_mad_dummy_example` has `main` with `mod.toml` declaring `name = "mad_dummy_example"`.

## Task 2: Convert Main Repository To Project Submodules

- [ ] **Step 1: Remove old ignored worktree directories from the main checkout**

Run from the main `souprune` repository:

```bash
rm -rf projects/example_mod projects/example_battle_mod projects/example_am_mod projects/undertale_preset
```

Expected: only local ignored worktree checkouts are removed; no tracked main-repo files are deleted.

- [ ] **Step 2: Update `.gitignore` to allow maintained project submodules**

Edit `.gitignore` so the project section reads:

```gitignore
# Project mods (local user projects are ignored; maintained examples are git submodules)
projects/*/
!projects/config.toml
!projects/undertale_preset/
!projects/mad_dummy_example/
```

- [ ] **Step 3: Add the two project submodules**

Run:

```bash
git submodule add https://github.com/Bli-AIk/souprune_undertale_preset.git projects/undertale_preset
git submodule add https://github.com/Bli-AIk/souprune_mad_dummy_example.git projects/mad_dummy_example
```

Expected: `.gitmodules` has both new entries, and `git status --short` shows gitlinks for the two project paths.

- [ ] **Step 4: Remove old worktree installer files**

Run:

```bash
git rm mods.toml scripts/setup_mods.sh scripts/setup_mods.ps1
```

Expected: the old `mods.toml` and setup scripts are staged for deletion.

## Task 3: Update Main Repository References

- [ ] **Step 1: Update active project config**

Modify `projects/config.toml` to:

```toml
[project]
mod_name = "mad_dummy_example"
language = "en-US"

[window]
resolution_scale = 4
```

- [ ] **Step 2: Update fallback config**

In `crates/souprune/src/config.rs`, change the fallback project name and message from `example_mod` to `mad_dummy_example`.

- [ ] **Step 3: Update test support project roots**

In `crates/souprune/tests/test_support.rs`, replace the default project root with `projects/mad_dummy_example`, remove the `example_am_mod` root, and map:

```rust
"mad_dummy_example" => PROJECT_ROOT.as_path(),
"undertale_preset" => PROJECT_PRESET_ROOT.as_path(),
```

The helper functions `parse_project_ron` and `list_project_files_with_suffix` should default to `mad_dummy_example`.

- [ ] **Step 4: Update smoke tests**

In `crates/souprune/tests/asset_parse_smoke.rs`:

- Replace `example_mod` with `mad_dummy_example`.
- Remove `example_am_mod` from candidate lists.
- Keep `undertale_preset`.
- Update skip messages from `worktree` to `project submodule`.

- [ ] **Step 5: Rename tracked fixture directories**

Run:

```bash
git mv crates/souprune_cauld_ron/tests/fixtures/battle_dialogue_channel_regressions/example_mod crates/souprune_cauld_ron/tests/fixtures/battle_dialogue_channel_regressions/mad_dummy_example
git mv crates/souprune_cauld_ron/tests/fixtures/project_ron_baselines/example_mod crates/souprune_cauld_ron/tests/fixtures/project_ron_baselines/mad_dummy_example
git rm -r crates/souprune_cauld_ron/tests/fixtures/project_ron_baselines/example_am_mod
```

Expected: fixture baselines align with the new active example and no longer preserve the archived AM example fixture.

- [ ] **Step 6: Update cauld-ron tests**

Replace `example_mod` with `mad_dummy_example` in:

- `crates/souprune_cauld_ron/tests/battle_dialogue_channel_regressions.rs`
- `crates/souprune_cauld_ron/tests/cotton_first_turn_component.rs`
- `crates/souprune_cauld_ron/tests/cotton_first_turn_snapshot.rs`
- `crates/souprune_cauld_ron/tests/project_performance_snapshots.rs`

Update skip and assertion messages to say `mad_dummy_example`.

- [ ] **Step 7: Update packaging allowlist**

Modify `scripts/pack.sh` to remove the `MODS_TOML` dependency and package this explicit list:

```bash
PROJECT_MODS=("undertale_preset" "mad_dummy_example")
```

The packaging loop should iterate `PROJECT_MODS[@]`, check `projects/$mod_name`, and copy git-tracked files from each submodule with `git -C "${mod_dir}" ls-files -z`.

- [ ] **Step 8: Update documentation**

Update:

- `readme.md`
- `readme_zh-hans.md`
- `CONTRIBUTING.md`
- `CONTRIBUTING_zh-hans.md`
- `crates/souprune_lint/readme.md`
- `crates/souprune_lint/readme_zh-hans.md`

Required documentation content:

- No default `scripts/setup_mods.*` flow.
- `git submodule update --init --recursive` is the setup path.
- `mad_dummy_example` is the default runnable example.
- `souprune_undertale_preset` and `souprune_mad_dummy_example` are the maintained example repositories.
- `souprune_example_mods` is archived legacy content.

## Task 4: Verify Main Repository Migration

- [ ] **Step 1: Search for stale active references**

Run:

```bash
rg -n "example_mod|example_am_mod|example_battle_mod|setup_mods|mods\\.toml|souprune_example_mods" readme.md readme_zh-hans.md CONTRIBUTING.md CONTRIBUTING_zh-hans.md scripts crates projects Cargo.toml .gitignore .gitmodules -S
```

Expected: no stale default-path references. References to `souprune_example_mods` may remain only as archived legacy notes.

- [ ] **Step 2: Verify submodules**

Run:

```bash
git submodule update --init --recursive
test -f projects/undertale_preset/mod.toml
test -f projects/mad_dummy_example/mod.toml
rg -n '^name = "mad_dummy_example"$' projects/mad_dummy_example/mod.toml
```

Expected: all commands succeed.

- [ ] **Step 3: Format and run quality checks**

Run:

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -D warnings
```

Expected: both commands succeed.

- [ ] **Step 4: Commit main repository migration**

Run:

```bash
git status --short
git add .gitmodules .gitignore projects/config.toml scripts/pack.sh readme.md readme_zh-hans.md CONTRIBUTING.md CONTRIBUTING_zh-hans.md crates/souprune/src/config.rs crates/souprune/tests crates/souprune_cauld_ron/tests crates/souprune_lint/readme.md crates/souprune_lint/readme_zh-hans.md projects/undertale_preset projects/mad_dummy_example
git commit -m "chore: migrate maintained mods to submodules"
```

Expected: main repository commit contains submodule gitlinks and all reference updates.

## Task 5: Deprecate And Archive Old Example Mods Repository

- [ ] **Step 1: Clone old repository main branch**

Run:

```bash
rm -rf /tmp/souprune_example_mods_archive
git clone --single-branch --branch main https://github.com/Bli-AIk/souprune_example_mods.git /tmp/souprune_example_mods_archive
```

Expected: main branch checkout is ready for README update.

- [ ] **Step 2: Replace old repository README files**

Write `/tmp/souprune_example_mods_archive/readme.md` with:

```markdown
# souprune_example_mods

This repository is deprecated and archived.

Active maintained SoupRune example content moved to:

- https://github.com/Bli-AIk/souprune_undertale_preset
- https://github.com/Bli-AIk/souprune_mad_dummy_example

The old branch-based examples remain here only for historical reference:

- `mod/undertale_preset`
- `mod/example_mod`
- `mod/example_battle_mod`
- `mod/example_am_mod`

New users should clone https://github.com/Bli-AIk/souprune and initialize its submodules with:

```bash
git submodule update --init --recursive
```
```

Write `/tmp/souprune_example_mods_archive/readme_zh-hans.md` with equivalent Simplified Chinese text.

- [ ] **Step 3: Commit and push old repository deprecation**

Run:

```bash
cd /tmp/souprune_example_mods_archive
git add readme.md readme_zh-hans.md
git commit -m "docs: archive branch-based example mods"
git push origin main
```

Expected: old repository main branch clearly points users to the new repositories.

- [ ] **Step 4: Archive old repository**

Run:

```bash
gh repo archive Bli-AIk/souprune_example_mods --yes
gh repo view Bli-AIk/souprune_example_mods --json isArchived
```

Expected: JSON output contains `"isArchived":true`.

## Task 6: Final Verification

- [ ] **Step 1: Confirm remote repositories**

Run:

```bash
gh repo view Bli-AIk/souprune_undertale_preset --json name,url,visibility
gh repo view Bli-AIk/souprune_mad_dummy_example --json name,url,visibility
gh repo view Bli-AIk/souprune_example_mods --json name,url,isArchived
```

Expected: two active public repositories exist; old repository is archived.

- [ ] **Step 2: Confirm final main repository status**

Run:

```bash
git status --short --branch
git log --oneline -3
```

Expected: branch `chore/mod-repo-migration-design` contains the design and migration commits; worktree is clean except for acceptable untracked local temp files, if any.
