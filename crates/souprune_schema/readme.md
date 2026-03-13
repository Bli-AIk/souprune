# souprune_schema

[![license](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](LICENSE) <img src="https://img.shields.io/github/repo-size/Bli-AIk/souprune.svg"/> <img src="https://img.shields.io/github/last-commit/Bli-AIk/souprune.svg"/> <br>
<img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" />

> Current Status: 🚧 Early Development

**souprune_schema** — Pure Rust data types defining the schema for all SoupRune RON configuration files.

| English | Simplified Chinese          |
|---------|-----------------------------|
| English | [简体中文](./readme_zh-hans.md) |

## Introduction

`souprune_schema` provides type definitions for serialization and deserialization of SoupRune game configuration files. It is a data-only crate with zero Bevy/game-engine dependencies, designed for use by both the main game engine and standalone tools (linters, editors, CI validators).

This enables type-safe parsing and validation of `.view.ron`, `.enemy.ron`, `.sequence.ron`, `.performance.ron`, and other configuration formats without pulling in heavy engine dependencies.

## Features

* **Zero engine dependencies** — Pure `serde` + `ron`, no Bevy required
* **Comprehensive schema coverage**:
  - `view` — View/scene layout definitions
  - `enemy` — Enemy definitions and combat properties
  - `battle` — Battle configuration and mechanics
  - `danmaku` — Bullet spawn/behavior configurations
  - `character` — Character/NPC definitions
  - `overworld` — Overworld gameplay settings
  - `sequence` — Sequence/event scripting
  - `item` — Item/inventory definitions
  - `fre` — FRE expression definitions
  - `config` — Global game configuration
* **File kind detection** — `RonFileKind` enum for automatic file type identification from path
* **Bevy-compatible types** — Color, vector, and transform types without Bevy dependency

## How to Use

Add to `Cargo.toml`:
```toml
[dependencies]
souprune_schema = { path = "../souprune_schema" }
```

Parse a RON file:
```rust
use souprune_schema::from_ron_str;
use souprune_schema::danmaku::PerformanceDef;

let ron_str = std::fs::read_to_string("demo_attack.performance.ron")?;
let performance: PerformanceDef = from_ron_str(&ron_str)?;
```

## How to Build

```bash
cargo build -p souprune_schema
cargo test -p souprune_schema
```

## Dependencies

| Crate                                            | Version | Description             |
|--------------------------------------------------|---------|-------------------------|
| [serde](https://crates.io/crates/serde)         | 1.0     | Serialization framework |
| [ron](https://crates.io/crates/ron)              | 0.12    | RON format support      |

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
