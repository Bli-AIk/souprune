# Souprune Haxe Mod Example

This is a Hello World example mod for Souprune written in Haxe using hxcpp.

## Prerequisites

- Haxe 4.3+ (https://haxe.org/)
- hxcpp (install via `haxelib install hxcpp`)
- A C++ compiler (g++, clang++, MSVC)

## Building

```bash
# Install dependencies
haxelib install hxcpp

# Build the mod
haxe build.hxml
```

## Project Structure

```
mod_example_haxe/
├── build.hxml          # Haxe build configuration
├── src/
│   ├── Main.hx         # Entry point
│   └── souprune/
│       └── ffi/
│           └── SoupruneApi.hx  # Generated FFI bindings
└── README.md
```

## Notes

The `SoupruneApi.hx` file is auto-generated from `souprune_api`. 
Do not edit it manually. To regenerate, run:

```bash
cd /path/to/souprune
cargo run -p souprune_api --features bindgen --bin souprune_bindgen
```
