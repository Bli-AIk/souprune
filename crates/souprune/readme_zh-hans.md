# souprune

[![license](https://img.shields.io/badge/license-LGPL--3.0--or--later-blue)](LICENSE) <img src="https://img.shields.io/github/repo-size/Bli-AIk/souprune.svg"/> <img src="https://img.shields.io/github/last-commit/Bli-AIk/souprune.svg"/> <br>
<img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" />

> 当前状态：🚧 早期开发中

**souprune** — 基于 Bevy 的 Deltarune/Undertale 风格同人游戏框架。

| 英语 | 简体中文 |
|------|----------|
| [English](./readme.md) | 简体中文 |

## 介绍

`souprune` 是主游戏框架 crate，基于 Bevy 0.18 构建。它为 Deltarune/Undertale 风格的同人游戏提供完整的基础设施——包括对话、战斗、大世界、动画、碰撞和 mod 系统——同时不包含具体游戏逻辑。

开发者通过 RON 配置文件和 WASM mod 定义游戏内容，框架负责渲染、物理、输入和状态管理。

## 功能

* **应用状态管理** — AppSetup → Menu → Overworld → Battle 生命周期
* **弹幕系统** — 基于时间轴的弹幕模式和行为，完全通过 WASM 调度
* **视图系统** — RON 驱动的 UI，支持 SDF 渲染和热重载布局
* **对话系统** — Mortar 脚本对话，带打字机效果
* **WASM Mod 系统** — wasmtime 42 + Component Model，支持可扩展游戏逻辑
* **FRE 桥接** — Fact-Rule-Event 规则引擎集成，实现数据驱动的游戏玩法
* **输入系统** — 键盘、手柄和触屏输入，通过 leafwing-input-manager
* **碰撞系统** — 基于触发器的碰撞检测，用于战斗机制
* **动画系统** — 精灵动画、角色动画和 Alight Motion 导入
* **音频系统** — 通过 bevy_kira_audio 播放音效和音乐

## 使用方法

添加到 `Cargo.toml`：
```toml
[dependencies]
souprune = { path = "../souprune" }
```

运行演示：
```bash
cargo run -p souprune
```

启用调试功能（检查器 + 性能 HUD）：
```bash
cargo run -p souprune --features debug
```

## 构建方法

### 前置要求

* Rust 1.85 或更高版本
* 系统依赖（Linux）：
  ```bash
  sudo apt-get install -y g++ pkg-config libx11-dev libasound2-dev libudev-dev \
      libwayland-dev libxkbcommon-dev
  ```

### 构建步骤

1. **初始化子模块**：
   ```bash
   git submodule update --init --recursive
   ```

2. **构建项目**：
   ```bash
   cargo build --release -p souprune
   ```

3. **运行测试**：
   ```bash
   cargo test --workspace
   ```

## 依赖

本 crate 使用以下主要依赖：

| Crate                                                              | 版本   | 描述                      |
|--------------------------------------------------------------------|--------|---------------------------|
| [bevy](https://crates.io/crates/bevy)                             | 0.18   | 游戏引擎                  |
| [wasmtime](https://crates.io/crates/wasmtime)                     | 42     | WASM 运行时，用于 mod 系统 |
| [leafwing-input-manager](https://crates.io/crates/leafwing-input-manager) | 0.20 | 基于动作的输入处理       |
| [bevy_kira_audio](https://crates.io/crates/bevy_kira_audio)       | 0.25   | 音频播放                  |
| [bevy_ecs_tiled](https://crates.io/crates/bevy_ecs_tiled)         | 0.11   | Tiled 地图支持            |
| [ron](https://crates.io/crates/ron)                                | 0.12   | RON 配置解析              |
| [fasteval](https://crates.io/crates/fasteval)                     | 0.2    | 表达式求值                |

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
