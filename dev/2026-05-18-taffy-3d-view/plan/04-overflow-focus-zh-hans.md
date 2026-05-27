# 阶段 4：Overflow、Scroll 与 Focus Stack

## 当前口径

- View 是作者侧界面能力；本阶段不新增独立 UI 系统，也不使用 Bevy UI 作为作者语义。
- 本阶段目标是把 overflow metadata、scroll state 和焦点栈接入现有 View runtime，形成可测试的基础能力。
- 3D View 不在本阶段实现；后续阶段再处理空间锚点和射线输入。
- 人工验收继续使用 `view_taffy_layout` example，直接读取 `.view.ron`，不依赖 `projects/*` mod。

## 不做

- 不实现项目专用菜单行为。
- 不实现 text input 的完整编辑体验。
- 不实现 3D ray input。
- 不手改 `projects/**/*.ron` 生成产物。

## Task 1：Schema 与 Layout Metadata

**Files:**

- Modify: `crates/souprune_schema/src/view.rs`
- Modify: `crates/souprune/src/core/view/layout/view_schema.rs`
- Modify: `crates/souprune/src/core/view/layout/slots.rs`
- Modify: `crates/souprune/src/core/view/layout/taffy.rs`
- Modify: `crates/souprune/src/core/view/layout.rs`

- [x] 新增 `ViewOverflowDef`、`ViewOverflowAxisDef`；`Scroll` 轴直接生成 `ViewScrollState`，不另加独立 scroll mode enum。
- [x] 新增 `ViewFocusPolicyDef`，挂在 `ViewNodeDef`，不放入 `StyleDef`。
- [x] `StyleDef` 新增 `overflow: Option<ViewOverflowDef>`。
- [x] `ViewLayoutSlots` 产出可按 path 查询的 `ViewClipRect` / `ViewScrollState` metadata。
- [x] 初始 View 生成时把 `ViewClipRect` / `ViewScrollState` 插入对应实体，供 runtime system 查询。
- [x] Taffy style 映射 overflow，不改变默认 visible 布局行为。
- [x] 增加 schema parse、schema -> runtime roundtrip、slot metadata 单测。

## Task 2：Focus Stack 与输入路由

**Files:**

- Modify: `crates/souprune/src/core/view/components/view_element.rs`
- Modify: `crates/souprune/src/core/view/components.rs`
- Modify: `crates/souprune/src/core/view/lifecycle.rs`
- Create: `crates/souprune/src/core/view/lifecycle/focus.rs`
- Modify: `crates/souprune/src/core/view/input.rs`
- Modify: `crates/souprune/src/core/view/plugin.rs`
- Modify: `crates/souprune/src/core/view/ron_view/spawn.rs`
- Modify: `crates/souprune/src/core/view/messages.rs`

- [x] 新增 `ViewFocusScope` component。
- [x] 新增 `ViewFocusStack` resource，支持 push 去重、remove、top、clear。
- [x] `ActiveView` 改为 focus stack top 的派生 marker。
- [x] 新增 lifecycle 系统：新增 scope 入栈、despawn/removed cleanup、同步唯一 `ActiveView`。
- [x] 输入事务只路由到 `ViewFocusStack.top()`；无 stack resource 时保留旧 `ActiveView` fallback。
- [x] RON 节点上的 `focus_policy: Focusable | Scope` 会让对应布局根进入 focus scope 候选。
- [x] 保留现有单 View 行为。
- [x] 增加焦点栈和输入路由单测。

## Task 3：Example 人工验收

**Files:**

- Modify: `crates/souprune/examples/view_taffy_layout.rs`
- Modify: `crates/souprune/examples/assets/view/taffy_minimal.view.ron`

- [x] 增加 Stage 4 acceptance 区域，覆盖 hidden/scroll/focus 可观察状态。
- [x] 增加本地 facts 和 `demo` interface expects。
- [x] example 按键更新本地 facts：`H/C` hidden probes，`Up/Down` scroll offset，`Tab/Enter/Escape/0` focus state。
- [x] 不在画面中加入使用说明文本。

## 验收

- [x] `cargo fmt --all`
- [x] `cargo test -p souprune view::layout`
- [x] `cargo test -p souprune view::lifecycle::focus --lib`
- [x] `cargo test -p souprune view::input --lib`
- [x] `cargo check -p souprune --example view_taffy_layout`
- [x] `cargo test -p souprune_schema view --lib`
- [x] `cargo test -p souprune view::ron_view::spawn --lib`
- [x] `cargo test -p souprune view::ron_view::spawn_nodes --lib`
- [x] `cargo test -p souprune`
- [x] `cargo clippy --workspace --all-targets -- -D warnings`
- [x] `cargo test -p souprune view::layout::taffy::tests::manual_acceptance_view_asset_parses_and_solves --lib` 直接读取 `.view.ron` 并校验 Stage 4 hidden/scroll/focus 字段。
