# SoupRune 代码风格指南

> 本文档定义了 SoupRune 项目的代码风格与文档规范。
> 英文版请参见 [style.md](style.md)。

---

## 1. 命名规范

### 1.1 通用规则

| 项目                      | 大小写                          | 示例                                  |
|-------------------------|------------------------------|-------------------------------------|
| 模块与文件                   | `snake_case`                 | `fre_bridge`, `chapter_schema`      |
| 类型（struct, enum, trait） | `UpperCamelCase`             | `FactDatabase`, `ViewRoot`          |
| 函数与方法                   | `snake_case`                 | `evaluate_conditions`, `get_by_str` |
| 常量与静态变量                 | `SCREAMING_SNAKE_CASE`       | `MAX_ENEMIES`, `DEFAULT_SPEED`      |
| Feature flags           | `lowercase`                  | `debug`, `unsafe_gpu`               |
| 资产目录                    | `lowercase_with_underscores` | `battle_sprites/`                   |
| ECS 组件与资源               | `UpperCamelCase`             | `PlayerHealth`, `EnumRegistry`      |

以 [Rust API Guidelines — Naming](https://rust-lang.github.io/api-guidelines/naming.html) 作为基线参考。

### 1.2 公共 API：优先使用完整单词

SoupRune 面向同人游戏社区——许多贡献者是爱好者而非系统程序员。**公共 API 名称应使用完整的、描述性的单词。**

```rust
// ✅ 推荐
pub struct HealthBarSource {
    ..
}
pub fn evaluate_conditions(..) -> bool { .. }
pub enum AlightMotionEntity {..}

// ❌ 避免
pub struct HpBarSrc {
    ..
}
pub fn eval_conds(..) -> bool { .. }
pub enum AmEntity {..}
```

私有/局部代码有更大灵活性——当上下文清晰时，短名称是可以的。

### 1.3 概念内部保持一致

对同一概念选定**一种**拼写方式并始终使用。不要混合大小写或缩写：

```rust
// ❌ 不一致
HPBarSourceDef   // "HP" 全大写
HpSourceType     // "Hp" 首字母大写
DesiredHpBar     // "Hp" 又一种写法

// ✅ 一致——选一种坚持使用
HealthBarSourceDef
HealthBarSourceType
DesiredHealthBar
```

---

## 2. 缩写与术语使用规则

### 2.1 允许的缩写

以下缩写在 Rust 生态中广为人知，可在任何地方使用：

| 缩写                    | 含义                 |
|-----------------------|--------------------|
| `ctx`                 | Context（上下文）       |
| `db`                  | Database（数据库）      |
| `buf`                 | Buffer（缓冲区）        |
| `iter`                | Iterator（迭代器）      |
| `impl`                | Implementation（实现） |
| `config` / `cfg`（属性中） | Configuration（配置）  |
| `err`                 | Error（错误）          |
| `msg`                 | Message（消息）        |
| `id`                  | Identifier（标识符）    |
| `idx`                 | Index（索引，仅限局部变量）   |

### 2.2 项目特定术语

以下项目特定缩写是允许的，因为它们有文档记录且使用一致：

| 术语  | 全称                      | 用途                                     |
|-----|-------------------------|----------------------------------------|
| FRE | Fact-Rule-Event         | 系统名称、crate 名称 (`bevy_fact_rule_event`) |
| SDF | Signed Distance Field   | 渲染技术（来自 `bevy_alight_motion`）          |
| ECS | Entity Component System | Bevy 的架构                               |
| RON | Rusty Object Notation   | 数据格式                                   |

### 2.3 公共 API 中禁止使用

在**公共类型、trait 和函数名**中避免这些缩写，应使用完整单词：

| 避免                | 改用                                  |
|-------------------|-------------------------------------|
| `Mgr`             | `Manager`                           |
| `Svc`             | `Service`                           |
| `Util` / `Helper` | 描述实际用途                              |
| `Misc`            | 描述实际用途                              |
| `Info`            | 具体化：`Metadata`, `Status`, `Details` |
| 单字母前缀（`Am`, `Hp`） | 完整单词（`AlightMotion`, `Health`）      |

### 2.4 Def 后缀约定

`Def`（Definition 的缩写）在代码库中用于**从 RON 文件反序列化的纯数据结构体**。此约定已建立，应继续维护：

```rust
pub struct RuleDef {
    ..
}       // 从 .fre.ron 反序列化
pub struct ViewNodeDef {
    ..
}   // 从 .view.ron 反序列化
pub struct EnemyDef {
    ..
}      // 从敌人配置反序列化
```

> ⚠️ **重要**：在 RPG / 同人游戏语境中，"def" 也常作为 "defense"（防御力）的缩写。为避免歧义，**涉及战斗属性时必须使用 "
defense" 全称**。`Def` 后缀专用于 "Definition" 结构体。
>
> ```rust
> // ✅ 正确
> pub struct EnemyDefence { pub value: i32 }  // 防御力使用全称 "Defence"
>
> // ❌ 容易混淆
> pub struct EnemyDef { pub def: i32 }        // "def" 字段与 "Def" 后缀冲突
> ```

---

## 3. 模块与目录结构

### 3.1 目录结构

```
crates/souprune/src/
├── core/           # 引擎级系统（输入、视图、摄像机、碰撞……）
├── app_state/      # 游戏状态模块（菜单、大世界、战斗）
├── extra/          # 可选工具（调试工具、格式加载器）
└── lib.rs          # 插件注册、应用构建器
```

### 3.2 何时拆分模块

- 文件接近 **~500 总行数**时，就该停下来想一想：它现在是不是还只在做一件事
- **硬性规定：800 总行数**（由 `tokei_check.sh` 强制执行）——超过此限制**必须**拆分
- **硬性规定：`tokei` 代码行数 500**（同样由 `tokei_check.sh` 强制执行）——就算注释和空行不多，只要真实逻辑太密，也必须拆分
- 模块有**不同的职责** → 按职责拆分

### 3.3 模块风格：Rust 2018+

**禁止使用 `mod.rs` 文件。** 使用 Rust 2018+ 模块命名规范：

```
// ❌ 旧风格（2018 前）
src/core/view/mod.rs

// ✅ 新风格（Rust 2018+）
src/core/view.rs          // 模块根文件
src/core/view/            // 子模块目录
```

唯一的例外是 `examples/` 目录，因为 Cargo 将 examples 中的 `.rs` 文件视为独立二进制文件，此时 `mod.rs` 是可接受的。

此规则由 `tokei_check.sh` 强制执行。

### 3.4 文件命名

- 每个文件一个模块（或目录）
- 文件名与模块名匹配：`mod fre_bridge` → `fre_bridge.rs`；如果它有子模块，就放进 `fre_bridge/` 目录
- 测试模块紧随源码：文件底部的 `#[cfg(test)] mod tests { .. }`

### 3.5 单一职责与边界归属

**单一职责原则必须遵守。**

- 一个模块读起来应该像是在做**一件事**。如果 schema、资源加载、运行时系统、编辑器辅助逻辑全都挤在一起，这个模块就已经太宽了
- 插件根文件是装配文件。它们应该负责注册插件、系统和资源，真正的行为放进专门的子模块里
- 如果一个文件不断变大，是因为各种无关改动都往里塞，就应该按职责拆开，而不是等到行数爆炸才处理

### 3.6 分层规则

目录结构本身不算分层，依赖方向也必须符合架构。

- 不是放进 `core/` 目录就自动变成基础设施层了。如果一个模块需要知道 battle、overworld 或编辑器细节，它大概率就不该放在 `core/`
- 一个简单判断规则是：`app_state/` 可以依赖 `core/`，但 `core/` 不应该反过来伸手依赖 `app_state/`
- 编辑器 crate 应该依赖**公开的引擎 API**或**schema crate**，不要因为眼前方便就直接穿透到深层内部路径
- 状态专属行为应放在对应 game state 或专门的共享 gameplay 层，而不是塞进通用基础设施模块

### 3.7 Schema 唯一真源

同一种资产格式必须且只能有**一套权威 schema 类型**。

- 先选定一套 schema，然后把它当成唯一真源
- 不要让运行时、lint 和编辑器各自维护同一种格式的不同版本
- 运行时包装层可以添加 `Asset`、`Reflect` 或转换逻辑，但不能复制一份字段定义然后各自演化

### 3.8 内部代码的导入与重导出

- 让读代码的人一眼就能看出这个文件依赖了什么
- 生产代码中避免使用通配导入，生态约定俗成的 prelude 例如 `bevy::prelude::*` 除外
- 内部模块里少用桶式重导出（`pub use foo::*;`）；优先显式写出要暴露的项
- 只有在明确设计过的公共边界才整模块重导出，例如 crate `prelude` 或稳定的顶层 API

### 3.9 兼容与删除策略

SoupRune 处于活跃开发阶段，**不以向后兼容为优先目标**。

- 既然允许删旧设计，就应该真的去删，而不是一直背着它们前进
- 临时兼容代码一旦加进来，就必须同时写明它什么时候会被删掉
- 迁移完成后应尽快删除旧路径；不要让两套系统半死不活地长期并存
- 如果资产或规则已经迁到新格式，就应在同一阶段把旧字段名、旧事件名和旧桥接系统一起删掉
- “以后再清理” 不能作为保留死抽象或并行系统的理由

### 3.10 Bevy 插件应该长什么样

下面这些 Bevy 相关规则，来自 Bevy 官方的插件开发指导和 `bevy_best_practices` 的思路；这里不照抄原文，而是用这个仓库里能直接执行的人话写出来。

如果一个模块叫 Bevy 插件，那它读起来就应该像某个子系统的入口，而不是另一个超大杂物间。

- 插件文件应该主要做装配：注册插件、资源、资产、调度和状态钩子，然后就收手
- 不要让 `Plugin::build` 悄悄长成真正的业务逻辑区、解析逻辑区或运行时分支区
- 在库 crate 里，优先暴露一个有名字的 `Plugin` struct，而不是只有 `plugin(app)` 这种自由函数。这样以后要加配置时，不会把调用方一起拖下水
- 第三方 Bevy crate 的配置，应该跟使用它的那个子系统放在一起，而不是散落在远处的全局启动代码里

换成人话就是：别人打开一个插件文件时，应该很快看懂**这个子系统注册了什么**，而不是被迫从头倒推**这个子系统到底怎么运行**。

### 3.11 状态边界、调度与清理

Bevy 很容易让人把东西全塞进 `Update`。不要这么做。

- 进入 `Update` 的系统，通常都应该有明确的 `State`、`run_if` 或 `SystemSet` 边界
- 如果一个系统只应该在某个界面、某个模式或某场战斗中运行，就把这件事直接写在调度上，不要靠“反正大概不会触发”来碰运气
- 同一个状态的 `OnEnter`、`Update`、`OnExit` 最好放在相近位置注册，让读代码的人一眼看到它怎么启动、怎么清理
- 实体必须有清理方案。用 `StateScoped` 或显式 cleanup marker，不要先生成一堆场景实体，再赌以后会有人记得把它们删掉
- 顶层生成的实体通常都应该有 `Name`，因为没有名字的根实体只会让调试和世界检查更痛苦

一句话规则：一个状态启动了什么，附近的代码也应该明确说明它什么时候结束、怎么结束。

### 3.12 优先用事件，不要偷偷紧耦合

两个子系统如果只是要互相通知，就优先用事件，不要直接互相掏内部状态。

- 子系统之间传请求、结果和事实变化时，优先用事件，不要让一个系统跨很多层去摸另一个系统的内部细节
- 如果事件写入方和读取方要求同帧配合，就用 `.before(...)`、`.after(...)`、`.chain()` 或有顺序的 `SystemSet` 把顺序写清楚
- 如果一个系统存在的意义只是响应事件，就把这件事体现在调度上，不要让它每帧空跑
- 事件名和 payload 应该描述领域含义，而不是描述一时的实现细节

说白了：事件是连接系统时最便宜、最诚实的方式；隐形跨模块假设不是。

---

## 4. 公共 API 设计原则

### 4.1 函数命名

执行动作的函数使用**动词 + 名词**模式：

```rust
pub fn evaluate_conditions(..) -> bool { .. }
pub fn register_rules(..) { .. }
pub fn spawn_view_entity(..) -> Entity { .. }
```

纯 getter 使用**名词**模式：

```rust
pub fn active_view(&self) -> Option<Entity> { .. }
pub fn fact_count(&self) -> usize { .. }
```

### 4.2 Trait 命名

- 能力型 trait：使用形容词或 `-able` 后缀 → `FactReader`, `Serializable`
- 行为型 trait：使用动词形式 → `Evaluate`, `Dispatch`

### 4.3 避免模糊名称

不要创建名为 `util`、`helper`、`common` 或 `misc` 的模块或类型。用它们实际做的事情来命名。

---

## 5. 文档注释规范

### 5.1 双语文档

所有文档必须**双语（英文 + 中文）**。先英文，后中文翻译：

```rust
/// Evaluates all conditions in a rule against the current facts.
///
/// 根据当前 facts 评估规则中的所有条件。
pub fn evaluate_conditions(..) -> bool { .. }
```

模块级文档：

```rust
//! # fre_bridge
//!
//! ## Module Overview
//!
//! Bridge between the game and the FRE (Fact-Rule-Event) system.
//!
//! ## 模块概述
//!
//! 游戏与 FRE（Fact-Rule-Event）系统之间的桥接层。
```

### 5.2 文档结构

对于复杂项目，使用以下结构：

1. **单行摘要**（英文）
2. **单行摘要**（中文）
3. 扩展说明（如需要，双语）
4. `# Examples` 部分（如适用）

保持文档**简洁**——避免重述代码已经表达清楚的内容。链接到外部资源而非在文档中详细解释常见概念。

### 5.3 行内注释

谨慎使用行内注释。使用时，非平凡逻辑推荐双语：

```rust
// Enum resolution: Int vs String → try enum lookup
// 枚举解析：Int vs String → 尝试枚举查找
```

---

## 6. 代码风格

### 6.1 格式化工具

使用 `rustfmt` 默认设置。除非团队达成一致，不设置 `rustfmt.toml` 覆盖项。

每次提交前运行：

```bash
cargo fmt --all
```

### 6.2 Clippy

Clippy 配置（`clippy.toml`）：

```toml
cognitive-complexity-threshold = 20
excessive-nesting-threshold = 4
too-many-lines-threshold = 120
too-many-arguments-threshold = 20
type-complexity-threshold = 1000
```

**禁止使用 `#[allow(clippy::...)]`。** 应使用 `#[expect(clippy::...)]` 并附带必需的 `// reason:` 注释。此规则由
`tokei_check.sh` 强制执行。

```rust
// ❌ 禁止
#[allow(clippy::too_many_arguments)]
fn complex_function(..) { .. }

// ✅ 可接受（仅在有合理理由时）
#[expect(clippy::too_many_arguments)]
// reason: Bevy 系统参数需要所有这些参数；重构为 SystemParam 已记录在 #123
fn complex_system(..) { .. }
```

即使 `#[expect]` 也不应滥用——它是最后手段。优先修复根本问题：

- 嵌套过深 → 提取辅助函数、使用 `let-else`、使用迭代器方法
- 参数过多 → 使用配置结构体或 builder 模式

### 6.3 导入排序

按以下顺序分组导入，组间用空行分隔：

1. `mod` 声明和 `pub use` 重导出
2. 标准库（`std::`）
3. 外部 crate（`bevy::`、`serde::` 等）
4. 内部 crate（`crate::`、`super::`）

```rust
mod eval;
pub use eval::register_condition_evaluator_system;

use std::collections::HashMap;

use bevy::prelude::*;
use bevy_fact_rule_event::{EnumRegistry, FactReader};

use crate::core::view::components::ViewRoot;
```

### 6.4 嵌套

最大嵌套深度为 **4 层**。使用以下方式扁平化：

- `let-else` 绑定
- 提前返回 / 守卫语句
- `if-let` 链（`if let Some(x) = a && let Some(y) = b { .. }`）
- 提取辅助函数

```rust
// ❌ 嵌套过深
if let Some(a) = get_a() {
if let Some(b) = get_b() {
if condition {
do_something(a, b);
}
}
}

// ✅ 扁平化
let Some(a) = get_a() else { return };
let Some(b) = get_b() else { return };
if condition {
do_something(a, b);
}
```

### 6.5 Match 分支

使用一致的格式。简短分支可保持一行；复杂分支使用代码块：

```rust
match value {
FactValue::Int(v) => FactValue::Int( * v),
FactValue::String(v) => FactValue::String(v.clone()),
FactValue::Bool(v) => {
info ! ("Processing bool: {}", v);
FactValue::Bool( * v)
}
}
```

---

## 7. 错误处理与 Result 类型约定

### 7.1 何时使用 Result vs Option

- `Result<T, E>` — 操作可能失败并有明确的错误信息
- `Option<T>` — 值可能存在也可能不存在（无错误语义）

### 7.2 错误类型

对于 crate 级错误，定义专用的错误枚举：

```rust
#[derive(Debug, thiserror::Error)]
pub enum FreError {
    #[error("Unknown enum variant '{variant}' for group '{group}'")]
    UnknownEnumVariant { group: String, variant: String },
}
```

### 7.3 Panic

- **库代码**：避免 panic。返回 `Result` 或 `Option`。
- **应用代码**：对于真正不可恢复的状态，panic 是可接受的。
- **资源加载**：使用 `warn!` + 合理默认值，而非在数据格式错误时 panic。

---

## 8. 日志与调试输出

### 8.1 日志级别

| 级别       | 用途                 |
|----------|--------------------|
| `error!` | 破坏功能的不可恢复故障        |
| `warn!`  | 意料之外但可恢复的情况        |
| `info!`  | 重要的状态变更（场景切换、资源加载） |
| `debug!` | 开发调试信息（规则评估详情）     |
| `trace!` | 非常细粒度的数据（逐帧数值）     |

### 8.2 规则

- 库代码中**绝不**使用 `println!` 或 `eprintln!`。使用 `bevy::log` 宏。
- 日志消息包含上下文：`info!("FRE Bridge: SetLocalFact({}, {:?})", key, value)`
- 详细日志通过 `#[cfg(feature = "debug")]` 或 `debug!`/`trace!` 级别控制。

---

## 9. 测试代码规范

### 9.1 测试位置

- 单元测试：源文件底部的 `#[cfg(test)] mod tests { .. }`
- 集成测试：crate 根目录的 `tests/` 目录（如需要）

### 9.2 测试命名

使用描述性名称，说明被测试的行为：

```rust
#[test]
fn enum_registry_resolves_known_variant() { .. }

#[test]
fn evaluate_conditions_returns_false_when_fact_missing() { .. }
```

### 9.3 测试结构

- ECS 测试使用最小化的 `bevy::prelude::App` 配置
- 优先使用**确定性断言**——避免帧数依赖
- 每个测试函数测试一个行为

### 9.4 Workspace 级质量门槛

风格规则必须能在整个 workspace 中被执行，而不只是主 crate。

- 规则只有真正被仓库检查到时，才算规则
- 行数检查、lint 检查与结构检查应覆盖所有自维护的一方 crate
- 如果某个子 crate 自带质量脚本，根仓质量入口就应该调用它，而不是静默跳过
- 任何无法在 CI 中检查的规则，都应视为建议而不是硬性规定

---

## 10. 文档语言与多语言规范

### 10.1 两份规范文档

| 文件                     | 语言   | 用途     |
|------------------------|------|--------|
| `doc/style.md`         | 英文   | 主要风格指南 |
| `doc/style_zh-hans.md` | 简体中文 | 中文翻译版  |

两份文档必须保持**相同的结构**（相同的章节、相同的编号）。更新一份时，同步更新另一份。

> ⚠️ 目前没有专门的本地化负责人。因此，**所有文档变更必须同时更新所有语言版本**。此策略可能会随着团队扩大而调整。

### 10.2 代码注释与文档

- 源码文档（`///`、`//!`）为**双语**——先英文后中文
- 行内注释（`//`）非平凡逻辑推荐双语
- 提交信息使用**英文**（conventional commit 格式）
- RON 文件注释（`//`）可使用任一语言

### 10.3 README

- `readme.md` — 英文
- `readme_zh-hans.md` — 简体中文
