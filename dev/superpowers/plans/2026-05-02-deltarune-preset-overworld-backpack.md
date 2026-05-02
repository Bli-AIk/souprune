# Deltarune Preset Overworld Backpack Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 创建 `deltarune_preset` 前置 mod，并提供基础可用的 Deltarune overworld 背包/菜单与本地 smoke 验收 mod。

**Architecture:** `deltarune_preset` 是与 `undertale_preset` 平级的独立 GitHub 仓库，通过主仓库 `projects/deltarune_preset` submodule 接入。第一阶段主要复用现有 content crate 生成流程和 FRE/View 数据管线，用 DR 命名、DR 640x480 左上原点坐标、三角色 party facts 和 ITEM/STORAGE/KEYITEM 菜单状态替换 UT 背包概念。

**Tech Stack:** Rust、Bevy、SoupRune content library、FRE RON、View RON、Git submodule、GitHub CLI。

---

## File Structure

- Modify: `.gitmodules`
  - Add `projects/deltarune_preset` submodule.
- Modify: `.gitignore`
  - Add `!projects/deltarune_preset/`.
- Create: `projects/deltarune_preset/`
  - New submodule repository `Bli-AIk/souprune_deltarune_preset`.
  - Content structure copied from `undertale_preset`, then reduced and renamed for DR overworld backpack.
- Create in submodule: `projects/deltarune_preset/mod.toml`
  - Declares `name = "deltarune_preset"` and `entry_point = false`.
- Create in submodule: `projects/deltarune_preset/app/*.ron`
  - Global rules, input config, flow config required by preset.
- Create in submodule: `projects/deltarune_preset/overworld/rules/dark_menu.fre.ron`
  - Handles menu open/close, category switching, list cursor movement, and unusable key item feedback.
- Create in submodule: `projects/deltarune_preset/overworld/view/dark_menu.view.ron`
  - Defines DR-style 640x480 menu layout.
- Create in submodule: `projects/deltarune_preset/content/src/**/*.rs`
  - Generated-content registry source modules for the DR content files.
- Create local ignored fixture: `projects/deltarune_smoke_test/`
  - Copied from `mad_dummy_example` minimal project and changed to depend on `deltarune_preset`.
  - Initializes party, money, inventory, storage, key item, and menu facts for manual acceptance.

## Task 1: Write Plan and Repository Baseline

**Files:**
- Create: `dev/superpowers/plans/2026-05-02-deltarune-preset-overworld-backpack.md`

- [ ] **Step 1: Verify baseline worktree**

Run:

```bash
git submodule update --init --recursive --jobs 1
cargo build
```

Expected: `cargo build` exits 0 before feature changes.

- [ ] **Step 2: Commit this plan**

Run:

```bash
git add -f dev/superpowers/plans/2026-05-02-deltarune-preset-overworld-backpack.md
git commit -m "docs: plan deltarune preset overworld backpack"
```

Expected: a plan-only commit.

## Task 2: Create Remote Repository and Submodule

**Files:**
- Modify: `.gitmodules`
- Modify: `.gitignore`
- Create: `projects/deltarune_preset/`

- [ ] **Step 1: Check whether the GitHub repo exists**

Run:

```bash
gh repo view Bli-AIk/souprune_deltarune_preset --json nameWithOwner,visibility
```

Expected: repo exists, or the command exits non-zero.

- [ ] **Step 2: Create the repo if missing**

Run if Step 1 exits non-zero:

```bash
gh repo create Bli-AIk/souprune_deltarune_preset --public --description "Deltarune-style preset mod for SoupRune." --confirm
```

Expected: repo is created and `gh repo view Bli-AIk/souprune_deltarune_preset` succeeds.

- [ ] **Step 3: Seed the repository from `undertale_preset`**

Run:

```bash
tmp_dir="$(mktemp -d)"
git clone https://github.com/Bli-AIk/souprune_deltarune_preset.git "$tmp_dir"
rsync -a --delete --exclude .git --exclude target --exclude .build projects/undertale_preset/ "$tmp_dir/"
cd "$tmp_dir"
git add .
git commit -m "feat: seed deltarune preset structure"
git push origin main
```

Expected: remote repo has initial content.

- [ ] **Step 4: Add the submodule to the main repo**

Run:

```bash
git submodule add https://github.com/Bli-AIk/souprune_deltarune_preset.git projects/deltarune_preset
```

Expected: `.gitmodules` and `projects/deltarune_preset` gitlink are staged/modified.

- [ ] **Step 5: Allow the maintained submodule path**

Edit `.gitignore` local project section to include:

```gitignore
!projects/deltarune_preset/
```

- [ ] **Step 6: Commit submodule wiring**

Run:

```bash
git add .gitmodules .gitignore projects/deltarune_preset
git commit -m "feat: add deltarune preset submodule"
```

Expected: commit contains only submodule wiring and `.gitignore`.

## Task 3: Rename and Reduce Deltarune Preset Content

**Files:**
- Modify in submodule: `projects/deltarune_preset/mod.toml`
- Modify in submodule: `projects/deltarune_preset/readme.md`
- Modify in submodule: `projects/deltarune_preset/readme_zh-hans.md`
- Modify in submodule: `projects/deltarune_preset/content/src/lib.rs`
- Modify in submodule: `projects/deltarune_preset/content/Cargo.toml`
- Modify in submodule: `projects/deltarune_preset/content/Cargo.lock`
- Remove or ignore in submodule: UT-only battle content not needed for this stage.

