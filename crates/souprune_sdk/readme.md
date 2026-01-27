# souprune_sdk

[![license](https://img.shields.io/badge/license-LGPL--3.0--or--later-blue)](LICENSE) <img src="https://img.shields.io/github/repo-size/Bli-AIk/souprune.svg"/> <img src="https://img.shields.io/github/last-commit/Bli-AIk/souprune.svg"/> <br>
<img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" />

> Current Status: 🚧 Early Development

**souprune_sdk** — Modding SDK for SoupRune game framework.

| English | Simplified Chinese          |
|---------|-----------------------------|
| English | [简体中文](./readme_zh-hans.md) |

## Introduction

`souprune_sdk` is the Software Development Kit for creating mods and extensions for SoupRune games.  
It solves the problem of complex mod development by providing a unified, high-level API, allowing users to create game modifications without deep engine knowledge.

With `souprune_sdk`, you only need to implement your mod logic using the provided interfaces and load it into any SoupRune-based game.  
In the future, it may also support visual modding tools and mod marketplace integration.

## Features

* High-level modding API built on souprune_api
* Type-safe mod interfaces
* Easy integration with SoupRune games
* (Planned) Mod lifecycle management
* (Planned) Hot-reload support for development
* (Planned) Visual mod editor

## How to Use

1. **Install Rust** (if not already installed):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Add to Cargo.toml**:

   ```toml
   [dependencies]
   souprune_sdk = "0.1"
   ```

3. **Create a mod**:

   ```rust
   use souprune_sdk::prelude::*;

   // Your mod implementation here
   ```

4. **Build your mod**:

   ```bash
   cargo build --release
   ```

## Dependencies

This project uses the following crates:

| Crate                                             | Version | Description                 |
| ------------------------------------------------- | ------- | --------------------------- |
| [souprune_api](https://crates.io/crates/souprune_api) | 0.0.1   | FFI API layer |

## Warning

⚠️ **This is an early development release.**

- API is unstable and may change significantly
- Limited documentation and examples
- Not recommended for production use
- Breaking changes expected in future versions

## Contributing

Contributions are welcome!
Whether you want to fix a bug, add a feature, or improve documentation:

* Submit an **Issue** or **Pull Request**.
* Share ideas and discuss design or architecture.

## License

This project is licensed under

* GNU Lesser General Public License v3.0 or later ([LICENSE](LICENSE) or [https://www.gnu.org/licenses/lgpl-3.0.html](https://www.gnu.org/licenses/lgpl-3.0.html))
