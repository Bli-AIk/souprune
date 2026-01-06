# bevy_alight_motion

[![license](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](LICENSE-APACHE) <img src="https://img.shields.io/github/repo-size/Bli-AIk/bevy_alight_motion.svg"/> <img src="https://img.shields.io/github/last-commit/Bli-AIk/bevy_alight_motion.svg"/> <br>
<img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" />

> Current Status: 🚧 Early Development (Initial version in progress)

**bevy_alight_motion** — Bevy plugin for loading and playing Alight Motion project files.

| English | Simplified Chinese |
|---------|--------------------|
| English | [简体中文](./readme_zh-hans.md) |

## Introduction

`bevy_alight_motion` is a plugin for the [Bevy](https://bevyengine.org/) game engine that allows you to import and animate assets directly from [Alight Motion](https://alightmotion.com/) project files.  
It solves the problem of manual animation recreation in code, allowing designers to create complex animations in Alight Motion and developers to run them directly in Bevy.

With `bevy_alight_motion`, you only need to export your Alight Motion project as an `.amproj` file and load it with a single function call.  
In the future, it may also support more complex effects and shaders exported from Alight Motion.

## Features

* Load `.amproj` ZIP archives and standalone `.xml` project files.
* Automatic keyframe animation with cubic-bezier and step easing support.
* Coordinate system conversion (Alight Motion top-left origin to Bevy center origin).
* Support for nested scenes (pre-compositions).
* Customizable playback control via ECS components.
* (Planned) Support for more shape types and effects.

## How to Use

1. **Add Dependency** to your `Cargo.toml`:
   ```toml
   [dependencies]
   bevy_alight_motion = { git = "https://github.com/Bli-AIk/souprune", path = "crates/bevy_alight_motion" }
   ```

2. **Register the Plugin** in your Bevy App:
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

3. **Load a Project**:
   ```rust
   fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
       commands.spawn(Camera2d);
       // Load the AM project from your assets folder
       load_am_project(&mut commands, &asset_server, "am/project.amproj");
   }
   ```

4. **Run the Example Player**:
   ```bash
   cargo run --example player
   ```

## How to Build

### Prerequisites

* Rust 1.80 or later (uses 2024 edition)

### Build Steps

1. **Clone the repository**:
   ```bash
   git clone https://github.com/Bli-AIk/souprune.git
   cd souprune/crates/bevy_alight_motion
   ```

2. **Build the project**:
   ```bash
   cargo build --release
   ```

3. **Run tests**:
   ```bash
   cargo test
   ```

## Dependencies

This project uses the following crates:

| Crate | Version | Description |
|-------|---------|-------------|
| [bevy](https://crates.io/crates/bevy) | 0.17.2 | Game engine |
| [quick-xml](https://crates.io/crates/quick-xml) | 0.37 | High-performance XML pull-parser/serializer |
| [serde](https://crates.io/crates/serde) | 1.0 | Serialization/deserialization framework |
| [zip](https://crates.io/crates/zip) | 2.2 | ZIP archive reading/writing |
| [thiserror](https://crates.io/crates/thiserror) | 2.0 | Error derive macros |

## Contributing

Contributions are welcome!
Whether you want to fix a bug, add a feature, or improve documentation:

* Submit an **Issue** or **Pull Request**.
* Share ideas and discuss design or architecture.

## License

This project is licensed under either of

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))
* MIT license ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))

at your option.