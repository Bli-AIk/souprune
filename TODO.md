# Souprune 项目微任务清单

本清单将任务拆解为极小的原子操作。每完成一项打钩即可。

## ✅ 已完成：v0.1.x (基础与物理)

### v0.1.0 - v0.1.3: 早期基础建设
- [x] 将单体项目重构为 Rust Workspace 结构
- [x] 创建 `crates/souprune` 核心包与 `crates/bevy_mortar_bond`
- [x] 集成 `bevy_ecs_tiled` 并实现 `.tmx` 地图加载
- [x] 实现基础角色生成与动画状态机
- [x] 实现像素完美视口适配

### v0.1.4: SDF 物理与碰撞系统
- [x] 实现基于 SDF 的梯度碰撞检测算法
- [x] 实现 Tilemap 碰撞数据提取与合并
- [x] 实现角色与 SDF 碰撞体的交互
- [x] 实现碰撞层 Debug 可视化

---

## ✅ 已完成：v0.2.x (RPG 交互与 UI)

### v0.2.0: 基础 UI 与交互
- [x] 实现 `OverworldUIBox` 摄像机锚点系统
- [x] 实现 `BoxCursor` 导航逻辑
- [x] 实现 `Backpack` 菜单状态管理
- [x] 集成打字机效果插件 `bevy_ecs_typewriter`

---

## ✅ 已完成：v0.3.x (架构数据化)

### v0.3.0: 数据驱动与 Mod 支持
- [x] 设计并实现 `.char.ron` 和 `.anim.ron`
- [x] 设计并实现 `.item.ron` 及物品注册表
- [x] 设计并实现 `.ui.ron` 布局系统
- [x] 实现 `MultiSourceAssetReader` (Mod 资源加载)
- [x] 重构资源路径结构

---

## 🚀 待办：v0.4.x (战斗系统)

目标：构建完整的弹幕战斗体验，从基础状态切换到复杂的流程控制。

### v0.4.0: 基础架构 (State Management)

#### 基础部分

- [x] 在 `crates/souprune/src/app_state/battle.rs` 创建空的 `BattlePlugin`
- [x] 在 `AppState` 枚举中添加 `Battle` 变体
- [x] 在 `get_game_plugins` 中注册 `BattlePlugin`
- [x] 定义 `OverworldEntity` 组件与 `cleanup_overworld_entities` 系统
- [x] 添加 F6 进入战斗状态的调试按键
- [x] 限制 `OverworldPlugin` 仅在 `AppState::Overworld` 激活
- [x] 添加 `bevy_brp_extras` 依赖
- [x] 修复 ron UI 多次创建的问题
- [x] 定义 `BattleEntity` 组件与 `cleanup_battle_entities` 系统
- [x] 把 `cleanup_entities` 系统改为泛型
- [x] 为 `inspector.rs` 中的物体添加 "Debug:" 前缀

#### 战斗流程数据化 (Battle Sequence)
- [x] 定义 `Chapter` 枚举
- [x] 定义 `BattleSequencer` 作为 序列器
- [x] 实现调度器系统
- [x] 定义 `PlayerAction` 与执行逻辑 (Spawn, Teleport, SetActive)
- [x] 定义 `UIAction` 与执行逻辑 (LoadLayout)
- [x] 定义 `CameraAction` 与执行逻辑 (SetPosition, SetZoom)
- [x] 实现 `Wait` 章节执行逻辑
- [x] 实现 Chapter ron 文件雏形
- [x] 实现战斗碰撞系统 (基于 SDF 的 BattleBox 约束)
- [x] 创建 `souprune_api` crate (定义 HostApi/VTable 协议)
- [x] 实现 Mod SDK (Context 封装与 Safe Rust 接口)
- [x] 实现 Host 端 FFI 函数 (连接 Bevy ECS 与 C ABI)
- [x] 实现 Native Mod Loader (DLL 加载与符号解析)
- [x] 重构 UI 框架以支持战斗 UI
- [x] 实现战斗系统 UI 热重载
- [x] 重构 UIBox 结构 为 SmudShape 层
- [x] 加入 UI 轴点
- [x] 加入 Debug 调整玩家等级
- [x] 搭建 Battle UI

### v0.4.1: 战斗系统执行器 (通用化重构)

#### 1. 核心架构重构: 从 SoulMode 到 Behavior
目标：将 "SoulMode" (灵魂模式) 泛化为通用的 "Behavior" (行为) 系统，使其可用于任何实体（如敌人、弹幕、过场角色）。

