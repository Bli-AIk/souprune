# souprune_builtins

[![license](https://img.shields.io/badge/license-LGPL--3.0--or--later-blue)](LICENSE) <img src="https://img.shields.io/github/repo-size/Bli-AIk/souprune.svg"/> <img src="https://img.shields.io/github/last-commit/Bli-AIk/souprune.svg"/> <br>
<img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" />

> 当前状态：🚧 早期开发中

**souprune_builtins** — 为 SoupRune 提供内置生成模式和弹幕行为的预编译 WASM 模块。

| 英语                     | 简体中文 |
|------------------------|------|
| [English](./readme.md) | 简体中文 |

## 介绍

`souprune_builtins` 是一个 WASM 客户端模块（编译为 `cdylib`，目标 `wasm32-wasip2`），实现了 SoupRune 弹幕系统使用的所有内置生成模式和子弹行为。

框架不再硬编码游戏逻辑，所有内置行为都通过与社区 mod 开发者相同的 `souprune_sdk` 接口实现为 WASM 模块。这确保了内置内容和自定义内容之间的一致性。

## 功能

**生成模式：**

* `builtin.single` — 在中心生成单个子弹
* `builtin.ring` — 圆形排列（数量、半径、起始角度）
* `builtin.line` — 线性排列（数量、间距、方向）
* `builtin.edge` — 边缘生成子弹（数量、边、间距、边距）

**弹幕行为：**

* `builtin.linear` — 匀速直线运动
* `builtin.orbital` — 圆周/轨道运动
* `builtin.sine` — 正弦波运动
* `builtin.stationary` — 静止不动
* `builtin.aimed` — 追踪玩家
* `builtin.tween` — 动画属性变化（透明度、缩放、位置、旋转，支持缓动函数）

## 构建方法

### 前置要求

* Rust 并安装 `wasm32-wasip2` 目标：
  ```bash
  rustup target add wasm32-wasip2
  ```

### 构建步骤

```bash
cd crates/souprune_builtins
cargo build --target wasm32-wasip2 --release
```

编译后的 WASM 二进制文件位于：

```
target/wasm32-wasip2/release/souprune_builtins.wasm
```

## 依赖

| Crate                           | 版本  | 描述           |
|---------------------------------|-----|--------------|
| [souprune_sdk](../souprune_sdk) | 0.2 | WASM 客户端 SDK |

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

* GNU Lesser General Public License v3.0 或更高版本（[LICENSE](../../LICENSE.md)
  或 [https://www.gnu.org/licenses/lgpl-3.0.html](https://www.gnu.org/licenses/lgpl-3.0.html)）
