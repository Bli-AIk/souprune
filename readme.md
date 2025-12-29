# SoupRune

[![license](https://img.shields.io/github/license/Bli-AIk/souprune)](LICENSE.md) <img src="https://img.shields.io/github/repo-size/Bli-AIk/souprune.svg"/> <img src="https://img.shields.io/github/last-commit/Bli-AIk/souprune.svg"/> <br>
<img src="https://img.shields.io/badge/Deltarune / Undertale-black?style=for-the-badge&logo=undertale&logoColor=ff0000" /> <img src="https://img.shields.io/badge/Bevy-232326?style=for-the-badge&logo=bevy&logoColor=white" /> <br>
<img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" />
<img src="https://img.shields.io/badge/C%23-239120?style=for-the-badge&logo=csharp&logoColor=white" />
<img src="https://img.shields.io/badge/Haxe-EA8220?style=for-the-badge&logo=haxe&logoColor=white" />
<img src="https://img.shields.io/badge/Nim-Coming_Soon-FFE953?style=for-the-badge&logo=nim&logoColor=black" />
<img src="https://img.shields.io/badge/Nelua-Coming_Soon-9CA3AF?style=for-the-badge&logo=lua&logoColor=white" />

> **Status**: 🚧 Initial development phase (framework structure is still evolving rapidly)

[![](https://dcbadge.limes.pink/api/server/5YXK5DRjPZ)](https://discord.gg/5YXK5DRjPZ)

**SoupRune** is an experimental game framework designed specifically for creating RPG / STG games similar to 
**[Deltarune](https://deltarune.com/) / [Undertale](https://undertale.com/)**.

| English         | Simplified Chinese          |
|-----------------|-----------------------------|
| English Version | [简体中文](./readme_zh-hans.md) |

## 🥣 Introduction

Hey, don’t be scared by that **“Rust”** badge above — **SoupRune isn’t just a niche tool for hardcore programmers!**

Why is that? Because:

### SoupRune is a Language-Agnostic Framework!

SoupRune adopts a **Framework - Project (Mod)** architecture. The core engine built with `bevy` handles the low-level
execution, while your game logic resides in an independent "Project" that interacts with the core via standard
interfaces.

We achieve true **Language-Agnosticism** through the `C ABI`. In choosing supported languages, we prioritize those
friendly to **traditional Undertale / Deltarune developers**, aiming to build a bridge for developers from **GameMaker
** (Haxe), **Lua** (Nelua), or **Python / GDScript** (Nim) backgrounds.

We warmly welcome developers from all language communities to join us in improving support for these languages!

### SoupRune is a Community-Oriented Framework!

SoupRune — the **“Rune”** in the name isn’t just random.

While the Undertale fangame framework space may already be saturated, the Deltarune community is thriving.

SoupRune was created to seize this opportunity as a fangame framework, 
~~while also serving as a spiritual successor to [Undertale Changer Template](https://github.com/Bli-AIk/Undertale-Changer-Template)~~.

But that doesn’t mean SoupRune is only a Deltarune framework — we also value support for Undertale fangames!

SoupRune aims to become a true **“community-oriented fangame framework”** — open, flexible, modern, yet still retaining
that familiar DR / UT style.

### SoupRune is an Open Source Framework!

SoupRune uses the **LGPL-3.0** license. This license applies only to the framework core code.

This means:

* The Project (Mod) you develop can use other open-source licenses or be closed-source;
* You can use SoupRune's source code in closed-source projects;
* But if you modify the framework's core code, you must release those changes under the LGPL;
* If you wish to use modified SoupRune core code in a fully closed-source environment, you can contact me for a
  commercial license.

Additionally, you must still follow Toby Fox’s rules for fan games.

### SoupRune is an Experimental Framework!

Although "experimental" implies instability and imperfection, it also represents infinite possibilities.

Use it to make a dr / ut fangame? Or try to recreate other classic games? Or even develop a brand new original game?
Anything is possible!

SoupRune is currently still in the **initial development phase**, and the framework structure and design philosophy are
evolving rapidly. We welcome community members to actively participate in discussions and contributions to shape the
future of this framework.

Join our [Discord](https://discord.gg/5YXK5DRjPZ)!

## 🧭 S.O.U.P Principles

Yes, **Soup**Rune is a pun — and here’s what it stands for:

| Abbreviation | Full Form         | Meaning                                                                     |
|:------------:|:------------------|:----------------------------------------------------------------------------|
|    **S**     | **Strong**        | Built on **Bevy** and **Rust**, powerful and modern architecture.           |
|    **O**     | **Open**          | Uses the **LGPL** open-source license, allowing free use and extension.     |
|    **U**     | **User-friendly** | Offers multi-language scripting to lower the learning curve.                |
|    **P**     | **Polyglot**      | Supports multiple programming languages (Rust, C#, Haxe, Nim, Nelua, etc.). |

## ⚙️ Technical Foundation

* Core built with **Bevy engine** and **Rust**, ensuring performance and extensibility;
* Design goals: **clear structure, modular expansion, and easy customization**.

## 🏗️ Project Architecture

SoupRune adopts a multi-crate workspace architecture:

| Crate                                                   | Description                                                                                  |
|:--------------------------------------------------------|:---------------------------------------------------------------------------------------------|
| [`souprune`](./crates/souprune)                         | **Core Framework**: The main framework body, application entry point, and core logic.        |
| [`souprune_api`](./crates/souprune_api)                 | **Protocol Layer**: Defines interface standards for Project (Mod) interaction with the core. |
| [`souprune_sdk`](./crates/souprune_sdk)                 | **Development Kit**: A wrapper for the API, provided for external Project (Mod) scripts.     |
| [`souprune_mod_test`](./crates/souprune_mod_test)       | **Sample Mod**： Sample test library for scripting systems.                                   |
| [`bevy_mortar_bond`](./crates/bevy_mortar_bond)         | **Plugin**: Bridge between Mortar scripting and Bevy, handling dialogue and logic.           |
| [`bevy_ecs_typewriter`](./crates/bevy_ecs_typewriter)   | **Plugin**: ECS-based typewriter implementation, supporting rich text and multi-language.    |
| [`bevy_fact_rule_event`](./crates/bevy_fact_rule_event) | **Plugin**: Complex event system based on the "Fact-Rule-Event" model.                       |

## 🧩 Script Layer Support

SoupRune's scripting system is built on **C ABI** to achieve high-performance interoperability. We have carefully
selected a series of languages that support **AOT compilation**, aiming to correspond to different development paradigms
so that developers from other engines can smoothly migrate their experience:

|        Language         | Target Users                                      | Description                                                                                                |
|:-----------------------:|:--------------------------------------------------|:-----------------------------------------------------------------------------------------------------------|
|        **Rust**         | System-level developers / Rustaceans / Bevy users | Native support, best performance.                                                                          |
|      **.NET (C#)**      | Unity / Godot / C# users                          | Industry standard language. Seamless integration and high performance via **Native AOT**.                  |
|        **Haxe**         | **Haxe** users / **GameMaker** users              | Powerful high-level language. Its syntax is similar to GML, making it an excellent choice for development. |
|  **Nim** (Coming Soon)  | **Python** / **GDScript** (Godot) users           | Python-like indentation syntax, compiles to C, combining elegance with efficiency.                         |
| **Nelua** (Coming Soon) | **Lua** users                                     | Inherits Lua's minimalist syntax style but compiles to native machine code for extreme performance.        |

If you are interested in helping with SoupRune's multi-language support, contributions are welcome!

## Citation Instructions

This project uses the following open-source projects as libraries, dependencies, or references:

### Original Games

| Project                             | Description             |
|-------------------------------------|-------------------------|
| [Undertale](https://undertale.com/) | UNDERTALE Original Game |
| [Deltarune](https://deltarune.com/) | DELTARUNE Original Game |

### Predecessor

| Project                                                                             | Version | License                                                                                                   | Description                                                                                                                |
|-------------------------------------------------------------------------------------|---------|-----------------------------------------------------------------------------------------------------------|----------------------------------------------------------------------------------------------------------------------------|
| [Undertale-Changer-Template](https://github.com/Bli-AIk/Undertale-Changer-Template) | 1.0.7   | [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | SoupRune is the **spiritual successor** of Undertale-Changer-Template, continuing its core concepts and design philosophy. |

### Game Engine Core

| Project                               | Version | License                                                                                                                                                                                                       | Description      |
|---------------------------------------|---------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|------------------|
| [Bevy](https://crates.io/crates/bevy) | 0.17.2  | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | Game Engine Core |

### Bevy Plugin Ecosystem

| Project                                                                   | Version                                                                                                                               | License                                                                                                                                                                                                       | Description                                                                                            |
|---------------------------------------------------------------------------|---------------------------------------------------------------------------------------------------------------------------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|--------------------------------------------------------------------------------------------------------|
| [bevy_ecs_typewriter](https://github.com/Bli-AIk/bevy_ecs_typewriter)     | 0.0.0                                                                                                                                 | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | Typewriter implementation based on ECS                                                                 |
| [bevy_fact_rule_event](https://github.com/Bli-AIk/bevy_fact_rule_event)   | 0.0.0                                                                                                                                 | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | Implementation of an event system based on fact-rule-event                                             |
| [bevy_mortar_bond](https://github.com/Bli-AIk/bevy_mortar_bond)           | 0.0.0                                                                                                                                 | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | A bridge library between Bevy and the Mortar language                                                  |
| [leafwing-input-manager](https://crates.io/crates/leafwing-input-manager) | 0.19.0                                                                                                                                | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | Manages game input, handling keyboard, mouse, and controller operation mappings                        |
| [seldom_state](https://crates.io/crates/seldom_state)                     | 0.15.0                                                                                                                                | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | Finite State Machine implementation                                                                    |
| [bevy_ecs_tiled](https://crates.io/crates/bevy_ecs_tiled)                 | dev (GitHub branch)                                                                                                                   | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | Integrates Bevy ECS and Tiled map editor for tile-based game levels                                    |
| [bevy_tween](https://crates.io/crates/bevy_tween)                         | 0.10.0                                                                                                                                | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | Tweening animation library for Bevy, used for smooth animations and transitions                        |
| [bevy-inspector-egui](https://crates.io/crates/bevy-inspector-egui)       | 0.35.0                                                                                                                                | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | Optional editor/debugging tool for Bevy, used for real-time inspection of the ECS world                |
| [iyes_perf_ui](https://crates.io/crates/iyes_perf_ui)                     | dev (My [fork](https://github.com/Bli-AIk/iyes_perf_ui) based on GitHub PR [#35](https://github.com/IyesGames/iyes_perf_ui/pull/35) ) | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | Optional performance monitoring UI for Bevy, displaying FPS, system timings, and profiling information |
| [bevy_smud](https://crates.io/crates/bevy_smud)                           | 0.12.0                                                                                                                                | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | 2D SDF shape renderer plugin for Bevy                                                                  |
| [bevy_rich_text3d](https://crates.io/crates/bevy_rich_text3d)             | 0.5.1                                                                                                                                 | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | Mesh-based rasterized rich text implementation                                                         |
| [bevy_kira_audio](https://crates.io/crates/bevy_kira_audio)               | 0.24.0                                                                                                                                | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | Audio playback plugin using Kira, supports WAV, OGG, FLAC, MP3 formats                                 |
| [bevy_brp_extras](https://crates.io/crates/bevy_brp_extras)               | 0.17.2                                                                                                                                | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | Extra features for Bevy Remote Protocol (BRP)                                                          |

### Rust Crates

| Project                                 | Version | License                                                                                                                                                                                                       | Description                                                                                                    |
|-----------------------------------------|---------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|----------------------------------------------------------------------------------------------------------------|
| [serde](https://crates.io/crates/serde) | 1.0     | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | Serialization/deserialization framework supporting `derive` macros for convenient (de)serialization of structs |
| [toml](https://crates.io/crates/toml)   | 0.9.8   | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | TOML parsing                                                                                                   |
| [ron](https://crates.io/crates/ron)     | 0.10    | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | Rusty Object Notation parsing                                                                                  |

### Asset References

| Project                                             | Description                                               |
|-----------------------------------------------------|-----------------------------------------------------------|
| [DTTVL-Fonts](https://github.com/UTCLC/DTTVL-Fonts) | Font files used in the DELTATRAVELER localization project |

**Heartfelt thanks to every contributor to the project! 🎔**

## Contributors

The following individuals have contributed to the project.

<a href = "https://github.com/Bli-AIk/souprune/Python/graphs/contributors">
<img src = "https://contrib.rocks/image?repo=Bli-AIk/souprune" alt=""/>
</a>

**Heartfelt thanks to each and every one of you! 🎔**

## 🤝 Join Us

Whether you:

* Want to create your own DR/UT-style game;
* Want to try Bevy and Rust;
* Or simply love open-source and experimentation —

You’re welcome to contribute to **SoupRune**:

* Submit Issues or Pull Requests!
* Share ideas and discuss architecture!
* Or just chat about game development in the community!

**Let’s cook the most delicious Soup together!**
