# souprune_mod_test

[![license](https://img.shields.io/badge/license-LGPL--3.0--or--later-blue)](LICENSE) <img src="https://img.shields.io/github/repo-size/Bli-AIk/souprune.svg"/> <img src="https://img.shields.io/github/last-commit/Bli-AIk/souprune.svg"/> <br>
<img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" />

> 当前状态：🚧 早期开发中 - 实验性

**souprune_mod_test** — 用于验证 SoupRune SDK 功能的测试模组实现。

| 英语 | 简体中文 |
|------|----------|
| [English](./readme.md) | 简体中文 |

## 介绍

`souprune_mod_test` 是 SoupRune SDK 的参考实现和测试模组。  
它通过提供使用 souprune_sdk 创建模组的工作示例来解决 SDK 验证问题。

使用 `souprune_mod_test`，开发者可以学习模组结构、测试 SDK 安装、并查看如何实现常见模组模式的实际示例。  
这个 crate 主要用于测试和参考目的。

## 功能

* souprune_sdk 的参考实现
* 演示模组结构和模式
* SDK 集成测试
* 为模组开发者提供示例代码
* 构建为 cdylib 以支持动态加载

## 使用方法

1. **安装 Rust**（如果尚未安装）：
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **构建测试模组**（需要 `wasm32-wasip2` 编译目标）：

   ```bash
   rustup target add wasm32-wasip2
   cargo build -p souprune_mod_test --target wasm32-wasip2
   ```

3. **加载到 SoupRune 游戏中**：

   编译后的 `.wasm` 组件可以被兼容的 SoupRune 游戏加载，也可以使用 `souprune_mock_host` 进行测试。

4. **学习源代码**：

   查看 `src/lib.rs` 中的实现以了解模组模式。

## 依赖

本项目使用以下 crate：

| Crate                                             | 版本 | 描述                 |
| ------------------------------------------------- | ------- | --------------------------- |
| [souprune_sdk](https://crates.io/crates/souprune_sdk) | 0.0.1   | 模组 SDK |

## 警告

⚠️ **这是一个实验性的早期版本。**

- API 不稳定，可能会发生重大变化
- 仅用于测试和参考目的
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
