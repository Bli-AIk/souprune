# bevy_alight_motion

[![license](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](LICENSE-APACHE) <img src="https://img.shields.io/github/repo-size/Bli-AIk/bevy_alight_motion.svg"/> <img src="https://img.shields.io/github/last-commit/Bli-AIk/bevy_alight_motion.svg"/> <br>
<img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" />

> 当前状态: 🚧 早期开发 (初始版本进行中)

**bevy_alight_motion** — 用于加载和播放 Alight Motion 项目文件的 Bevy 插件。

| English | 简体中文 |
|---------|--------------------|
| [English](./readme.md) | 简体中文 |

## 简介

`bevy_alight_motion` 是一个为 [Bevy](https://bevyengine.org/) 游戏引擎开发的插件，允许你直接从 [Alight Motion](https://alightmotion.com/) 项目文件中导入资产和动画。  
它解决了在代码中手动重建动画的问题，让设计师可以在 Alight Motion 中创建复杂的动画，并由开发者在 Bevy 中直接运行。

使用 `bevy_alight_motion`，你只需将 Alight Motion 项目导出为 `.amproj` 文件，然后通过一个函数调用即可加载。  
未来，它还可能支持从 Alight Motion 导出的更复杂的特效和着色器。

## 功能

* 加载 `.amproj` ZIP 归档和独立的 `.xml` 项目文件。
* 自动关键帧动画，支持 cubic-bezier (三次贝塞尔) 和 step (步进) 缓动。
* 坐标系转换 (Alight Motion 的左上角原点转换为 Bevy 的中心原点)。
* 支持嵌套场景 (预合成)。
* 通过 ECS 组件实现可自定义的播放控制。
* (计划中) 支持更多的形状类型和特效。

## 如何使用

1. **添加依赖** 到你的 `Cargo.toml`:
   ```toml
   [dependencies]
   bevy_alight_motion = { git = "https://github.com/Bli-AIk/souprune", path = "crates/bevy_alight_motion" }
   ```

2. **注册插件** 在你的 Bevy App 中:
   ```rust
   use bevy::prelude::*;
   use bevy_alight_motion::prelude::*;

   fn main() {
       App::new()
           .add_plugins(DefaultPlugins)
           .add_plugins(AlightMotionPlugin)
           .add_systems(Startup, setup)
           .run();
   }
   ```

3. **加载项目**:
   ```rust
   fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
       commands.spawn(Camera2d);
       // 从你的 assets 文件夹加载 AM 项目
       load_am_project(&mut commands, &asset_server, "am/project.amproj");
   }
   ```

4. **运行示例播放器**:
   ```bash
   cargo run --example player
   ```

## 如何构建

### 前置条件

* Rust 1.80 或更高版本 (使用 2024 edition)

### 构建步骤

1. **克隆仓库**:
   ```bash
   git clone https://github.com/Bli-AIk/souprune.git
   cd souprune/crates/bevy_alight_motion
   ```

2. **构建项目**:
   ```bash
   cargo build --release
   ```

3. **运行测试**:
   ```bash
   cargo test
   ```

## 依赖项

本项目使用了以下 crate:

| Crate | 版本 | 描述 |
|-------|---------|-------------|
| [bevy](https://crates.io/crates/bevy) | 0.17.2 | 游戏引擎 |
| [quick-xml](https://crates.io/crates/quick-xml) | 0.37 | 高性能 XML 解析/序列化库 |
| [serde](https://crates.io/crates/serde) | 1.0 | 序列化/反序列化框架 |
| [zip](https://crates.io/crates/zip) | 2.2 | ZIP 归档读写库 |
| [thiserror](https://crates.io/crates/thiserror) | 2.0 | 错误派生宏 |

## 贡献

欢迎贡献！
无论你是想修复 Bug、添加新功能还是改进文档：

* 提交 **Issue** 或 **Pull Request**。
* 分享想法并讨论设计或架构。

## 许可

本项目采用以下任一许可协议授权：

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) 或 [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))
* MIT license ([LICENSE-MIT](LICENSE-MIT) 或 [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))

可根据你的选择使用。