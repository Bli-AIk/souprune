# interoptopus_backend_haxe

[![license](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](LICENSE-APACHE) <img src="https://img.shields.io/github/repo-size/Bli-AIk/souprune.svg"/> <img src="https://img.shields.io/github/last-commit/Bli-AIk/souprune.svg"/> <br>
<img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" />

> Current Status: 🚧 Early Development - Experimental

**interoptopus_backend_haxe** — Haxe (hxcpp) backend for Interoptopus FFI bindings generator.

| English | Simplified Chinese          |
|---------|-----------------------------|
| English | [简体中文](./readme_zh-hans.md) |

## Introduction

`interoptopus_backend_haxe` is an experimental Haxe code generator backend for the [Interoptopus](https://github.com/ralfbiedert/interoptopus) FFI bindings tool.  
It solves the problem of manually writing FFI bindings between Rust and Haxe (hxcpp), allowing users to automatically generate type-safe Haxe wrapper code from Rust libraries.

With `interoptopus_backend_haxe`, you only need to annotate your Rust code with Interoptopus attributes, and the tool will generate corresponding Haxe bindings.  
In the future, it may also support advanced features like callback handling and complex type mappings.

## Features

* Automatic Haxe (hxcpp) binding generation from Rust FFI
* Integration with Interoptopus workflow
* Type-safe code generation
* (Planned) Callback support
* (Planned) Complex type marshalling
* (Planned) Documentation generation

## How to Use

1. **Install Rust** (if not already installed):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Add to Cargo.toml**:

   ```toml
   [build-dependencies]
   interoptopus = "0.15.0-alpha.24"
   interoptopus_backend_haxe = "0.0.1"
   ```

3. **Basic usage** (in your build.rs or bindings generator):

   ```rust
   use interoptopus::Interop;
   use interoptopus_backend_haxe::Generator;

   // Define your FFI interface
   // ... (see Interoptopus documentation)

   // Generate Haxe bindings
   let inventory = my_inventory();
   let generator = Generator::new();
   generator.write_to_file(&inventory, "generated_bindings.hx").unwrap();
   ```

## Dependencies

This project uses the following crates:

| Crate                                             | Version | Description                 |
| ------------------------------------------------- | ------- | --------------------------- |
| [interoptopus](https://crates.io/crates/interoptopus) | 0.15.0-alpha.24   | FFI bindings framework |
| [interoptopus_backend_utils](https://crates.io/crates/interoptopus_backend_utils) | 0.15.0-alpha.24   | Shared backend utilities |
| [derive_builder](https://crates.io/crates/derive_builder) | 0.20.2   | Builder pattern macros |
| [heck](https://crates.io/crates/heck) | 0.5   | Case conversion utilities |

## Warning

⚠️ **This is an experimental early release.**

- API is unstable and may change significantly
- Limited test coverage
- Not recommended for production use
- Breaking changes expected in future versions

Please report any issues or contribute improvements!

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
