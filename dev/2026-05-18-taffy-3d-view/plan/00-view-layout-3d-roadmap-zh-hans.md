# View Taffy 布局与 3D 空间扩展路线图

> **给执行 agent：**按 Superpowers 流程执行本计划时，优先使用 `superpowers:subagent-driven-development`，否则使用 `superpowers:executing-plans`。执行时按 checkbox 推进。本文是总路线图，后续每个阶段都应拆出独立阶段计划。

**目标：**把 Taffy 引入现有 View 系统，让 `.view.ron` 支持真实的盒模型与高级布局，并渐进扩展到 3D 视角/透视空间中的 View 表达能力。

**架构方向：**View 是 SoupRune 的视觉界面系统，不另造一套系统，也不把 Bevy UI 作为作者侧语义。Taffy 只作为 View 内部布局求解器；schema、content crate、RON、运行时实体树、FRE/View 交互、reconcile 仍属于现有 View 边界。3D View 是附加能力：先让二维布局结果可以挂到 3D 平面，最终支持带透视、锚点、朝向和局部平面布局的空间 View。

**技术栈：**Rust、Bevy、Taffy、RON/schema、Cauld-ron 内容生成、View runtime、FRE/LocalState、BRP/screenshot/manual example 验收。

---

## 当前口径

- 不使用“UI”命名 SoupRune 的作者侧能力；只在说明 Bevy crate 或外部参照时使用 `bevy_ui` / Kristal UI。
- View 是传统界面能力的超集：可以做 HUD、菜单、对话框、战斗面板、世界空间标记、以及未来的 3D 透视界面。
- 第一阶段可以小，但必须建立正确方向：现有 `StyleDef` 不是空字段，Taffy 计算结果要实际影响 View 实体。
- 最终阶段要形成完整 3D View 支持；不过 3D 不是主力路径，2D/屏幕空间 View 仍是默认和主要用例。
- 人工验收不依赖现有 mod。每个 breaking 阶段至少提供一个读取 `.view.ron` 的 example，用专门测试资产验证新能力。

## 执行分支纪律

- 长期开发基线分支为 `feat/taffy_view`，基于当前 `refactor/framework-rearchitecture` 创建。
- 后续所有本专题更新都必须从 `feat/taffy_view` 开始。
- 每个阶段创建独立阶段分支，例如 `feat/view-taffy-layout-01-minimal`。
- 每个阶段用若干子 agent 并行实现，但子 agent 只允许修改文件和报告结果，禁止执行任何 git 操作。
- 主 agent 负责阶段分支创建、集成、验证、提交、合并和推送。
- 阶段完成后，阶段分支保留在本地，不删除。
- 阶段完成后只把阶段分支合并回 `feat/taffy_view`。
- 只有 `feat/taffy_view` 可以推送到远端；阶段分支禁止推送。
- 所有阶段完成前，不切回其他长期开发分支执行本专题更新。

## 参考约束

- Kristal Shadow 文档把界面表达建立在 component tree、sizing、margin/padding、layout、overflow、focus stack、菜单组件上。SoupRune 不复制 Lua API，但采用“树状组件 + 布局 + 输入焦点”的能力目标。
- Taffy 官方 crate 提供 Flexbox、CSS Grid、Block 等布局算法，以及 `TaffyTree` / lower-level tree integration。SoupRune 第一批只接 Flexbox 语义，Grid/Block 留到后续阶段。
- 当前 SoupRune 已有 `ViewLayoutAsset`、`ViewNodeDef.children`、`StyleDef`、`world_space`、`coordinate_space`、reconcile、FRE local state 和动态 transform；计划应复用这些入口。
- 当前 `StyleDef` 已声明 `width/height/left/right/top/bottom/position_type/flex_direction/justify_content/align_items`，但 spawn 路径没有布局求解。阶段 1 应先补齐最小闭环，而不是新增一套作者面对的并行 schema。
- 当前 camera-relative View 只查询 `Camera2d`，3D 阶段必须拆出更通用的 View anchor/camera target 选择。

## 不做

