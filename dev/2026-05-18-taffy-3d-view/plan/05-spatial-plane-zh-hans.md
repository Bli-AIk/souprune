# 阶段 5：3D 平面 View

> **给执行 agent：**按 Superpowers 流程执行本计划时，使用 `superpowers:subagent-driven-development`。阶段分支为 `feat/view-spatial-05-plane`，子 agent 禁止执行任何 git 操作。

**目标：**让二维 Taffy layout 结果可以作为 View 平面挂到 3D 世界中，并用透视相机 example 直接读取 `.view.ron` 验收。

**架构：**View 仍是唯一作者侧界面能力；Taffy 继续只负责局部二维布局。Stage 5 新增空间 root 语义，把布局 slot 的二维坐标映射到一个 3D 平面 basis；2D View 默认路径保持不变。

**Tech Stack：**Rust、Bevy 0.18、Taffy、RON/schema、View runtime、Camera3d/Perspective、example fixture。

---

## 当前口径

- 本阶段只做显示，不做 ray input、hover、3D hit test。
- 本阶段不把 View 子树变成任意三维布局；每个 spatial root 仍是一个局部二维平面。
- 新字段使用 enum/struct，不使用 magic string。
- `world_space: bool` 暂不删除；新增 `space` 字段作为未来替代路径，并在最终收口阶段再移除旧布尔字段。
- 人工验收必须通过 `view_spatial_3d` example 直接读取 `spatial_plane.view.ron`，不依赖 `projects/*` mod。

## 不做

- 不实现项目专用 3D 菜单语义。
- 不实现射线输入。
- 不迁移现有项目内容。
- 不手改 `projects/**/*.ron`。

## Task 1：Schema 与 Spatial Def

**Files:**

- Modify: `crates/souprune_schema/src/view.rs`
- Modify: `crates/souprune/src/core/view/layout/view_schema.rs`
- Modify: `crates/souprune/src/core/view/layout.rs`

- [x] 写 shared schema parse 红测：`ViewLayoutAsset.space: Some(World3dPlane(...))` 可从 RON 解析。
- [x] 写 runtime roundtrip 红测：shared schema `World3dPlane` 可转换为 runtime `ViewLayoutAsset`。
- [x] 新增 `ViewSpaceDef` enum：`Camera2dRelative`、`World2d`、`World3dPlane(ViewWorld3dPlaneDef)`。
- [x] 新增 `ViewWorld3dPlaneDef` struct，字段：
  - `transform: SerializableTransform`
  - `rotation_degrees: Option<SerializableVec3>`
  - `plane_size: (f32, f32)`
  - `pixels_per_unit: f32`
  - `camera: ViewCameraTargetDef`
- [x] 新增 `ViewCameraTargetDef` enum：`Main`、`Named(String)`。
- [x] runtime schema 与 shared schema 字段保持同构。
- [x] `ViewLayoutAsset` 新增 `space: Option<ViewSpaceDef>`，不删除 `world_space`。

## Task 2：Spatial Transform 求解

**Files:**

- Create: `crates/souprune/src/core/view/spatial.rs`
- Modify: `crates/souprune/src/core/view.rs` 或对应 View module export 文件

- [x] 写红测：`World3dPlane` 中 `pixels_per_unit = 100` 时，slot `(x=100,y=50)` 映射到平面局部 `(1.0,-0.5,0.0)`。
- [x] 写红测：平面 `transform.translation` 会成为 root 世界位置，layout offset 只作为子节点局部偏移。
- [x] 新增 `ViewSpatialRoot` component，保存 `ViewWorld3dPlaneDef` 的 runtime 结果。
- [x] 新增纯函数 `layout_slot_to_plane_translation(slot, plane)`，把 pixel 坐标转换为 Bevy 3D 坐标。
- [x] 新增纯函数 `spatial_root_transform(plane)`，从 schema transform 生成 Bevy `Transform`。
- [x] 不在此任务接入 spawn，先保证纯函数可测。

## Task 3：Spawn 路径接入 Camera3d

