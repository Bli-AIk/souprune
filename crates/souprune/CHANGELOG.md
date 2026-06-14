# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.6.0](https://github.com/Bli-AIk/souprune/compare/souprune-v0.5.1...souprune-v0.6.0) - 2026-05-06

### Added

- *(dialogue)* add configurable text animation styles ([#121](https://github.com/Bli-AIk/souprune/pull/121))
- *(dialogue)* add enemy speech bubbles ([#116](https://github.com/Bli-AIk/souprune/pull/116))
- Add Vessel execution support for RON authoring ([#115](https://github.com/Bli-AIk/souprune/pull/115))
- *(ui)* refactor coordinate space and view layout system ([#111](https://github.com/Bli-AIk/souprune/pull/111))
- [**breaking**] overhaul code-first content authoring and mod tooling ([#110](https://github.com/Bli-AIk/souprune/pull/110))
- [**breaking**] add battle sequencer with loop, enemy turns, and danmaku enhancements ([#108](https://github.com/Bli-AIk/souprune/pull/108))
- *(dialogue)* add auto-pause and voice config systems ([#105](https://github.com/Bli-AIk/souprune/pull/105))
- *(example_battle)* optimize view system, refine resource resolution and config loading ([#96](https://github.com/Bli-AIk/souprune/pull/96))
- *(view)* add text reconstruction with genetic algorithm optimization ([#90](https://github.com/Bli-AIk/souprune/pull/90))
- *(battle)* add animated merge with gap policy and easing support ([#88](https://github.com/Bli-AIk/souprune/pull/88))
- [**breaking**] Battle System API Enhancement & Text Rendering Backend Overhaul ([#83](https://github.com/Bli-AIk/souprune/pull/83))
- [**breaking**] Comprehensive FRE system refactor and animation/editor enhancements ([#78](https://github.com/Bli-AIk/souprune/pull/78))
- *(editor)* add experimental souprune_editor crate with asset browser and view editor ([#77](https://github.com/Bli-AIk/souprune/pull/77))
- Add Android platform support with touch controls and build pipeline ([#73](https://github.com/Bli-AIk/souprune/pull/73))
- [**breaking**] Implement comprehensive dialogue system and restructure project layout ([#71](https://github.com/Bli-AIk/souprune/pull/71))
- Integrate cargo-deny and optimize CI pipeline ([#59](https://github.com/Bli-AIk/souprune/pull/59))
- [**breaking**] integrate Fact-Rule-Event system into battle and debug tools ([#45](https://github.com/Bli-AIk/souprune/pull/45))
- [**breaking**] Unified Interactive View & Data-Driven Input System ([#44](https://github.com/Bli-AIk/souprune/pull/44))
- *(battle)* Add Tween View Element Animation System ([#43](https://github.com/Bli-AIk/souprune/pull/43))
- add ModifyViewElement command & refactor ui to view ([#41](https://github.com/Bli-AIk/souprune/pull/41))

### Fixed

- *(core)* fix elastic easing function parentheses
- *(ci)* copy WIT files into each crate for cargo publish compatibility

### Miscellaneous Tasks

- [**breaking**] migrate maintained mods to submodules ([#120](https://github.com/Bli-AIk/souprune/pull/120))
- *(core)* standardize format across 184 files
- clean up test changes from main
- *(deps)* bump wasmtime 42→43, ariadne 0.4→0.6

### Other

- [**breaking**] view interaction system and generic sequence engine (breaking) ([#67](https://github.com/Bli-AIk/souprune/pull/67))

### Refactor

- refactor!(overworld): unify trigger system and fix dialogue/puzzle bugs ([#106](https://github.com/Bli-AIk/souprune/pull/106))
- WASM mod framework — generalize architecture and abolish config.toml ([#95](https://github.com/Bli-AIk/souprune/pull/95))
- *(workspace)* raise structural guards and land architecture cleanup ([#89](https://github.com/Bli-AIk/souprune/pull/89))
- [**breaking**] migrate mod system from C ABI to WASM Component Model ([#79](https://github.com/Bli-AIk/souprune/pull/79))
- sequence-driven architecture with dynamic modes and extensible dispatch ([#75](https://github.com/Bli-AIk/souprune/pull/75))
- [**breaking**] upgrade to Bevy 0.18 ([#66](https://github.com/Bli-AIk/souprune/pull/66))
- [**breaking**] migrate HP bar to generic ShaderMaterial and introduce DynamicMaterial2d ([#65](https://github.com/Bli-AIk/souprune/pull/65))
- *(view)* replace evalexpr with fasteval for expression evaluation ([#63](https://github.com/Bli-AIk/souprune/pull/63))
- refactor!(view): adopt FRE-driven reconciliation view system ([#58](https://github.com/Bli-AIk/souprune/pull/58))
- [**breaking**] Remove hardcoded paths and introduce unified Visual system ([#47](https://github.com/Bli-AIk/souprune/pull/47))
- replace debug visualizers with Gizmos & simplify core logic ([#46](https://github.com/Bli-AIk/souprune/pull/46))
- Replace bevy_smud with custom SDF rendering ([#42](https://github.com/Bli-AIk/souprune/pull/42))
- Refactor RON backends to separate schema from logic ([#40](https://github.com/Bli-AIk/souprune/pull/40))

### Testing

- modify inside crate (should trigger CLA)

## [0.5.0](https://github.com/Bli-AIk/souprune/releases/tag/souprune-v0.5.0) - 2026-01-27

### Added

- Unified Danmaku, Overworld Chase, Alight Motion Integration, & Docs Overhaul ([#29](https://github.com/Bli-AIk/souprune/pull/29))
- Danmaku system & Multi-language SDK support ([#22](https://github.com/Bli-AIk/souprune/pull/22))
- *(battle)* implement battle system core infrastructure & dynamic UI (v0.4.0) ([#20](https://github.com/Bli-AIk/souprune/pull/20))
- *(ui)* Implement Overworld UI Infrastructure ([#15](https://github.com/Bli-AIk/souprune/pull/15))
- Implement Collision Detection System and Optimize Overworld Components ([#13](https://github.com/Bli-AIk/souprune/pull/13))
- Add Tilemap based on Tiled ([#11](https://github.com/Bli-AIk/souprune/pull/11))
- Sprite configuration files and sprite animation functions ([#7](https://github.com/Bli-AIk/souprune/pull/7))
- *(core)* Adding a standard character state machine ([#6](https://github.com/Bli-AIk/souprune/pull/6))
- *(ci)* add GitHub Actions workflows and documentation ([#1](https://github.com/Bli-AIk/souprune/pull/1))
- initial project setup

### Other

- *(souprune)* exclude fonts and audio assets from crates.io package
- *(souprune)* add version constraints for all path dependencies
- *(Cargo.toml)* reorder dependencies and add exclude list
- *(core)* bump version to 0.4.1
- Basic Data-driven Architecture ([#16](https://github.com/Bli-AIk/souprune/pull/16))
- *(core)* bump version to 0.2.0
- Restructure core modules and implement plugin-based architecture ([#12](https://github.com/Bli-AIk/souprune/pull/12))
- add line breaks to readme badges
- restructure readme files with improved content and layout ([#5](https://github.com/Bli-AIk/souprune/pull/5))
