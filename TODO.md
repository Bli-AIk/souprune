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
- [x] 定义 `PlayerAction`
- [x] 实现 Chapter ron 文件雏形
- [x] 创建 `souprune_api` crate (定义 HostApi/VTable 协议)
- [ ] 实现 Mod SDK (Context 封装与 Safe Rust 接口)
- [ ] 实现 Host 端 FFI 函数 (连接 Bevy ECS 与 C ABI)
- [ ] 实现 Native Mod Loader (DLL 加载与符号解析)
- [ ] 实现执行器系统
  - [ ] 实现 `SoulMode` 组件 (持有当前 Mod 的 VTable)
  - [ ] 实现 `ModUpdateSystem` (驱动 on_update 生命周期)
- [ ] 定义 `BattleSetup`
- [ ] 实现 `.battle.ron` 的 `AssetLoader`
- [ ] 定义 `BattleContext` 运行时资源
- [ ] 实现 Chapter 执行器：根据当前 Chapter 类型切换子状态

### v0.4.1: 弹幕系统 (Danmaku Core)
- [ ] 定义 `BulletBehavior` 枚举 (Functional, Tween, Composite)
- [ ] 实现 `BulletSpawner` 系统
- [ ] 实现基于 `Functional` 的运动逻辑系统
- [ ] 实现基于 `Tween` 的运动逻辑系统

### v0.4.2 OverworldSession 部分

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
- [ ] 扩展 `.ui.ron` 支持 `Sprite` 元素 (用于战斗 UI)