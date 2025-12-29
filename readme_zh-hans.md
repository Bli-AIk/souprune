# SoupRune

[![license](https://img.shields.io/github/license/Bli-AIk/souprune)](LICENSE.md) <img src="https://img.shields.io/github/repo-size/Bli-AIk/souprune.svg"/> <img src="https://img.shields.io/github/last-commit/Bli-AIk/souprune.svg"/> <br>
<img src="https://img.shields.io/badge/Deltarune / Undertale-black?style=for-the-badge&logo=undertale&logoColor=ff0000" /> <img src="https://img.shields.io/badge/Bevy-232326?style=for-the-badge&logo=bevy&logoColor=white" /> <br>
<img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" />
<img src="https://img.shields.io/badge/C%23-239120?style=for-the-badge&logo=csharp&logoColor=white" />
<img src="https://img.shields.io/badge/Haxe-EA8220?style=for-the-badge&logo=haxe&logoColor=white" />
<img src="https://img.shields.io/badge/Nim-即将支持-FFE953?style=for-the-badge&logo=nim&logoColor=black" />
<img src="https://img.shields.io/badge/Nelua-即将支持-9CA3AF?style=for-the-badge&logo=lua&logoColor=white" />

> **状态**：🚧 初始开发阶段（框架结构仍在快速演进中）

[![](https://dcbadge.limes.pink/api/server/5YXK5DRjPZ)](https://discord.gg/5YXK5DRjPZ)

**SoupRune** 是一个专为创作类似 **[Deltarune](https://deltarune.com/) / [Undertale](https://undertale.com/)** 的
RPG / STG 游戏而设计的实验性游戏框架。

| 英语                     | 简体中文 |
|------------------------|------|
| [English](./readme.md) | 简体中文 |

## 🥣 简介

嘿，别被上面那个 **「Rust」** 徽章吓到 —— **SoupRune 可不只是给某些专业程序员玩的冷门工具！**

为什么这么说呢？因为：

### SoupRune 是语言无关的框架！

SoupRune 采用了 **框架 - Project (Mod)** 架构。通过 `bevy` 构建的核心引擎负责底层运行，而您的游戏逻辑则位于独立的
"Project" 中，通过标准接口与核心交互。

我们通过 `C ABI` 使使得 SoupRune 能够真正做到 **语言无关（Language-Agnostic）**。在语言支持的选择上，我们优先考虑了对 **传统
Undertale / Deltarune 开发者友好** 的语言，旨在为来自 **GameMaker** (Haxe)、**Lua** (Nelua) 或 **Python / GDScript** (Nim)
背景的开发者搭建一座桥梁。

我们热烈欢迎来自各个语言社区的开发者加入，共同完善这些语言的支持！

### SoupRune 是面向社区的框架！

SoupRune——它名字里的 **「Rune」** 不是随便取的。

Undertale 的 Fangame 框架兴许已经饱和，但 Deltarune 社区正在蓬勃发展。

SoupRune 就是趁此机会打造的 Fangame 框架，
~~同时顺带充当 [Undertale Changer Template](https://github.com/Bli-AIk/Undertale-Changer-Template) 的精神续作。~~

但这也不意味着 SoupRune 只是一个 Deltarune 框架 —— 对于 Undertale Fangame 的支持，我们也同样看重！

SoupRune 的目标是成为真正意义上的 **「面向社区的 Fangame 框架」** —— 开放、灵活、现代，同时依然保持那个熟悉的 DR / UT 风格。

### SoupRune 是开源的框架！

SoupRune 采用 **LGPL-3.0** 许可协议开源发布。该许可证仅适用于框架核心代码。

这意味着：

* 你开发的 Project (Mod) 可以使用其他开源许可证，也可以闭源；
* 你可以在闭源项目中使用 SoupRune 的源代码；
* 但若你修改了框架核心代码，需将这些修改以 LGPL 方式开源；
* 若你希望在完全闭源环境下使用修改版 SoupRune 核心代码，可联系我获取商业许可。

此外，仍需遵守 Toby Fox 对粉丝游戏的相关使用规定。

### SoupRune 是实验性的框架！

尽管实验性意味着不稳定和不完善，但它也代表着无限的可能性。

用它来做 dr / ut 的 fangame？或者试试还原其他的经典游戏？甚至开发全新的原创游戏？ 一切皆有可能！

SoupRune 目前仍处于 **初始开发阶段**，框架结构和设计理念仍在快速演进中。
我们欢迎社区成员积极参与讨论和贡献，共同塑造这个框架的未来。

加入我们的 [Discord](https://discord.gg/5YXK5DRjPZ) 吧！

## 🧭 S.O.U.P 原则

是的，**Soup**Rune当然是一个双关啦——

|  缩写   | 全称                | 含义                                  |
|:-----:|:------------------|:------------------------------------|
| **S** | **Strong**        | 基于 **Bevy** 与 **Rust**，性能强劲、架构现代。   |
| **O** | **Open**          | 采用 **LGPL** 开源协议，允许自由使用与拓展。         |
| **U** | **User-friendly** | 提供多语言脚本层，降低上手门槛。                    |
| **P** | **Polyglot**      | 支持多种编程语言（Rust、C#、Haxe、Nim、Nelua 等）。 |

## ⚙️ 技术基础

* 核心使用 **Bevy 引擎** 与 **Rust 语言** 实现，保证性能与可扩展性；
* 设计目标是：**结构清晰、可模块化扩展、易于定制**；

## 🏗️ 项目架构

SoupRune 采用多 Crate 的工作空间架构：

| Crate                                                   | 描述                                            |
|:--------------------------------------------------------|:----------------------------------------------|
| [`souprune`](./crates/souprune)                         | **核心框架**：框架本体，游戏的主入口和核心逻辑实现。                  |
| [`souprune_api`](./crates/souprune_api)                 | **协议层**：定义了 Project (Mod) 与框架核心交互的接口标准。       |
| [`souprune_sdk`](./crates/souprune_sdk)                 | **开发工具包**：对 API 的封装，提供给外部 Project (Mod) 脚本使用。 |
| [`souprune_mod_test`](./crates/souprune_mod_test)       | **样例 Mod**： 脚本系统的示例测试库。                       |
| [`bevy_mortar_bond`](./crates/bevy_mortar_bond)         | **功能插件**：Mortar 脚本语言与 Bevy 的桥接层，负责对话与逻辑。      |
| [`bevy_ecs_typewriter`](./crates/bevy_ecs_typewriter)   | **功能插件**：基于 ECS 的打字机实现，支持富文本与多语言。             |
| [`bevy_fact_rule_event`](./crates/bevy_fact_rule_event) | **功能插件**：基于“事实-规则-事件”模型的复杂事件系统。               |

## 🧩 脚本层支持

SoupRune 的脚本系统基于 **C ABI** 构建，以实现高性能互操作。我们精心挑选了一系列支持 **AOT 编译**
的语言，旨在对应不同的开发范式，让来自其他引擎的开发者能平滑迁移经验：

|           语言           | 适用人群                                 | 描述                                     |
|:----------------------:|:-------------------------------------|:---------------------------------------|
|        **Rust**        | 系统层开发者 / Rustacean / Bevy 用户         | 原生支持，性能最佳。                             |
|     **.NET (C#)**      | Unity / Godot / C# 用户                | 工业标准语言。通过 **Native AOT** 技术实现无缝集成与高性能。 |
|        **Haxe**        | **Haxe** 用户 / **GameMaker** 用户       | 强大的高级语言。其语法与 GML 有相似之处，是开发的绝佳选择。       |
|  **Nim**(Coming Soon)  | **Python** / **GDScript** (Godot) 用户 | 类似 Python 的缩进式语法，却能编译成 C 代码，兼具优雅与高效。   |
| **Nelua**(Coming Soon) | **Lua** 用户                           | 继承了 Lua 的极简语法风格，但编译为原生机器码，提供极致性能。      |

如果你有兴趣为 SoupRune 的多语言支持提供帮助，欢迎参与贡献！

## 引用说明

本项目使用了以下开源项目作为库、依赖或参考：

### 原作

| 项目                                  | 描述           |
|-------------------------------------|--------------|
| [Undertale](https://undertale.com/) | UNDERTALE 原作 |
| [Deltarune](https://deltarune.com/) | DELTARUNE 原作 |

### 前身

| 项目                                                                                  | 版本    | 许可证                                                                                                       | 功能                                                              |
|-------------------------------------------------------------------------------------|-------|-----------------------------------------------------------------------------------------------------------|-----------------------------------------------------------------|
| [Undertale-Changer-Template](https://github.com/Bli-AIk/Undertale-Changer-Template) | 1.0.7 | [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | SoupRune 是 Undertale-Changer-Template 的 **精神续作**，延续了其核心理念与设计哲学。 |

### 游戏引擎核心

| 项目                                    | 版本     | 许可证                                                                                                                                                                                                           | 功能     |
|---------------------------------------|--------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|--------|
| [Bevy](https://crates.io/crates/bevy) | 0.17.2 | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | 游戏引擎核心 |

### bevy 插件生态

| 项目                                                                        | 版本                                                                                                                             | 许可证                                                                                                                                                                                                           | 功能                                     |
|---------------------------------------------------------------------------|--------------------------------------------------------------------------------------------------------------------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|----------------------------------------|
| [bevy_ecs_typewriter](https://github.com/Bli-AIk/bevy_ecs_typewriter)     | 0.0.0                                                                                                                          | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | 基于 ECS 的打字机实现                          |
| [bevy_fact_rule_event](https://github.com/Bli-AIk/bevy_fact_rule_event)   | 0.0.0                                                                                                                          | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | 基于 fact-rule-event 的事件系统实现             |
| [bevy_mortar_bond](https://github.com/Bli-AIk/bevy_mortar_bond)           | 0.0.0                                                                                                                          | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | Bevy 与 Mortar 语言的桥接库                   |
| [leafwing-input-manager](https://crates.io/crates/leafwing-input-manager) | 0.19.0                                                                                                                         | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | 管理游戏输入，处理键盘、鼠标和控制器的操作映射                |
| [seldom_state](https://crates.io/crates/seldom_state)                     | 0.15.0                                                                                                                         | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | 有限状态机实现                                |
| [bevy_ecs_tiled](https://crates.io/crates/bevy_ecs_tiled)                 | dev（GitHub 分支）                                                                                                                 | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | 集成 Bevy ECS 和 Tiled 地图编辑器，用于基于瓦片的游戏关卡  |
| [bevy_tween](https://crates.io/crates/bevy_tween)                         | 0.10.0                                                                                                                         | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | Bevy 的补间动画库，用于平滑动画和过渡                  |
| [bevy-inspector-egui](https://crates.io/crates/bevy-inspector-egui)       | 0.35.0                                                                                                                         | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | Bevy 的可选编辑器/调试工具，用于实时检查 ECS 世界         |
| [iyes_perf_ui](https://crates.io/crates/iyes_perf_ui)                     | dev（我基于 GitHub PR [#35](https://github.com/IyesGames/iyes_perf_ui/pull/35) 创建的 [分支](https://github.com/Bli-AIk/iyes_perf_ui) ) | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | Bevy 的可选性能监控 UI，显示 FPS、系统计时和性能分析信息     |
| [bevy_smud](https://crates.io/crates/bevy_smud)                           | 0.12.0                                                                                                                         | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | Bevy 的 2d sdf 形状渲染器插件                  |
| [bevy_rich_text3d](https://crates.io/crates/bevy_rich_text3d)             | 0.5.1                                                                                                                          | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | 基于网格的栅格富文本实现                           |
| [bevy_kira_audio](https://crates.io/crates/bevy_kira_audio)               | 0.24.0                                                                                                                         | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | 使用 Kira 的音频播放插件，支持 WAV、OGG、FLAC、MP3 格式 |
| [bevy_brp_extras](https://crates.io/crates/bevy_brp_extras)               | 0.17.2                                                                                                                         | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | Bevy 远程协议 (BRP) 的扩展功能                  |

### Rust crates

| 项目                                      | 版本    | 许可证                                                                                                                                                                                                           | 功能                                       |
|-----------------------------------------|-------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|------------------------------------------|
| [serde](https://crates.io/crates/serde) | 1.0   | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | 序列化/反序列化框架，支持 `derive` 宏以方便地对结构体进行（反）序列化 |
| [toml](https://crates.io/crates/toml)   | 0.9.8 | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | TOML 解析                                  |
| [ron](https://crates.io/crates/ron)     | 0.10  | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | Rusty Object Notation 解析                 |

### 资源引用

| 项目                                                  | 描述                           |
|-----------------------------------------------------|------------------------------|
| [DTTVL-Fonts](https://github.com/UTCLC/DTTVL-Fonts) | DELTATRAVELER 中文本地化项目使用的字体文件 |

**衷心感谢每一个项目的贡献者们！🎔**

## 贡献者

以下人员为本项目做出了贡献。

<a href = "https://github.com/Bli-AIk/souprune/Python/graphs/contributors">
<img src = "https://contrib.rocks/image?repo=Bli-AIk/souprune" alt=""/>
</a>

**衷心感谢你们每一个人！🎔**

## 🤝 加入我们

无论你是：

* 想做自己的 DR/UT 风格游戏；
* 想尝试 Bevy 与 Rust；
* 还是单纯热爱开源与实验精神——

都欢迎参与 **SoupRune** 的建设：

* 提交 Issue 或 Pull Request！
* 分享想法、讨论架构！
* 或者在社区里单纯聊聊游戏开发！

**让我们一起煮出一锅最美味的 Soup 吧！**