- [ ] **Step 1: Rename mod metadata**

Set `projects/deltarune_preset/mod.toml` to use:

```toml
name = "deltarune_preset"
version = "0.1.0"
authors = ["SoupRune Contributors"]
description = "Deltarune-style game preset for SoupRune — overworld menu views, FRE rules, and party data helpers."
entry_point = false
```

Keep resource and content library sections. Remove the runtime library section unless a runtime crate is retained.

- [ ] **Step 2: Rename content crate docs**

Set `projects/deltarune_preset/content/src/lib.rs` module docs to:

```rust
//! Cauld-ron content guest for `deltarune_preset`.
//!
//! `deltarune_preset` 的 Cauld-ron 内容 guest。
```

- [ ] **Step 3: Remove UT-only generated content**

Delete or stop registering UT battle files and UT backpack files that are not needed for DR overworld menu. Keep only shared app/input/flow, basic player behavior if needed, DR menu rules, DR menu view, and minimal support modules required by the content generator.

- [ ] **Step 4: Commit submodule metadata reduction**

Run inside `projects/deltarune_preset`:

```bash
git add .
git commit -m "feat: rename preset for deltarune"
```

Expected: submodule commit contains rename/reduction changes.

## Task 4: Implement DR Overworld Menu State, Facts, and View

**Files:**
- Create in submodule: `projects/deltarune_preset/overworld/rules/dark_menu.fre.ron`
- Create in submodule: `projects/deltarune_preset/overworld/view/dark_menu.view.ron`
- Create in submodule: matching generated source modules under `projects/deltarune_preset/content/src/overworld/...`
- Modify in submodule: `projects/deltarune_preset/app/global.fre.ron`
- Modify in submodule: `projects/deltarune_preset/mod.toml`

- [ ] **Step 1: Define menu facts**

Use facts with DR-specific names:

```text
dr.menu.open
dr.menu.layer
dr.menu.top_index
dr.menu.category_index
dr.menu.item_cursor
dr.menu.storage_cursor
dr.menu.key_item_cursor
dr.party.count
dr.party.0.name
dr.party.0.hp
dr.party.0.max_hp
dr.inventory.count
dr.storage.count
dr.key_items.count
dr.money
```

- [ ] **Step 2: Implement FRE interactions**

`dark_menu.fre.ron` must support:

- Open menu into top menu.
- Cancel closes or goes one layer back.
- Confirm enters ITEM category or selected list.
- Left/right cycles top/category/list column.
- Up/down moves list cursor by two entries.
- Confirm on key item records unusable feedback instead of executing an item.

- [ ] **Step 3: Implement DR coordinate view**

`dark_menu.view.ron` must express the menu as a 640x480 layout with left-top DR coordinates. Include top buttons, money text, three party boxes at x chunks `0`, `212`, `424`, category selector, and two-column lists.

- [ ] **Step 4: Commit DR menu implementation**

Run inside `projects/deltarune_preset`:

```bash
git add .
git commit -m "feat: add dark world overworld menu"
```

Expected: submodule commit builds its content crate.

## Task 5: Create Local Smoke Fixture

**Files:**
- Create ignored: `projects/deltarune_smoke_test/`

- [ ] **Step 1: Copy minimal example**

Run:

```bash
rsync -a --exclude .git --exclude target --exclude .build projects/mad_dummy_example/ projects/deltarune_smoke_test/
```

- [ ] **Step 2: Change dependency**

Set `projects/deltarune_smoke_test/mod.toml` dependencies to:

```toml
[dependencies]
deltarune_preset = "0.1.0"
```

- [ ] **Step 3: Seed DR facts**

Update the fixture entry sequence or FRE rules so smoke data includes Kris, Susie, Ralsei, sample ITEM/STORAGE/KEYITEM entries, and a money value.

- [ ] **Step 4: Verify fixture remains ignored**

Run:

```bash
git status --short --ignored projects/deltarune_smoke_test
```

Expected: fixture appears only as ignored files.

## Task 6: Verification and Final Commit

**Files:**
- Main repo submodule pointer for `projects/deltarune_preset`
- Any main repo docs/config touched during implementation.

- [ ] **Step 1: Build content crate**

Run:

```bash
cargo build
```

Expected: exits 0.

- [ ] **Step 2: Format**

Run:

```bash
cargo fmt --all
```

Expected: exits 0.

- [ ] **Step 3: Clippy**

Run:

```bash
cargo clippy --workspace --all-targets -D warnings
```

Expected: exits 0.

- [ ] **Step 4: Verify submodule**

Run:

```bash
git submodule status projects/deltarune_preset
git -C projects/deltarune_preset log --oneline -3
```

Expected: main repo points at latest pushed submodule commit.

- [ ] **Step 5: Commit main repo pointer**

Run:

```bash
git add projects/deltarune_preset
git commit -m "feat: implement deltarune overworld backpack preset"
```

Expected: main repo commit updates submodule pointer after verified implementation.

