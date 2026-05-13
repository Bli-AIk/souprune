# SoupRune

[![license](https://img.shields.io/github/license/Bli-AIk/souprune)](LICENSE.md) <img src="https://img.shields.io/github/repo-size/Bli-AIk/souprune.svg"/> <img src="https://img.shields.io/github/last-commit/Bli-AIk/souprune.svg"/> <br>
<img src="https://img.shields.io/badge/Deltarune-Undertale-black?style=for-the-badge&labelColor=001225&logo=undertale&logoColor=ff0000" /> <img src="https://img.shields.io/badge/Bevy-232326?style=for-the-badge&logo=bevy&logoColor=white" /> <br>
<img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" />
<img src="https://img.shields.io/badge/WASM-624DE8?style=for-the-badge&logo=webassembly&logoColor=white" />
<img src="https://img.shields.io/badge/JS-planned-F7DF1E?style=for-the-badge&logo=javascript&logoColor=black" />
<img src="https://img.shields.io/badge/.NET-planned-512BD4?style=for-the-badge&logo=dotnet&logoColor=white" />

> **Status**: 🚧 Initial development phase (framework structure is still evolving rapidly)

[![](https://dcbadge.limes.pink/api/server/5YXK5DRjPZ)](https://discord.gg/5YXK5DRjPZ)

**SoupRune** is a distinctive modern framework for the next generation of fangames,
designed specifically for creating RPG / STG games similar to
**[Deltarune](https://deltarune.com/) / [Undertale](https://undertale.com/)**.

| English | Simplified Chinese          |
|---------|-----------------------------|
| English | [简体中文](./readme_zh-hans.md) |

## 🥣 What is this?

**SoupRune** is a distinctive modern framework for the next generation of fangames, built on the `bevy` engine and born
for creating RPG and Danmaku (STG)
games in the style of **[Deltarune](https://deltarune.com/) / [Undertale](https://undertale.com/)**.

It strives to be the next-generation community-driven Fangame framework—bringing a unique flavor while embracing high
performance and modern development paradigms at its core.

## 🧭 S.O.U.P Principles

Our design philosophy is concentrated in a delicious bowl of **S.O.U.P**:

* **S (Strong) - Powerful Core**: Built on **Bevy Engine** and **Rust**, enjoying the performance and parallel computing
  advantages of the ECS architecture.
* **O (Open) - Truly Open Source**: Licensed under **LGPL-3.0**. The framework core is open source, while your Project /
  Mod (game) belongs to you.
* **U (User-friendly) - Easy to Start**: Provides out-of-the-box RPG features (dialogue, battle, maps), bullet
  sequencers (STG - supports turn-based), and visual tool integration (supports Alight Motion projects), allowing you to
  focus on creativity.
* **P (Polyglot) - Multi-language Support**: Language-agnostic via the **WASM Component Model**. Any language that
  compiles to WebAssembly can be your "utensil" to enjoy this soup—safely sandboxed and cross-platform.

## 🚀 Quick Start

SoupRune is currently in the **🚧 Initial Development Phase**, but if you're eager to try it out, you can start with
these steps:

1. **Prepare Environment**: Install the [Rust Development Environment](https://www.rust-lang.org/).
2. **Clone Repository**: `git clone https://github.com/Bli-AIk/souprune.git`
3. **Enter Directory**: `cd souprune`
4. **Update Submodules**: `git submodule update --init --recursive`
5. **Run Example (Debug Mode)**: `cargo run --package souprune --bin souprune --features debug`

<details>
<summary>Maintained Example Projects</summary>

Maintained example content is included as Git submodules:

- [`souprune_undertale_preset`](https://github.com/Bli-AIk/souprune_undertale_preset) — reusable Undertale-style preset
- [`souprune_mad_dummy_example`](https://github.com/Bli-AIk/souprune_mad_dummy_example) — default runnable Mad Dummy example project

The old [`souprune_example_mods`](https://github.com/Bli-AIk/souprune_example_mods) repository is archived and preserved
only for historical branch-based examples.

Configure active mod in `projects/config.toml`:

```toml
[project]
mod_name = "mad_dummy_example"
language = "en-US"
```

</details>

## ⚙️ Design Philosophy

On a technical level, SoupRune:

* Uses **Bevy engine** and **Rust** for the core implementation, ensuring performance and extensibility;
* Aims for: **clear structure, modular expansion, and easy customization**;

In terms of project architecture, SoupRune adopts a design that separates the **Engine** and the **Project**.

You can think of this as the relationship between a **"Game Console"** and a **"Game Cartridge"**:

* **Engine (Core)**: The underlying "Console". It handles the heavy, complex tasks—like "how to render brilliant
  graphics" and "how to make physical collisions more realistic". This part is built with high-performance Rust and
  usually doesn't require your attention.
* **Project / Mod (Content)**: The "Game Cartridge" you create. This is where your creativity lives—character dialogue,
  exciting battles, and moving stories.

### 📝 Development Workflow: Embracing Configuration & Scripting

In SoupRune, game development doesn't mean staring at tedious code all day. We promote a **"Data-Driven"** approach,
combining **RON configuration files** with **custom scripts**:

1. **RON Configuration Files (Content Description)**:
   Most game content is created by writing **RON (Rusty Object Notation)** files. It's like **filling out a form** or *
   *playing with blocks**—clear and intuitive.
    * Want to design a battle? Write a `.performance.ron` to schedule the bullet timeline.
    * Need to layout a UI? Use a `.ui_layout.ron` to define buttons and text.
    * Defining character stats? Set animations and colliders in a `.character.ron`.
    * *Even without a programming background, you can create rich game content by modifying these configs.*

2. **Custom Scripts (Behavior & Algorithms)**:
   You only need to write scripts when you need unique or complex logic (like a brand-new spiral homing algorithm or a
   special boss mechanic).
   This is where SoupRune shines—you can choose **Rust** (or any language that compiles to WASM) to write this logic!
    * Your code is compiled into a **WASM component** and loaded into the framework at runtime via a sandboxed runtime.
    * The framework communicates with your mod through a well-defined **WIT (WebAssembly Interface Types)** contract,
      giving you **high performance** and **memory safety** while keeping mods isolated from the framework core.

**In short: SoupRune prepares the broth (underlying framework); you just follow the recipe (RON configs) and pick your
favorite spoon (any WASM-compatible language) to add the ingredients and cook your delicious game!**

<details>
<summary><strong>So, what was the original intention behind SoupRune?</strong></summary>

If you're interested in the design thinking behind SoupRune, here are more details:

### 🌌 Why "Rust" the Soul? Why Rust and Bevy?

> To make a 2D pixel RPG, do you really need high-performance ECS engines like Rust and Bevy? Isn't that overkill?

I think this is a question most people would ask—whether you are a systems developer or a creator who loves retro/doujin
games.

Yes, we are doing something that seems contradictory:
Using **Rust**, a systems language so hardcore and performance-obsessed, to reconstruct a game genre that is most
emotional, retro, and least concerned with graphics and technical prowess.

Because—aesthetics can be retro, but technology should be cutting-edge.

SoupRune is an exploration—we want to prove that ECS can run not only massive tech demos but also delicately handle a
touching dialogue or a carefully choreographed performance. We turned Bevy into a paintbrush.

SoupRune is also a weapon—for those great community creators who yearn to unleash all their talents. You have amazing
creativity, but are often trapped by the performance bottlenecks of outdated tools. Lagging danmaku, crashing saves,
complex mechanisms that are hard to implement.

Before this, no one had ever envisioned the combination of the two—and now, it stands here.

This is a map prepared for you; this is a weapon forged for you.

We hope to use avant-garde rendering pipelines and architectural design to build an indestructible stage. Even if you
don't know code, your work deserves industrial-grade performance.

This is the goal of this project, the original intention of its design: to build a bridge.

### 🌉 Our Promise: A Bridge Across the Gap

> A bridge between technology and art—the intersection of light and dark, the boundary between dream and reality.

SoupRune is not just a framework; it is **technical equality**.

* **Simplicity is not just simple**:
  In SoupRune, "simple" refers to your experience when creating, not the crudeness of the underlying layer.
  We have shouldered all the underlying complexity (multi-threading, memory management, rendering pipelines) just so you
  can write logic as naturally as breathing.
* **Powered by Love, but also Explosive**:
  We believe that doujin works should not be synonymous with "making do".
  Beauty and technical power should not be bound—even pixel games deserve 4K clarity, 144Hz smoothness, tens of
  thousands of particles and bullets, and a carnival of rendering effects.

### 🏗️ Architecture: Why separate Core and Project? Why be "Language-Agnostic"?

SoupRune adopts a **Core (Engine) - Project (Mod)** separated architecture.

* **Core**: Driven by Rust and Bevy, responsible for all underlying heavy lifting (rendering, physics, ECS scheduling).
* **Project**: Communicates with the core via a standardized **WASM Component Model** interface defined in WIT.

We know that developers in the Undertale / Deltarune community come from diverse backgrounds—and that's exactly
why we chose WASM. Instead of binding to one specific language, the WASM Component Model provides a universal,
standardized compilation target. Any language with a WASM backend can produce mods for SoupRune:

* **Rust** developers get native-quality performance with zero overhead;
* **JavaScript / TypeScript** developers can prototype mods rapidly (planned via `jco` toolchain);
* **C# (.NET)** developers can leverage their Unity/Godot experience (planned via .NET WASI SDK);
* And as the WASM ecosystem grows, **more languages will become available naturally**—without requiring any
  changes to SoupRune itself.

The best part? WASM mods run in a **sandbox**. A buggy mod cannot crash the framework, corrupt save data,
or access the filesystem without permission. This is safety by design—not an afterthought.

Our original intention is to let every creator in the community **define game content in a simple and easy-to-use way**;
Our goal is to let every developer in the community **write code logic in their own familiar way**.

### ⚖️ License: About LGPL Open Source

We chose **LGPL-3.0** to find a balance between "open source contribution" and "creator rights". Simply put:

* ✅ **Your game, your rules**: Projects (Mods) you develop based on SoupRune can be closed-source or commercialized; you
  don't need to open-source your game logic.
* 🤝 **Give back to the community**: If you modify SoupRune's **framework core code** (the Engine part), you must
  open-source those changes so everyone can benefit.
* 🏢 **Commercial Licensing**: If you need to modify the core code in a closed-source environment, you can contact me for
  a commercial license.

### 🔮 Vision: Towards Community-Driven—The Next Generation Playground

The **"Rune"** in SoupRune's name both pays tribute to Deltarune and symbolizes a heritage.

It is the spiritual successor to [Undertale Changer Template](https://github.com/Bli-AIk/Undertale-Changer-Template). We
still carry the determination to change everything—our goal is not to make a closed tool, but an **open, modern—and most
importantly, a community-oriented next-gen playground**.

While "next-gen" implies evolving frontiers, it also represents infinite possibilities—whether recreating
classics or creating entirely original works, we hope SoupRune becomes the tool of choice in your hands.

### 🤝 Join Us: We Need Your Determination

**(Join Us: Bring Your Determination)**

This is an open-source **next-gen playground** belonging to the community.
It may not be perfect yet, just like how every great adventure begins.

* If you are a **Rustacean**, come help us forge this sword and challenge the limits of ECS architecture in narrative
  games.
* If you are a **Renderer**, come help us polish this sword and create a stylized rendering pipeline that is both
  efficient and beautiful, unprecedented in retro RPG / STG.
* If you are a **Creator**, try wielding this sword and tell us what kind of functions are needed to carry your
  imagination.

The prophecy of the RUNE should be written by us together.

Let us write a new future, line by line.

</details>

## 🤝 Contributing

SoupRune welcomes contributions of all kinds — whether you're an artist, musician,
game designer, translator, or developer.

**👉 See [CONTRIBUTING.md](./CONTRIBUTING.md) for detailed guides on how to get involved.**

Not sure where to start? Join our [Discord](https://discord.gg/5YXK5DRjPZ) and say hi!

## Contributors

The following individuals have contributed to the SoupRune project!

<a href = "https://github.com/Bli-AIk/souprune/Python/graphs/contributors">
<img src = "https://contrib.rocks/image?repo=Bli-AIk/souprune" alt=""/>
</a>

### Non-Code Contributors

* **71**: Provided many Alight Motion example projects (including Undertale bullet patterns and visual PVs) during
  testing. Great help with AM integration!
* **陈皮**: Provided many Alight Motion example projects (mostly Undertale bullet patterns), providing great help for AM
  integration.

**Heartfelt thanks to each one of you! 🎔**

## 🤝 Join Us

Whether you:

* Want to make your own DR/UT style game;
* Want to try Bevy and Rust;
* Or simply love the open-source and next-gen spirit—

You're welcome to participate in the construction of **SoupRune**:

* Submit Issues for any questions or suggestions!
* Contribute to SoupRune by providing Pull Requests!
* Share ideas and discuss architecture in Discord or GitHub Discussions!
* Or just chat about game development in the community!

**Let's cook the most delicious Soup together!**

## 🏗️ Project Architecture

SoupRune adopts a multi-crate workspace architecture:

| Crate                                                   | Description                                                                                 |
|:--------------------------------------------------------|:--------------------------------------------------------------------------------------------|
| [`souprune`](./crates/souprune)                         | **Core Framework**: The main framework body, application entry point, and core logic.       |
| [`souprune_api`](./crates/souprune_api)                 | **Protocol Layer**: WIT interface definitions and shared types for the WASM mod system.     |
| [`souprune_sdk`](./crates/souprune_sdk)                 | **Development Kit**: Rust WASM guest SDK for building mod components.                       |
| [`souprune_mod_test`](./crates/souprune_mod_test)       | **Sample Mod**: Example WASM mod component (target: wasm32-wasip2).                         |
| [`souprune_mock_host`](./crates/souprune_mock_host)     | **Tool**: Standalone test host for loading and running WASM mods via Wasmtime.              |
| [`souprune_schema`](./crates/souprune_schema)           | **Schema**: Pure data schema types for SoupRune RON files. No Bevy dependency.              |
| [`souprune_lint`](./crates/souprune_lint)               | **Linter**: RON file linter for SoupRune projects.                                          |
| [`bevy_mortar_bond`](./crates/bevy_mortar_bond)         | **Plugin**: Bridge between Mortar scripting and Bevy, handling dialogue and logic.          |
| [`bevy_ecs_typewriter`](./crates/bevy_ecs_typewriter)   | **Plugin**: ECS-based typewriter implementation, supporting rich text and multi-language.   |
| [`bevy_fact_rule_event`](./crates/bevy_fact_rule_event) | **Plugin**: Complex event system based on the "Fact-Rule-Event" model.                      |
| [`bevy_alight_motion`](./crates/bevy_alight_motion)     | **Plugin**: Integration for parsing and playing Alight Motion animation projects.           |

## 🧩 Script Layer Support

SoupRune's modding system is built on the **WASM Component Model** to achieve safe, sandboxed, cross-platform
mod loading. Mods are compiled to `.wasm` components and loaded at runtime via Wasmtime. The interface contract
is defined using WIT (WebAssembly Interface Types).

|        Language         | Target Users                                      | Description                                                                  |
|:-----------------------:|:--------------------------------------------------|:-----------------------------------------------------------------------------|
|        **Rust**         | System-level developers / Rustaceans / Bevy users | Native support via `souprune_sdk` + `wit-bindgen`. Best performance.         |
|    **JS** (Planned)     | Web developers / scripting users                  | Via `jco` (JavaScript Component Object) toolchain for rapid mod prototyping. |
| **.NET (C#)** (Planned) | Unity / Godot / C# users                          | Via .NET WASI SDK for C# mod development.                                    |

If you are interested in helping with SoupRune's multi-language support, contributions are welcome!

> 🤝 **Friendly Recommendation / Looking for Lua (Love2D) Ecosystem?**
>
> SoupRune is built around the WASM Component Model and focuses on compiled, sandboxed mod components.
> If you prefer the flexibility of **Lua dynamic scripting** or are used to the **Love2D** workflow, we strongly
> recommend you try another excellent framework in the community:
>
> **[SoulEngine](https://github.com/AnskiyyRenew/love2d-undertale-template)** — Designed for rapid prototyping and
> dynamic building.
>
> *We want to build not just a framework, but a thriving ecosystem for UT/DR fangame development together. ❤️*

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
| [Bevy](https://crates.io/crates/bevy) | 0.18    | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | Game Engine Core |

### Bevy Plugin Ecosystem

| Project                                                                   | Version | License                                                                                                                                                                                                       | Description                                                                                            |
|---------------------------------------------------------------------------|---------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|--------------------------------------------------------------------------------------------------------|
| [bevy_ecs_typewriter](https://github.com/Bli-AIk/bevy_ecs_typewriter)     | 0.2.0   | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | Typewriter implementation based on ECS                                                                 |
| [bevy_fact_rule_event](https://github.com/Bli-AIk/bevy_fact_rule_event)   | 0.3.0   | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | Implementation of an event system based on fact-rule-event                                             |
| [bevy_mortar_bond](https://github.com/Bli-AIk/bevy_mortar_bond)           | 0.3.0   | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | A bridge library between Bevy and the Mortar language                                                  |
| [bevy_alight_motion](https://github.com/Bli-AIk/bevy_alight_motion)       | 0.3.0   | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | Integration for Alight Motion animations, includes SDF shape rendering                                 |
| [bevy_tween (fork)](https://github.com/Bli-AIk/bevy_tween)                | 0.12    | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | Tweening animation library for Bevy, used for smooth animations and transitions                        |
| [iyes_perf_ui (fork)](https://github.com/Bli-AIk/iyes_perf_ui)            | 0.5.0   | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | Optional performance monitoring UI for Bevy, displaying FPS, system timings, and profiling information |
| [leafwing-input-manager](https://crates.io/crates/leafwing-input-manager) | 0.20    | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | Manages game input, handling keyboard, mouse, and controller operation mappings                        |
| [bevy_ecs_tiled](https://crates.io/crates/bevy_ecs_tiled)                 | 0.11    | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT)                                                                                                           | Integrates Bevy ECS and Tiled map editor for tile-based game levels                                    |
| [bevy-inspector-egui](https://crates.io/crates/bevy-inspector-egui)       | 0.36.0  | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | Optional editor/debugging tool for Bevy, used for real-time inspection of the ECS world                |
| [bevy_rich_text3d](https://crates.io/crates/bevy_rich_text3d)             | 0.6     | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | Mesh-based rasterized rich text implementation                                                         |
| [bevy_kira_audio](https://crates.io/crates/bevy_kira_audio)               | 0.25    | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | Audio playback plugin using Kira, supports WAV, OGG, FLAC, MP3 formats                                 |
| [bevy_brp_extras](https://crates.io/crates/bevy_brp_extras)               | 0.18    | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | Extra features for Bevy Remote Protocol (BRP)                                                          |

### Rust Crates

| Project                                           | Version | License                                                                                                                                                                                                       | Description                                                                                                    |
|---------------------------------------------------|---------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|----------------------------------------------------------------------------------------------------------------|
| [serde](https://crates.io/crates/serde)           | 1.0     | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | Serialization/deserialization framework supporting `derive` macros for convenient (de)serialization of structs |
| [toml](https://crates.io/crates/toml)             | 0.9.8   | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | TOML parsing                                                                                                   |
| [ron](https://crates.io/crates/ron)               | 0.12    | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | Rusty Object Notation parsing                                                                                  |
| [serde_json](https://crates.io/crates/serde_json) | 1.0     | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | JSON serialization/deserialization                                                                             |
| [regex](https://crates.io/crates/regex)           | 1.10    | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | Regular expressions                                                                                            |
| [fasteval](https://crates.io/crates/fasteval)     | 0.2     | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT)                                                                                                           | Fast and safe evaluation of algebraic expressions                                                              |
| [wasmtime](https://crates.io/crates/wasmtime)     | 42      | [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE)                                                                                                     | WebAssembly runtime for WASM mod loading                                                                       |

### Asset References

| Project                                             | Description                                               |
|-----------------------------------------------------|-----------------------------------------------------------|
| [DTTVL-Fonts](https://github.com/UTCLC/DTTVL-Fonts) | Font files used in the DELTATRAVELER localization project |

**Heartfelt thanks to the contributors of each of these projects! 🎔**
