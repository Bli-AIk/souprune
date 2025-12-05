# SoupRune

[![license](https://img.shields.io/github/license/Bli-AIk/souprune)](LICENSE.md) <img src="https://img.shields.io/github/repo-size/Bli-AIk/souprune.svg"/> <img src="https://img.shields.io/github/last-commit/Bli-AIk/souprune.svg"/> <br>
<img src="https://img.shields.io/badge/Deltarune / Undertale-black?style=for-the-badge&logo=undertale&logoColor=ff0000" /> <img src="https://img.shields.io/badge/Bevy-232326?style=for-the-badge&logo=bevy&logoColor=white" /> <br>
<img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" /> <img src="https://img.shields.io/badge/GameMaker Language-Coming Soon-9CA3AF?style=for-the-badge&logo=gamemaker&logoColor=white" />
<img src="https://img.shields.io/badge/Lua-Coming Soon-9CA3AF?style=for-the-badge&logo=lua&logoColor=white" />
<img src="https://img.shields.io/badge/.NET-Coming Soon-9CA3AF?style=for-the-badge&logo=dotnet&logoColor=white" />

> **Status**: 🚧 Initial iteration (features and structure may change frequently)

**SoupRune** is a game framework designed specifically for *
*[Deltarune](https://deltarune.com/) / [Undertale](https://undertale.com/) fangames**.

| English         | Simplified Chinese          |
|-----------------|-----------------------------|
| English Version | [简体中文](./readme_zh-hans.md) |

## 🥣 Introduction

Hey, don’t be scared by that **“Rust”** badge above — **SoupRune isn’t just a niche tool for hardcore programmers!**

It supports **Gamemaker**, *
*[Unitale](https://github.com/lvk/Unitale) / [Create Your Frisk](https://github.com/RhenaudTheLukark/CreateYourFrisk)**,
and the **.NET (C#)** ecosystem.

Whether you’re used to coding in GML, scripting in Lua, or developing in Unity / Godot, you may find something familiar
here.

**And the “Rune” in the name isn’t random either —**

While the Undertale fangame framework space may already be saturated, the Deltarune community is thriving.

SoupRune was created to seize this opportunity as a fangame framework, ~~while also serving as a spiritual successor
to [Undertale Changer Template](https://github.com/Bli-AIk/Undertale-Changer-Template)~~.

But that doesn’t mean SoupRune is only a Deltarune framework — we also value support for Undertale fangames!

SoupRune aims to become a true **“community-oriented fangame framework”** — open, flexible, modern, yet still retaining
that familiar DR / UT style.

## 🧭 S.O.U.P Principles

Yes, **Soup**Rune is a pun — and here’s what it stands for:

| Abbreviation | Full Form         | Meaning                                                                 |
|:------------:|:------------------|:------------------------------------------------------------------------|
|    **S**     | **Strong**        | Built on **Bevy** and **Rust**, powerful and modern architecture.       |
|    **O**     | **Open**          | Uses the **LGPL** open-source license, allowing free use and extension. |
|    **U**     | **User-friendly** | Offers multi-language scripting to lower the learning curve.            |
|    **P**     | **Popular**       | Compatible with mainstream DR/UT community ecosystems and habits.       |

## ⚙️ Technical Foundation

* Core built with **Bevy engine** and **Rust**, ensuring performance and extensibility;
* Design goals: **clear structure, modular expansion, and easy customization**.

## 🧩 Script Layer Support

SoupRune provides multi-language scripting support so developers from different backgrounds can get started quickly:

|                                Language                                 | Target Users                                                                                                 | Description                                                                                            |
|:-----------------------------------------------------------------------:|:-------------------------------------------------------------------------------------------------------------|:-------------------------------------------------------------------------------------------------------|
|                                **Rust**                                 | System-level developers / Rustaceans / Bevy users                                                            | Native support, best performance.                                                                      |
| **[COL (Configurable Open Language)](https://github.com/Bli-AIk/col/)** | GML / GameMaker users                                                                                        | My self-developed open-source GML alternative, fully compatible with GML syntax and easily extensible. |
|                                 **Lua**                                 | [Unitale](https://github.com/lvk/Unitale) / [CYF](https://github.com/RhenaudTheLukark/CreateYourFrisk) users | Lightweight scripting language, easy to pick up.                                                       |
|                           **.NET (C#/VB/F#)**                           | Unity / Godot users                                                                                          | Familiar syntax, smooth migration.                                                                     |

Future plans include:

* Providing a familiar development experience for [CYF](https://github.com/RhenaudTheLukark/CreateYourFrisk) / Gamemaker
  enthusiasts (e.g., similar event systems or script APIs);
* Offering migration guides
  from  [CYF](https://github.com/RhenaudTheLukark/CreateYourFrisk) / [Undertale Engine](https://github.com/TML233/UndertaleEngine)
  projects.

## 💬 Open Source & License

SoupRune uses the **LGPL-3.0** license.

This means:

* You can use SoupRune in closed-source projects;
* If you modify the framework’s core, you must release those changes under the LGPL;
* If you want to use a modified version in a fully closed-source environment, contact me for a commercial license.

Additionally, you must still follow Toby Fox’s rules for fan games.

## Citation Instructions

This project uses the following open-source projects as libraries, dependencies, or references:

### Original Games

| Project                             | Description             |
|-------------------------------------|-------------------------|
| [Undertale](https://undertale.com/) | UNDERTALE Original Game |
| [Deltarune](https://deltarune.com/) | DELTARUNE Original Game |

### Engine / Framework References

| Project                                                                  | Description                                                            |
|--------------------------------------------------------------------------|------------------------------------------------------------------------|
| [Unitale](https://github.com/lvk/Unitale)                                | An Undertale engine built with Unity 5, moddable via Lua               |
| [Create Your Frisk](https://github.com/RhenaudTheLukark/CreateYourFrisk) | A fork of Unitale by Rhenaud The Lukark                                |
| [Undertale Engine](https://github.com/TML233/UndertaleEngine)            | A GameMaker project template designed for creating Undertale fan games |

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