- 不引入新的作者侧“UI 系统”或 `Ui*` public API。
- 不把 View 节点改成 Bevy `Node` 作为长期作者语义。
- 不让框架 core 写入项目专用菜单语义；菜单行为仍通过 schema/FRE/WASM/runtime composition 表达。
- 不手改 `projects/*/*.ron` 生成产物。示例资产如果需要 RON，应由 example fixture 或专门 content 源生成；阶段计划中必须明确来源。
- 不为历史语法保留兼容层。若字段语义需要改变，走完整 deprecation 清理阶段，不保留旧别名。

## 总体阶段拆分

1. 阶段 1：Taffy 最小闭环。让 View `style` 的尺寸、间距、flex 方向、gap、主轴/交叉轴对齐、相对/绝对定位生效，并提供独立 example 人工验收。
2. 阶段 2：盒模型与内容测量。补齐 margin/padding/border、Fit/Fill/Fixed 等作者友好的 sizing 表达，接入文本/精灵/view_box 的测量。
3. 阶段 3：动态布局与 reconcile。fact、repeat、热重载、窗口/相机尺寸变化后重新求解布局，并把布局结果纳入 desired/current tree。
4. 阶段 4：Overflow、scroll、焦点栈。支持 visible/hidden/scroll、裁剪边界、滚动输入、焦点恢复和菜单导航基础能力。
5. 阶段 5：3D 平面 View。二维 Taffy 布局结果挂到 3D 世界中的平面、billboard 或实体锚点；支持透视相机人工验收。
6. 阶段 6：完整空间 View。支持 3D View root、空间锚点、朝向策略、局部平面、深度排序、射线输入和 2D/3D 混合嵌套边界。
7. 阶段 7：收口与文档。删除临时桥、补齐 schema/SDK/cauld-ron 文档与示例，固定长期验收矩阵。

## 文件职责地图

- `crates/souprune_schema/src/view.rs`：作者侧 View schema 的权威结构；新增布局 enum、长度单位、边距/填充、overflow、anchor/space 配置都先落在这里。
- `crates/souprune/src/core/view/layout/view_schema.rs`：运行时 asset schema；必须与 shared schema 保持同构，避免 schema 和 runtime 双轨漂移。
- `crates/souprune/src/core/view/layout/serde_types.rs`：长度、尺寸、rect、alignment 等 serde helper 的运行时表示。
- `crates/souprune/src/core/view/layout/taffy.rs`：新增。把 `ViewLayoutAsset` / `ViewNodeDef` 转换为 Taffy tree，执行求解，再产出 View layout slots。
- `crates/souprune/src/core/view/layout/measure.rs`：新增。集中处理文本、精灵、view_box、容器的内容测量。
- `crates/souprune/src/core/view/layout/slots.rs`：新增。存储求解后的节点 rect、content rect、clip rect、z/layer 以及节点 key。
- `crates/souprune/src/core/view/ron_view/spawn_nodes.rs`：消费 layout slots，生成实体时把 Taffy 输出合成到 `Transform`，不再只读显式 transform。
- `crates/souprune/src/core/view/reconcile/compute.rs`：动态布局阶段把布局结果纳入 `DesiredElement`。
- `crates/souprune/src/core/view/reconcile/tree.rs`：保存 desired/current layout rect，支持 diff layout 变化。
- `crates/souprune/src/core/view/reconcile/delta.rs`：应用布局变化产生的 transform/clip/scroll 更新。
- `crates/souprune/src/core/view/input.rs`：后续 overflow/scroll/focus 阶段消费输入事务，保持 View 内部焦点栈。
- `crates/souprune/src/core/view/components/*.rs`：新增或拆分 `ViewLayoutNode`、`ViewLayoutRect`、`ViewClipRect`、`ViewFocusScope`、`ViewSpatialRoot` 等组件。
- `crates/souprune/src/core/view/spatial.rs`：新增。3D root、相机/实体锚点、billboard、平面 basis、射线拾取和空间 transform 合成。
- `crates/souprune/examples/view_taffy_layout.rs`：新增第一阶段人工验收 example，直接读取 `.view.ron`。
- `crates/souprune/examples/view_spatial_3d.rs`：新增 3D 阶段人工验收 example，直接读取 `.view.ron`。
- `crates/souprune/examples/assets/view/*.view.ron`：example 专用 View 资产；不依赖现有 mod。
- `crates/souprune/tests/view_layout_schema.rs`：schema/RON 解析与共享 schema 转运行时 asset 的回归测试。
- `crates/souprune/tests/architecture_boundaries.rs`：新增边界检查，禁止出现新的作者侧 `ui` 模块命名或 View 外第二套界面系统。

