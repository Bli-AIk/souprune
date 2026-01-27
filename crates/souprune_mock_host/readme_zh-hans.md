# souprune_mock_host

[![license](https://img.shields.io/badge/license-LGPL--3.0--or--later-blue)](LICENSE) <img src="https://img.shields.io/github/repo-size/Bli-AIk/souprune.svg"/> <img src="https://img.shields.io/github/last-commit/Bli-AIk/souprune.svg"/> <br>
<img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" />

> 当前状态：🚧 早期开发中 - 实验性

**souprune_mock_host** — 用于测试 SoupRune 模组的模拟宿主环境，无需完整游戏。

| 英语 | 简体中文 |
|------|----------|
| [English](./readme.md) | 简体中文 |

## 介绍

`souprune_mock_host` 是一个用于加载和测试 SoupRune 模组的最小宿主环境。  
它通过提供一个轻量级运行时来解决模组测试问题，该运行时可以动态加载模组而无需完整的游戏设置。

使用 `souprune_mock_host`，模组开发者可以快速测试他们的创作、验证 FFI 绑定、并在隔离环境中调试模组行为。  
它专为开发和 CI/CD 测试工作流程设计。

## 功能

* 通过 libloading 实现动态模组加载
* souprune_api 的最小宿主实现
* 隔离测试环境
* 快速模组开发迭代
* 适合 CI/CD 自动化测试
* 无需完整游戏引擎

## 使用方法

1. **安装 Rust**（如果尚未安装）：
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **构建你的模组**：

   ```bash
   cargo build -p souprune_mod_test --release
   ```

3. **运行模拟宿主**：

   ```bash
   cargo run -p souprune_mock_host -- path/to/your/mod.so
   ```

4. **测试模组功能**：

   模拟宿主将加载你的模组并执行测试场景。

## 依赖

本项目使用以下 crate：

| Crate                                             | 版本 | 描述                 |
| ------------------------------------------------- | ------- | --------------------------- |
| [souprune_api](https://crates.io/crates/souprune_api) | 0.0.1   | FFI API 层 |
| [libloading](https://crates.io/crates/libloading) | 0.9.0   | 动态库加载 |

## 警告

⚠️ **这是一个实验性的早期版本。**

- API 不稳定，可能会发生重大变化
- 仅用于测试目的
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
