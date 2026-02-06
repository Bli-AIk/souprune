# SoupRune

[![license](https://img.shields.io/github/license/Bli-AIk/souprune)](LICENSE.md) <img src="https://img.shields.io/github/repo-size/Bli-AIk/souprune.svg"/> <img src="https://img.shields.io/github/last-commit/Bli-AIk/souprune.svg"/> <br>
<img src="https://img.shields.io/badge/Deltarune / Undertale-black?style=for-the-badge&logo=undertale&logoColor=ff0000" /> <img src="https://img.shields.io/badge/Bevy-232326?style=for-the-badge&logo=bevy&logoColor=white" /> <br>
<img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" />
<img src="https://img.shields.io/badge/C%23-实验性支持-239120?style=for-the-badge&logo=csharp&logoColor=white" />
<img src="https://img.shields.io/badge/Haxe-实验性支持-EA8220?style=for-the-badge&logo=haxe&logoColor=white" />
<img src="https://img.shields.io/badge/Nim-即将支持-FFE953?style=for-the-badge&logo=nim&logoColor=black" />
<img src="https://img.shields.io/badge/Nelua-即将支持-9CA3AF?style=for-the-badge&logo=lua&logoColor=white" />

> **状态**：🚧 初始开发阶段（框架结构仍在快速演进中）

[![](https://dcbadge.limes.pink/api/server/5YXK5DRjPZ)](https://discord.gg/5YXK5DRjPZ)

**SoupRune** 是一个令人耳目一新的现代化框架，为次世代同人游戏而生，
专为创作类似 **[Deltarune](https://deltarune.com/) / [Undertale](https://undertale.com/)** 的 RPG / STG 游戏而设计。

| 英语                     | 简体中文 |
|------------------------|------|
| [English](./readme.md) | 简体中文 |

## 🥣 这是什么？

**SoupRune** 是一个令人耳目一新的现代化框架，它基于 `bevy` 引擎制作，为次世代同人游戏而生，
专为创作类似 **[Deltarune](https://deltarune.com/) / [Undertale](https://undertale.com/)** 风格的 RPG / 弹幕射击游戏。

它致力于成为下一代社区驱动的 Fangame 引擎——既带来了独特的味道，又在底层架构上拥抱了高性能与现代开发范式。

## 🧭 S.O.U.P 原则

我们的设计哲学凝聚在一碗美味的 **S.O.U.P** 里：

* **S (Strong) - 强劲内核**：基于 **Bevy 引擎** 与 **Rust** 构建，享受 ECS 架构带来的高性能与并行计算优势。
* **O (Open) - 自由开源**：采用 **LGPL-3.0** 协议。核心代码库开源，但你的 Project / Mod（游戏）属于你自己。
* **U (User-friendly) - 易于上手**：提供开箱即用的 RPG 功能（对话、战斗、地图）、弹幕序列器（STG-支持回合制）、以及可视化工具集成（支持
  Alight Motion 工程），让你专注于创意。
* **P (Polyglot) - 多语言支持**：通过 C ABI 实现语言无关，你可以选择最顺手的“餐具”（C#、Haxe、Rust 等）来享用这碗汤。

## 🚀 快速开始

SoupRune 目前仍处于 **🚧 初始开发阶段**，但如果你渴望尝鲜，可以按以下步骤起步：

1. **准备环境**：安装 [Rust 开发环境](https://www.rust-lang.org/)。
2. **克隆仓库**：`git clone https://github.com/Bli-AIk/souprune.git`
3. **进入目录**：`cd souprune`
4. **拉取子模块**：`git submodule update --init --recursive`
5. **在Debug模式下，运行示例**：`cargo run --package souprune --bin souprune --features debug`

## ⚙️ 设计理念

在技术层面，SoupRune：

* 核心使用 **Bevy 引擎** 与 **Rust 语言** 实现，保证性能与可扩展性；
* 设计目标是：**结构清晰、可模块化扩展、易于定制**；

在项目架构上，SoupRune 采用了 **Engine (引擎)** 与 **Project (项目)** 分离的设计。

你可以把这想象成 **“游戏机”** 与 **“游戏卡带”** 的关系：

* **Engine (核心)**：是底层的“游戏机”。它负责处理最繁重、最复杂的工作，比如“怎么画出绚丽的画面”、“怎么让物理碰撞更真实”。这部分由高性能的
  Rust 打造，通常不需要你去操心。
* **Project / Mod (内容)**：是你制作的“游戏卡带”。这里装着你的创意——角色的对话、精彩的战斗、感人的剧情。

### 📝 开发模式：拥抱配置与脚本

在 SoupRune 中，开发游戏不再意味着整天面对枯燥的代码。我们推崇 **“数据驱动”** 的开发方式，通过 **RON 配置文件** 与
**自定义脚本** 的结合来实现创意：

1. **RON 配置文件（内容描述）**：
   绝大多数的游戏内容是通过编写 **RON (Rusty Object Notation)** 文件完成的。这就像是 **填写表格** 或 **拼积木**，清晰且直观。
    * 想设计一场战斗流程？写一个 `.performance.ron` 安排弹幕时间轴。
    * 想摆放 UI 界面？写一个 `.ui_layout.ron` 定义按钮和文本。
    * 想定义角色属性？写一个 `.character.ron` 设置动画和碰撞箱。
    * *即使没有编程基础，也能通过修改这些配置做出丰富的游戏内容。*

2. **自定义脚本 (行为与算法)**：
   当你需要实现独特的、复杂的逻辑（比如一个从未见过的螺旋追踪弹幕算法、特殊的 BOSS 机制）时，才需要编写脚本。
   这也是 SoupRune 强大的地方——你可以选择 **Rust**, **C# (.NET)** 或 **Haxe** 来编写这些逻辑！
    * 代码会被编译成动态库（`.dll` 或 `.so`），像插件一样“插入”到引擎中运行。
    * 引擎通过标准接口（C ABI）与你的脚本对话，让你在享受 **高性能** 的同时，还能使用你 **最熟悉的语言**。

**总结来说：SoupRune 负责把汤底熬好（底层引擎），
你只需要看着菜谱（RON 配置），再选一把顺手的勺子（编程语言）往里加料，就能煮出属于你的美味游戏！**

<details>
<summary><strong>那么，SoupRune 的设计初衷是什么呢？</strong></summary>

如果你对 SoupRune 背后的设计思考感兴趣，这里有更多细节：

### 🌌 为什么把“灵魂”给“锈”掉？为什么要用 Rust 和 Bevy？

> 做一个 2D 像素 RPG，真的需要 Rust 和 Bevy 这样高性能的 ECS 引擎吗？这不是杀鸡用牛刀吗？

我想这会是绝大多数人提出的一个问题——无论你是系统级开发者、还是喜欢复古/同人游戏的创作者。

是的，我们在做一件看起来很矛盾的事：
用如此硬核、如此追求性能的系统语言 **Rust**，去重构一个最感性、最复古、最不看重画质和技术能力的游戏类型。

因为——美学可以复古，但技术应当走在前沿。

SoupRune 是一次探索——我们想证明 ECS 不仅仅能跑千军万马的 Demo，它也能细腻地处理一段感人的对话，一场精心编排的演出。我们把
Bevy 变成了画笔。

SoupRune 也是一件利器——对于那些渴望发挥自己所有才能对、伟大的社区创作者们，你们拥有惊人的创造力，但往往受困于陈旧工具的性能瓶颈。卡顿的弹幕、崩溃的存档、难以实现的复杂机制。

在此之前，从未有人设想过这两者的结合——而现在，它正站在这里。

这是为你们准备的地图，这是为你们打造好的武器。

我们希望用前卫的渲染管线和架构设计，打造一个坚不可摧的舞台。哪怕你甚至不懂代码，你的作品也值得拥有工业级的性能。

这便是这个项目成立的目标，这个项目设计的初衷：架起一座桥梁。

### 🌉 我们的承诺：一座跨越鸿沟的桥

> 技术与艺术之间的桥——光和暗的交汇、梦与现实的界限。

SoupRune 不仅仅是一个框架，它是一次**技术平权**。

* **简单不仅是简单**：
  在 SoupRune 里，"简单"是指你创作时的体验，而不是底层的简陋。
  我们扛下了底层所有的复杂（多线程、内存管理、渲染管线），只为了让你在写逻辑时能像呼吸一样自然。
* **为爱发电，但也炸裂全场**：
  我们相信，同人作品不应是“凑合”的代名词。
  美与技术力不应绑定——哪怕是像素游戏，也配得上 4K 的清晰度、144Hz 的丝滑和数万个粒子、弹幕，以及渲染效果的狂欢。

### 🏗️ 架构：为何要将核心与项目分开？为何要做到“语言无关”？

SoupRune 采用了 **核心 (Engine) - 项目 (Project/Mod)** 分离的架构。

* **核心**：由 Rust 和 Bevy 驱动，负责所有底层的繁重工作（渲染、物理、ECS 调度）。
* **项目**：通过标准的 **C ABI** 与核心对话。

我们深知 Undertale / Deltarune 社区的开发者背景各异——

* 来自 **Clickteam Fusion** 的开发者，会发现 **Ron** 只是另一种形式的事件表；
* 来自 **GameMaker** 的爱好者，也会发觉 **Haxe** 亲切自然；
* 习惯 **Unity/Godot** 的专业人士，可以继续享受 **C#** (通过 Native AOT 获得极佳性能)；
* 习惯 **Lua** 或 **Python / GDScript** 的朋友，未来也能通过 **Nelua** 或 **Nim** 无缝接入。

让社区的每一位创作者都能 **用简单易上手的方式** 来定义游戏内容，是我们设计的初衷；
让社区的每一位开发者都能 **用自己熟悉的方式** 来编写代码逻辑，是我们设计的目标。

### ⚖️ 协议：关于 LGPL 开源

我们选择 **LGPL-3.0** 是为了在“开源贡献”和“创作者权益”之间找到平衡。简而言之：

* ✅ **你的游戏由你做主**：你基于 SoupRune 开发的 Project (Mod) 可以闭源，也可以商业化销售，无需开源你的游戏逻辑代码。
* 🤝 **回馈社区**：如果你修改了 SoupRune 的**框架核心代码**（Engine 部分），则必须将这些修改开源，让所有人受益。
* 🏢 **商业许可**：如果你确实需要在闭源环境下修改核心代码，可以联系我获取商业授权。

### 🔮 愿景：走向社区驱动——属于社区的次世代游乐场

SoupRune 名字里的 **「Rune」** 既致敬了 Deltarune，也象征着一种传承。

它是 [Undertale Changer Template](https://github.com/Bli-AIk/Undertale-Changer-Template)
的精神续作。我们仍然怀揣着改变一切的决心——我们的目标不是做一个封闭的工具，而是一个**开放的、现代的——且最重要的，
面向社区的次世代游乐场**。

虽然“次世代”意味着处于不断进化的前沿，但它也代表了无限的可能性——无论是复刻经典，还是创造全新的原创作品，我们都希望
SoupRune 能成为你手中的利器。

### 🤝 加入我们：我们需要你的决心

**(Join Us: Bring Your Determination)**

这是一个开源的、属于社区的**次世代游乐场**。
它现在可能还不完美，就像每一个伟大的冒险开始时那样。

* 如果你是 **Rustacean**，来帮我们铸造这把剑，挑战 ECS 架构在叙事游戏中的极限。
* 如果你是 **Renderer**，来帮我们打磨这把剑，打造出前所未有的，存在于复古 RPG / STG 中的，既高效又美观的风格化渲染管线。
* 如果你是 **Creator**，来试着挥舞这把剑，告诉我们需要什么样的功能才能承载你的想象。

符文 (RUNE) 的预言应当由我们共同书写。

让我们一行一行书写新的未来吧。

</details>

## 贡献者

以下人员为 SoupRune 项目做出了贡献！

<a href = "https://github.com/Bli-AIk/souprune/Python/graphs/contributors">
<img src = "https://contrib.rocks/image?repo=Bli-AIk/souprune" alt=""/>
</a>

### 非代码贡献者

* **71**: 他在此项目测试过程中提供了许多 Alight Motion 示例工程（包括 Undertale 弹幕，以及一些视觉 PV），为 AM 集成提供了很大帮助！
* **陈皮**: 他提供了许多 Alight Motion 示例工程（主要是 Undertale 弹幕），为 AM 集成提供了很大帮助！

**衷心感谢你们每一个人！🎔**

## 🤝 加入我们

无论你是：

* 想做自己的 DR/UT 风格游戏；
* 想尝试 Bevy 与 Rust；
* 还是单纯热爱开源与先锋精神——

都欢迎参与 **SoupRune** 的建设：

* 针对任何 问题、建议，提交 Issue，提出你的宝贵意见！
* 为 SoupRune 填砖加瓦，提供 Pull Request！
* 在 Discord 或 Github Discussions 分享想法、讨论架构！
* 或者在社区里单纯聊聊游戏开发！

**让我们一起煮出一锅最美味的 Soup 吧！**

## 🏗️ 项目架构

SoupRune 采用多 Crate 的工作空间架构：

| Crate                                                             | 描述                                               |
|:------------------------------------------------------------------|:-------------------------------------------------|
| [`souprune`](./crates/souprune)                                   | **核心框架**：框架本体，游戏的主入口和核心逻辑实现。                     |
| [`souprune_api`](./crates/souprune_api)                           | **协议层**：定义了 Project (Mod) 与框架核心交互的接口标准。          |
| [`souprune_sdk`](./crates/souprune_sdk)                           | **开发工具包**：对 API 的封装，提供给外部 Project (Mod) 脚本使用。    |
| [`souprune_mod_test`](./crates/souprune_mod_test)                 | **样例 Mod**： 脚本系统的示例测试库。                          |
| [`bevy_mortar_bond`](./crates/bevy_mortar_bond)                   | **功能插件**：Mortar 脚本语言与 Bevy 的桥接层，负责对话与逻辑。         |
| [`bevy_ecs_typewriter`](./crates/bevy_ecs_typewriter)             | **功能插件**：基于 ECS 的打字机实现，支持富文本与多语言。                |
| [`bevy_fact_rule_event`](./crates/bevy_fact_rule_event)           | **功能插件**：基于“事实-规则-事件”模型的复杂事件系统。                  |
| [`bevy_alight_motion`](./crates/bevy_alight_motion)               | **功能插件**：解析并播放 Alight Motion 动画工程的集成插件。          |
| [`interoptopus_backend_haxe`](./crates/interoptopus_backend_haxe) | **工具库**：Interoptopus FFI 绑定生成器的 Haxe (hxcpp) 后端。 |

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

> 🤝 **友情推荐 / 寻找 Lua (Love2D) 生态？**
>
> SoupRune 专注于 AOT 编译与静态类型语言的极致性能。
> 如果您更偏爱 **Lua 动态脚本** 的灵活性，或者习惯 **Love2D** 的工作流，我们强烈推荐您尝试社区中另一款优秀的框架：
>
> **[SoulEngine](https://github.com/AnskiyyRenew/love2d-undertale-template)** —— 专为快速原型与动态构建设计。
>
> *我们要做的不仅仅是框架，而是共同繁荣的 UT/DR 同人开发生态。❤️*

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
| [bevy_ecs_tiled](https://crates.io/crates/bevy_ecs_tiled)                 | 0.10.0                                                                                                                         | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT)                                                                                                           | 集成 Bevy ECS 和 Tiled 地图编辑器，用于基于瓦片的游戏关卡  |
| [bevy_tween](https://crates.io/crates/bevy_tween)                         | 0.10.0                                                                                                                         | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | Bevy 的补间动画库，用于平滑动画和过渡                  |
| [bevy_alight_motion](https://github.com/Bli-AIk/bevy_alight_motion)       | 0.0.0                                                                                                                          | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | Alight Motion 动画集成，包含 SDF 形状渲染         |
| [bevy_seedling](https://crates.io/crates/bevy_seedling)                   | 0.6.1                                                                                                                          | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | 可选的音频后端                                |
| [bevy-inspector-egui](https://crates.io/crates/bevy-inspector-egui)       | 0.35.0                                                                                                                         | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | Bevy 的可选编辑器/调试工具，用于实时检查 ECS 世界         |
| [iyes_perf_ui](https://crates.io/crates/iyes_perf_ui)                     | dev（我基于 GitHub PR [#35](https://github.com/IyesGames/iyes_perf_ui/pull/35) 创建的 [分支](https://github.com/Bli-AIk/iyes_perf_ui) ) | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | Bevy 的可选性能监控 UI，显示 FPS、系统计时和性能分析信息     |
| [bevy_rich_text3d](https://crates.io/crates/bevy_rich_text3d)             | 0.5.1                                                                                                                          | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | 基于网格的栅格富文本实现                           |
| [bevy_kira_audio](https://crates.io/crates/bevy_kira_audio)               | 0.24.0                                                                                                                         | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | 使用 Kira 的音频播放插件，支持 WAV、OGG、FLAC、MP3 格式 |
| [bevy_brp_extras](https://crates.io/crates/bevy_brp_extras)               | 0.17.2                                                                                                                         | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | Bevy 远程协议 (BRP) 的扩展功能                  |

### Rust crates

| 项目                                                    | 版本              | 许可证                                                                                                                                                                                                           | 功能                                       |
|-------------------------------------------------------|-----------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|------------------------------------------|
| [serde](https://crates.io/crates/serde)               | 1.0             | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | 序列化/反序列化框架，支持 `derive` 宏以方便地对结构体进行（反）序列化 |
| [toml](https://crates.io/crates/toml)                 | 0.9.8           | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | TOML 解析                                  |
| [ron](https://crates.io/crates/ron)                   | 0.10            | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | Rusty Object Notation 解析                 |
| [serde_json](https://crates.io/crates/serde_json)     | 1.0             | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | JSON 序列化/反序列化                            |
| [regex](https://crates.io/crates/regex)               | 1.10            | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | 正则表达式                                    |
| [fasteval](https://crates.io/crates/fasteval)         | 0.2             | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT)                                                                                                           | 快速、安全的代数表达式求值                            |
| [interoptopus](https://crates.io/crates/interoptopus) | 0.15.0-alpha.24 | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT)                                                                                                           | 多语言绑定生成器                                 |
| [mortar_compiler](https://github.com/Bli-AIk/mortar)  | 0.5.0           | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | Mortar 语言编译器                             |

### 资源引用

| 项目                                                  | 描述                           |
|-----------------------------------------------------|------------------------------|
| [DTTVL-Fonts](https://github.com/UTCLC/DTTVL-Fonts) | DELTATRAVELER 中文本地化项目使用的字体文件 |

**衷心感谢每一个项目的贡献者们！🎔**