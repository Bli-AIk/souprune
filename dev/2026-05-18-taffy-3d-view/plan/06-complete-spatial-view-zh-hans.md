# 阶段 6：完整空间 View 运行时基础

> **给执行 agent：**按 Superpowers 流程执行本计划时，使用 `superpowers:subagent-driven-development`。阶段分支为 `feat/view-spatial-06-complete`，子 agent 禁止执行任何 git 操作。

**目标：**把 Stage 5 的 3D 平面挂载扩展为可长期使用的空间 View runtime：支持空间锚点、朝向策略、每帧同步、深度策略声明、射线命中和 2D/3D View 共存边界。

**架构：**View 仍是唯一作者侧界面能力。Taffy 继续产出二维局部布局；空间层只负责把 root 平面挂到世界、相机或命名实体锚点，并把 pointer ray 转换回平面内的布局坐标。Stage 6 不引入项目专用菜单语义，也不引入第二套 View API。

**Tech Stack：**Rust、Bevy 0.18、Taffy、RON/schema、View runtime、Camera3d、Ray3d、Gizmos、direct `.view.ron` example。

---

## 当前口径

- 3D 是 View 的附加空间承载能力；2D camera-relative 和 2D world-space 仍保持默认路径。
- `World3dPlane` 是 root 级空间边界；子树仍在局部二维平面上排布。
- 输入命中只负责把鼠标/触摸 ray 投到空间平面，并记录命中组件/局部坐标；具体菜单选择行为继续交给 View/FRE/LocalState 后续组合。
- 命名实体锚点使用 `Name` 查找。名称是开放标识符，允许字符串；固定策略仍使用 enum，避免 magic string。
- 人工验收继续使用 `view_spatial_3d` 直接读取 `spatial_plane.view.ron`，不依赖 `projects/*` mod。

## 不做

- 不实现项目专用 3D 菜单。
- 不实现任意 3D 子节点布局；每个空间 root 仍是一个局部平面。
- 不做复杂遮挡/物理拾取；本阶段只做数学平面 ray hit。
- 不手改 `projects/**/*.ron`。

## Task 1：空间策略 schema

**Files:**

- Modify: `crates/souprune_schema/src/view.rs`
- Modify: `crates/souprune/src/core/view/layout/view_schema.rs`
- Modify: `crates/souprune/src/core/view/layout.rs`
- Modify: `crates/souprune/examples/assets/view/spatial_plane.view.ron`
- Test: `crates/souprune/src/core/view/layout.rs`

- [x] 新增 `ViewSpatialAnchorDef` enum：`World`、`Named(String)`。
- [x] 新增 `ViewSpatialOrientationDef` enum：`Fixed`、`FaceCamera`、`FaceCameraYaw`。
- [x] 新增 `ViewSpatialDepthDef` enum：`TreeOrder`、`LayoutZ`、`DistanceToCamera`。
- [x] 新增 `ViewSpatialInputDef` enum：`Disabled`、`PlaneRay`。
- [x] 给 `ViewWorld3dPlaneDef` 新增 `anchor`、`orientation`、`depth`、`input` 字段，均有默认值。
- [x] shared/runtime schema 保持同构。
- [x] roundtrip 测试覆盖四个新增策略字段。
- [x] `spatial_plane.view.ron` 使用 `orientation: FaceCameraYaw` 和 `input: PlaneRay`。

## Task 2：空间 root 同步与锚点

**Files:**

- Modify: `crates/souprune/src/core/view/spatial.rs`
- Modify: `crates/souprune/src/core/view/reconcile/system.rs`
- Modify: `crates/souprune/src/core/view/plugin.rs`
- Test: `crates/souprune/src/core/view/spatial.rs`

- [x] 写红测：`Named("Anchor")` 锚点会把平面 transform 合成到命名实体世界 transform 下。
- [x] 写红测：`FaceCameraYaw` 只在水平面朝向相机，保持 root 高度。
- [x] 新增 `resolve_spatial_root_transform(plane, anchor, camera)` 纯函数。
- [x] 新增 `sync_spatial_view_roots_system`，每帧刷新 `World3dPlane` root 的 `Transform` 与 `ViewSpatialRoot`。
- [x] reconcile 不再重复写空间 root transform，只负责在 asset 更新时触发后续同步。
- [x] 如果命名锚点缺失，保留当前 root transform 并 `debug!`，不 panic。

## Task 3：空间平面射线命中

**Files:**

- Create: `crates/souprune/src/core/view/spatial/input.rs`
- Modify: `crates/souprune/src/core/view/spatial.rs`
- Modify: `crates/souprune/src/core/view/plugin.rs`
- Test: `crates/souprune/src/core/view/spatial/input.rs`

- [x] 新增 `ViewSpatialHit` component，包含 `world_position: Vec3`、`plane_position: Vec2`、`layout_position: Vec2`、`distance: f32`。
- [x] 写红测：中心 ray 命中平面后得到 `(layout_x, layout_y)`，并按 `pixels_per_unit` 转换。
- [x] 写红测：命中点在 `plane_size` 外时返回 `None`。
- [x] 实现纯函数 `intersect_spatial_plane(ray, root_transform, plane)`。
- [x] 新增 `update_spatial_view_hits_system`：读取 primary window cursor、active 3D main camera、`ViewSpatialRoot`，给命中的 root 插入 `ViewSpatialHit`，未命中则移除。
- [x] 只处理 `input: PlaneRay` 的空间 root。

## Task 4：example 验收

**Files:**

- Modify: `crates/souprune/examples/view_spatial_3d.rs`
- Modify: `crates/souprune/examples/assets/view/spatial_plane.view.ron`
- Test: `crates/souprune/src/core/view/layout/taffy.rs`

- [x] `view_spatial_3d` 增加一个命名锚点实体，并让 `.view.ron` 的空间平面锚到该实体。
- [x] example 用 gizmos 绘制平面、layout rect、命中点。
- [x] 直接 RON 验收测试断言 `anchor`、`orientation`、`input` 字段可解析。
- [x] `cargo check -p souprune --example view_spatial_3d` 通过。

## Task 5：阶段验证与集成

**Files:**

- Modify: `dev/2026-05-18-taffy-3d-view/plan/06-complete-spatial-view-zh-hans.md`

- [x] `cargo fmt --all`
- [x] `cargo test -p souprune view::spatial --lib`
- [x] `cargo test -p souprune view::layout::taffy::tests::spatial_plane_view_asset_parses_and_solves --lib`
- [x] `cargo check -p souprune --example view_spatial_3d`
- [x] `cargo test -p souprune`
- [x] `cargo clippy --workspace --all-targets -- -D warnings`
- [x] `git diff --check`
- [x] 阶段分支提交后合并回 `feat/taffy_view`，只推送 `feat/taffy_view`。

## 人工验收重点

- `view_spatial_3d` 直接读取 `crates/souprune/examples/assets/view/spatial_plane.view.ron`。
- View 平面跟随命名锚点移动。
- `FaceCameraYaw` 下平面朝向 3D 相机，但不改变锚点高度。
- 鼠标移入平面时显示命中点；移出平面后命中点消失。
- 2D `view_taffy_layout` 路径不受空间同步和射线命中系统影响。

## 风险

- 当前可视验收仍使用 gizmos，因为现有 Bevy feature set 没有启用 PBR/light；Stage 7 再决定长期 3D 渲染后端。
- `Named(String)` 锚点允许开放标识符；这是实体名称引用，不属于固定选项 magic string。
- 射线命中只选择 root 平面，不在本阶段解析子节点 hover/press；后续可用 `ViewLayoutRect` 在平面局部坐标内做二次命中。
