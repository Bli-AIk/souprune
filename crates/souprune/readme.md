# souprune

[![license](https://img.shields.io/badge/license-LGPL--3.0--or--later-blue)](LICENSE-APACHE) <img src="https://img.shields.io/github/repo-size/Bli-AIk/souprune.svg"/> <img src="https://img.shields.io/github/last-commit/Bli-AIk/souprune.svg"/> <br>
<img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" />

> Current Status: 🚧 Early Development (Initial version in progress)

**souprune** — A game framework designed specifically for Deltarune / Undertale fangames.

| English         | Simplified Chinese                 |
|-----------------|---------------------------------|
| English | [简体中文](./readme_zh-hans.md) |

## Introduction

`souprune` is a game framework designed specifically for Deltarune / Undertale fangames.  
It solves the challenge of creating Deltarune/Undertale style games, allowing users to build fan games with authentic mechanics and features.

With `souprune`, you only need to focus on content creation while the framework handles the core game systems.  
In the future, it may also support additional game mechanics and enhanced modding capabilities.

## Features

* Deltarune/Undertale style game mechanics
* Built on Bevy engine for performance and flexibility
* Integrated text animation system
* Fact-Rule-Event system for game logic
* Mortar language integration
* (Planned) Enhanced battle system
* (Planned) Save/load functionality
* (Planned) Dialogue system

## How to Use

1. **Install Rust** (if not already installed):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Add to Cargo.toml**:

   ```toml
   [dependencies]
   souprune = "0.1.2"
   ```

3. **Basic game setup**:

   ```rust
   // <待补充>
   ```

4. **Debug features**:

   * Enable debug features: `cargo run --features debug`
   * Inspector and performance UI available in debug mode

## How to Build

### Prerequisites

* Rust 1.70 or later

### Build Steps

1. **Clone the repository**:

   ```bash
   git clone https://github.com/Bli-AIk/souprune.git
   cd souprune
   ```

2. **Build the project**:

   ```bash
   cargo build --release
   ```

3. **Run tests**:

   ```bash
   cargo test
   ```

4. **Install globally** (optional):

   ```bash
   cargo install --path .
   ```

## Dependencies

This project uses the following crates:

| Crate                                             | Version | Description                 |
| ------------------------------------------------- | ------- | --------------------------- |
| [bevy](https://crates.io/crates/bevy) | 0.17.2   | Game engine |
| [leafwing-input-manager](https://crates.io/crates/leafwing-input-manager) | 0.19.0   | Input handling |
| [seldom_state](https://crates.io/crates/seldom_state) | 0.15.0   | State management |
| [serde](https://crates.io/crates/serde) | 1.0   | Serialization framework |
| [toml](https://crates.io/crates/toml) | 0.9.8   | Configuration parsing |
| [bevy_tween](https://crates.io/crates/bevy_tween) | 0.10.0   | Animation tweening |
| bevy_rich_text_3d_animator | 0.1.0   | Text animation system |
| bevy_fact_rule_event | 0.1.0   | Fact-Rule-Event system |
| bevy_mortar_bond | 0.1.0   | Mortar language integration |

## Contributing

Contributions are welcome!
Whether you want to fix a bug, add a feature, or improve documentation:

* Submit an **Issue** or **Pull Request**.
* Share ideas and discuss design or architecture.

## License

This project is licensed under LGPL-3.0-or-later.