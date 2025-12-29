# Overworld System

SoupRune uses the **Tiled Map Editor** as its primary level design tool.

## File Formats

In the `projects/<mod>/overworld/levels/` directory, you will find the following files:

*   **`.tiled-project`**: Tiled project file, managing Tilesets and Object Templates.
*   **`.world`**: Tiled world file, used to combine multiple maps (`.tmx` or internal formats) together to build a seamless overworld.

## Workflow

1.  **Download Tiled**: Visit [mapeditor.org](https://www.mapeditor.org/) to download the latest version.
2.  **Create Project**: Create a new project under `overworld/levels`.
3.  **Draw Maps**:
    *   **Tile Layers**: Used for drawing terrain, walls, and other static visual elements.
    *   **Object Layers**: Used for placing colliders, NPCs, portals, and other interactive objects.
4.  **Define Properties**:
    *   You can add custom properties to objects (e.g., `script` pointing to a Mortar script, or `target_map` pointing to a teleport destination). The engine reads these properties upon loading and binds the corresponding logic.

## Collision Detection

Typically, you need to create a dedicated "collision layer" in Tiled or use shapes (rectangles, polygons) in an Object Layer to define walkable areas and obstacles.