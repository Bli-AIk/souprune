# souprune-lint

[![license](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](LICENSE) <img src="https://img.shields.io/github/repo-size/Bli-AIk/souprune.svg"/> <img src="https://img.shields.io/github/last-commit/Bli-AIk/souprune.svg"/> <br>
<img src="https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white" />

> Current Status: 🚧 Early Development

**souprune-lint** — Command-line linter for validating SoupRune RON configuration files.

| English | Simplified Chinese          |
|---------|-----------------------------|
| English | [简体中文](./readme_zh-hans.md) |

## Introduction

`souprune-lint` is a standalone binary tool for static validation of SoupRune game configuration files (`.view.ron`, `.sdf.ron`, `.enemy.ron`, `.performance.ron`, etc.). It checks RON files for structural correctness and schema violations, producing diagnostics in multiple output formats suitable for CI/CD pipelines and IDE integration.

## Features

* **Schema Validation** — Validates RON files against `souprune_schema` type definitions
* **File Kind Detection** — Automatically identifies file type from extension (`.view.ron`, `.sdf.ron`, `.enemy.ron`, `.items.ron`, `.sequence.ron`, `.fre.ron`, `.performance.ron`, etc.)
* **Recursive Scanning** — Lint entire directories with `walkdir`
* **Multiple Output Formats**:
  - `pretty` — Colorful ariadne-based diagnostic display
  - `jetbrains` — IDE-compatible `file:line:col: level: message` format
  - `json` — Machine-readable structured output

## How to Use

```bash
# Lint a single file
cargo run -p souprune-lint -- check path/to/file.view.ron

# Lint a directory recursively
cargo run -p souprune-lint -- check projects/mad_dummy_example/

# Specify output format
cargo run -p souprune-lint -- check --format jetbrains projects/
```

## How to Build

```bash
cargo build --release -p souprune-lint
```

The binary will be at `target/release/souprune-lint`.

## Dependencies

| Crate                                                  | Version | Description              |
|--------------------------------------------------------|---------|--------------------------|
| [souprune_schema](../souprune_schema)                  | 0.1     | Schema type definitions  |
| [ron](https://crates.io/crates/ron)                    | 0.12    | RON parsing              |
| [clap](https://crates.io/crates/clap)                 | 4       | CLI argument parsing     |
| [ariadne](https://crates.io/crates/ariadne)           | 0.4     | Diagnostic rendering     |
| [walkdir](https://crates.io/crates/walkdir)           | 2       | Recursive file traversal |

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

* GNU Lesser General Public License v3.0 or later ([LICENSE](../../LICENSE.md) or [https://www.gnu.org/licenses/lgpl-3.0.html](https://www.gnu.org/licenses/lgpl-3.0.html))
