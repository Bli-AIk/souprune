# 架构指南

> 英文版请参阅 [architecture.md](architecture.md)。

SoupRune 是基于 Rust/Bevy 的 RPG+STG 同人游戏框架（Deltarune/Undertale 风格）。
本文档描述其分层架构。

---

## 三层架构

```
┌──────────────────────────────────────────┐
│  用户内容层（Mod / 游戏项目）              │  RON、Mortar 脚本、
│  projects/<mod>/                         │  WASM 组件、资产文件
├──────────────────────────────────────────┤
│  预设层（Rust 原生）                       │  游戏特定逻辑：
│  crates/souprune/src/preset/             │  战斗、大地图、物品、
│                                          │  敌人、UI 布局
├──────────────────────────────────────────┤
│  核心引擎层（Rust 原生）                    │  通用基础设施：
│  crates/souprune/src/core/               │  FRE、View、Mortar、弹幕、
│                                          │  碰撞、WASM 运行时
└──────────────────────────────────────────┘
```

**依赖规则**：Core ← Preset ← 用户内容。依赖方向严格单向。

### 核心层（`core/`）

引擎层。只知道机制，不知道语义。

- **FRE**（事实-规则-事件）：数据驱动的规则引擎。事件触发规则；规则修改事实；View 响应事实变化。
- **View**：RON 驱动的声明式 UI，支持 SDF 渲染和响应式更新。
- **Mortar VM**：字节码虚拟机，处理分支对话和脚本演出。
- **弹幕系统**：高性能子弹引擎——生命周期管理、行为栈、内置运动模式。
- **碰撞系统**：基于 SDF 的碰撞检测，含事件缓冲和冷却去重。
- **Mod 系统**：基于 wasmtime 的 WASM 运行时，加载用户自定义行为和模式。

核心层提供弹幕、碰撞、对话和 View，但不知道"HP""物品""战斗阶段"等概念。

### 预设层（`preset/`）

游戏逻辑层。将通用核心转化为完整的 RPG+STG 框架。

- 战斗状态机、回合流程、伤害计算
- 大地图：玩家控制器、NPC 交互、瓦片地图
- 物品/敌人注册表、FRE 动作处理器
- View 的 DataPath/Condition/ExprFunction 解析器
- MortarFactBindings 对话变量注入

预设层保持单一整体模块——目标用户需要完整的 RPG+STG 工具套件。

### 用户内容层（`projects/`）

通过数据和脚本创作游戏内容：

- RON：物品/敌人定义、View 布局、弹幕演出
- Mortar：对话、过场动画、事件序列
- FRE 规则：游戏逻辑、状态转换
- WASM 组件：自定义子弹行为、特殊机制
- 资产：精灵图、音频、瓦片地图

---

## 核心子系统

### FRE（事实-规则-事件）

引擎的核心。将数据与行为解耦：

| 概念            | 职责               | 示例                             |
|---------------|------------------|--------------------------------|
| **Fact（事实）**  | 全局键值存储           | `"player:hp" = 20`             |
| **Event（事件）** | 标记发生的事情          | `CollisionEnter { a, b }`      |
| **Rule（规则）**  | 声明式逻辑，将事件绑定到事实变更 | `On TakeDamage → hp -= amount` |

流程：事件 → 规则评估 → 事实变更 → View 响应式更新。

### View 系统

RON 驱动的 UI，配备解析器注册表。View 响应 Fact 变化，而非命令式调用。
预设层通过 `DataPathResolvers`、`ConditionResolvers`、`ExprFunctionResolvers` 注入游戏特定数据。

### 弹幕系统

一流的 STG 支持——SoupRune 的核心竞争力：

- 内置运动：线性、轨道、正弦、缓动、静止、瞄准
- 自定义运动：通过 WASM 组件实现特殊弹幕
- 时间轴驱动的生成序列

---

## WASM 扩展模型

WASM 是面向 mod 作者的扩展点，不是 Rust 系统的替代品。

- **使用 WASM**：自定义子弹行为、特殊生成模式、mod 专有逻辑
- **保留在 Rust 中**：核心运动、碰撞、渲染、UI 布局
- **WIT 接口**：`behavior`、`danmaku`、`spawn-pattern`、`custom-action-handler`、`mode-lifecycle`、`rule-provider`

---

## 边界规则

✅ Core **可以**：定义弹幕原语、碰撞、对话渲染、View 布局、FRE 评估、通用调度

❌ Core **禁止**：导入 preset、硬编码游戏特定的 fact key、了解具体实体（Item、Enemy）、定义游戏状态机

---

## 目录结构

```
crates/
├── souprune/src/
│   ├── core/               # 引擎基础设施
│   │   ├── danmaku/        #   子弹引擎（特权模块）
│   │   ├── dialogue/       #   对话 + Mortar 集成
│   │   ├── view/           #   RON 驱动的 UI
│   │   ├── collision.rs    #   SDF 碰撞
│   │   ├── fre_bridge.rs   #   FRE ↔ ECS 桥接
│   │   └── mod_system.rs   #   WASM mod 加载
│   ├── preset/             # 游戏逻辑（战斗、大地图、物品）
│   └── app_state/          # 应用状态管理
├── bevy_fact_rule_event/   # FRE 引擎（子模块）
├── bevy_mortar_bond/       # Mortar 脚本（子模块）
├── bevy_alight_motion/     # Alight Motion + SDF（子模块）
├── souprune_api/           # WIT 接口定义
└── souprune_sdk/           # Rust WASM guest SDK
```
