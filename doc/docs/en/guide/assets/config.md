# Mod Configuration (mod.toml)

Every Mod directory must contain a `mod.toml` file, which tells the engine how to load your Mod.

## Example Configuration

```toml
name = "example_mod"
version = "0.1.0"
authors = ["Your Name"]
description = "An example mod for SoupRune."

[dependencies]
# List other Mods your Mod depends on here

[soul_modes]
# Define dynamic link libraries corresponding to Soul Modes
"soul_red" = "libmod_example.so"
"soul_blue" = "libmod_example.so"
```

## Field Descriptions

*   **name**: Unique identifier for the Mod.
*   **version**: Version number, following Semantic Versioning.
*   **soul_modes**: A key-value mapping.
    *   Key: The ID of the Soul Mode (e.g., "soul_red").
    *   Value: The name of the compiled library file containing the logic for that mode (e.g., "libmod_example.so"). This allows you to write custom Soul movement and interaction logic in Rust.