# souprune_schema

[![license](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](LICENSE) <img src="https://img.shields.io/github/repo-size/Bli-AIk/souprune.svg"/> <img src="https://img.shields.io/github/last-commit/Bli-AIk/souprune.svg"/> <br>
<img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" />

> 当前状态：🚧 早期开发中

**souprune_schema** — 定义所有 SoupRune RON 配置文件 schema 的纯 Rust 数据类型。

| 英语 | 简体中文 |
|------|----------|
| [English](./readme.md) | 简体中文 |

## 介绍

`souprune_schema` 提供 SoupRune 游戏配置文件的序列化和反序列化类型定义。这是一个纯数据 crate，没有任何 Bevy/游戏引擎依赖，设计用于主游戏引擎和独立工具（检查器、编辑器、CI 验证器）。

它实现了 `.view.ron`、`.enemy.ron`、`.sequence.ron`、`.performance.ron` 等配置格式的类型安全解析和验证，无需引入沉重的引擎依赖。

## 功能

* **零引擎依赖** — 纯 `serde` + `ron`，不需要 Bevy
* **全面的 schema 覆盖**：
  - `view` — 视图/场景布局定义
  - `enemy` — 敌人定义和战斗属性
  - `battle` — 战斗配置和机制
  - `danmaku` — 弹幕生成/行为配置
  - `character` — 角色/NPC 定义
  - `overworld` — 大世界游戏设置
  - `sequence` — 序列/事件脚本
  - `item` — 物品/背包定义
  - `fre` — FRE 表达式定义
  - `config` — 全局游戏配置
* **文件类型检测** — `RonFileKind` 枚举，根据路径自动识别文件类型
* **Bevy 兼容类型** — 颜色、向量和变换类型，无需 Bevy 依赖

## 使用方法

添加到 `Cargo.toml`：
```toml
[dependencies]
souprune_schema = { path = "../souprune_schema" }
```

解析 RON 文件：
```rust
use souprune_schema::from_ron_str;
use souprune_schema::danmaku::PerformanceDef;

let ron_str = std::fs::read_to_string("demo_attack.performance.ron")?;
let performance: PerformanceDef = from_ron_str(&ron_str)?;
```

## 构建方法

```bash
cargo build -p souprune_schema
cargo test -p souprune_schema
```

## 依赖

| Crate                                            | 版本  | 描述           |
|--------------------------------------------------|-------|----------------|
| [serde](https://crates.io/crates/serde)         | 1.0   | 序列化框架     |
| [ron](https://crates.io/crates/ron)              | 0.12  | RON 格式支持   |

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