---

## 阶段 1：Taffy 最小闭环

分支建议：`feat/view-taffy-layout-01-minimal`

目标：让 `.view.ron` 中已有/最小新增的布局字段通过 Taffy 求解，并实际影响 View 实体位置。此阶段只保证固定尺寸容器、子节点、flex row/column、gap、alignment、absolute/relative positioning 的闭环。

不做：

- 不做 overflow/scroll。
- 不做 3D。
- 不做 text/sprite 精确内容测量；没有显式尺寸的可视节点先要求提供尺寸或使用保守测量。
- 不迁移现有 mod。

**Files:**

- Modify: `crates/souprune/Cargo.toml`
- Modify: `crates/souprune_schema/src/view.rs`
- Modify: `crates/souprune/src/core/view/layout/view_schema.rs`
- Modify: `crates/souprune/src/core/view/layout.rs`
- Create: `crates/souprune/src/core/view/layout/taffy.rs`
- Create: `crates/souprune/src/core/view/layout/slots.rs`
- Modify: `crates/souprune/src/core/view/ron_view/spawn_nodes.rs`
- Create: `crates/souprune/examples/view_taffy_layout.rs`
- Create: `crates/souprune/examples/assets/view/taffy_minimal.view.ron`
- Test: `crates/souprune/src/core/view/layout/taffy.rs`
- Test: `crates/souprune/tests/view_layout_schema.rs`

任务：

- [ ] 加入 `taffy` 依赖，版本在阶段执行时按当前 Bevy/Rust 约束选择，并记录选择理由。
- [ ] 扩展 `StyleDef`：补齐 `margin`、`padding`、`gap`、`align_self`、`display`，全部使用 enum/struct，不使用 magic string。
- [ ] 实现 schema -> runtime 转换测试，覆盖新增字段。
- [ ] 新增 `ViewLayoutSlot` / `ViewLayoutSlots`，用稳定节点路径或 View element key 关联求解结果。
- [ ] 实现 `compute_taffy_layout(asset, viewport_size, measure_context)`，先只支持静态长度和 Percent。
- [ ] 在 spawn 时先求解 root slots，再把 slot 的 `x/y` 合成到节点 `Transform.translation`；显式 transform 作为 layout 后的局部偏移。
- [ ] 新增 example：`cargo run -p souprune --example view_taffy_layout --features "debug,bevy/dynamic_linking"` 能读取 `taffy_minimal.view.ron` 并显示三类布局：居中、横向按钮列、绝对子节点。
- [ ] example 按键或事实切换不要求动态重排；人工只验收静态布局。

验收：

- [ ] 运行 `cargo fmt --all`。
- [ ] 运行 `cargo test -p souprune view::layout::taffy`。
- [ ] 运行 `cargo test -p souprune --test view_layout_schema`。
- [ ] 运行 `cargo clippy --workspace --all-targets -D warnings`。
- [ ] 人工运行 `view_taffy_layout` example，确认 example 直接读取 `.view.ron`，不依赖 `projects/*` mod。
- [ ] 人工截图或 BRP 检查：三组节点的 transform 与预期布局位置一致。

人工测试重点：

- Root 为 640x480 时，居中元素位于画面中心。
- Row 容器的子节点间距一致，`SpaceBetween` / `Center` 可见。
- Absolute 子节点不参与兄弟 flex 排布，但仍相对父容器定位。
- 显式 transform 仍作为 layout 后偏移，不覆盖 layout 结果。

---

## 阶段 2：盒模型与内容测量

分支建议：`feat/view-taffy-layout-02-measure`

目标：把 View 的作者侧布局表达提升到可长期使用的盒模型。支持 Fixed/Fill/Fit sizing、margin/padding/border、文本/精灵/view_box 的基础测量。

不做：

- 不做 scroll。
- 不做 3D。
- 不要求 Grid/Block。

**Files:**

