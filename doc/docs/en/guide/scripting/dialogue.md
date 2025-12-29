# Dialogue and Events

Mortar's most powerful feature is the precise synchronization of text with events. This is crucial for creating expressive RPG dialogue.

## Event Binding (with events)

After the `text` field, you can use a `with events` block to define events that trigger at specific points in time.

```rust
node Intro {
    text: "Look! What is that?"
    
    with events: [
        // Format: Index, Event Function
        
        // Play sound at the 0th character (start)
        0, play_sound("audio/surprise.wav"),
        
        // Change color at the 2nd character
        2, set_color("#FF0000"),
        
        // Shake screen at the 5th character
        5, shake_screen(1.0)
    ]
}
```

### Index Types

*   **Integer (Int)**: Represents character index. For example, `2` means trigger when the typewriter types the 3rd character.
*   **Float**: Represents time (seconds). This is useful in non-typewriter modes or when fine control over audio synchronization is needed.

## Common Event Patterns

While specific event functions depend on your Mod code implementation (registered in Rust), here are some common design patterns:

### 1. Changing Text Color

```rust
fn set_color(hex: String);

node ColorDemo {
    text: "This is normal, this is red."
    with events: [
        16, set_color("#FF0000"),
        21, set_color("#FFFFFF") // Revert to white
    ]
}
```

### 2. Playing Voices/SFX

```rust
fn play_sfx(name: String);

node SoundDemo {
    text: "Boom! It exploded."
    with events: [
        0, play_sfx("explosion")
    ]
}
```

### 3. Character Animation

```rust
fn set_face(expression: String);

node FaceDemo {
    text: "I am happy... Now I am angry!"
    with events: [
        0, set_face("happy"),
        14, set_face("angry")
    ]
}
```