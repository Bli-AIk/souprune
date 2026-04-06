# 参与贡献 🥣

感谢你对 SoupRune 的关注！不论你是游戏创作者、美术创作者、音乐人、翻译者还是开发者——这里都有属于你的位置。

每种贡献都能让 SoupRune 变得更好！我们不按"难度"给贡献排序——而是按**你想做什么**来区分。

**不知道从哪开始？** 来 [Discord](https://discord.gg/5YXK5DRjPZ) 和大家一起聊聊吧！

| 英语                           | 简体中文 |
|------------------------------|------|
| [English](./CONTRIBUTING.md) | 简体中文 |

---

## 📋 目录

- [行为准则](#-行为准则)
- [Bug 反馈与建议](#-bug-反馈与建议)
- [路径 A：社区与创作](#-路径-a社区与创作)
- [路径 B：框架核心开发](#-路径-b框架核心开发)
- [路径 C：生态与工具链](#-路径-c生态与工具链)
- [开发流程](#-开发流程)
- [许可证与 CLA](#-许可证与-cla)

---

## 📜 行为准则

请阅读并遵守我们的[行为准则](./CODE_OF_CONDUCT.md)（基于 Contributor Covenant 3.0）。我们是一个连接 UTDR 同人社区和
Rust/Bevy 生态的社区——不同背景之间的互相尊重至关重要。

---

## 🐛 Bug 反馈与建议

一份好的 Bug 报告或功能建议，和代码贡献一样有价值。这是改善 SoupRune 最直接的方式。

**任何人都可以参与——不需要编程知识。**

### 报告 Bug

使用 [Bug 报告模板](https://github.com/Bli-AIk/souprune/issues/new?template=bug-report.md)，并包含：

- SoupRune 版本和你的操作系统
- 清晰的复现步骤
- 你期望的结果 vs. 实际发生的情况
- 错误信息、日志或截图

### 提出功能建议

使用[功能请求模板](https://github.com/Bli-AIk/souprune/issues/new?template=feature-request.md)，描述：

- 你想解决的问题或改善的工作流
- 你的建议方案，以及你考虑过的替代方案

### 提出重构建议

使用[重构请求模板](https://github.com/Bli-AIk/souprune/issues/new?template=refactor-request.md)来提出代码结构改进。

### 不确定是不是 Bug？

先来 [Discord](https://discord.gg/5YXK5DRjPZ) 聊聊——我们很乐意帮你弄清楚！

---

## 🎮 路径 A：社区与创作

**适合**：游戏创作者、美术/音乐创作者、翻译者、Mod 开发者、文档贡献者

**核心理念**：用 SoupRune 创作你自己的 Mod，或为社区打磨公共前置 Mod。

### A1. 制作你的 Mod

SoupRune 是为创作 Deltarune / Undertale 风格同人游戏而生的框架。你可以：

- **Mortar 脚本** — 用 Mortar（SoupRune 内置的脚本语言）编写对话和事件逻辑
- **FRE 规则** — 使用 FRE（Fact-Rule-Event，事实-规则-事件）系统，用数据驱动的方式定义游戏逻辑
- **View 布局** — 通过 RON（Rusty Object Notation）配置文件设计 UI
- **关卡设计** — 使用 [Tiled](https://www.mapeditor.org/) 地图编辑器创建地图
- **WASM Mod** — 用任何能编译到 WebAssembly 的语言来构建 Mod

📚 **入门指南**：参见 [Mod 开发指南](./doc/mod_development.md)
和[示例 Mod 仓库](https://github.com/Bli-AIk/souprune_example_mods)。

### A2. 贡献社区 Mod

- 为社区制作高质量的 Mod 模板或前置 Mod
- 贡献可复用的游戏资源包
- 编写 Mod 开发教程和最佳实践

### A3. 美术与音频

- 精灵、UI、动画资源
- 音乐和音效
- 命名规范：`小写字母_下划线分隔`
- 放置目录：`projects/<mod_name>/assets/<category>`

### A4. 文档与翻译

- 修正错别字、改善措辞、补充遗漏的说明
- 翻译文档和 README（我们维护英文和简体中文两个版本）
- 在 Discord 帮助他人——社区支持也是一种贡献！

> 🎮 **路径 A 不需要签署 CLA。** 你在 `projects/` 目录下创建的 Mod 和游戏完全属于你自己。SoupRune 的架构设计保证了 LGPL 的"
> 传染性"不会触及你的创作——你通过明确定义的接口（WIT、RON、Mortar 沙盒）与框架交互。你可以自由选择你的作品的许可证。

---

## 🔧 路径 B：框架核心开发

**适合**：Rust 开发者（Rustaceans）

**你需要**：Rust 基础、了解 [Bevy](https://bevyengine.org/) 和 ECS（Entity-Component-System，实体-组件-系统）架构

### 如何开始

1. **搭建开发环境**：参见 README 中的[快速开始](./readme_zh-hans.md#-快速开始)
2. **阅读代码风格指南**：[`doc/style.md`](./doc/style.md) — 写代码前必读
3. **了解架构**：[`doc/architecture.md`](./doc/docs/architecture.md) — SoupRune 的整体结构

### 可以参与的方向

- `src/core/` — 核心系统（animation, camera, collision, danmaku, input, view, FRE bridge）
- `src/app_state/` — 应用状态（AppSetup → Menu → Overworld → Battle）
- `src/extra/` — 扩展工具（Markdown, TOML, Mortar 加载器, debug 工具）
- `crates/souprune_api` / `crates/souprune_sdk` — WASM 宿主端接口

### 寻找 Issue

查看标记为 [`good first issue`](https://github.com/Bli-AIk/souprune/labels/good%20first%20issue) 或 [
`help wanted`](https://github.com/Bli-AIk/souprune/labels/help%20wanted) 的 Issue。

### 学习资源

刚接触 Rust 或 Bevy？以下是推荐资源（我们不自己写教程——这些更专业）：

- [The Rust Programming Language](https://doc.rust-lang.org/book/)（Rust 官方书籍）
- [Rust 语言圣经](https://course.rs/)（中文 Rust 教程）
- [Bevy Book](https://bevyengine.org/learn/book/introduction/)（Bevy 官方指南）
- [Unofficial Bevy Cheat Book](https://bevy-cheatbook.github.io/)（非官方 Bevy 速查手册）

> ⚖️ **路径 B 需要签署 CLA。** 向核心代码库（`crates/`）提交 PR 需要签署[贡献者许可协议](./CLA.md)
> 。详见[许可证与 CLA](#%EF%B8%8F-许可证与-cla)。

---

## 🧩 路径 C：生态与工具链

**适合**：Bevy 生态贡献者、工具开发者、跨语言绑定开发者

### C1. Bevy Crate 家族

以下独立 Bevy crate 由同一团队（Bli-AIk）维护，作为 git submodule 集成到 SoupRune。每个 crate 有自己的仓库和 Issue 跟踪：

| Crate                    | 描述                            | 仓库                                                        |
|--------------------------|-------------------------------|-----------------------------------------------------------|
| **bevy_mortar_bond**     | Mortar 脚本语言的 Bevy 绑定（对话与事件系统） | [GitHub](https://github.com/Bli-AIk/bevy_mortar_bond)     |
| **bevy_fact_rule_event** | FRE 数据驱动规则引擎                  | [GitHub](https://github.com/Bli-AIk/bevy_fact_rule_event) |
| **bevy_ecs_typewriter**  | 打字机文本效果                       | [GitHub](https://github.com/Bli-AIk/bevy_ecs_typewriter)  |
| **bevy_alight_motion**   | Alight Motion 动画导入与 SDF 渲染    | [GitHub](https://github.com/Bli-AIk/bevy_alight_motion)   |
| **bevy_bitmap_text**     | 位图字体渲染                        | [GitHub](https://github.com/Bli-AIk/bevy_bitmap_text)     |

**请到对应的独立仓库提交 Issue 和 PR**，而不是 SoupRune 主仓库。改进这些 crate 会同时惠及 SoupRune 和其他使用它们的项目。

### C2. 编辑器 (souprune_editor)

> ⚠️ **实验性项目** — 编辑器目前不是开发重点，但这是一个非常有趣的探索方向。

- 基于 Bevy + egui 构建
- 当前状态：概念验证阶段
- 可贡献方向：UI 原型、工具面板实验、可视化调试探索
- 注意：API 可能频繁变动

### C3. SDK 与多语言绑定

- `souprune_sdk` — Rust WASM guest SDK
- C# (.NET AOT) 绑定
- Haxe 绑定
- **想添加新语言支持？** Go、Python、Zig 等都欢迎！

### C4. CI / 构建 / 工具链

- 构建优化
- CI 流水线改进
- 开发者体验工具（justfile 配方、脚手架等）

> ⚖️ **路径 C 需要签署 CLA**（针对 Bli-AIk 维护的 crate）。详见[许可证与 CLA](#%EF%B8%8F-许可证与-cla)。

---

## 🔄 开发流程

### Fork & Branch

1. Fork 本仓库
2. 创建特性分支：`git checkout -b feat/my-feature`
3. 初始化子模块：`git submodule update --init --recursive`

### 提交规范

我们使用 [Conventional Commits](https://www.conventionalcommits.org/)：

```
feat(battle): add new damage calculation system
fix(overworld): resolve NPC collision edge case
docs(readme): update quick start section
refactor(view): simplify layout parsing logic
chore(deps): bump bevy to 0.18
```

### 提交 PR 前

运行完整的检查套件：

```bash
cargo fmt --all                                          # 格式化
cargo clippy --workspace --all-targets -D warnings       # 静态检查
cargo test --workspace                                   # 测试
```

### PR 检查清单

- [ ] 代码编译无警告（`cargo clippy`）
- [ ] 所有测试通过（`cargo test --workspace`）
- [ ] 代码已格式化（`cargo fmt --all`）
- [ ] PR 描述说明了做了什么以及为什么
- [ ] 关联了相关 Issue（如适用）
- [ ] 标注了破坏性变更（如适用）

---

## ⚖️ 许可证与 CLA

### 双重授权模式

SoupRune 采用**双重授权**模式：

| 用户类型 | 许可证          | 说明                        |
|------|--------------|---------------------------|
| 开源用户 | **LGPL-3.0** | 免费使用和修改框架；对核心的修改必须开源      |
| 商业用户 | **商业许可证**    | 允许在闭源环境下修改核心框架代码（如主机平台移植） |

这一模式在开源界有成熟先例（Qt、MySQL 等）。

### 谁需要签署 CLA？

**分界线是架构边界，不是个人身份。**

```
┌──────────────────────────────────────────────────┐
│  projects/ — Mod 与游戏项目                       │
│  ❌ 不需要 CLA                                    │
│  你的游戏通过 WIT / RON / Mortar 沙盒接口         │
│  与 SoupRune 交互。LGPL 的传染性在架构边界处       │
│  被完全阻断。你可以自由选择你的作品的许可证。      │
├──────────────────────────────────────────────────┤
│  crates/ — 框架核心 / Preset / SDK               │
│  ✅ 必须签署 CLA                                  │
│  任何提交到 crates/ 的 Rust 代码 PR              │
│  都需要签署贡献者许可协议。                       │
└──────────────────────────────────────────────────┘
```

### 📝 关于代码所有权与商业授权的透明说明

SoupRune 致力于成为一个长期、稳定、由社区驱动的次世代游乐场。为了保证框架的健康发展，我们要求所有向核心仓库提交代码的开发者签署
CLA（贡献者许可协议）。

我们希望在此坦诚地向你解释这背后的原因及资金运作方式：

#### 1. 为什么要签署 CLA？

对于基于 SoupRune 开发游戏的创作者（Project / Mod 层），你可以 **100% 拥有你的游戏并自由处置**，无需任何授权。

但对于 SoupRune 核心框架，我们采用 LGPL-3.0 协议。这意味着如果未来有商业团队希望将 SoupRune 用于闭源主机平台（如 Switch /
PS5，受限于平台机制无法动态链接），他们需要购买"商业闭源授权"。CLA 赋予了核心维护团队发放此类授权的法律权利。

坦白说，这在短期内真的只是一种可能性——但我们需要为这种可能性做好法律准备。毕竟，选择一个开源且可持续的框架，总比被锁定在某些昂贵且前途未卜的商业引擎里好得多。

#### 2. 商业授权的收入如何处理？

由于追踪具体代码行的商业价值并按比例分成的管理成本过高，我们目前不提供针对单个 PR 的直接资金分成。

签署 CLA 意味着你同意将代码权利授权给项目组。如果有幸产生商业授权收入，这笔资金将全额归属 SoupRune 核心维护团队（目前为
Bli-AIk），并用于以下途径：

- 💰 **基础设施开销**：服务器、域名、CI/CD 等硬性成本
- 🛠️ **核心维护**：资助核心维护者的开发时间，确保 Issue 能被及时处理，文档能持续更新
- 🏆 **社区悬赏**（未来规划）：当资金充足时，设立 Bounties，对解决复杂核心 Issue 的开发者给予直接的资金奖励

#### 3. 你的贡献意味着什么？

你的每一行代码都是在为那些无法支付高昂商业引擎费用的同人创作者铺路。感谢你的理解与决心！

#### 签署 CLA 意味着什么

- ✅ 你**保留你的版权**
- ✅ 你授予 SoupRune 使用和**再许可**你的贡献的权利
- ✅ 你确认你有权提交该代码
- ✅ 你的贡献仍然在 LGPL-3.0 下对所有开源用户开放

#### 签署 CLA 不意味着什么

- ❌ 不转让你的版权
- ❌ 不限制你在其他项目中使用相同代码
- ❌ 不影响你在开源社区的任何权利

📄 **阅读完整 CLA**：[CLA.md](./CLA.md)

---

*感谢你为 SoupRune 贡献力量！我们正在一起为同人游戏社区创造一些特别的东西。🥣*