- Modify: `crates/souprune_schema/src/view.rs`
- Modify: `crates/souprune/src/core/view/layout/view_schema.rs`
- Modify: `crates/souprune/src/core/view/layout/taffy.rs`
- Create: `crates/souprune/src/core/view/layout/measure.rs`
- Modify: `crates/souprune/src/core/view/ron_view/spawn_helpers.rs`
- Modify: `crates/souprune/src/core/view/text.rs`
- Modify: `crates/souprune/examples/assets/view/taffy_minimal.view.ron`
- Test: `crates/souprune/src/core/view/layout/measure.rs`

任务：

- [ ] 新增 `ViewSizingDef` enum：`Fixed { width, height }`、`Fill`、`Fit`、以及必要的 per-axis 组合，避免布尔标志。
- [ ] 将 `SerializableVal::Auto` 映射到 Taffy auto，`Percent` 映射到 percent，`Px` 映射到 length；`Vw/Vh` 使用 viewport size 转换。
- [ ] 实现文本测量：读取字体/字号/line height/spacing 的保守 bounds，不能测量时返回明确 fallback 并打 debug log。
- [ ] 实现精灵测量：读取 image size 或 animation frame size；无法同步得到 asset size 时允许下一帧重排。
- [ ] 实现 `view_box` 测量：使用 `ViewBoxLogicDef.width/height`。
- [ ] 增加单测：Fit 容器会包住文本/精灵/view_box 的测量尺寸。
- [ ] 扩展 example：加入 Fit box、Fill row、带 padding 的菜单面板。

验收：

- [ ] 运行 `cargo fmt --all`。
- [ ] 运行 `cargo test -p souprune view::layout::measure`。
- [ ] 运行 `cargo clippy --workspace --all-targets -D warnings`。
- [ ] 人工运行 `view_taffy_layout` example，确认 Fit/Fill/padding/margin 在 `.view.ron` 修改后立即可观察。

人工测试重点：

- 文本容器不需要手写固定高度也能撑开。
- 填充容器扣除父 padding 和自身 margin。
- 精灵尺寸参与排布时，不挤压相邻节点。

---

## 阶段 3：动态布局与 reconcile

分支建议：`feat/view-taffy-layout-03-dynamic`

目标：让 layout 进入 View 的动态更新链路。repeat 数量、facts、窗口尺寸、相机尺寸、热重载、资源测量变化都能触发布局重算，并通过 reconcile 最小更新实体。

不做：

- 不做 scroll/focus。
- 不做 3D。
- 不改 FRE 语义。

**Files:**

- Modify: `crates/souprune/src/core/view/reconcile/compute.rs`
- Modify: `crates/souprune/src/core/view/reconcile/tree.rs`
- Modify: `crates/souprune/src/core/view/reconcile/diff.rs`
- Modify: `crates/souprune/src/core/view/reconcile/delta.rs`
- Modify: `crates/souprune/src/core/view/reconcile/system.rs`
- Modify: `crates/souprune/src/core/view/ron_view/reload.rs`
- Modify: `crates/souprune/src/core/view/plugin.rs`
- Test: `crates/souprune/src/core/view/reconcile/diff.rs`

任务：

- [ ] `DesiredElement` 保存 layout rect 和 transform 的合成结果。
- [ ] `CurrentViewTree` 读取实体上的 `ViewLayoutRect`。
- [ ] diff 检测 rect 变化并生成 `UpdateLayout` 或 transform update。
- [ ] fact/repeat 变化触发对应 View asset 的 pending reconciliation。
- [ ] 窗口或相机 visible size 变化触发 camera-relative View 重新布局。
- [ ] asset hot reload 后重建 Taffy tree 并保留 ViewRoot local state。
- [ ] 增加单测：repeat 数组长度变化后，desired tree 节点数量和 layout rect 更新。
- [ ] example 加入按键或 FRE fact 修改节点数量/文本长度，人工观察动态重排。

验收：

- [ ] 运行 `cargo fmt --all`。
- [ ] 运行 `cargo test -p souprune view::reconcile`。
- [ ] 运行 `cargo clippy --workspace --all-targets -D warnings`。
- [ ] 人工运行 `view_taffy_layout` example，修改 fact 后布局重排且没有整棵树闪烁重建。

人工测试重点：

