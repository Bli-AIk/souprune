# souprune

[![license](https://img.shields.io/badge/license-LGPL--3.0--or--later-blue)](LICENSE-APACHE) <img src="https://img.shields.io/github/repo-size/Bli-AIk/souprune.svg"/> <img src="https://img.shields.io/github/last-commit/Bli-AIk/souprune.svg"/> <br> <img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" />

> 当前状态：🚧 早期开发中（初始版本正在开发）

**souprune** — 专为 Deltarune / Undertale 同人游戏设计的游戏框架。

| 英语             | 简体中文                      |
| --------------- | --------------------------- |
| [English](./readme.md) | 简体中文 |

## 介绍

`souprune` 是一个专为 Deltarune / Undertale 同人游戏设计的游戏框架。
它解决了创建 Deltarune/Undertale 风格游戏的挑战，让用户能够构建具有正宗机制和功能的同人游戏。

使用 `souprune`，你只需要专注于内容创作，框架会处理核心游戏系统。
未来还计划支持更多游戏机制和增强的模组功能。

## 功能

* Deltarune/Undertale 风格的游戏机制
* 基于 Bevy 引擎，性能优异且灵活
* 集成文本动画系统
* 用于游戏逻辑的事实-规则-事件系统
* Mortar 语言集成
* （计划中）增强的战斗系统
* （计划中）保存/加载功能
* （计划中）对话系统

## 使用方法

1. **安装 Rust**（如果尚未安装）：

   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **添加到 Cargo.toml**：

   ```toml
   [dependencies]
   souprune = "0.1.2"
   ```

3. **基本游戏设置**：

   ```rust
   // <待补充>
   ```

4. **调试功能**：

   * 启用调试功能：`cargo run --features debug`
   * 调试模式下可使用检查器和性能 UI

## 构建方法

### 前置要求

* Rust 1.70 或更高版本

### 构建步骤

1. **克隆仓库**：

   ```bash
   git clone https://github.com/Bli-AIk/souprune.git
   cd souprune
   ```

2. **构建项目**：

   ```bash
   cargo build --release
   ```

3. **运行测试**：

   ```bash
   cargo test
   ```

4. **全局安装**（可选）：

   ```bash
   cargo install --path .
   ```

## 依赖

本项目使用以下 crate：

| Crate                                             | 版本    | 描述   |
| ------------------------------------------------- | ----- | ---- |
| [bevy](https://crates.io/crates/bevy) | 0.17.2 | 游戏引擎 |
| [leafwing-input-manager](https://crates.io/crates/leafwing-input-manager) | 0.19.0 | 输入处理 |
| [seldom_state](https://crates.io/crates/seldom_state) | 0.15.0 | 状态管理 |
| [serde](https://crates.io/crates/serde) | 1.0 | 序列化框架 |
| [toml](https://crates.io/crates/toml) | 0.9.8 | 配置解析 |
| [bevy_tween](https://crates.io/crates/bevy_tween) | 0.10.0 | 动画补间 |
| bevy_rich_text_3d_animator | 0.1.0 | 文本动画系统 |
| bevy_fact_rule_event | 0.1.0 | 事实-规则-事件系统 |
| bevy_mortar_bond | 0.1.0 | Mortar 语言集成 |

## 贡献指南

欢迎贡献！
无论你想修复错误、添加功能或改进文档：

* 提交 **Issue** 或 **Pull Request**。
* 分享想法并讨论设计或架构。

## 许可证

本项目使用 LGPL-3.0-or-later 许可证。