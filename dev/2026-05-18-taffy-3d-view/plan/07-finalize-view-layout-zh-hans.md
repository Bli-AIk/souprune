# 阶段 7：View 布局与空间能力收口

> **给执行 agent：**按 Superpowers 流程执行本计划时，使用 `superpowers:subagent-driven-development`。阶段分支为 `feat/view-layout-07-finalize`，子 agent 禁止执行任何 git 操作。阶段完成后保留本地阶段分支，合并回 `feat/taffy_view`，只推送 `feat/taffy_view`。

**目标：**删除阶段性兼容桥，固定 View/Taffy/空间 View 的长期 schema、example、文档口径与架构边界。

**架构：**View 是 SoupRune 的唯一作者侧视觉布局能力，不再保留 `world_space` 布尔语义。`space: None` 表示默认 camera-relative View，`space: Some(World2d)` 表示二维世界空间 View，`space: Some(World3dPlane(_))` 表示 3D 空间平面 View。`depth` 本阶段只保留为 schema 声明，等待后续渲染后端接入，不宣称已有深度排序行为。

**Tech Stack：**Rust、Serde/RON、Bevy、Taffy、Cauld-ron content source、direct `.view.ron` examples、architecture boundary tests。

---

## 不做

- 不新增第二套作者侧能力或 `ui` public API。
- 不保留 `world_space` 反序列化兼容层。
- 不手改 `projects/**/*.ron` 作为来源；项目生成产物只通过 content 源重新生成。
- 不在本阶段补半套 3D 深度排序渲染行为。

## Task 1：红测锁定收口边界

**Files:**

- Modify: `crates/souprune/tests/architecture_boundaries.rs`

- [x] 新增测试：框架、schema、content 源、examples、cauld-ron fixtures 中不再出现 `world_space`。
- [x] 新增测试：公开 docs 不再使用作者侧 `UI` 口径，历史文件名也不能继续是 `3.1_ui.md`。
- [x] 新增测试：公开作者侧 API 不新增 `ui` 模块。
- [x] 运行 `cargo test -p souprune --test architecture_boundaries view_authoring_surface_uses_space_not_legacy_boolean`，确认失败来自现存 `world_space`。
- [x] 运行 `cargo test -p souprune --test architecture_boundaries public_docs_use_view_terminology_for_authoring_surface`，确认失败来自现存 docs。

## Task 2：删除 `world_space` schema 与 runtime 路径

**Files:**

- Modify: `crates/souprune_schema/src/view.rs`
- Modify: `crates/souprune/src/core/view/layout/view_schema.rs`
- Modify: `crates/souprune/src/core/view/layout.rs`
- Modify: `crates/souprune/src/core/view/layout/taffy.rs`
- Modify: `crates/souprune/src/core/view/camera.rs`
- Modify: `crates/souprune/src/core/view/reconcile/compute.rs`
- Modify: `crates/souprune/src/core/view/reconcile/system.rs`
- Modify: `crates/souprune/src/core/view/ron_view/spawn.rs`

- [x] 从 shared/runtime `ViewLayout` 删除 `world_space` 字段。
- [x] `camera_relative_parent_for_view` 改为：`None` 与 `Camera2dRelative` 使用相机父节点；`World2d` 与 `World3dPlane` 不使用相机父节点。
- [x] `camera_relative_view_offset` 改为：仅 `World2d` 与 `World3dPlane` 返回零偏移；默认 camera-relative 仍应用坐标空间偏移。
- [x] 所有测试和 struct literal 改用 `space`。
- [x] 运行 `cargo test -p souprune view::ron_view::spawn --lib`。

## Task 3：迁移 content 源与 RON fixtures

**Files:**

- Modify: `projects/mad_dummy_example/content/src/battle/view/battle_bg.rs`
- Modify: `projects/mad_dummy_example/content/src/battle/view/mad_dummy.rs`
- Modify: `projects/undertale_preset/content/src/battle/view/undertale.rs`
- Modify: `crates/souprune/examples/assets/view/taffy_minimal.view.ron`
- Modify: `crates/souprune/examples/assets/view/spatial_plane.view.ron`
- Modify: `crates/souprune/examples/view_text_reconstruction/search.rs`
- Modify: `crates/souprune_cauld_ron/tests/semantic_output_preservation.rs`
- Regenerate/update generated files under `projects/**` and `crates/souprune_cauld_ron/tests/fixtures/**` as needed.