- [x] **ABI 层重构 (`souprune_api`)**
  - [x] 将 `SoulModeVTable` 重命名为 `BehaviorVTable`
  - [x] 将 `ContextHandle` 的相关函数签名中的命名统一化
  - [x] 确保 FFI 接口名称变更 (`get_soul_mode_count` -> `get_behavior_count` 等)
- [x] **SDK 层重构 (`souprune_sdk`)**
  - [x] 将 `SoulMode` trait 重命名为 `Behavior`
  - [x] 更新 `declare_souls!` 宏为 `declare_behaviors!`
  - [x] 更新 `Context` 封装
  - [x] **修复状态丢失缺陷 (Critical Fix)**:
    - *原因*: 当前实现每帧都在栈上重新创建 Mod 实例 (`let mut mode = $constructor()`)，导致 Mod 无法保存任何状态 (如计数器、计时器)。必须将实例生命周期托管到堆上。
    - [x] **ABI 变更**: 修改 `create_behavior` 签名，返回 `(*mut c_void, BehaviorVTable)` 而非仅 `BehaviorVTable`。返回的指针指向堆上的 Mod 实例。
    - [x] **VTable 变更**: 修改 `BehaviorVTable` 中的所有函数指针 (如 `on_update`)，增加 `instance: *mut c_void` 作为第一个参数 (相当于 `self`)。
    - [x] **SDK 宏重写**: 更新 `declare_behaviors!` 宏：
      - 在 `create_behavior` 中使用 `Box::new($constructor()).into_raw()` 创建并泄露实例所有权给 Host。
      - 在 `wrapper_on_update` 等回调中，使用 `(instance as *mut $mod_type).as_mut()` 恢复引用来调用方法，避免重新构造。
    - [x] **资源清理**: 在 `BehaviorVTable` 中增加 `destroy` 函数，用于在 Host 销毁 Behavior 时调用 `Box::from_raw(instance)` 重新获取所有权并 Drop，防止内存泄漏。
- [x] **引擎层重构 (`crates/souprune`)**
  - [x] **重命名组件与资源**:
    - [x] `SoulRegistry` -> `BehaviorRegistry`
    - [x] `SoulParams` -> `BehaviorParams`
    - [x] `SoulState` -> `BehaviorState` (注意：实际实现中因机制优化已移除该组件)
  - [x] **性能优化**:
    - [x] 实现 `ActiveBehavior` 组件 (在 `Added<BehaviorParams>` 时查询并缓存 VTable 指针)
    - [x] 重写 `update_behaviors_system`: 直接遍历 `ActiveBehavior` 调用函数指针，移除每帧 Hash 查找
  - [x] **迁移示例 Mod**: 更新 `example_mod` 以适配新的 API

#### 2. 战斗资源与定义重构
目标：理清 "Battle" (整场战斗) 与 "Chapter" (战斗中的一步) 的关系。

- [x] **资产重命名**
  - [x] 将 `BattleFlowAsset` 重命名为 `BattleAsset` (对应 `.battle.ron`)
  - [x] 将 `demo.chapter.ron` 重命名为 `demo.battle.ron`
  - [x] 更新 `AssetLoader` 注册逻辑
- [x] **Chapter 定义完善**
  - [x] 审查 `Chapter` 枚举，确保其作为“战斗步骤”的定义清晰 (无需重命名，BattleAsset = Vec<Chapter> 关系明确)

#### 3. 战斗执行器 (Battle Executor)
目标：实现一个状态机，能够读取 `BattleAsset` 并按顺序执行其中的 `Chapter`。

- [x] **运行时资源 (`BattleContext`)**
  - [x] 定义 `BattleContext` 资源，包含：
    - [x] `chapters`: `Vec<Chapter>`
    - [x] `state`: `BattleExecutionState` (Idle, Processing, Waiting)
- [x] **执行器系统 (`BattleExecutorSystem`)**
  - [x] **Dispatch 逻辑**: 根据当前 `Chapter` 类型分发处理
  - [x] **Wait 处理**: 实现 `Wait(f32)` 的计时与自动步进
  - [x] **Action 处理**: 实现 `SetPlayer`, `SetCamera`, `SetUI` 的即时执行与步进
  - [x] **UI 交互处理**: 实现 `UIInteraction` 的挂起逻辑 (等待 UI 信号/事件 - 目前作为非阻塞处理，需未来增强)
  - [x] **嵌套处理**: 实现 `Sequence` (队列展开) 和 `Parallel` (父子追踪) 的执行逻辑

