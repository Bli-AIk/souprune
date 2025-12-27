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

- [ ] **ABI 层重构 (`souprune_api`)**
  - [ ] 将 `SoulModeVTable` 重命名为 `BehaviorVTable`
  - [ ] 将 `ContextHandle` 的相关函数签名中的命名统一化
  - [ ] 确保 FFI 接口名称变更 (`get_soul_mode_count` -> `get_behavior_count` 等)
- [ ] **SDK 层重构 (`souprune_sdk`)**
  - [ ] 将 `SoulMode` trait 重命名为 `Behavior`
  - [ ] 更新 `declare_souls!` 宏为 `declare_behaviors!`
  - [ ] 更新 `Context` 封装
- [ ] **引擎层重构 (`crates/souprune`)**
  - [ ] **重命名组件与资源**:
    - [ ] `SoulRegistry` -> `BehaviorRegistry`
    - [ ] `SoulParams` -> `BehaviorParams`
    - [ ] `SoulState` -> `BehaviorState`
  - [ ] **性能优化**:
    - [ ] 实现 `ActiveBehavior` 组件 (在 `Added<BehaviorParams>` 时查询并缓存 VTable 指针)
    - [ ] 重写 `update_behaviors_system`: 直接遍历 `ActiveBehavior` 调用函数指针，移除每帧 Hash 查找
  - [ ] **迁移示例 Mod**: 更新 `example_mod` 以适配新的 API

#### 2. 战斗资源与定义重构
目标：理清 "Battle" (整场战斗) 与 "Chapter" (战斗中的一步) 的关系。

- [ ] **资产重命名**
  - [ ] 将 `BattleFlowAsset` 重命名为 `BattleAsset` (对应 `.battle.ron`)
  - [ ] 将 `demo.chapter.ron` 重命名为 `demo.battle.ron`
  - [ ] 更新 `AssetLoader` 注册逻辑
- [ ] **Chapter 定义完善**
  - [ ] 审查 `Chapter` 枚举，确保其作为“战斗步骤”的定义清晰

#### 3. 战斗执行器 (Battle Executor)
目标：实现一个状态机，能够读取 `BattleAsset` 并按顺序执行其中的 `Chapter`。

- [ ] **运行时资源 (`BattleContext`)**
  - [ ] 定义 `BattleContext` 资源，包含：
    - [ ] `current_step`: `usize` (当前执行到的 Chapter 索引)
    - [ ] `wait_timer`: `Timer` (用于 Wait 类型的等待)
    - [ ] `state`: `BattleExecutionState` (Idle, Processing, Waiting)
- [ ] **执行器系统 (`BattleExecutorSystem`)**
  - [ ] **Dispatch 逻辑**: 根据当前 `Chapter` 类型分发处理
  - [ ] **Wait 处理**: 实现 `Wait(f32)` 的计时与自动步进
  - [ ] **Action 处理**: 实现 `SetPlayer`, `SetCamera`, `SetUI` 的即时执行与步进
  - [ ] **UI 交互处理**: 实现 `UIInteraction` 的挂起逻辑 (等待 UI 信号/事件)
  - [ ] **嵌套处理**: 实现 `Nested` 的递归或堆栈执行逻辑 (可选，视复杂度而定)

### v0.4.2: 弹幕系统 (Danmaku Core)
核心架构：基于堆栈的复合弹幕系统，利用 `bevy_tween` 的 `delta` 特性实现叠加。

- [ ] **依赖集成**
  - [x] 在 `Cargo.toml` 中添加 `bevy_tween` 依赖。
    - [ ] 启用 `serde` feature。
  - [ ] 注册 `bevy_tween` 插件，并配置自定义的插值器（如果需要）。

- [ ] **核心数据结构定义**
  - [ ] 定义 `DanmakuBlueprint` 资产 (`.danmaku.ron`): 包含外观、运动堆栈、子生成器。
  - [ ] 定义 `MotionTrack` 枚举 (封装 `bevy_tween` 的类型):
    - [ ] `TranslationTrack`: 对应 `bevy_tween::interpolate::Translation` (开启 `delta: true`)。
    - [ ] `RotationTrack`: 对应 `bevy_tween::interpolate::Rotation`。
    - [ ] `ScriptTrack`: 脚本驱动的轨道 (自定义 `Interpolator`)。
  - [ ] 定义 `ChildSpawner` 结构。

- [ ] **运行时系统 (Runtime Systems)**
  - [ ] **Danmaku Spawner**:
    - [ ] 解析 BluePrint，生成 Bullet Entity。
    - [ ] **Stack 实现**: 遍历 `motion_tracks`，为每个轨道生成一个独立的 "Driver Entity"，挂载 `Tween` 组件，`Target` 指向 Bullet Entity。这样实现多层叠加。
  - [ ] **生命周期管理**: 确保 Driver Entity 随 Bullet Entity 销毁。
  - [ ] **Child Spawning**: 实现触发器逻辑。

- [ ] **可视化调试**
  - [ ] 实现弹幕路径预测绘制。

### v0.4.3: API 桥接与 SDK
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

### v0.4.4: OverworldSession 部分
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