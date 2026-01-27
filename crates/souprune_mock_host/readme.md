# souprune_mock_host

[![license](https://img.shields.io/badge/license-LGPL--3.0--or--later-blue)](LICENSE) <img src="https://img.shields.io/github/repo-size/Bli-AIk/souprune.svg"/> <img src="https://img.shields.io/github/last-commit/Bli-AIk/souprune.svg"/> <br>
<img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" />

> Current Status: 🚧 Early Development - Experimental

**souprune_mock_host** — Mock host environment for testing SoupRune mods without a full game.

| English | Simplified Chinese          |
|---------|-----------------------------|
| English | [简体中文](./readme_zh-hans.md) |

## Introduction

`souprune_mock_host` is a minimal host environment for loading and testing SoupRune mods.  
It solves the problem of mod testing by providing a lightweight runtime that can dynamically load mods without requiring a complete game setup.

With `souprune_mock_host`, mod developers can quickly test their creations, verify FFI bindings, and debug mod behavior in an isolated environment.  
It's designed for development and CI/CD testing workflows.

## Features

* Dynamic mod loading via libloading
* Minimal host implementation of souprune_api
* Isolated testing environment
* Fast iteration for mod development
* CI/CD friendly for automated testing
* No full game engine required

## How to Use

1. **Install Rust** (if not already installed):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Build your mod**:

   ```bash
   cargo build -p souprune_mod_test --release
   ```

3. **Run the mock host**:

   ```bash
   cargo run -p souprune_mock_host -- path/to/your/mod.so
   ```

4. **Test mod functionality**:

   The mock host will load your mod and execute test scenarios.

## Dependencies

This project uses the following crates:

| Crate                                             | Version | Description                 |
| ------------------------------------------------- | ------- | --------------------------- |
| [souprune_api](https://crates.io/crates/souprune_api) | 0.0.1   | FFI API layer |
| [libloading](https://crates.io/crates/libloading) | 0.9.0   | Dynamic library loading |

## Warning

⚠️ **This is an experimental early release.**

- API is unstable and may change significantly
- For testing purposes only
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
