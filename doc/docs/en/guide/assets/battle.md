# Battle System Configuration

SoupRune's battle system is highly data-driven. The battle flow is primarily defined via `.chapter.ron` files, located in the `projects/<mod>/battle/chapters/` directory.

## Chapter Files (RON)

RON (Rusty Object Notation) is a format similar to JSON but supports Rust types. A battle chapter is typically a list of actions.

### Example Structure

```rust
[
    // 1. Initialize Camera
    SetCamera(SetZoom(0.4)),
    
    // 2. Load UI Layout
    UIInteraction(ui_layout: "battle/ui/undertale.ui_layout.ron"),
    
    // 3. Spawn Player (Soul)
    SetPlayer(Spawn(
        config_path: "battle/players/player.battle_player.ron", 
        position: (0.0, -80.0)
    )),
    
    // 4. Wait for 5 Seconds
    Wait(5.0),
    
    // 5. Execute Bullet Pattern
    BulletPattern(
        pattern_id: ["flowey_pellets_circle"]
    ),
    
    // 6. Nested Sequence
    Nested([
        Wait(0.5),
        SetPlayer(Despawn),
    ]),
]
```

## Common Instructions

*   **SetCamera**: Controls camera zoom and position during battle.
*   **UIInteraction**: Loads or manipulates the battle UI.
*   **SetPlayer**: Manages the spawning (`Spawn`) and despawning (`Despawn`) of the player's Soul.
*   **Wait**: Waits for a specified amount of time (in seconds).
*   **BulletPattern**: Triggers a predefined bullet pattern. Bullet logic is usually implemented in Rust code and referenced by ID.
*   **Nested**: Executes a nested sequence of actions.

## Character Configuration

Battle characters are defined in the `battle/players/` directory, typically containing the character's stats (HP, ATK) and appearance configuration.