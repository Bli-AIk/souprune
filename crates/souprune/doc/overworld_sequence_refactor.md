# Overworld 可配置化重构：Sequence 驱动的场景构建

## 问题陈述

当前 Overworld 和 Battle 是两个硬编码的 `AppState`。Battle 已经通过 `sequence.ron` 实现了声明式场景编排，但 Overworld 仍然是以代码驱动的方式组织的——进入 Overworld 时直接执行一系列固定的初始化系统（加载地图、生成玩家、设置相机、启动 BGM 等）。

**目标**：让 Overworld 也能通过 `sequence.ron` 来定义场景的构建和行为流程，实现"第三条道路"——不是 Unity/UE 的"场景为王"，也不是 Godot 的"节点为王"，而是**"序列为王"**：通过 `sequence.ron` 编排所有场景生命周期。

## 核心原则回顾

1. **完全弃用旧语法** — 不考虑向后兼容性
2. **若无必要，不引入新语法** — 最大化复用已有的 Chapter 变体和 FRE 系统
3. **最小化 API 表面** — 用组合实现复杂行为

---

## 现状分析

### Battle 的 Sequence 架构（已实现）

```
AppState::Battle
  ├─ OnEnter: setup_camera, setup_input
  ├─ Sequencer: 从 .sequence.ron 加载 chapters
  │   ├─ SpawnView / SetViewFact / TweenViewElement
  │   ├─ DanmakuPerformance / AmPerformance
  │   ├─ SetPlayer / SetCamera
  │   ├─ Conditional / FactSwitch / AwaitFact
  │   ├─ LoadFre / ModifyFact / EmitFactEvent
  │   ├─ Sequence / Parallel / Wait
  │   └─ RunSequence (嵌套外部序列)
  └─ OnExit: cleanup
```

Battle 的 23 个 Chapter 变体已经覆盖了：
- UI 视图管理（SpawnView, SetViewFact, TweenViewElement, ModifyViewElement）
- 动画和演出（DanmakuPerformance, AmPerformance）
- 控制流（Sequence, Parallel, Wait, Conditional, FactSwitch, RunSequence）
- 游戏状态（SetPlayer, SetCamera, SetUI）
- FRE 集成（AwaitFact, ModifyFact, EmitFactEvent, LoadFre）

### Overworld 的现状（硬编码）

```
AppState::Overworld
  ├─ OnEnter (硬编码):
  │   ├─ create_overworld_entities_system → spawn player
  │   ├─ setup_action_handlers_system → FRE trigger handlers
  │   └─ set_overworld_danmaku_context
  ├─ Update (固定系统集):
  │   ├─ TilemapPlugin: initialize, collision, objects, camera, z-order, bgm
  │   ├─ PlayerPlugin: direction, spawn, state transition
  │   ├─ CollisionPlugin, TriggerPlugin, ChasePlugin
  │   └─ FRETriggerSet: rules, triggers, interactables, danmaku
  └─ OnExit: cleanup entities, stop bgm, clear FRE
```

**Overworld 中硬编码的操作（需要序列化的）**：

| 操作 | 现有实现方式 | 是否可复用现有 Chapter |
|------|-------------|----------------------|
| 加载瓦片地图 | `initialize_tilemap_system` 读取 config.initial_map_path | ❌ 需新 Chapter |
| 生成碰撞体 | `generate_collision_tiles_system` | ❌ 需新 Chapter 或合并到 LoadMap |
| 处理对象属性 | `process_map_object_properties_system` | ❌ 同上 |
| 设置相机边界 | `setup_camera_bounds_system` | ✅ SetCamera 可扩展 |
| 生成玩家 | `spawn_player_on_event` | ✅ SetPlayer 已存在 |
| 设置 FRE handler | `setup_action_handlers_system` | ✅ LoadFre 已存在 |
| 启动 BGM | `update_map_bgm_system` | ❌ 需新 Chapter |
| 加载状态配置 | StateConfig from states.ron | ✅ LoadFre 可复用 |

---

## 重构方案

### 第一步：统一 Sequencer — 从 Battle 专属到全局共享

**关键变化**：Sequencer 不再是 Battle 的专属组件，而是成为核心基础设施。

```
当前:
  AppState::Battle → SequencerPlugin (Battle 专属)

目标:
  Core → SequencerPlugin (全局)
  AppState::Overworld → 使用 Sequencer
  AppState::Battle → 使用 Sequencer
```

