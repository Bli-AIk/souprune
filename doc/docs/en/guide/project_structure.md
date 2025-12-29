# Project Structure

SoupRune's project structure is designed to be clear, separating core engine code from user content. As a Mod Creator, you will primarily focus on the `projects/` directory.

## Directory Overview

```
souprune/
├── crates/                 # Engine Core Source Code (Rust)
├── projects/               # User Projects Directory
│   ├── config.toml         # Global Configuration
│   └── example_mod/        # Example Mod
│       ├── mod.toml        # Mod Metadata
│       ├── battle/         # Battle-related Resources
│       ├── overworld/      # Overworld-related Resources
│       ├── code/           # Scripts and Logic Code
│       └── shared/         # Shared Resources (Images, Text, etc.)
└── Cargo.toml              # Workspace Configuration
```

## Mod Structure in Detail

A standard Mod folder (e.g., `example_mod`) contains the following parts:

### 1. mod.toml
The core configuration file for the Mod, defining the Mod's name, version, author, and Soul Mode bindings.

### 2. battle/
Contains all data for the battle system:
*   **chapters/**: Defines the flow of battle chapters (in `.ron` format).
*   **players/**: Defines battle character attributes.
*   **ui/**: Layout configuration for the battle interface.

### 3. overworld/
Contains data for the Overworld:
*   **levels/**: Tiled map project files (`.tiled-project`, `.world`).
*   **characters/**: Overworld character definitions.

### 4. code/
Stores the logical code for the Mod.
*   **mod_example/**: Typically a Rust Crate that can be compiled into a dynamic link library (`.so` / `.dll`) to extend advanced game logic.

### 5. shared/
Stores common game assets:
*   **textures/**: Texture images.
*   **items/**: Item definitions.
*   **locales/**: Localization text files.