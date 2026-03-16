# bevy_bitmap_text

[![license](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](LICENSE-APACHE)
<img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" />

> 当前状态：🚧 早期开发中（初始版本开发中）

**bevy_bitmap_text** — 基于 Bevy 的 Glyph-as-Entity 动态图集文本渲染。

| English                | 简体中文 |
|------------------------|------|
| [English](./readme.md) | 简体中文 |

## 简介

`bevy_bitmap_text` 是一个面向 [Bevy](https://bevyengine.org/) 的文本渲染后端，将每个字符作为独立的 ECS 实体生成，并附带
`Sprite` 组件。  
它解决了整体式文本网格的局限性，允许用户对每个字符施加 ECS 驱动的动画效果，如抖动、波浪和颜色变化。

使用 `bevy_bitmap_text`，你只需添加 `TextBlock` 组件并配置 `TextBlockStyling`——插件会自动处理光栅化、图集打包、排版布局和实体同步。

## 功能特性

* **Glyph-as-Entity 架构** — 每个字符是独立的 `Sprite` 实体，通过标准 ECS 实现逐字动画和样式控制
* **动态字形图集** — 使用 [fontdue](https://github.com/mooman219/fontdue)
  按需光栅化，通过 [etagere](https://crates.io/crates/etagere) 进行矩形装箱
* **打字机效果** — 内置 `GlyphReveal` 组件，实现逐字显示
* **逐字特效** — `ShakeEffect` 和 `WaveEffect` 组件，提供抖动和正弦波动画
* **颜色标签** — 内联 `{#RRGGBB:文本}` 标记，实现分段着色
* **文本样式** — 可配置字体、大小、颜色、对齐、锚点、行高、字间距和词间距
* **自动换行** — 可选的 `max_width` 实现自动换行

## 使用方法

1. **添加依赖** 到 `Cargo.toml`：

   ```toml
   [dependencies]
   bevy_bitmap_text = { path = "crates/bevy_bitmap_text" }
   ```

2. **注册插件**：

   ```rust
   use bevy::prelude::*;
   use bevy_bitmap_text::{BitmapTextPlugin, FontDirectories};

   fn main() {
       App::new()
           .add_plugins(DefaultPlugins)
           .insert_resource(FontDirectories {
               directories: vec!["assets/fonts".to_string()],
           })
           .add_plugins(BitmapTextPlugin::default())
           .add_systems(Startup, setup)
           .run();
   }
   ```

3. **生成文本块**：

   ```rust
   use bevy_bitmap_text::*;

   fn setup(mut commands: Commands) {
       commands.spawn(Camera2d);
       commands.spawn((
           TextBlock::new("你好，世界！"),
           TextBlockStyling {
               font: FontId::from_name("my_font"),
               size_px: 32,
               world_scale: 32.0,
               ..default()
           },
       ));
   }
   ```

4. **打字机效果**：

   ```rust
   commands.spawn((
       TextBlock::new("逐字显示的文本..."),
       TextBlockStyling { /* ... */ },
       GlyphReveal { visible_count: 0 },
   ));
   // 每帧递增 `visible_count` 以逐字显示。
   ```

5. **颜色标签**：

   ```rust
   use bevy_bitmap_text::parse_text_to_segments;
   let segments = parse_text_to_segments("{#FF0000:红色} 和 {#00FF00:绿色}");
   commands.spawn((
       TextBlock::from_segments(segments),
       TextBlockStyling { /* ... */ },
   ));
   ```

## 构建方法

### 前置要求

* Rust 1.85 或更高版本（edition 2024）
* Bevy 0.18

### 构建步骤

1. **构建 crate**：

   ```bash
   cargo build -p bevy_bitmap_text
   ```

2. **运行测试**：

   ```bash
   cargo test -p bevy_bitmap_text
   ```

3. **运行演示**：

   ```bash
   cargo run -p bevy_bitmap_text --example demo
   ```

## 依赖

本项目使用以下 crate：

| Crate                                       | 版本   | 说明                  |
|---------------------------------------------|------|---------------------|
| [bevy](https://crates.io/crates/bevy)       | 0.18 | 游戏引擎框架（资产、颜色、渲染、精灵） |
| [fontdue](https://crates.io/crates/fontdue) | 0.7  | 轻量级字体光栅化            |
| [etagere](https://crates.io/crates/etagere) | 0.2  | 矩形图集装箱              |
| [log](https://crates.io/crates/log)         | 0.4  | 日志门面                |

## 贡献

欢迎贡献！
无论是修复 bug、添加功能还是改进文档：

* 提交 **Issue** 或 **Pull Request**。
* 分享想法，讨论设计或架构。

## 许可证

本项目采用以下任一许可证：

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE)
  或 [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))
* MIT license ([LICENSE-MIT](LICENSE-MIT) 或 [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))

由你选择。
