# souprune_editor

[![license](https://img.shields.io/badge/license-LGPL--3.0--or--later-blue)](LICENSE) <img src="https://img.shields.io/github/repo-size/Bli-AIk/souprune.svg"/> <img src="https://img.shields.io/github/last-commit/Bli-AIk/souprune.svg"/> <br>
<img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" />

> 当前状态：🚧 早期开发中

**souprune_editor** — 基于 Bevy + egui 的可视化编辑器，用于实时创建和编辑 SoupRune 游戏内容。

| 英语 | 简体中文 |
|------|----------|
| [English](./readme.md) | 简体中文 |

## 介绍

`souprune_editor` 是基于 bevy_workbench 和 egui 构建的可视化游戏编辑环境。它在视口中渲染游戏，同时允许实时编辑场景、视图和配置。编辑器与独立游戏共享相同的插件基础设施，确保编辑时的行为与运行时一致。

## 功能

* **视图编辑器** — 编辑 `.view.ron` 场景布局，带实时预览
* **FRE 编辑器** — 编辑 Fact-Rule-Event 表达式
* **RON 源码编辑器** — RON 文件原始编辑，具有语法感知
* **实时预览** — 在隔离视口中实时渲染游戏
* **国际化** — 支持英文和中文界面
* **平台支持** — 桌面端和移动端自适应布局

## 使用方法

编辑器通常通过工作空间运行：
```bash
cargo run -p souprune_editor
```

## 构建方法

### 前置要求

* Rust 1.85 或更高版本
* 系统依赖（与 souprune 主 crate 相同）

### 构建步骤

```bash
cargo build --release -p souprune_editor
```

## 依赖

| Crate                                                    | 版本   | 描述               |
|----------------------------------------------------------|--------|--------------------|
| [bevy](https://crates.io/crates/bevy)                   | 0.18   | 游戏引擎           |
| [bevy_egui](https://crates.io/crates/bevy_egui)         | 0.39   | egui UI 集成       |
| [egui](https://crates.io/crates/egui)                   | 0.33   | 即时模式 UI        |
| [ron](https://crates.io/crates/ron)                      | 0.12   | RON 配置解析       |
| [rfd](https://crates.io/crates/rfd)                     | 0.17   | 原生文件对话框     |
| [souprune](../souprune)                                  | —      | 主游戏框架         |

## 警告

⚠️ **这是一个早期开发版本。**

- API 不稳定，可能会发生重大变化
- 文档有限
- 不推荐用于生产环境
- 未来版本预计会有破坏性更改

## 贡献指南

欢迎贡献！
无论你想修复错误、添加功能或改进文档：

* 提交 **Issue** 或 **Pull Request**。
* 分享想法并讨论设计或架构。

## 许可证

本项目可依据以下许可证进行分发：

* GNU Lesser General Public License v3.0 或更高版本（[LICENSE](../../LICENSE.md) 或 [https://www.gnu.org/licenses/lgpl-3.0.html](https://www.gnu.org/licenses/lgpl-3.0.html)）
