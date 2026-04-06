# 架构指南

> 英文版请参阅 [architecture.md](architecture.md)。

本文档描述 SoupRune 的内部架构。

---

## 全局三层架构

SoupRune 将关注点分离为三个清晰的层级。
你可以把它想象成 **硬件 → 固件 → 游戏卡带** 的关系：

```
┌──────────────────────────────────────────────┐
│  用户内容层（Mod / 游戏项目）                   │
│  projects/<mod>/                              │
│  ── RON 配置、Mortar 脚本、WASM 模组 ────────  │
├──────────────────────────────────────────────┤
│  预设层（Rust 原生代码）                        │
│  crates/souprune/src/preset/                  │
│  ── 战斗、大地图、物品、敌人 ─────────────────  │
├──────────────────────────────────────────────┤
│  核心引擎层（Rust 原生代码）                     │
│  crates/souprune/src/core/                    │
│  ── FRE、View、Mortar、弹幕、碰撞 ───────────  │
└──────────────────────────────────────────────┘
```

**依赖箭头严格单向**：Core ← Preset ← 用户内容。
Core 永远不会导入 preset。Preset 永远不会导入用户 mod。

---

## 核心引擎层（`core/`）

核心层是 SoupRune 的"运行时"——它知道事物**如何运作**，
但不知道它们**意味着什么**。它提供弹幕运动、碰撞检测、对话渲染
和响应式 UI——但完全不了解 "HP" "物品" "敌人" 或 "战斗阶段" 等概念。

### FRE（事实-规则-事件）—— 引擎的心脏

FRE 是 SoupRune 的数据驱动规则引擎。它通过三个简单概念实现数据与行为的解耦：

| 概念            | 做什么               | 示例                             |
|---------------|-------------------|--------------------------------|
| **Fact（事实）**  | 全局键值存储——唯一的真相来源   | `"player:hp" = 20`             |
| **Event（事件）** | 表示发生了某事的信号——不携带逻辑 | `CollisionEnter { a, b }`      |
| **Rule（规则）**  | 声明式逻辑，响应事件并修改事实   | `On TakeDamage → hp -= amount` |

**运作流程**：事件触发 → FRE 引擎评估匹配的规则 →
规则修改事实 → View 系统响应式地更新 UI。

这意味着你永远不需要写 `button.set_color(gray)`。取而代之的是，
控制按钮的事实发生了变化，View 自动做出反应。

### View 系统 —— 声明式 UI

View 通过 `.view_layout.ron` 文件定义——而非 Rust 代码。
系统使用 SDF 渲染（基于 `bevy_alight_motion`）和网格文本
（基于 `bevy_rich_text3d`）来呈现高质量的视觉效果。

Preset 通过 **解析器注册表** 向 View 注入游戏特定数据：

- `DataPathResolvers` —— 将 `"player.hp"` 等路径解析为实际值
- `ConditionResolvers` —— 评估 `"has_item('sword')"` 等条件
- `ExprFunctionResolvers` —— 提供自定义表达式函数

这意味着核心的 View 系统是完全通用的——它不知道 `"player.hp"` 意味着什么，
直到 preset 告诉它如何解析这个路径。

### Mortar VM —— 对话系统虚拟机

Mortar 是专为分支对话和脚本演出设计的字节码虚拟机。
它处理对话树、条件文本和定时事件序列中固有的复杂逻辑——
让这些复杂性远离 FRE 规则和 Rust 代码。

Mortar 脚本发出抽象事件；FRE 捕获这些事件来更新游戏状态。
文本内容存在于 Mortar 中；游戏逻辑存在于 FRE 规则中。

### 弹幕系统 —— STG 引擎

SoupRune **归根到底是一个 RPG/STG 框架**。弹幕系统不是附加功能——
它是享有 `core/` 特权地位的一等公民引擎特性。

- **子弹生命周期**：生成 → 行为栈 → 逐帧运动更新 → 销毁
- **内置运动模式**（原生 Rust，零 WASM 开销）：
  线性、轨道、正弦、缓动、静止、瞄准
- **自定义运动**：用户通过 WASM 组件实现特殊弹幕
- **时间轴演出**：RON 驱动的生成序列，支持可配置的模式
- **高性能**：为 60fps 下数千同时存在的子弹而优化

### 碰撞系统

基于 SDF 的碰撞检测，配合 `EventPhase` 缓冲机制提供
基于冷却的事件去重。系统发出通用碰撞事件——
由预设层来解释它们的含义（例如"与玩家碰撞 = 受到伤害"）。

### Mod 系统（WASM 运行时）

基于 wasmtime 的运行时，加载用户提供的 WASM 组件。
Mod 可以提供自定义子弹行为、生成模式、动作处理器、
模式生命周期钩子和规则提供器——全部通过定义明确的 WIT 接口。

---

## 预设层（`preset/`）

预设层将通用核心引擎转化为完整的 UT/DR 游戏体验。
它用 **原生 Rust** 编写（不是 WASM），以获得最大的性能和类型安全。

这一层故意保持 **整体式** 设计——目标用户（同人游戏创作者）
需要一套完整的 RPG+STG 工具包，而不是需要拼装的可选微型 crate。