### v0.4.2: 弹幕系统 (Danmaku Core)
核心架构：基于堆栈的复合弹幕系统，采用数据驱动设计。

- [x] **依赖集成**
  - [x] 在 `Cargo.toml` 中添加 `bevy_tween` 依赖。
  - [x] 使用 Bevy 0.17 Message API 替代旧版 Event API。

- [x] **素材注册**
  - [x] 在 `textures/battle/config.toml` 注册 `flowey_pellet` 动画 (36帧)。
  - [x] 在 `textures/battle/config.toml` 注册 `spear` 精灵。

- [x] **核心数据结构定义 (数据驱动)**
  - [x] 定义 `DanmakuBlueprint` 资产 (`.danmaku.ron`)
    - [x] `BulletVisual`: Sprite, SpriteRef 或 Animation 视觉表现
    - [x] `SpawnPattern`: Single, RingGenerator, LineGenerator, EdgeGenerator 生成模式
    - [x] `MotionTrack`: Linear, Orbital (原 Circular), Sine, Homing, Tween, Custom (原 Algo) 运动轨道
    - [x] `ChildSpawner`: 子发射器（嵌套弹幕支持）
  - [x] 定义 `SpawnPatternEvent` 消息事件 (引用 blueprint 路径)
  - [x] 定义 `BulletMotionState` 和 `BulletMotionTracks` 运行时组件

- [x] **运行时系统 (Runtime Systems)**
  - [x] **Blueprint Loader**: 实现 `.danmaku.ron` 资产加载器
  - [x] **Danmaku Spawner**: 实现 `process_spawn_pattern_events` + `spawn_bullets_from_blueprints` 系统
  - [x] **运动堆栈评估**: 实现 `update_bullet_motion` 系统 (支持多轨道叠加)
  - [x] **生命周期管理**: 实现 `update_bullet_lifetime` 和 `cleanup_dead_bullets` 系统

- [x] **具体弹幕实现 (Example Blueprints)**
  - [x] 实现 `BulletPattern` 章节执行器逻辑 (加载 blueprint 路径)
  - [x] 创建 `flowey_pellet.danmaku.ron`
    - [x] 配置：RingGenerator 生成 (12个弹幕, 半径 120)
    - [x] 运动：Orbital 轨道 (旋转 + 向内收缩)
    - [x] 视觉：Animation (flowey_pellet, 36帧)
  - [x] 创建 `undyne_spear.danmaku.ron`
    - [x] 配置：EdgeGenerator 生成 (5支矛, 从左侧)
    - [x] 运动：Linear 轨道 (向右移动)
    - [x] 视觉：SpriteRef (battle/spear)

- [x] **API 改进与 SDK 封装**
  - [x] 重命名 API 以提高清晰度：
    - [x] `Circular` -> `Orbital` (避免与 Tween 混淆)
    - [x] `Circle`/`Box` -> `CircleCollider`/`BoxCollider`
    - [x] `Ring`/`Line`/`Edge` -> `RingGenerator`/`LineGenerator`/`EdgeGenerator`
  - [x] 添加 `SpriteRef` 视觉类型，通过 config.toml 名称引用精灵
  - [x] 添加 `TriggerCollider` 组件到弹幕实体以支持 F3 碰撞箱可视化
  - [x] Timeline 支持 `absolute` 字段（绝对时间）和默认的相对时间
  - [x] Timeline 支持 `behaviors` 字段（内联行为定义）
  - [x] 修复动画帧排序问题（实现自然数字排序）
  - [x] SDK 封装 unsafe：提供 `BulletContext`、`Vec2` 安全封装，支持命名属性 (props) 和 `get_float` API

### v0.4.3: API 桥接与 SDK
见 https://github.com/Bli-AIk/souprune/issues/19

- [ ] 重写 readme 以反映新的多语言支持愿景
- [ ] **Phase 1: Interoptopus 迁移**
  - [ ] 在 `souprune_api` 添加 `interoptopus` 依赖
  - [ ] 用 `#[ffi_type]` 重构 `ContextHandle` 和 `HostApi`
  - [ ] 创建 `bindgen` bin target 并实现基础 C 绑定生成
