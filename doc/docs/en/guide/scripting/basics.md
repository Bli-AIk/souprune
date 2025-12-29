# Mortar Scripting Basics

**Mortar** is the dedicated scripting language for SoupRune, used for writing dialogue, story flows, and simple logic control. Its syntax is designed to be concise, resembling scriptwriting.

## Nodes

Mortar scripts are composed of multiple **Nodes**. Each node represents a segment of dialogue or story.

```rust
node Start {
    text: "Hello, world!"
} -> NextNode
```

### Node Structure

*   `node Name { ... }`: Defines a node.
*   `text: "..."`: The text content displayed by the node.
*   `-> Target`: Defines the jump target after the node ends.

## Variables and Interpolation

You can use `{}` within text to insert variables or the results of function calls.

```rust
node Greeting {
    text: $"Hello, {get_player_name()}!"
}
```
Note: When using interpolation, the string must be prefixed with the `$` symbol.

## External Functions

To use logic defined on the Rust side within Mortar, you need to declare an `fn`.

```rust
// Declare a function to play sound
fn play_sound(path: String);

node Music {
    // Use it later in events
    text: "Listen to this music..."
}
```

## Choices

You can provide choices for the player to select, and even display different options based on conditions.

```rust
node Question {
    text: "Where do you want to go?"
    
    choice: [
        "Forest" -> Forest,
        "Castle" -> Castle,
        
        // Option with a condition
        "Secret Base" when has_key() -> SecretBase,
        
        // Nested choices
        "Check Inventory" -> [
            "Apple" -> EatApple;
            "Bread" -> EatBread;
        ]
    ]
}
```

*   `when condition()`: The option is only available when the condition is met.
*   `-> [ ... ]`: Choices can be nested.