**Files:**

- Modify: `crates/souprune/src/core/view/ron_view/spawn.rs`
- Modify: `crates/souprune/src/core/view/ron_view/spawn_nodes.rs`
- Modify: `crates/souprune/src/core/view/plugin.rs`
- Modify: `crates/souprune/src/core/view/spatial.rs`

- [x] 写红测：`layout_uses_3d_plane_space` 返回 true 时不要求 `Camera2d`。
- [x] 拆出 camera target 查询：2D path 继续查询 `Camera2d`，3D path 查询 `Camera3d` 且允许 perspective。
- [x] spatial root 生成时插入 `ViewSpatialRoot`，并使用 `World3dPlane.transform` 与 `rotation_degrees` 作为 root `Transform`。
- [x] spatial View 生成子节点时，使用 `layout_slot_to_plane_translation` 合成局部 Transform。
- [x] reconcile 路径复用相同 spatial plane 映射，避免 hot reload/fact 更新后从世界单位退回 raw pixels。
- [x] reconcile 路径刷新 spatial root `Transform` 与 `ViewSpatialRoot`。
- [x] 2D path 的 `Camera2dRelative`、`World2d`、旧 `world_space` 行为保持现有测试通过。
- [x] 为没有匹配 camera target 的 3D View 输出 `warn!` 并跳过生成，不 panic。

## Task 4：Example 与 RON 验收

**Files:**

- Create: `crates/souprune/examples/view_spatial_3d.rs`
- Create: `crates/souprune/examples/assets/view/spatial_plane.view.ron`
- Modify: `crates/souprune/src/core/view/layout/taffy.rs`

- [x] 新增 `spatial_plane.view.ron`，包含一个 `space: Some(World3dPlane(...))` root，至少三个节点：panel、row item、absolute marker。
- [x] 新增 `view_spatial_3d` example：启动 `Camera3d` 透视相机，并通过 `SpawnViewRequest` 读取 `view/spatial_plane.view.ron`。当前 Bevy feature set 未启用 PBR/light，示例使用 3D gizmos 绘制实际生成的 View layout rect 与参考平面/轴标记，避免依赖 2D sprite render phase。
- [x] example 不显示使用说明文本；画面第一视口直接呈现 3D View 平面。
- [x] 在 `taffy.rs` 的直接 RON 验收测试中读取 `spatial_plane.view.ron`，验证可解析并求解 layout slots。
- [x] `cargo check -p souprune --example view_spatial_3d` 通过。

## Task 5：阶段验证与集成

**Files:**

- Modify: `dev/2026-05-18-taffy-3d-view/plan/05-spatial-plane-zh-hans.md`

- [x] `cargo fmt --all`
- [x] `cargo test -p souprune view::spatial --lib`
- [x] `cargo test -p souprune view::layout::taffy::tests::spatial_plane_view_asset_parses_and_solves --lib`
- [x] `cargo check -p souprune --example view_spatial_3d`
- [x] `cargo test -p souprune`
- [x] `cargo clippy --workspace --all-targets -- -D warnings`
- [x] `git diff --check`
- [ ] 阶段分支提交后合并回 `feat/taffy_view`，只推送 `feat/taffy_view`。

## 人工验收重点

- `view_spatial_3d` 直接读取 `crates/souprune/examples/assets/view/spatial_plane.view.ron`。
- 透视相机下，View 平面保持世界空间位置。
- 平面倾斜后，子节点保持二维排布关系。
- `view_taffy_layout` 2D example 不受 3D 支持影响。

## 风险

- Bevy sprite/text 在 3D camera 下的渲染路径可能需要材质或 mesh 适配；本阶段优先把 transform/空间挂载打通，若 Sprite2d 在 Camera3d 下不可见，则 example 使用 ViewBox/mesh-backed probe 做验收。
- `world_space` 与新 `space` 会短期共存；Stage 7 负责删除长期兼容路径。
- Stage 5 不做 ray input，因此 focus/input 仍按 Stage 4 规则运行。
