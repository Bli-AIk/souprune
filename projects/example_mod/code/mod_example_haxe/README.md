# Souprune Haxe Mod Example

This is an example mod for Souprune written in Haxe using hxcpp.
It implements the same three danmaku behaviors as the Rust and C# versions.

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
├── build.hxml                  # Haxe build configuration
├── src/
│   ├── Main.hx                 # Entry point (mod registration)
│   └── behaviors/
│       ├── SpiralHomingDanmaku.hx
│       ├── WaveBurstDanmaku.hx
│       └── GravityDropDanmaku.hx
└── README.md
```

## Implemented Behaviors

1. **SpiralHomingDanmaku** - Spirals outward while slowly tracking the player
2. **WaveBurstDanmaku** - Moves in sine wave pattern with burst acceleration  
3. **GravityDropDanmaku** - Falls with gravity and bounces

These behaviors are identical to the Rust (`mod_example`) and C# (`mod_example_csharp`) versions.

## SDK Reference

The SDK is located at `crates/souprune_sdk_haxe/` and provides:
- `IDanmakuBehavior` interface
- `BulletContext`, `BulletOutput`, `Vec2` types
- `DanmakuRegistry` for behavior registration