**具体做法**：
- 将 `src/app_state/battle/sequencer/` 移至 `src/core/sequencer/`
- 泛化 `BattleContext` → `SequenceContext`（通用序列执行上下文）
- `SequencerPlugin` 不再依赖 `AppState::Battle`，改为在任何状态下可用
- 保留所有现有 Chapter 变体不变

### 第二步：新增 Overworld 专属 Chapter 变体

分析 Overworld 需要的操作，**仅新增无法由现有 Chapter 组合完成的**：

#### 必须新增的 Chapter

```rust
/// 加载 Tiled 瓦片地图
LoadMap {
    /// 地图路径 (.tmx)，支持 Expr 动态路径
    path: String,
    /// 是否生成碰撞体（默认 true）
    generate_collision: bool,
    /// 是否处理对象属性（触发区、交互物、NPC 等，默认 true）
    process_objects: bool,
    /// 是否设置相机边界（默认 true）
    setup_camera_bounds: bool,
}
```

**理由**：加载瓦片地图是 Overworld 独有的操作，无法由现有 Chapter 组合实现。将碰撞体生成、对象处理、相机边界作为选项合并到 `LoadMap` 中，因为它们在逻辑上是不可分割的（对象和碰撞依赖地图数据）。

```rust
/// 播放/切换 BGM
SetBgm {
    /// BGM 路径（None = 停止）
    path: Option<String>,
    /// 淡入时长（秒）
    fade_in: Option<f32>,
}
```

**理由**：Battle 的 `DanmakuPerformance` 和 `AmPerformance` 面向视觉动画，BGM 控制是完全不同的领域。这是真正缺失的 Chapter。

#### 无需新增的操作

| 操作 | 复用方式 |
|------|---------|
| 生成玩家 | `SetPlayer(Spawn { ... })` — 已存在 |
| 设置相机跟随 | `SetCamera(Follow { ... })` — 已存在 |
| 加载 FRE 规则 | `LoadFre { files, aggregate }` — 已存在 |
| 设置 Fact | `ModifyFact { modifications }` — 已存在 |
| 显示 UI | `SpawnView { view_layout, bindings }` — 已存在 |
| 等待条件 | `AwaitFact { condition }` — 已存在 |
| 条件分支 | `Conditional { condition, then, else }` — 已存在 |

### 第三步：Overworld 入口序列化

**以前（硬编码）**：
```rust
// overworld.rs OnEnter
fn create_overworld_entities_system(...) {
    spawn_player();
    setup_action_handlers();
}
```

**以后（数据驱动）**：

`overworld_entry.sequence.ron`:
```ron
(
    rules_file: Some("overworld/rules/global.fre.ron"),
    chapters: [
        // 1. 加载地图
        LoadMap(
            path: "levels/town.tmx",
            generate_collision: true,
            process_objects: true,
            setup_camera_bounds: true,
        ),
        // 2. 生成玩家
        SetPlayer(Spawn(
            position: Some((160.0, 120.0)),
        )),
        // 3. 相机跟随玩家
        SetCamera(Follow(
            smooth: true,
        )),
        // 4. 加载 FRE 规则
        LoadFre(
            files: [
                "overworld/rules/triggers.fre.ron",
                "overworld/rules/states.fre.ron",
            ],
            aggregate: {},
        ),
        // 5. 启动 BGM
        SetBgm(
            path: Some("audio/bgm/town.ogg"),
            fade_in: Some(1.0),
        ),
        // 6. 显示 HUD
        SpawnView(
            view_layout: "ui/overworld_hud.view.ron",
            bindings: {
                "player_data": LocalLayer,
            },
        ),
    ],
)
```

### 第四步：配置路径更新

`mod.toml` / `config.toml` 变更：

```toml
[game]
# 旧：initial_map_path = "levels/town.tmx"
# 新：
initial_sequence_path = "sequences/overworld_entry.sequence.ron"

# 旧：initial_battle_path = "battles/test.sequence.ron"
# 新：（保持不变，或统一）
initial_battle_path = "battles/test.sequence.ron"
```

**更进一步**（可选）：
```toml
[game]
# 完全统一入口，不再区分 Overworld 和 Battle
entry_sequence = "sequences/main.sequence.ron"
```

`main.sequence.ron` 可以根据 FRE 条件决定进入 Overworld 还是 Battle：
```ron
(
    chapters: [
        Conditional(
            condition: Exists("debug_battle_mode"),
            then_branch: Some(RunSequence(path: "battles/test.sequence.ron")),
            else_branch: Some(RunSequence(path: "sequences/overworld_entry.sequence.ron")),
        ),
    ],
)
```

