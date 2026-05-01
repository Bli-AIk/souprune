# souprune-lint

[![license](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](LICENSE) <img src="https://img.shields.io/github/repo-size/Bli-AIk/souprune.svg"/> <img src="https://img.shields.io/github/last-commit/Bli-AIk/souprune.svg"/> <br>
<img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" />

> 当前状态：🚧 早期开发中

**souprune-lint** — 用于验证 SoupRune RON 配置文件的命令行检查工具。

| 英语 | 简体中文 |
|------|----------|
| [English](./readme.md) | 简体中文 |

## 介绍

`souprune-lint` 是一个独立的二进制工具，用于对 SoupRune 游戏配置文件（`.view.ron`、`.sdf.ron`、`.enemy.ron`、`.performance.ron` 等）进行静态验证。它检查 RON 文件的结构正确性和 schema 违规，以多种输出格式生成诊断信息，适用于 CI/CD 管线和 IDE 集成。

## 功能

* **Schema 验证** — 根据 `souprune_schema` 类型定义验证 RON 文件
* **文件类型检测** — 根据扩展名自动识别文件类型（`.view.ron`、`.sdf.ron`、`.enemy.ron`、`.items.ron`、`.sequence.ron`、`.fre.ron`、`.performance.ron` 等）
* **递归扫描** — 使用 `walkdir` 检查整个目录
* **多种输出格式**：
  - `pretty` — 基于 ariadne 的彩色诊断显示
  - `jetbrains` — IDE 兼容的 `file:line:col: level: message` 格式
  - `json` — 机器可读的结构化输出

## 使用方法

```bash
# 检查单个文件
cargo run -p souprune-lint -- check path/to/file.view.ron

# 递归检查目录
cargo run -p souprune-lint -- check projects/mad_dummy_example/

# 指定输出格式
cargo run -p souprune-lint -- check --format jetbrains projects/
```

## 构建方法

```bash
cargo build --release -p souprune-lint
```

二进制文件位于 `target/release/souprune-lint`。

## 依赖

| Crate                                                  | 版本  | 描述               |
|--------------------------------------------------------|-------|--------------------|
| [souprune_schema](../souprune_schema)                  | 0.1   | Schema 类型定义    |
| [ron](https://crates.io/crates/ron)                    | 0.12  | RON 解析           |
| [clap](https://crates.io/crates/clap)                 | 4     | CLI 参数解析       |
| [ariadne](https://crates.io/crates/ariadne)           | 0.4   | 诊断信息渲染       |
| [walkdir](https://crates.io/crates/walkdir)           | 2     | 递归文件遍历       |

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
