# bevy_bitmap_text

[![license](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](LICENSE-APACHE)
<img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" />

> Current Status: 🚧 Early Development (Initial version in progress)

**bevy_bitmap_text** — Glyph-as-Entity dynamic atlas text rendering for Bevy.

| English | Simplified Chinese          |
|---------|-----------------------------|
| English | [简体中文](./readme_zh-hans.md) |

## Introduction

`bevy_bitmap_text` is a text rendering backend for [Bevy](https://bevyengine.org/) that spawns each character as an
individual ECS entity with a `Sprite` component.  
It solves the limitation of monolithic text meshes, allowing users to apply per-character ECS-driven animations such as
shake, wave, and color changes.

With `bevy_bitmap_text`, you only need to add a `TextBlock` component and configure `TextBlockStyling` — the plugin
handles rasterization, atlas packing, layout, and entity synchronization automatically.

## Features

* **Glyph-as-Entity architecture** — each character is a separate `Sprite` entity, enabling per-character animation and
  styling via standard ECS
* **Dynamic glyph atlas** — on-demand rasterization with [fontdue](https://github.com/mooman219/fontdue) and rectangle
  packing via [etagere](https://crates.io/crates/etagere)
* **Typewriter effect** — built-in `GlyphReveal` component for progressive character reveal
* **Per-character effects** — `ShakeEffect` and `WaveEffect` components for jitter and sine-wave animations
* **Color tags** — inline `{#RRGGBB:text}` markup for per-segment coloring
* **Text styling** — configurable font, size, color, alignment, anchor, line height, character spacing, and word spacing
* **Auto line-wrapping** — optional `max_width` for automatic word wrap

## How to Use

1. **Add the dependency** to your `Cargo.toml`:

   ```toml
   [dependencies]
   bevy_bitmap_text = { path = "crates/bevy_bitmap_text" }
   ```

2. **Register the plugin**:

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

3. **Spawn a text block**:

   ```rust
   use bevy_bitmap_text::*;

   fn setup(mut commands: Commands) {
       commands.spawn(Camera2d);
       commands.spawn((
           TextBlock::new("Hello, world!"),
           TextBlockStyling {
               font: FontId::from_name("my_font"),
               size_px: 32,
               world_scale: 32.0,
               ..default()
           },
       ));
   }
   ```

4. **Typewriter effect**:

   ```rust
   commands.spawn((
       TextBlock::new("Revealing text..."),
       TextBlockStyling { /* ... */ },
       GlyphReveal { visible_count: 0 },
   ));
   // Increment `visible_count` each frame to reveal characters one by one.
   ```

5. **Color tags**:

   ```rust
   use bevy_bitmap_text::parse_text_to_segments;
   let segments = parse_text_to_segments("{#FF0000:Red} and {#00FF00:Green}");
   commands.spawn((
       TextBlock::from_segments(segments),
       TextBlockStyling { /* ... */ },
   ));
   ```

## How to Build

### Prerequisites

* Rust 1.85 or later (edition 2024)
* Bevy 0.18

### Build Steps

1. **Build the crate**:

   ```bash
   cargo build -p bevy_bitmap_text
   ```

2. **Run tests**:

   ```bash
   cargo test -p bevy_bitmap_text
   ```

3. **Run the demo**:

   ```bash
   cargo run -p bevy_bitmap_text --example demo
   ```

## Dependencies

This project uses the following crates:

| Crate                                       | Version | Description                                          |
|---------------------------------------------|---------|------------------------------------------------------|
| [bevy](https://crates.io/crates/bevy)       | 0.18    | Game engine framework (asset, color, render, sprite) |
| [fontdue](https://crates.io/crates/fontdue) | 0.7     | Lightweight font rasterization                       |
| [etagere](https://crates.io/crates/etagere) | 0.2     | Rectangle atlas packing                              |
| [log](https://crates.io/crates/log)         | 0.4     | Logging facade                                       |

## Contributing

Contributions are welcome!
Whether you want to fix a bug, add a feature, or improve documentation:

* Submit an **Issue** or **Pull Request**.
* Share ideas and discuss design or architecture.

## License

This project is licensed under either of

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE)
  or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))
* MIT license ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))

at your option.
