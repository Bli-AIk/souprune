# souprune_sdk

[![license](https://img.shields.io/badge/license-LGPL--3.0--or--later-blue)](LICENSE) <img src="https://img.shields.io/github/repo-size/Bli-AIk/souprune.svg"/> <img src="https://img.shields.io/github/last-commit/Bli-AIk/souprune.svg"/> <br>
<img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" />

> 当前状态：🚧 早期开发中

**souprune_sdk** — SoupRune 游戏框架的模组开发 SDK。

| 英语 | 简体中文 |
|------|----------|
| [English](./readme.md) | 简体中文 |

## 介绍

`souprune_sdk` 是用于为 SoupRune 游戏创建模组和扩展的软件开发工具包。  
它通过提供统一的高级 API 解决了复杂的模组开发问题，让用户无需深入了解引擎即可创建游戏修改。

使用 `souprune_sdk`，你只需要使用提供的接口实现模组逻辑，就可以将其加载到任何基于 SoupRune 的游戏中。  
未来还计划支持可视化模组工具和模组市场集成。

## 功能

* 基于 souprune_api 构建的高级模组 API
* 类型安全的模组接口
* 与 SoupRune 游戏轻松集成
* （计划中）模组生命周期管理
* （计划中）开发热重载支持
* （计划中）可视化模组编辑器

## 使用方法

1. **安装 Rust**（如果尚未安装）：
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **添加到 Cargo.toml**：

   ```toml
   [dependencies]
   souprune_sdk = "0.1"
   ```

3. **创建模组**：

   ```rust
   use souprune_sdk::prelude::*;

   // 在这里实现你的模组
   ```

4. **构建模组**：

   ```bash
   cargo build --release
   ```

## 依赖

本项目使用以下 crate：

| Crate                                             | 版本 | 描述                 |
| ------------------------------------------------- | ------- | --------------------------- |
| [souprune_api](https://crates.io/crates/souprune_api) | 0.0.1   | FFI API 层 |

## 警告

⚠️ **这是一个早期开发版本。**

- API 不稳定，可能会发生重大变化
- 文档和示例有限
- 不推荐用于生产环境
- 未来版本预计会有破坏性更改

## 贡献指南

欢迎贡献！
无论你想修复错误、添加功能或改进文档：

* 提交 **Issue** 或 **Pull Request**。
* 分享想法并讨论设计或架构。

## 许可证

本项目可依据以下许可证进行分发：

* GNU Lesser General Public License v3.0 或更高版本（[LICENSE](LICENSE) 或 [https://www.gnu.org/licenses/lgpl-3.0.html](https://www.gnu.org/licenses/lgpl-3.0.html)）
