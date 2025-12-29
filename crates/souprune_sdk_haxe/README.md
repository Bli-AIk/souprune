# Souprune Haxe SDK

This directory contains the official Haxe SDK for creating Souprune mods.

## Contents

- `SoupruneApi.hx` - Auto-generated FFI bindings (do not edit)
- `souprune_api.h` - C header for hxcpp linking
- `Sdk/DanmakuBehavior.hx` - Danmaku behavior interface and types
- `Sdk/NativeExports.hx` - Native export infrastructure

## Usage

To use this SDK in your Haxe mod project:

1. Copy this entire directory to your project or add it to your classpath
2. Implement `IDanmakuBehavior` for your custom behaviors
3. Register behaviors using `DanmakuRegistry.register()`
4. Export the required C functions

## Regenerating Bindings

The `SoupruneApi.hx` file is auto-generated. To regenerate:

```bash
cd /path/to/souprune
cargo run -p souprune_api --features bindgen --bin souprune_bindgen
cp generated/SoupruneApi.hx crates/souprune_sdk_haxe/
cp generated/souprune_api.h crates/souprune_sdk_haxe/
```

## Example

See `projects/example_mod/code/mod_example_haxe/` for a complete example.
