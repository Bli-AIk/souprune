# Quick Start

This section guides you on how to run SoupRune and launch the example Mod.

## Prerequisites

Before starting, please ensure you have the following tools installed:

1.  **Rust Toolchain**: Visit [rustup.rs](https://rustup.rs/) to install.
2.  **System Dependencies (Linux)**:
    ```bash
    sudo apt-get install g++ pkg-config libx11-dev libasound2-dev libudev-dev libwayland-dev libxkbcommon-dev
    ```

## Running the Project

SoupRune is a Rust workspace. To run the main program and load the example Mod, execute the following command in the project root directory:

```bash
cargo run --package souprune
```

The first compilation may take some time, so please be patient.

## Running Examples

If you want to test a specific component individually (e.g., the dialogue system UI), you can run a specific Example:

```bash
cargo run -p bevy_mortar_bond --example dialogue_ui
```

## Next Steps

After a successful run, you should see the game window. Next, you can check the [Project Structure](project_structure.md) to learn how to modify and create your own Mod.