# SoupRune

[![license](https://img.shields.io/github/license/Bli-AIk/souprune)](LICENSE) 
<img src="https://img.shields.io/github/repo-size/Bli-AIk/souprune.svg"/> 
<img src="https://img.shields.io/github/last-commit/Bli-AIk/souprune.svg"/>

<img src="https://img.shields.io/badge/Deltarune / Undertale-black?style=for-the-badge&logo=undertale&logoColor=ff0000" />
<img src="https://img.shields.io/badge/Bevy-232326?style=for-the-badge&logo=bevy&logoColor=white" />
<br>
<img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" />
<img src="https://img.shields.io/badge/Gamemaker language-black?style=for-the-badge&logo=gamemaker&logoColor=white" />
<img src="https://img.shields.io/badge/Lua-2C2D72?style=for-the-badge&logo=lua&logoColor=white" />
<img src="https://img.shields.io/badge/.NET-512BD4?style=for-the-badge&logo=dotnet&logoColor=white" />

> **状态**：🚧 初始开发阶段（框架结构仍在快速演进中）

**SoupRune** 是一个专为 **[Deltarune](https://deltarune.com/) / [Undertale](https://undertale.com/) Fangame** 设计的游戏框架。


| 英语                     | 简体中文 |
|------------------------|------|
| [English](./readme.md) | 简体中文 |

---

## 🥣 简介

嘿，别被上面那个 **「Rust」** 徽章吓到 —— **SoupRune 可不只是给某些专业程序员玩的冷门工具！**

它支持 **Gamemaker**、**[Unitale](https://github.com/lvk/Unitale) / [Create Your Frisk](https://github.com/RhenaudTheLukark/CreateYourFrisk)**、以及 **.NET (C#)** 生态，

无论你是习惯码 GML、写 Lua、还是 Unity / Godot 开发者，也许都能在这里找到熟悉的感觉。

**名字里的「Rune」也不是随便取的 ——**

Undertale 的 Fangame 框架兴许已经饱和，但 Deltarune 的社区正在蓬勃发展。

SoupRune 就是趁此机会打造的 Fangame 框架，~~同时顺带充当 [Undertale Changer Template](https://github.com/Bli-AIk/Undertale-Changer-Template) 的精神续作~~。

但这也不意味着 SoupRune 只是一个 Deltarune 框架 —— 对于 Undertale Fangame 的支持，我们也同样看重！

SoupRune 的目标是成为真正意义上的 **「面向社区的 Fangame 框架」** —— 开放、灵活、现代，同时依然保持那个熟悉的 DR / UT 风格。

---

## 🧭 S.O.U.P 原则

是的，**Soup**Rune当然是一个双关啦——


|  缩写   | 全称                | 含义                                  |
|:-----:|:------------------|:------------------------------------|
| **S** | **Strong**        | 基于 **Bevy** 与 **Rust**，性能强劲、架构现代。   |
| **O** | **Open**          | 采用 **LGPL** 开源协议，允许自由使用与拓展。         |
| **U** | **User-friendly** | 提供多语言脚本层，降低上手门槛。                    |
| **P** | **Popular**       | 兼容 DR/UT 社区主流生态与使用习惯。               |

---

## ⚙️ 技术基础

* 核心使用 **Bevy 引擎** 与 **Rust 语言** 实现，保证性能与可扩展性；
* 设计目标是：**结构清晰、可模块化扩展、易于定制**；

---

## 🧩 脚本层支持

SoupRune 将提供多语言脚本支持，让不同背景的开发者都能快速上手：

|                                   语言                                   | 适用人群                                                                                                      | 描述                          |
|:----------------------------------------------------------------------:|:----------------------------------------------------------------------------------------------------------|:----------------------------|
|                                **Rust**                                | 系统层开发者 / Rustacean / Bevy 用户                                                                              | 原生支持，性能最佳。                  |
| **[COL（Configurable Open Language）](https://github.com/Bli-AIk/col/)** | GML / GameMaker 用户                                                                                        | 我自研的开源 GML 替代语言，完全兼容语法、易扩展。 |
|                                **Lua**                                 | [Unitale](https://github.com/lvk/Unitale) / [CYF](https://github.com/RhenaudTheLukark/CreateYourFrisk) 用户 | 轻量级脚本语言，快速上手。               |
|                          **.NET (C#/VB/F#)**                           | Unity / Godot 用户                                                                                          | 熟悉语法？平滑迁移。                  |

未来还会：

* 尝试为 [CYF](https://github.com/RhenaudTheLukark/CreateYourFrisk) / Gamemaker 爱好者提供熟悉的开发体验（例如相似的事件系统或脚本接口）；
* 提供从 [CYF](https://github.com/RhenaudTheLukark/CreateYourFrisk) / [Undertale Engine](https://github.com/TML233/UndertaleEngine) 项目迁移的指南。

---

## 💬 开源与许可

SoupRune 采用 **LGPL-3.0** 许可协议。

这意味着：

* 你可以在闭源项目中使用 SoupRune；
* 但若你修改了框架核心代码，需将这些修改以 LGPL 方式开源；
* 若你希望在完全闭源环境下使用修改版 SoupRune，可联系我获取商业许可。

此外，仍需遵守 Toby Fox 对粉丝游戏的相关使用规定。

---

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