- repeat 列表增加/减少后，容器重新居中或重新分布。
- 文本变长后 Fit 容器变宽，兄弟节点位置更新。
- 热重载 `.view.ron` 后布局更新，View local facts 不丢失。

---

## 阶段 4：Overflow、scroll 与焦点栈

分支建议：`feat/view-taffy-layout-04-overflow-focus`

目标：补齐高级 View 的交互基础。支持 overflow visible/hidden/scroll，滚动区域、裁剪边界、焦点栈和菜单导航。

不做：

- 不实现项目专用菜单行为。
- 不做 text input 的完整编辑体验；只保留 schema 扩展点。
- 不做 3D 射线输入。

**Files:**

- Modify: `crates/souprune_schema/src/view.rs`
- Modify: `crates/souprune/src/core/view/layout/view_schema.rs`
- Modify: `crates/souprune/src/core/view/components/view_element.rs`
- Create: `crates/souprune/src/core/view/focus.rs`
- Create: `crates/souprune/src/core/view/overflow.rs`
- Modify: `crates/souprune/src/core/view/input.rs`
- Modify: `crates/souprune/src/core/view/plugin.rs`
- Modify: `crates/souprune/examples/view_taffy_layout.rs`
- Test: `crates/souprune/src/core/view/focus.rs`
- Test: `crates/souprune/src/core/view/overflow.rs`

任务：

- [ ] 新增 `ViewOverflowDef` enum：`Visible`、`Hidden`、`Scroll { axis, mode }`。
- [ ] 新增 `ViewFocusPolicyDef` enum：`None`、`Focusable`、`FocusScope`、`MenuList`。
- [ ] 为 hidden/scroll 区域生成 `ViewClipRect`，用于可视裁剪和输入命中测试。
- [ ] scroll 区域维护 `ViewScrollState`，输入事务只作用于当前 focus scope。
- [ ] 实现 `ViewFocusStack` resource：push/pop/top，View despawn 时恢复上一个 focus。
- [ ] 菜单列表只提供通用选择索引、confirm/cancel 事件或 FRE action 触发，不写项目语义。
- [ ] example 加入滚动菜单：上下移动、confirm 显示选中项、cancel 恢复上层 focus。

验收：

- [ ] 运行 `cargo fmt --all`。
- [ ] 运行 `cargo test -p souprune view::focus view::overflow`。
- [ ] 运行 `cargo clippy --workspace --all-targets -D warnings`。
- [ ] 人工运行 `view_taffy_layout` example，确认 scroll/hidden/focus stack 工作。

人工测试重点：

- hidden 区域外的子节点不可见且不可被输入命中。
- scroll 区域可以滚动，滚动不改变子节点自己的逻辑 layout。
- 子菜单关闭后，父菜单自动恢复 focus。

---

## 阶段 5：3D 平面 View

分支建议：`feat/view-spatial-05-plane`

目标：让二维 Taffy layout 可以被放置到 3D 空间中。此阶段 3D 是“二维布局结果 + 3D 平面挂载”，满足透视视角制作需求的第一批可用能力。

不做：

- 不做任意 3D 子树布局。
- 不做射线输入。
- 不要求所有现有 View 自动支持 3D；只支持声明为 spatial 的 View root。

**Files:**

- Modify: `crates/souprune_schema/src/view.rs`
- Modify: `crates/souprune/src/core/view/layout/view_schema.rs`
- Create: `crates/souprune/src/core/view/spatial.rs`
- Modify: `crates/souprune/src/core/view/ron_view/spawn.rs`
- Modify: `crates/souprune/src/core/view/ron_view/spawn_nodes.rs`
- Modify: `crates/souprune/src/core/view/plugin.rs`
- Create: `crates/souprune/examples/view_spatial_3d.rs`
- Create: `crates/souprune/examples/assets/view/spatial_plane.view.ron`
- Test: `crates/souprune/src/core/view/spatial.rs`

任务：

