# souprune-lint — Usage Guide

A standalone CLI tool for validating SoupRune RON files against schema definitions.

## Installation

Build from source (requires Rust toolchain):

```bash
cargo build -p souprune-lint --release
```

The binary will be at `target/release/souprune-lint`.

## Supported File Types

| Extension   | Schema         | Description               |
|-------------|----------------|---------------------------|
| `.view.ron` | `ViewLayout`   | View layout definitions   |
| `.sdf.ron`  | `SdfStructure` | SDF structure definitions |

## Commands

### `check` — Validate RON files

```bash
# Check a single file
souprune-lint check path/to/file.view.ron

# Check a directory recursively
souprune-lint check projects/

# Check multiple paths
souprune-lint check projects/mod_a/ projects/mod_b/some_file.view.ron
```

### Output Formats

Use `--format` to control output style:

#### `pretty` (default)

Rich diagnostic output with source context, powered by [ariadne](https://crates.io/crates/ariadne):

```
Error: Unexpected missing field named `name` in `ViewNodeDef`
   ╭─[path/to/file.view.ron:5:9]
   │
 5 │         ),
   │         ┬
   │         ╰── Unexpected missing field named `name` in `ViewNodeDef`
───╯
```

#### `jetbrains`

Single-line format compatible with JetBrains IDE File Watcher output filters:

```
path/to/file.view.ron:5:9: error: Unexpected missing field named `name` in `ViewNodeDef`
```

#### `json`

Machine-readable JSON for tool integration:

```json
{
  "file": "path/to/file.view.ron",
  "diagnostics": [
    { "severity": "error", "line": 5, "column": 9, "message": "Unexpected missing field named `name` in `ViewNodeDef`" }
  ]
}
```

## Exit Codes

| Code | Meaning                       |
|------|-------------------------------|
| `0`  | All files valid               |
| `1`  | One or more files have errors |

## Validation Layers

1. **RON Syntax** — Bracket matching, commas, quotes, RON extensions (`#![enable(implicit_some)]`)
2. **Schema Type** — Field names, required fields, type mismatches, invalid enum variants

## Notes

- Uses `ron` 0.12. Files authored for `ron` 0.10 may produce false positives if they use struct syntax
  `(x: v, y: v, z: v)` for tuple types — use positional syntax `(v, v, v)` instead.
- The `#![enable(implicit_some)]` pragma is respected automatically.
