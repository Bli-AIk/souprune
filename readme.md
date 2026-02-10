# SoupRune

[![license](https://img.shields.io/github/license/Bli-AIk/souprune)](LICENSE.md) <img src="https://img.shields.io/github/repo-size/Bli-AIk/souprune.svg"/> <img src="https://img.shields.io/github/last-commit/Bli-AIk/souprune.svg"/> <br>
<img src="https://img.shields.io/badge/Deltarune / Undertale-black?style=for-the-badge&logo=undertale&logoColor=ff0000" /> <img src="https://img.shields.io/badge/Bevy-232326?style=for-the-badge&logo=bevy&logoColor=white" /> <br>
<img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" />
<img src="https://img.shields.io/badge/C%23-experimental-239120?style=for-the-badge&logo=csharp&logoColor=white" />
<img src="https://img.shields.io/badge/Haxe-experimental-EA8220?style=for-the-badge&logo=haxe&logoColor=white" />
<img src="https://img.shields.io/badge/Nim-Coming_Soon-FFE953?style=for-the-badge&logo=nim&logoColor=black" />
<img src="https://img.shields.io/badge/Nelua-Coming_Soon-9CA3AF?style=for-the-badge&logo=lua&logoColor=white" />

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

It strives to be the next-generation community-driven Fangame engine—bringing a unique flavor while embracing high
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
* **P (Polyglot) - Multi-language Support**: Language-agnostic via C ABI. You can choose the "utensil" (C#, Haxe, Rust,
  etc.) that suits you best to enjoy this soup.

## 🚀 Quick Start

SoupRune is currently in the **🚧 Initial Development Phase**, but if you're eager to try it out, you can start with
these steps:

1. **Prepare Environment**: Install the [Rust Development Environment](https://www.rust-lang.org/).
2. **Clone Repository**: `git clone https://github.com/Bli-AIk/souprune.git`
3. **Enter Directory**: `cd souprune`
4. **Update Submodules**: `git submodule update --init --recursive`
5. **Run Example (Debug Mode)**: `cargo run --package souprune --bin souprune --features debug`

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
   This is where SoupRune shines—you can choose **Rust**, **C# (.NET)**, or **Haxe** to write this logic!
    * Your code is compiled into a dynamic library (`.dll` or `.so`) and "plugged" into the engine like a plugin.
    * The engine communicates with your script via a standard interface (C ABI), giving you **high performance** while
      letting you use the **language you know best**.

**In short: SoupRune prepares the broth (underlying engine); you just follow the recipe (RON configs) and pick your
favorite spoon (programming language) to add the ingredients and cook your delicious game!**

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
* **Project**: Communicates with the core via a standard **C ABI**.

We know that developers in the Undertale / Deltarune community come from diverse backgrounds—

* Developers from **Clickteam Fusion** will find **Ron** is just another form of event table;
* Enthusiasts from **GameMaker** will find **Haxe** familiar and natural;
* Professionals used to **Unity/Godot** can continue to enjoy **C#** (achieving excellent performance via Native AOT);
* Friends used to **Lua** or **Python / GDScript** will be able to join seamlessly via **Nelua** or **Nim** in the
  future.

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

| Crate                                                             | Description                                                                                  |
|:------------------------------------------------------------------|:---------------------------------------------------------------------------------------------|
| [`souprune`](./crates/souprune)                                   | **Core Framework**: The main framework body, application entry point, and core logic.        |
| [`souprune_api`](./crates/souprune_api)                           | **Protocol Layer**: Defines interface standards for Project (Mod) interaction with the core. |
| [`souprune_sdk`](./crates/souprune_sdk)                           | **Development Kit**: A wrapper for the API, provided for external Project (Mod) scripts.     |
| [`souprune_mod_test`](./crates/souprune_mod_test)                 | **Sample Mod**: Sample test library for scripting systems.                                   |
| [`bevy_mortar_bond`](./crates/bevy_mortar_bond)                   | **Plugin**: Bridge between Mortar scripting and Bevy, handling dialogue and logic.           |
| [`bevy_ecs_typewriter`](./crates/bevy_ecs_typewriter)             | **Plugin**: ECS-based typewriter implementation, supporting rich text and multi-language.    |
| [`bevy_fact_rule_event`](./crates/bevy_fact_rule_event)           | **Plugin**: Complex event system based on the "Fact-Rule-Event" model.                       |
| [`bevy_alight_motion`](./crates/bevy_alight_motion)               | **Plugin**: Integration for parsing and playing Alight Motion animation projects.            |
| [`interoptopus_backend_haxe`](./crates/interoptopus_backend_haxe) | **Tool**: Haxe (hxcpp) backend for Interoptopus FFI bindings generator.                      |

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

> 🤝 **Friendly Recommendation / Looking for Lua (Love2D) Ecosystem?**
>
> SoupRune focuses on the extreme performance of AOT compilation and statically typed languages.
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
| [Bevy](https://crates.io/crates/bevy) | 0.17.2  | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | Game Engine Core |

### Bevy Plugin Ecosystem

| Project                                                                   | Version                                                                                                                               | License                                                                                                                                                                                                       | Description                                                                                            |
|---------------------------------------------------------------------------|---------------------------------------------------------------------------------------------------------------------------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|--------------------------------------------------------------------------------------------------------|
| [bevy_ecs_typewriter](https://github.com/Bli-AIk/bevy_ecs_typewriter)     | 0.0.0                                                                                                                                 | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | Typewriter implementation based on ECS                                                                 |
| [bevy_fact_rule_event](https://github.com/Bli-AIk/bevy_fact_rule_event)   | 0.0.0                                                                                                                                 | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | Implementation of an event system based on fact-rule-event                                             |
| [bevy_mortar_bond](https://github.com/Bli-AIk/bevy_mortar_bond)           | 0.0.0                                                                                                                                 | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | A bridge library between Bevy and the Mortar language                                                  |
| [leafwing-input-manager](https://crates.io/crates/leafwing-input-manager) | 0.19.0                                                                                                                                | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | Manages game input, handling keyboard, mouse, and controller operation mappings                        |
| [seldom_state](https://crates.io/crates/seldom_state)                     | 0.15.0                                                                                                                                | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | Finite State Machine implementation                                                                    |
| [bevy_ecs_tiled](https://crates.io/crates/bevy_ecs_tiled)                 | 0.10.0                                                                                                                                | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT)                                                                                                           | Integrates Bevy ECS and Tiled map editor for tile-based game levels                                    |
| [bevy_tween](https://crates.io/crates/bevy_tween)                         | 0.10.0                                                                                                                                | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | Tweening animation library for Bevy, used for smooth animations and transitions                        |
| [bevy_alight_motion](https://github.com/Bli-AIk/bevy_alight_motion)       | 0.0.0                                                                                                                                 | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | Integration for Alight Motion animations, includes SDF shape rendering                                 |
| [bevy_seedling](https://crates.io/crates/bevy_seedling)                   | 0.6.1                                                                                                                                 | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | Alternative audio backend                                                                              |
| [bevy-inspector-egui](https://crates.io/crates/bevy-inspector-egui)       | 0.35.0                                                                                                                                | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | Optional editor/debugging tool for Bevy, used for real-time inspection of the ECS world                |
| [iyes_perf_ui](https://crates.io/crates/iyes_perf_ui)                     | dev (My [fork](https://github.com/Bli-AIk/iyes_perf_ui) based on GitHub PR [#35](https://github.com/IyesGames/iyes_perf_ui/pull/35) ) | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | Optional performance monitoring UI for Bevy, displaying FPS, system timings, and profiling information |
| [bevy_rich_text3d](https://crates.io/crates/bevy_rich_text3d)             | 0.5.1                                                                                                                                 | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | Mesh-based rasterized rich text implementation                                                         |
| [bevy_kira_audio](https://crates.io/crates/bevy_kira_audio)               | 0.24.0                                                                                                                                | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | Audio playback plugin using Kira, supports WAV, OGG, FLAC, MP3 formats                                 |
| [bevy_brp_extras](https://crates.io/crates/bevy_brp_extras)               | 0.17.2                                                                                                                                | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | Extra features for Bevy Remote Protocol (BRP)                                                          |

### Rust Crates

| Project                                               | Version         | License                                                                                                                                                                                                       | Description                                                                                                    |
|-------------------------------------------------------|-----------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|----------------------------------------------------------------------------------------------------------------|
| [serde](https://crates.io/crates/serde)               | 1.0             | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | Serialization/deserialization framework supporting `derive` macros for convenient (de)serialization of structs |
| [toml](https://crates.io/crates/toml)                 | 0.9.8           | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | TOML parsing                                                                                                   |
| [ron](https://crates.io/crates/ron)                   | 0.10            | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | Rusty Object Notation parsing                                                                                  |
| [serde_json](https://crates.io/crates/serde_json)     | 1.0             | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | JSON serialization/deserialization                                                                             |
| [regex](https://crates.io/crates/regex)               | 1.10            | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | Regular expressions                                                                                            |
| [fasteval](https://crates.io/crates/fasteval)         | 0.2             | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT)                                                                                                           | Fast and safe evaluation of algebraic expressions                                                              |
| [interoptopus](https://crates.io/crates/interoptopus) | 0.15.0-alpha.24 | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT)                                                                                                           | The polyglot binding generator                                                                                 |
| [mortar_compiler](https://github.com/Bli-AIk/mortar)  | 0.5.0           | [![License](https://img.shields.io/badge/license-MIT-blue.svg)](./third_party_licenses/LICENSE-MIT) [![License](https://img.shields.io/badge/license-Apache-blue.svg)](./third_party_licenses/LICENSE-APACHE) | Compiler for the Mortar language                                                                               |

### Asset References

| Project                                             | Description                                               |
|-----------------------------------------------------|-----------------------------------------------------------|
| [DTTVL-Fonts](https://github.com/UTCLC/DTTVL-Fonts) | Font files used in the DELTATRAVELER localization project |

**Heartfelt thanks to the contributors of each of these projects! 🎔**
