# souprune_api

[![license](https://img.shields.io/badge/license-LGPL--3.0--or--later-blue)](LICENSE) <img src="https://img.shields.io/github/repo-size/Bli-AIk/souprune.svg"/> <img src="https://img.shields.io/github/last-commit/Bli-AIk/souprune.svg"/> <br>
<img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" />

> Current Status: 🚧 Early Development

**souprune_api** — FFI API definitions and bindings generator for SoupRune game framework.

| English | Simplified Chinese          |
|---------|-----------------------------|
| English | [简体中文](./readme_zh-hans.md) |

## Introduction

`souprune_api` is the FFI (Foreign Function Interface) layer for the SoupRune game framework.  
It solves the problem of language interoperability, allowing users to interact with SoupRune from languages other than Rust (C#, Haxe, Lua, etc.).

With `souprune_api`, you only need to use the generated bindings in your target language to access SoupRune functionality.  
In the future, it may also support dynamic plugin loading and hot-reloading.

## Features

* FFI-safe API definitions for SoupRune
* Multi-language binding generation (C, C#, Haxe)
* Integration with Interoptopus for automatic code generation
* Type-safe cross-language communication
* (Planned) Plugin system support
* (Planned) Hot-reload capabilities

## How to Use

1. **Install Rust** (if not already installed):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Add to Cargo.toml**:

   ```toml
   [dependencies]
   souprune_api = "0.1"
   ```

3. **Generate bindings** (with bindgen feature):

   ```bash
   cargo run --bin souprune_bindgen --features bindgen
   ```

   This will generate:
   - C header files
   - C# bindings
   - Haxe bindings

4. **Use in your language**:

   See the generated binding files in the output directory for usage examples.

## Dependencies

This project uses the following crates:

| Crate                                             | Version | Description                 |
| ------------------------------------------------- | ------- | --------------------------- |
| [interoptopus](https://crates.io/crates/interoptopus) | 0.15.0-alpha.24   | FFI bindings framework |
| [interoptopus_backend_c](https://crates.io/crates/interoptopus_backend_c) | 0.15.0-alpha.24   | C backend |
| [interoptopus_backend_csharp](https://crates.io/crates/interoptopus_backend_csharp) | 0.15.0-alpha.24   | C# backend |
| [interoptopus_backend_haxe](https://crates.io/crates/interoptopus_backend_haxe) | 0.0.1   | Haxe backend |

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

* GNU Lesser General Public License v3.0 or later ([LICENSE](LICENSE) or [https://www.gnu.org/licenses/lgpl-3.0.html](https://www.gnu.org/licenses/lgpl-3.0.html))
