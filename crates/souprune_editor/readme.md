# souprune_editor

[![license](https://img.shields.io/badge/license-LGPL--3.0--or--later-blue)](LICENSE) <img src="https://img.shields.io/github/repo-size/Bli-AIk/souprune.svg"/> <img src="https://img.shields.io/github/last-commit/Bli-AIk/souprune.svg"/> <br>
<img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" />

> Current Status: 🚧 Early Development

**souprune_editor** — A Bevy + egui visual editor for creating and editing SoupRune game content in real-time.

| English | Simplified Chinese          |
|---------|-----------------------------|
| English | [简体中文](./readme_zh-hans.md) |

## Introduction

`souprune_editor` is a visual game authoring environment built on bevy_workbench and egui. It renders the game in a viewport while allowing live editing of scenes, views, and configurations. The editor shares the same plugin infrastructure as the standalone game, ensuring that edit-time behavior matches runtime behavior.

## Features

* **View Editor** — Edit `.view.ron` scene layouts with live preview
* **FRE Editor** — Edit Fact-Rule-Event expressions
* **RON Source Editor** — Raw RON file editing with syntax awareness
* **Live Preview** — Real-time game rendering in an isolated viewport
* **Internationalization** — English and Chinese UI support
* **Platform Support** — Desktop and mobile-aware layout

## How to Use

The editor is typically run via the workspace:
```bash
cargo run -p souprune_editor
```

## How to Build

### Prerequisites

* Rust 1.85 or later
* System dependencies (same as souprune main crate)

### Build Steps

```bash
cargo build --release -p souprune_editor
```

## Dependencies

| Crate                                                    | Version | Description                |
|----------------------------------------------------------|---------|----------------------------|
| [bevy](https://crates.io/crates/bevy)                   | 0.18    | Game engine                |
| [bevy_egui](https://crates.io/crates/bevy_egui)         | 0.39    | egui UI integration        |
| [egui](https://crates.io/crates/egui)                   | 0.33    | Immediate mode UI          |
| [ron](https://crates.io/crates/ron)                      | 0.12    | RON config parsing         |
| [rfd](https://crates.io/crates/rfd)                     | 0.17    | Native file dialogs        |
| [souprune](../souprune)                                  | —       | Main game framework        |

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
