# 架构指南

> 英文版请参阅 [architecture.md](architecture.md)。

本文档描述 SoupRune 当前的内部架构。

---

## 总览

SoupRune 是构建在 Bevy 之上的框架。本体已经不再存在编译进二进制的
`preset/` 层。分发的二进制只包含通用框架运行时；具体游戏语义由项目内容与
项目 WASM runtime 表达。

```
┌──────────────────────────────────────────────┐
│  项目层                                      │
│  projects/<project>/                         │
│  content crate 生成的 RON、Mortar、FRE 规则、 │
│  资源，以及 WASM runtime 玩法逻辑             │
├──────────────────────────────────────────────┤
│  框架层                                      │
│  crates/souprune/src/core/                   │
│  View、FRE bridge、Mortar 集成、弹幕、碰撞、  │
│  输入、模式运行时、schema-backed 内容加载     │
├──────────────────────────────────────────────┤
│  Schema 与 SDK 层                            │
│  crates/souprune_schema、souprune_api、       │
│  souprune_sdk、cauld-ron 工具链              │
└──────────────────────────────────────────────┘
```

依赖方向必须保持清晰：框架代码定义通用 host primitive 与 schema；项目内容和
项目 WASM runtime 把这些 primitive 组合成具体游戏。本体 Rust 代码禁止重新出现
`preset` 或 `host_runtime` 层。

---

## 框架运行时

`crates/souprune/src/core/` 包含可复用基础设施：

- `input/`：统一输入事务与按键配置。
- `view/`：RON 声明式 UI、`LocalState`、SDF 形状、文本与 reconcile。
- `fre_bridge/`：在 ECS 与 FRE 之间路由输入、View action、碰撞、自定义 action
  和 fact 写入。
- `sequencer/`：章节式流程控制、`Custom` action、`RunSequence`、`LoadMap`、
  对话与 View 章节。
- `danmaku/`：弹幕演出、时间线、内建运动 primitive 与 WASM 行为接入点。
- `collision/`：暴露给项目 runtime 的宿主碰撞 primitive。
- `mod_system/`：WASM 加载、行为分发、自定义 action 分发、host entity
  primitive 与音频副作用。
- `battle_runtime/`：通用 battle 模式调度、相机/输入初始化、sequencer/FRE 接线、
  弹幕集成与战斗对白表现。
- `overworld/`：通用俯视角模式运行时，包括地图、玩家移动、交互区和地图作用域
  FRE 集成。
- `content/`：当前项目格式需要的 schema-backed 资源加载与 fact 投影。

有些模块仍会出现 item、enemy、battle、overworld 等 RPG 词汇，因为这些是当前框架
支持的 schema 表面与运行时模式。边界不由词汇本身决定，而由职责归属决定：框架可以
加载类型化数据、投影 facts、提供通用模式胶水；物品使用效果、敌人回合选择、
BattleBox、玩家生成语义等项目玩法规则必须位于项目内容或项目 WASM runtime。

---

## 项目层

`projects/<project>/` 是 mod 作者面对的表面。

| 表面 | 归属 | 用途 |
| --- | --- | --- |
| `content/src/**` | 项目 content crate | 通过 cauld-ron 生成 RON 文件。 |
| `.ron` artifacts | 生成产物 | View 布局、sequence、规则、数据资源。 |
| `.mortar` | 项目脚本 | 对话树与文本脚本。 |
| `runtime/src/**` | 项目 WASM runtime | 自定义行为、自定义 action 与玩法语义。 |
| assets | 项目 | 精灵、音频、瓦片地图、Alight Motion 文件。 |

`projects/` 下的 RON 文件是生成产物。修改内容时应修改对应 content crate 源码，
重新构建后由 cauld-ron 输出 RON。

---

## 边界示例

框架拥有：

- `core::collision::region::CollisionRegion` 与移动约束。
- `core::mod_system` 中从 WASM 生成 sprite 或 view box 的 host entity primitive。
- `core::sequencer::Chapter::Custom` 与自定义 action 分发。
- `core::battle_runtime::BattleUpdate` 以及 battle 相机/输入初始化。
- `core::content::enemy::EnemyDef` 加载与 fact 投影。

项目 runtime 拥有：

- `SpawnBattleBox`、`SplitBattleBox`、`MergeBattleBoxes` 自定义 action。
- `SpawnBattlePlayer` 与红心行为。
- `UseItem`、`CheckItem`、`DropItem`。
- 敌人回合选择策略。
- Boss 专属弹幕行为与生成模式。

这样的拆分让二进制保持通用，同时允许前置项目通过自己的 runtime 模块提供完整玩法。

---

## 核心系统

### FRE

FRE 是数据驱动规则引擎。事件触发规则，规则修改 facts，View 与 runtime 系统响应
这些 facts。FRE 是框架机制，不是承载所有玩法行为的总管。属于项目的行为应通过
`Custom` action 分发给项目 WASM。

### View

View 布局由 RON 声明。View 拥有自己的 `LocalState`；外部系统通过受控快照读取，
并且只能通过显式命令或带作用域的 custom-action 副作用写入。View 局部状态不应再
被 FRE 隐式接管。

### 输入

输入以统一输入事务进入框架。View、FRE 与 WASM behavior 消费同一种事务模型，
但各自桥接层可以把它翻译为自己的本地命令。直接读取原始按键不再作为长期扩展点。

### WASM Runtime

项目 WASM runtime 是自定义行为扩展点。它可以读写 facts、响应输入 envelope、生成
host primitive、播放音频并处理 custom action。高频可复用 primitive 留在 Rust；
项目语义属于 WASM/content。

---

## 守卫规则

框架 Rust 允许：

- 通用渲染、输入、音频、碰撞、弹幕、对话、View、FRE 与 sequencer 基础设施。
- 当格式是框架支持的作者表面时，提供类型化 schema-backed 资源加载。
- 暴露给 WASM 的通用 host primitive。

框架 Rust 禁止：

- `crates/souprune/src/preset*` 或 `crates/souprune/src/host_runtime*`。
- 为已删除 preset 入口保留历史兼容别名。
- 把项目专属玩法命令硬编码进二进制。
- 在 `core/` 中保留 BattleBox/BattlePlayer 玩法抽象。
- 把物品使用或敌人回合选择行为放在项目 runtime 之外。

`scripts/check_core_boundaries.sh` 与 `crates/souprune/tests/architecture_boundaries.rs`
会检查这些边界中最关键的部分。
