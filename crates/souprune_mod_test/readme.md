# souprune_mod_test

[![license](https://img.shields.io/badge/license-LGPL--3.0--or--later-blue)](LICENSE) <img src="https://img.shields.io/github/repo-size/Bli-AIk/souprune.svg"/> <img src="https://img.shields.io/github/last-commit/Bli-AIk/souprune.svg"/> <br>
<img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" />

> Current Status: 🚧 Early Development - Experimental

**souprune_mod_test** — Test mod implementation for verifying SoupRune SDK functionality.

| English | Simplified Chinese          |
|---------|-----------------------------|
| English | [简体中文](./readme_zh-hans.md) |

## Introduction

`souprune_mod_test` is a reference implementation and test mod for the SoupRune SDK.  
It solves the problem of SDK verification by providing a working example of how to create a mod using souprune_sdk.

With `souprune_mod_test`, developers can learn mod structure, test their SDK installations, and see real examples of how to implement common modding patterns.  
This crate is primarily for testing and reference purposes.

## Features

* Reference implementation of souprune_sdk
* Demonstrates mod structure and patterns
* Integration testing for SDK
* Example code for mod developers
* Built as a cdylib for dynamic loading

## How to Use

1. **Install Rust** (if not already installed):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Build the test mod** (requires `wasm32-wasip2` target):

   ```bash
   rustup target add wasm32-wasip2
   cargo build -p souprune_mod_test --target wasm32-wasip2
   ```

3. **Load into a SoupRune game**:

   The compiled `.wasm` component can be loaded by compatible SoupRune games or tested with `souprune_mock_host`.

4. **Study the source**:

   Review the implementation in `src/lib.rs` to understand mod patterns.

## Dependencies

This project uses the following crates:

| Crate                                             | Version | Description                 |
| ------------------------------------------------- | ------- | --------------------------- |
| [souprune_sdk](https://crates.io/crates/souprune_sdk) | 0.0.1   | Modding SDK |

## Warning

⚠️ **This is an experimental early release.**

- API is unstable and may change significantly
- For testing and reference purposes only
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
