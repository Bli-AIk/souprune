# souprune

[![license](https://img.shields.io/badge/license-LGPL--3.0--or--later-blue)](LICENSE) <img src="https://img.shields.io/github/repo-size/Bli-AIk/souprune.svg"/> <img src="https://img.shields.io/github/last-commit/Bli-AIk/souprune.svg"/> <br>
<img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" />

> Current Status: 🚧 Early Development

**souprune** — A Bevy-based game framework for creating Deltarune/Undertale-style fangames.

| English | Simplified Chinese          |
|---------|-----------------------------|
| English | [简体中文](./readme_zh-hans.md) |

## Introduction

`souprune` is the main game framework crate, built on top of Bevy 0.18. It provides the complete infrastructure for Deltarune/Undertale-style fangames — including dialogue, battle, overworld, animation, collision, and mod systems — while avoiding concrete game logic.

With `souprune`, developers define game content through RON configuration files and WASM mods, and the framework handles rendering, physics, input, and state management.

## Features

* **App State Management** — AppSetup → Menu → Overworld → Battle lifecycle
* **Danmaku System** — Timeline-based bullet patterns and behaviors, fully WASM-dispatched
* **View System** — RON-driven UI with SDF rendering, hot-reloadable layouts
* **Dialogue System** — Mortar-scripted dialogue with typewriter effects
* **WASM Mod System** — wasmtime 42 + Component Model for extensible game logic
* **FRE Bridge** — Fact-Rule-Event engine integration for data-driven gameplay
* **Input System** — Keyboard, gamepad, and touch input via leafwing-input-manager
* **Collision System** — Trigger-based collision detection for battle mechanics
* **Animation System** — Sprite animation, character animation, and Alight Motion import
* **Audio System** — Sound and music playback via bevy_kira_audio

## How to Use

Add to your `Cargo.toml`:
```toml
[dependencies]
souprune = { path = "../souprune" }
```

Run the demo:
```bash
cargo run -p souprune
```

With debug features (inspector + performance HUD):
```bash
cargo run -p souprune --features debug
```

## How to Build

### Prerequisites

* Rust 1.85 or later
* System dependencies (Linux):
  ```bash
  sudo apt-get install -y g++ pkg-config libx11-dev libasound2-dev libudev-dev \
      libwayland-dev libxkbcommon-dev
  ```

### Build Steps

1. **Initialize submodules**:
   ```bash
   git submodule update --init --recursive
   ```

2. **Build the project**:
   ```bash
   cargo build --release -p souprune
   ```

3. **Run tests**:
   ```bash
   cargo test --workspace
   ```

## Dependencies

This crate uses the following key dependencies:

| Crate                                                              | Version | Description                       |
|--------------------------------------------------------------------|---------|-----------------------------------|
| [bevy](https://crates.io/crates/bevy)                             | 0.18    | Game engine                       |
| [wasmtime](https://crates.io/crates/wasmtime)                     | 42      | WASM runtime for mod system       |
| [leafwing-input-manager](https://crates.io/crates/leafwing-input-manager) | 0.20 | Action-based input handling     |
| [bevy_kira_audio](https://crates.io/crates/bevy_kira_audio)       | 0.25    | Audio playback                    |
| [bevy_ecs_tiled](https://crates.io/crates/bevy_ecs_tiled)         | 0.11    | Tiled map support                 |
| [ron](https://crates.io/crates/ron)                                | 0.12    | RON configuration parsing         |
| [fasteval](https://crates.io/crates/fasteval)                     | 0.2     | Expression evaluation             |

## Warning

⚠️ **This is an early development release.**

- API is unstable and may change significantly
- Limited documentation
- Not recommended for production use
- Breaking changes expected in future versions

## Contributing

Contributions are welcome!
Whether you want to fix a bug, add a feature, or improve documentation:

* Submit an **Issue** or **Pull Request**.
* Share ideas and discuss design or architecture.

## License

This project is licensed under

* GNU Lesser General Public License v3.0 or later ([LICENSE](../../LICENSE.md) or [https://www.gnu.org/licenses/lgpl-3.0.html](https://www.gnu.org/licenses/lgpl-3.0.html))