### 第五步：AppState 简化（远期）

```rust
// 旧
#[derive(States)]
pub enum AppState {
    AppSetup,
    Menu,
    Overworld,
    Battle,
}

// 新的可能性：
// Overworld 和 Battle 的区别仅在于可用的 Update 系统集
// 可以考虑合并为 InGame，由 sequence.ron 控制一切
#[derive(States)]
pub enum AppState {
    AppSetup,
    Menu,
    InGame, // 统一入口，序列决定行为
}
```

**但暂不建议立即合并**，原因：
- Overworld 和 Battle 的 Update 系统集完全不同（碰撞检测、物理、渲染层等）
- 合并后需要引入"模式"概念来控制系统激活，增加复杂度
- 可先保持两个状态，仅统一初始化流程

**建议路径**：
1. ✅ 先统一 Sequencer（本次重构）
2. ✅ Overworld 入口序列化（本次重构）
3. ⏳ 后续再考虑 AppState 合并（需要更多实践验证）

---

## 地图切换：Overworld 之间的转场

当前地图切换逻辑在 `trigger.rs` 中硬编码。序列化后：

`on_map_change.sequence.ron`：
```ron
(
    chapters: [
        // 1. 淡出
        TweenViewElement(
            selector: FullName("transition::fade_overlay"),
            target: Alpha(from: 0.0, to: 1.0),
            duration: 0.5,
            easing: EaseInOut,
            wait_for_completion: true,
        ),
        // 2. 清理旧地图
        EmitFactEvent(
            event_id: "map_cleanup",
            data: {},
        ),
        // 3. 加载新地图
        LoadMap(
            path: "$next_map_path",
            generate_collision: true,
            process_objects: true,
            setup_camera_bounds: true,
        ),
        // 4. 重置玩家位置
        SetPlayer(Teleport(
            position: "$spawn_point",
        )),
        // 5. 切换 BGM
        SetBgm(
            path: "$next_bgm_path",
            fade_in: Some(0.5),
        ),
        // 6. 淡入
        TweenViewElement(
            selector: FullName("transition::fade_overlay"),
            target: Alpha(from: 1.0, to: 0.0),
            duration: 0.5,
            easing: EaseInOut,
            wait_for_completion: true,
        ),
    ],
)
```

---

## 实施计划

### Phase 1：Sequencer 核心提取（影响最小）

1. 将 `battle/sequencer/` 移到 `core/sequencer/`
2. 泛化 `BattleContext` → `SequenceContext`
3. `SequencerPlugin` 改为全局注册
4. 确保 Battle 功能不受影响

### Phase 2：新增 Chapter 变体

1. 新增 `LoadMap` Chapter + 对应系统
2. 新增 `SetBgm` Chapter + 对应系统
3. 扩展 `SetPlayer` 变体（如果需要 `Teleport`）
4. 扩展 `SetCamera` 变体（如果需要新操作）

### Phase 3：Overworld 序列化

1. 创建 `overworld_entry.sequence.ron` 示例
2. 重构 `overworld.rs` 的 OnEnter 改为加载序列
3. 逐步迁移硬编码初始化到 Chapter
4. 保留 Update 系统集不变（碰撞、物理等仍然是代码驱动的）

### Phase 4：配置整合

1. 更新 `SoupruneConfig.game` 使用 `initial_sequence_path`
2. 更新文档和示例

---

## 不变的部分

以下系统仍然保持代码驱动（不适合序列化）：

- **每帧 Update 系统**：碰撞检测、物理、玩家输入、相机跟随、Z-ordering
- **OverworldSubState 状态机**：由 FRE 规则驱动（已经是数据驱动的）
- **Chase 追逐系统**：实时游戏逻辑，不适合序列编排
- **触发区 / 交互物的响应逻辑**：由 FRE 事件系统驱动（已经是数据驱动的）

这些系统本质上是**持续运行的 reactive 逻辑**，而非**一次性的初始化步骤**。Sequence.ron 的职责是编排初始化和转场，而非替代运行时系统。

---

## 风险与注意

1. **Sequencer 提取时的 import 路径变更**：Battle 中所有引用 `battle::sequencer::*` 的代码需要更新为 `core::sequencer::*`
2. **LoadMap 系统的复杂度**：加载 Tiled 地图涉及异步资源加载、碰撞体生成、对象实例化等，需要在 Sequencer 中支持异步等待
3. **测试覆盖**：需要确保序列化后的 Overworld 入口行为与原来完全一致
