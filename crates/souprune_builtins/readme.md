# souprune_builtins

[![license](https://img.shields.io/badge/license-LGPL--3.0--or--later-blue)](LICENSE) <img src="https://img.shields.io/github/repo-size/Bli-AIk/souprune.svg"/> <img src="https://img.shields.io/github/last-commit/Bli-AIk/souprune.svg"/> <br>
<img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" />

> Current Status: 🚧 Early Development

**souprune_builtins** — Pre-compiled WASM module providing built-in spawn patterns and danmaku behaviors for SoupRune.

| English | Simplified Chinese          |
|---------|-----------------------------|
| English | [简体中文](./readme_zh-hans.md) |

## Introduction

`souprune_builtins` is a WASM guest module (compiled as `cdylib` for `wasm32-wasip2`) that implements all built-in spawn
patterns and bullet behaviors used by the SoupRune danmaku system.

Rather than hardcoding game logic in the framework, all built-in behaviors are implemented as WASM modules using the
same `souprune_sdk` interface available to community mod developers. This ensures parity between built-in and custom
content.

## Features

**Spawn Patterns:**

* `builtin.single` — Single bullet at center
* `builtin.ring` — Circular arrangement (count, radius, start_angle)
* `builtin.line` — Linear arrangement (count, spacing, direction)
* `builtin.edge` — Edge-spawned bullets (count, side, spacing, margin)

**Danmaku Behaviors:**

* `builtin.linear` — Constant velocity movement
* `builtin.orbital` — Circular/orbital motion
* `builtin.sine` — Sinusoidal wave motion
* `builtin.stationary` — No movement
* `builtin.aimed` — Homing toward player
* `builtin.tween` — Animated property changes (opacity, scale, position, rotation with easing)

## How to Build

### Prerequisites

* Rust with `wasm32-wasip2` target:
  ```bash
  rustup target add wasm32-wasip2
  ```

### Build Steps

```bash
cd crates/souprune_builtins
cargo build --target wasm32-wasip2 --release
```

The compiled WASM binary will be at:

```
target/wasm32-wasip2/release/souprune_builtins.wasm
```

## Dependencies

| Crate                           | Version | Description    |
|---------------------------------|---------|----------------|
| [souprune_sdk](../souprune_sdk) | 0.2     | WASM guest SDK |

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

* GNU Lesser General Public License v3.0 or later ([LICENSE](../../LICENSE.md)
  or [https://www.gnu.org/licenses/lgpl-3.0.html](https://www.gnu.org/licenses/lgpl-3.0.html))
