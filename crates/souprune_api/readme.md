# souprune_api

[![license](https://img.shields.io/badge/license-LGPL--3.0--or--later-blue)](LICENSE) <img src="https://img.shields.io/github/repo-size/Bli-AIk/souprune.svg"/> <img src="https://img.shields.io/github/last-commit/Bli-AIk/souprune.svg"/> <br>
<img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" />

> Current Status: 🚧 Early Development

**souprune_api** — WIT interface definitions and shared types for the SoupRune mod system.

| English | Simplified Chinese          |
|---------|-----------------------------|
| English | [简体中文](./readme_zh-hans.md) |

## Introduction

`souprune_api` defines the contract between the SoupRune engine (host) and WASM mod components (guests)
using WIT (WebAssembly Interface Types). It provides shared Rust types used by both the host runtime
and the guest SDK (`souprune_sdk`).

## Features

* WIT interface definition (`wit/souprune-mod.wit`) — single source of truth
* Shared Rust types: `Vec2`, `BulletContext`, `BulletOutput`, `Action`
* Host-side: used by `souprune` (wasmtime) to define imports
* Guest-side: used by `souprune_sdk` (wit-bindgen) to implement exports

## WIT Interface

The WIT file defines three interfaces:

- **`host-api`** (imported by guest): `log`, `get-fact`, `set-fact`, `emit-event`
- **`behavior`** (exported by guest): `on-init`, `on-update`, `on-interact`
- **`danmaku`** (exported by guest): `init-bullet`, `update-bullet`

## How to Use

For **host development** (engine side), add to `Cargo.toml`:
```toml
[dependencies]
souprune_api = { path = "../souprune_api" }
```

For **mod development** (guest side), use `souprune_sdk` which re-exports the necessary types.

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
