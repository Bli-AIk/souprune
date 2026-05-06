# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0](https://github.com/Bli-AIk/souprune/compare/souprune_sdk-v0.2.0...souprune_sdk-v0.3.0) - 2026-05-06

### Added

- [**breaking**] Comprehensive FRE system refactor and animation/editor enhancements ([#78](https://github.com/Bli-AIk/souprune/pull/78))
- *(editor)* add experimental souprune_editor crate with asset browser and view editor ([#77](https://github.com/Bli-AIk/souprune/pull/77))
- Add Android platform support with touch controls and build pipeline ([#73](https://github.com/Bli-AIk/souprune/pull/73))

### Fixed

- *(ci)* copy WIT files into each crate for cargo publish compatibility

### Miscellaneous Tasks

- *(core)* standardize format across 184 files

### Refactor

- WASM mod framework — generalize architecture and abolish config.toml ([#95](https://github.com/Bli-AIk/souprune/pull/95))
- *(workspace)* raise structural guards and land architecture cleanup ([#89](https://github.com/Bli-AIk/souprune/pull/89))
- [**breaking**] migrate mod system from C ABI to WASM Component Model ([#79](https://github.com/Bli-AIk/souprune/pull/79))

## [0.1.0](https://github.com/Bli-AIk/souprune/releases/tag/souprune_sdk-v0.1.0) - 2026-01-27

### Added

- Danmaku system & Multi-language SDK support ([#22](https://github.com/Bli-AIk/souprune/pull/22))
- *(battle)* implement battle system core infrastructure & dynamic UI (v0.4.0) ([#20](https://github.com/Bli-AIk/souprune/pull/20))

### Other

- *(release)* prepare packages for crates.io publication
- *(crates)* bump version from 0.0.0 to 0.1.0 for multiple crates
