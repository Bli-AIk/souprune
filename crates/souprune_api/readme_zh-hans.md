# souprune_api

[![license](https://img.shields.io/badge/license-LGPL--3.0--or--later-blue)](LICENSE) <img src="https://img.shields.io/github/repo-size/Bli-AIk/souprune.svg"/> <img src="https://img.shields.io/github/last-commit/Bli-AIk/souprune.svg"/> <br>
<img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" />

> 当前状态：🚧 早期开发中

**souprune_api** — SoupRune 模组系统的 WIT 接口定义和共享类型。

| 英语 | 简体中文 |
|------|----------|
| [English](./readme.md) | 简体中文 |

## 介绍

`souprune_api` 定义了 SoupRune 引擎（宿主）与 WASM mod 组件（客户端）之间的契约，
使用 WIT（WebAssembly Interface Types）。它提供宿主运行时和客户端 SDK（`souprune_sdk`）共用的 Rust 类型。

## 功能

* WIT 接口定义（`wit/souprune-mod.wit`）——单一事实来源
* 共享 Rust 类型：`Vec2`、`BulletContext`、`BulletOutput`、`Action`
* 宿主侧：由 `souprune`（wasmtime）使用定义导入
* 客户端侧：由 `souprune_sdk`（wit-bindgen）使用实现导出

## WIT 接口

WIT 文件定义了三个接口：

- **`host-api`**（由客户端导入）：`log`、`get-fact`、`set-fact`、`emit-event`
- **`behavior`**（由客户端导出）：`on-init`、`on-update`、`on-interact`
- **`danmaku`**（由客户端导出）：`init-bullet`、`update-bullet`

## 使用方法

**引擎开发**（宿主侧），添加到 `Cargo.toml`：
```toml
[dependencies]
souprune_api = { path = "../souprune_api" }
```

**mod 开发**（客户端侧），使用 `souprune_sdk`，它会重新导出所需类型。

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