- [x] 将原 `world_space: true` content 源改为 `space: Some(ViewSpaceDef::World2d)`。
- [x] 直接 example `.view.ron` 删除 `world_space`，需要世界空间时写 `space`。
- [x] cauld-ron sparse preservation fixture 删除 `world_space`。
- [x] 通过 content crate 重新生成项目 `.view.ron`，不手改项目 RON 来源。
- [x] 运行 `rg -n "world_space" crates/souprune crates/souprune_schema crates/souprune_cauld_ron projects doc/docs AGENTS.md`，确认无命中。

## Task 4：文档与命名边界

**Files:**

- Rename: `doc/docs/en/part3_soul_dessert/3.1_ui.md` -> `doc/docs/en/part3_soul_dessert/3.1_view.md`
- Rename: `doc/docs/zh-hans/part3_soul_dessert/3.1_ui.md` -> `doc/docs/zh-hans/part3_soul_dessert/3.1_view.md`
- Modify: `doc/docs/en/README.md`
- Modify: `doc/docs/zh-hans/README.md`
- Modify: `doc/docs/zh-hans/README.serious.md`
- Modify: `doc/docs/zh-hans/SUMMARY.md`
- Modify: `AGENTS.md`
- Modify: selected internal View comments/log labels if they are project-specific or still say `UI` while meaning SoupRune View.

- [x] docs 中作者侧描述统一为 View / View layout / ViewBox。
- [x] AGENTS.md 增加 View terminology boundary。
- [x] core View runtime 注释从项目名示例改成通用描述。
- [x] 过度详细的 spawn-time `info!` 调试日志降为 `debug!`。
- [x] 运行 `cargo test -p souprune --test architecture_boundaries public_docs_use_view_terminology_for_authoring_surface`。

## Task 5：最终验证与集成

**Files:**

- Modify: `dev/2026-05-18-taffy-3d-view/plan/07-finalize-view-layout-zh-hans.md`
- Modify: `dev/2026-05-18-taffy-3d-view/plan/00-view-layout-3d-roadmap-zh-hans.md`

- [x] 更新路线图阶段 7 与总体验收矩阵状态。
- [x] `cargo fmt --all`
- [x] `cargo test -p souprune --test architecture_boundaries`
- [x] `cargo test -p souprune view::layout --lib`
- [x] `cargo test -p souprune view::spatial --lib`
- [x] `cargo test -p souprune --test view_layout_schema`
- [x] `cargo test -p souprune`
- [x] `cargo test -p souprune_cauld_ron`
- [x] `cargo check -p souprune --example view_taffy_layout`
- [x] `cargo check -p souprune --example view_spatial_3d`
- [x] `cargo clippy --workspace --all-targets -- -D warnings`
- [x] `git diff --check`
- [ ] 阶段分支提交后合并回 `feat/taffy_view`，只推送 `feat/taffy_view`。

## 人工验收矩阵

- `view_taffy_layout` 直接读取 `crates/souprune/examples/assets/view/taffy_minimal.view.ron`，覆盖 flex、spacing、sizing、absolute、display none、repeat、Fit ViewBox、visible/hidden、scroll、focus facts。
- `view_spatial_3d` 直接读取 `crates/souprune/examples/assets/view/spatial_plane.view.ron`，覆盖 `World3dPlane`、named anchor、yaw-facing orientation、plane ray input、layout rect gizmos、hit marker。
- 两个 example 均不依赖 `projects/*` mod。

## 风险

- 删除 `world_space` 会让旧 `.view.ron` 失效，这是预期的 complete deprecation。
- 项目 RON 文件是生成产物，必须由 content 源重新生成，不能把手改项目 RON 当作来源。
- `ViewSpatialDepthDef` 当前只固定 schema，不绑定渲染行为；文档和最终总结不能写成已实现深度排序。