- [ ] 新增 `ViewSpaceDef` enum，替代继续扩张 `world_space: bool` 的语义：`Camera2dRelative`、`World2d`、`World3dPlane`。
- [ ] 在 shared/runtime schema 中保留迁移计划：新资产使用 `space`，旧字段在最终收口阶段删除或转换，不长期兼容。
- [ ] `World3dPlane` 支持位置、rotation、scale、plane size、pixels-per-unit、camera target。
- [ ] `spawn_dynamic_view_system` 拆出 camera target 查询，允许 `Camera3d` + Perspective。
- [ ] layout slots 在平面局部坐标中求解，再由 spatial root basis 转为 3D transform。
- [ ] example 使用 `Camera3d` 透视相机、一个倾斜面板、多个 View 节点，直接读取 `spatial_plane.view.ron`。

验收：

- [ ] 运行 `cargo fmt --all`。
- [ ] 运行 `cargo test -p souprune view::spatial`。
- [ ] 运行 `cargo clippy --workspace --all-targets -D warnings`。
- [ ] 人工运行 `view_spatial_3d` example，确认透视相机下 View 平面正确显示。
- [ ] BRP/screenshot 验证画面非空、平面有透视缩放、节点相对平面布局稳定。

人工测试重点：

- 相机移动或旋转时，3D View 平面保持在世界空间位置。
- 平面倾斜后，子节点仍保持二维排布关系。
- 2D example 不受 3D 支持影响。

---

## 阶段 6：完整空间 View

分支建议：`feat/view-spatial-06-full`

目标：完成 3D View 系统的长期形态。支持空间 root、实体锚点、billboard、局部平面嵌套、深度排序、射线输入和 2D/3D 混合边界。

不做：

- 不实现项目专用 3D 菜单语义。
- 不支持真正三维体积内的 CSS-like layout；Taffy 仍只负责每个局部平面的二维排布。

**Files:**

- Modify: `crates/souprune_schema/src/view.rs`
- Modify: `crates/souprune/src/core/view/spatial.rs`
- Create: `crates/souprune/src/core/view/spatial/input.rs`
- Create: `crates/souprune/src/core/view/spatial/anchors.rs`
- Modify: `crates/souprune/src/core/view/input.rs`
- Modify: `crates/souprune/src/core/view/reconcile/compute.rs`
- Modify: `crates/souprune/examples/view_spatial_3d.rs`
- Test: `crates/souprune/src/core/view/spatial/input.rs`
- Test: `crates/souprune/src/core/view/spatial/anchors.rs`

任务：

- [ ] 新增 `ViewSpatialAnchorDef` enum：`WorldTransform`、`EntityTag`、`ViewElement`、`CameraRelative3d`。
- [ ] 新增 `ViewFacingDef` enum：`Fixed`、`BillboardCamera`、`AxisLockedBillboard`。
- [ ] 支持 spatial 子树：子节点可声明新局部平面，父空间 transform 与子平面 transform 合成。
- [ ] 深度排序策略使用 enum：`TreeOrder`、`ZOffset`、`DistanceToCamera`。
- [ ] 射线输入：从指针/触摸/鼠标生成 ray，命中 spatial plane 后转换为 layout local point，再走 existing View hit test。
- [ ] example 加入 billboard label、world anchored panel、ray hover/confirm 反馈。

验收：

- [ ] 运行 `cargo fmt --all`。
- [ ] 运行 `cargo test -p souprune view::spatial`。
- [ ] 运行 `cargo clippy --workspace --all-targets -D warnings`。
- [ ] 人工运行 `view_spatial_3d` example，确认 billboard、world anchor、ray input 可用。

人工测试重点：

- Billboard 节点始终朝向相机，但 layout 不抖动。
- Entity anchor 移动时，View 跟随实体。
- Ray 命中只触发最前方可交互节点。

---

## 阶段 7：收口、文档与边界审计

分支建议：`feat/view-layout-07-finalize`

目标：把阶段性桥接清理干净，固定长期 schema、example、文档、测试和架构边界。

**Files:**

- Modify: `crates/souprune_schema/src/view.rs`
- Modify: `crates/souprune/src/core/view/layout/view_schema.rs`
- Modify: `crates/souprune/src/core/view/layout.rs`
- Modify: `crates/souprune/src/core/view/ron_view/spawn.rs`
- Modify: `crates/souprune/tests/architecture_boundaries.rs`
- Rename: `doc/docs/en/part3_soul_dessert/3.1_ui.md` -> `doc/docs/en/part3_soul_dessert/3.1_view.md`
- Rename: `doc/docs/zh-hans/part3_soul_dessert/3.1_ui.md` -> `doc/docs/zh-hans/part3_soul_dessert/3.1_view.md`
- Modify: `AGENTS.md`
- Modify: `dev/2026-05-18-taffy-3d-view/plan/00-view-layout-3d-roadmap-zh-hans.md`