### 预设层提供什么

- **战斗系统**：回合制状态机、HP/伤害、敌人 AI、战斗框
- **大地图**：玩家控制器、NPC 交互、瓦片地图、区域触发器、追逐序列
- **物品系统**：`ItemRegistry`、物品效果（治疗、装备、音效）、FRE 事实注入
- **敌人系统**：`EnemyRegistry`、敌人数据、遭遇配置
- **FRE 集成**：游戏特定的动作处理器和规则定义
- **View 集成**：DataPath/Condition/ExprFunction 解析器，驱动响应式 UI
- **对话集成**：`MortarFactBindings` 将游戏数据注入对话变量

### 预设层如何与核心层通信

Preset 完全通过标准的 Bevy 和 FRE 机制与 Core 通信：

1. **Bevy ECS**：组件、资源、事件、系统、插件
2. **FRE**：规则、事实、事件、动作处理器
3. **解析器注册表**：动态注册数据解析器
4. **ViewActionExtensions**：可扩展的 View 事件分发
5. **MortarFactBindings**：动态 Mortar 函数/变量绑定

---

## 用户内容层（`projects/`）

这是游戏创作者工作的地方。内容完全通过数据和脚本来创作：

| 格式         | 用途      | 示例                           |
|------------|---------|------------------------------|
| **RON**    | 结构化游戏数据 | 物品定义、敌人属性、View 布局、弹幕演出       |
| **Mortar** | 脚本化序列   | 对话树、过场动画、事件链                 |
| **FRE 规则** | 游戏逻辑    | 状态转换、条件行为、伤害公式               |
| **WASM**   | 自定义代码   | 特殊弹幕、Boss 专属机制               |
| **资产**     | 媒体文件    | 精灵图、音频、瓦片地图、Alight Motion 项目 |

---

## WASM 扩展模型

WASM 是面向 **mod 作者的扩展点**——不是 Rust 系统的替代品。

| 使用 WASM    | 保留在 Rust 中 |
|------------|------------|
| 自定义子弹行为    | 核心运动原语     |
| 特殊生成模式     | 碰撞检测       |
| Mod 专有游戏逻辑 | 渲染和 UI 布局  |
| 特殊 Boss 机制 | FRE 规则评估   |

**WIT 接口** 定义了引擎与 mod 之间的契约：
`behavior`、`danmaku`、`spawn-pattern`、`custom-action-handler`、
`mode-lifecycle`、`rule-provider`

**性能提示**：WASM 在边界处有序列化开销。
热路径代码（子弹更新 × 数千子弹 × 60fps）应保留在 Rust 中。

---

## 边界规则

以下是保持 SoupRune 可维护性的架构不变量：

| ✅ Core 可以         | ❌ Core 禁止                 |
|-------------------|---------------------------|
| 定义弹幕运动原语          | 从 `preset/` 导入            |
| 定义碰撞形状和事件         | 硬编码游戏特定的 fact key         |
| 定义对话渲染和 Mortar VM | 了解 Item、Enemy 或 BattleBox |
| 定义 View 布局和响应式更新  | 定义游戏状态机                   |
| 定义 FRE 规则评估       | 注册游戏特定的 Mortar 函数         |
| 定义通用调度原语          | 包含 UT/DR 特定词汇             |

---

## Crate 地图

```
crates/
├── souprune/                     # 主框架 crate
│   └── src/
│       ├── core/                 # 第一层：引擎基础设施
│       │   ├── danmaku/          #   ★ STG 子弹引擎（特权模块）
│       │   ├── dialogue/         #   对话 UI & Mortar 集成
│       │   ├── view/             #   RON 驱动的声明式 UI
│       │   │   └── ron_view/
│       │   │       └── player_data.rs  # 解析器注册表
│       │   ├── collision.rs      #   SDF 碰撞检测
│       │   ├── fre_bridge.rs     #   FRE ↔ ECS 桥接
│       │   ├── fre_facts.rs      #   核心 fact key 常量
│       │   ├── mod_system.rs     #   WASM mod 加载和注册表
│       │   └── sequencer.rs      #   章节驱动的游戏流程
│       ├── preset/               # 第二层：UT/DR 游戏逻辑
│       │   ├── battle/           #   战斗状态机
│       │   ├── overworld/        #   大地图探索
│       │   ├── item.rs           #   物品注册表和数据
│       │   ├── item_actions.rs   #   物品 FRE 动作处理器
│       │   └── enemy.rs          #   敌人注册表和数据
│       └── app_state/            # 应用状态管理
│
├── bevy_fact_rule_event/         # FRE 引擎（git 子模块）
├── bevy_mortar_bond/             # Mortar 脚本（git 子模块）
├── bevy_ecs_typewriter/          # 打字机文本效果（git 子模块）
├── bevy_alight_motion/           # Alight Motion + SDF 渲染（git 子模块）
│
├── souprune_api/                 # WIT 接口定义
├── souprune_sdk/                 # Rust WASM guest SDK
├── souprune_mod_test/            # 示例 WASM mod
└── souprune_mock_host/           # 独立 WASM 测试宿主
```
