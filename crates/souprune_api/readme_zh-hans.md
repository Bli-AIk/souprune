# souprune_api

[![license](https://img.shields.io/badge/license-LGPL--3.0--or--later-blue)](LICENSE) <img src="https://img.shields.io/github/repo-size/Bli-AIk/souprune.svg"/> <img src="https://img.shields.io/github/last-commit/Bli-AIk/souprune.svg"/> <br>
<img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" />

> 当前状态：🚧 早期开发中

**souprune_api** — SoupRune 游戏框架的 FFI API 定义和绑定生成器。

| 英语 | 简体中文 |
|------|----------|
| [English](./readme.md) | 简体中文 |

## 介绍

`souprune_api` 是 SoupRune 游戏框架的 FFI（外部函数接口）层。  
它解决了语言互操作性问题，让用户能够从 Rust 以外的语言（C#、Haxe、Lua 等）与 SoupRune 交互。

使用 `souprune_api`，你只需要在目标语言中使用生成的绑定，就可以访问 SoupRune 的功能。  
未来还计划支持动态插件加载和热重载。

## 功能

* SoupRune 的 FFI 安全 API 定义
* 多语言绑定生成（C、C#、Haxe）
* 与 Interoptopus 集成实现自动代码生成
* 类型安全的跨语言通信
* （计划中）插件系统支持
* （计划中）热重载能力

## 使用方法

1. **安装 Rust**（如果尚未安装）：
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **添加到 Cargo.toml**：

   ```toml
   [dependencies]
   souprune_api = "0.1"
   ```

3. **生成绑定**（使用 bindgen 特性）：

   ```bash
   cargo run --bin souprune_bindgen --features bindgen
   ```

   这将生成：
   - C 头文件
   - C# 绑定
   - Haxe 绑定

4. **在你的语言中使用**：

   查看输出目录中生成的绑定文件以获取使用示例。

## 依赖

本项目使用以下 crate：

| Crate                                             | 版本 | 描述                 |
| ------------------------------------------------- | ------- | --------------------------- |
| [interoptopus](https://crates.io/crates/interoptopus) | 0.15.0-alpha.24   | FFI 绑定框架 |
| [interoptopus_backend_c](https://crates.io/crates/interoptopus_backend_c) | 0.15.0-alpha.24   | C 后端 |
| [interoptopus_backend_csharp](https://crates.io/crates/interoptopus_backend_csharp) | 0.15.0-alpha.24   | C# 后端 |
| [interoptopus_backend_haxe](https://crates.io/crates/interoptopus_backend_haxe) | 0.0.1   | Haxe 后端 |

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

* GNU Lesser General Public License v3.0 或更高版本（[LICENSE](LICENSE) 或 [https://www.gnu.org/licenses/lgpl-3.0.html](https://www.gnu.org/licenses/lgpl-3.0.html)）