任务：

- [x] 删除 `world_space` 长期兼容路径，或在 schema 中明确完成一次性迁移后移除。
- [x] 文档重命名或修订：对外称 View，不把 SoupRune View 称为 UI；历史文档文件名可单独阶段处理。
- [x] 增加架构测试：core 不新增 project-specific View 语义，不出现第二套作者侧界面系统。
- [x] 给 Cauld-ron 输出补齐新 enum 模板，确保 mod 作者通过 content crate 生成 RON。
- [x] 汇总 example 验收矩阵：2D layout、dynamic layout、scroll/focus、3D plane、spatial input。
- [x] 删除临时 debug log、fallback、旧别名、未使用组件和仅用于迁移的 helper。

验收：

- [x] 运行 `cargo fmt --all`。
- [x] 运行 `cargo test -p souprune`。
- [x] 运行 `cargo test -p souprune_cauld_ron`。
- [x] 运行 `cargo clippy --workspace --all-targets -- -D warnings`。
- [x] 运行 `cargo check -p souprune --example view_taffy_layout`。
- [x] 运行 `cargo check -p souprune --example view_spatial_3d`。
- [x] 人工确认 docs 与 examples 均使用 View 口径。

---

## 设计原则

- Taffy 是实现细节。作者面对的是 View schema，不是 Taffy API。
- Layout 只计算盒和局部坐标；渲染、材质、文本动画、FRE local state 仍由现有 View 子系统负责。
- 显式 transform 不消失，但语义变为 layout 后局部偏移；需要脱离布局的节点使用 absolute positioning 或 spatial anchor。
- 3D View 的 layout 仍以局部二维平面为基本单位；这能覆盖透视界面需求，同时避免发明三维 CSS。
- 所有固定选项使用 enum，不用 magic string；路径、用户文本、开放 id 仍可使用字符串。
- 每个阶段先做 example 验收资产，再迁移或扩展现有内容；现有 mod 不作为本重构的人工验收入口。

## 风险与应对

- **Taffy 测量需要异步资产尺寸。**先允许 placeholder 测量并在 asset ready 后重排；测试覆盖“尺寸变化触发 reconcile”。
- **`world_space: bool` 语义不足。**已删除旧布尔字段；`space` 是唯一 View 根放置语义。
- **reconcile 当前 spawn delta 对子节点不完整。**动态布局阶段必须先修正 tree/delta 对嵌套 spawn/update 的模型，再把 layout rect 纳入 diff。
- **3D 输入容易扩大战线。**阶段 5 只做显示，阶段 6 才做 ray input。
- **命名容易回到 UI。**文档和 public API 使用 View 命名；边界测试扫描新增 `ui` 作者侧模块名。

## 总体验收矩阵

- 自动：
  - [x] `cargo fmt --all`
  - [x] `cargo test -p souprune view::layout --lib`
  - [x] `cargo test -p souprune view::spatial --lib`
  - [x] `cargo test -p souprune --test view_layout_schema`
  - [x] `cargo test -p souprune --test architecture_boundaries`
  - [x] `cargo test -p souprune`
  - [x] `cargo test -p souprune_cauld_ron`
  - [x] `cargo clippy --workspace --all-targets -- -D warnings`
- 人工：
  - [x] `cargo check -p souprune --example view_taffy_layout`
  - [x] `cargo check -p souprune --example view_spatial_3d`
  - [x] 两个 example 都直接读取 `.view.ron`，不依赖 `projects/*` mod。
  - [x] 2D example 覆盖静态布局、动态重排、overflow/scroll/focus。
  - [x] 3D example 覆盖透视平面、named anchor、orientation、ray input。

## 后续拆分建议

- 阶段 7 已完成收口；后续如要让 `ViewSpatialDepthDef` 驱动真实渲染排序，应单独拆阶段并绑定具体 3D 渲染后端。