- [ ] **Phase 2: C# 集成 (.NET Native AOT)**
  - [ ] 在 `bindgen` 中启用 C# 后端
  - [ ] 建立 `souprune-sdk-dotnet` 项目结构
  - [ ] 实现 C# 版本的 `Hello World` Mod
- [ ] **Phase 3: Haxe 集成 (hxcpp)**
  - [ ] 在 `bindgen` 中实现 Haxe 代码生成逻辑
  - [ ] 建立 `souprune-sdk-haxe` 项目结构
  - [ ] 实现 Haxe 版本的 `Hello World` Mod
- [ ] **Phase 4: Nelua 集成**
  - [ ] 在 `bindgen` 中实现 Nelua 代码生成逻辑
  - [ ] 建立 `souprune-sdk-nelua` 项目结构
  - [ ] 实现 Nelua 版本的 `Hello World` Mod
- [ ] **Phase 5: Nim 集成**
  - [ ] 在 `bindgen` 中实现 Nim 代码生成逻辑
  - [ ] 建立 `souprune-sdk-nim` 项目结构
  - [ ] 实现 Nim 版本的 `Hello World` Mod

### v0.4.4: 战斗 UI 交互 (Battle UI Interaction)
目标：完善战斗中的 UI 交互流程，实现阻塞式对话与选择，并还原 Undertale 风格的基础战斗菜单。

- [ ] **基础交互架构**
  - [ ] 实现 `UIInteraction` 的真正阻塞逻辑 (BattleExecutor 暂停直到 UI 发送完成信号)
  - [ ] 定义 UI -> Battle 通信事件 (`BattleUIEvent::SelectionMade`, `BattleUIEvent::DialogueFinished`)
  - [ ] 更新 `sequencer.rs` 以处理这些事件并恢复流程
- [ ] **Undertale 战斗 UI 实现**
  - [ ] 实现四个主要按钮 (Fight, Act, Item, Mercy) 的 UI 布局与导航逻辑
  - [ ] 实现 **Fight** 逻辑: 选择敌人 -> 攻击动画 (QTE) -> 伤害计算
  - [ ] 实现 **Act** 逻辑: 选择敌人 -> 选择动作 -> 执行动作 (调用 Behavior 接口)
  - [ ] 实现 **Item** 逻辑: 读取背包 -> 选择物品 -> 使用效果
  - [ ] 实现 **Mercy** 逻辑: Spare (检查条件) / Flee (概率逃跑)

### v0.4.5: OverworldSession 部分
目标：实现 Overworld 状态的保存与恢复机制。

- [ ] 定义 `OverworldSession` 资源
- [ ] 实现 `save_overworld_state` (OnExit Overworld)
- [ ] 实现 `restore_overworld_state` (OnEnter Overworld)

---

## 🧠 待办：v0.5.x (逻辑与脚本增强)

目标：通过 FRE 系统解耦游戏逻辑，并增强脚本能力以支持复杂剧情。

### v0.5.0: FRE 事件系统 (Fact-Rule-Event)
- [ ] 定义 `FactDb` 资源与 `FactValue` 枚举
- [ ] 定义 `FREEvent` 事件总线
- [ ] 定义 `Rule` 结构体与 `.rule.ron` 加载器
- [ ] 实现 `process_rules` 核心系统 (Trigger -> Condition -> Action/Modification)

### v0.5.1: 脚本系统增强 (Scripting API)
- [ ] 为 Mortar 脚本暴露 `FRE` 接口 (读写 Fact, 触发 Event)
- [ ] 扩展 `bevy_mortar_bond` 支持新的逻辑指令
- [ ] 实现 `Dialogue` Chapter 与脚本系统的互操作

### v0.5.2: 背包系统补全
- [ ] 实现物品使用逻辑 (Item Use Effects)
- [ ] 实现物品查看逻辑 
- [ ] 实现物品丢弃逻辑

---

## 🎨 待办：v0.6.x (视听体验)

目标：提升游戏的视觉表现与听觉反馈。

### v0.6.0: 音频系统 (Firewheel Integration)
- [ ] 添加 `bevy_seedling` 依赖
- [ ] 配置音频通道 (Music, SFX, Voice)
- [ ] 实现音频播放与音量控制系统
- [ ] 在战斗与 UI 中集成音效触发

### v0.6.1: 视觉增强 (Shaders & VFX)
- [ ] 创建 `PostProcessPlugin`
- [ ] 实现基础 Post-processing Shader (CRT, 色差等)
- [ ] 实现 2D 光照组件 `PointLight2d`