# Architecture

> For the Simplified Chinese version, see [architecture_zh-hans.md](architecture_zh-hans.md).

SoupRune is a Rust/Bevy framework for RPG+STG fangames (Deltarune/Undertale style).
This document describes its layered architecture.

---

## Three-Tier Architecture

```
┌──────────────────────────────────────────┐
│  User Content (Mod / Game Project)       │  RON, Mortar scripts,
│  projects/<mod>/                         │  WASM components, assets
├──────────────────────────────────────────┤
│  Preset Layer (Rust)                     │  Game-specific logic:
│  crates/souprune/src/preset/             │  battle, overworld, items,
│                                          │  enemies, UI layouts
├──────────────────────────────────────────┤
│  Core Engine (Rust)                      │  Generic infrastructure:
│  crates/souprune/src/core/               │  FRE, View, Mortar, danmaku,
│                                          │  collision, WASM runtime
└──────────────────────────────────────────┘
```

**Dependency rule**: Core ← Preset ← User Content. The arrow is strictly one-directional.

### Core (`core/`)

The engine layer. Knows mechanics, not meaning.

- **FRE** (Fact-Rule-Event): Data-driven rule engine. Events trigger rules; rules mutate facts; Views react to facts.
- **View**: RON-driven declarative UI with SDF rendering and reactive updates.
- **Mortar VM**: Bytecode VM for branching dialogue and scripted sequences.
- **Danmaku**: High-performance bullet engine — lifecycle, behavior stack, builtin motions.
- **Collision**: SDF-based detection with event buffering and cooldown deduplication.
- **Mod System**: WASM runtime (wasmtime) for user-defined behaviors and patterns.

Core provides danmaku, collision, dialogue, and View — but does not know what "HP", "item", or "battle phase" means.

### Preset (`preset/`)

The game logic layer. Transforms the generic core into a complete RPG+STG framework.

- Battle state machine, turn flow, damage calculation
- Overworld: player controller, NPC interaction, tilemap
- Item/enemy registries, FRE action handlers
- DataPath/Condition/ExprFunction resolvers for View
- MortarFactBindings for dialogue variable injection

Preset is intentionally monolithic — the target audience needs a complete kit.

### User Content (`projects/`)

Game-specific content authored via data and scripts:

- RON: item/enemy definitions, View layouts, danmaku performances
- Mortar: dialogue, cutscenes, event sequences
- FRE rules: game logic, state transitions
- WASM components: custom bullet behaviors, special mechanics
- Assets: sprites, audio, tilemaps

---

## Core Subsystems

### FRE (Fact-Rule-Event)

The engine's heart. Decouples data from behavior:

| Concept   | Role                                               | Example                        |
|-----------|----------------------------------------------------|--------------------------------|
| **Fact**  | Global key-value store                             | `"player:hp" = 20`             |
| **Event** | Signal that something happened                     | `CollisionEnter { a, b }`      |
| **Rule**  | Declarative logic binding events to fact mutations | `On TakeDamage → hp -= amount` |

Flow: Event → Rule evaluation → Fact mutation → Reactive View update.

### View System

RON-driven UI with resolver registries. Views respond to Fact changes, not imperative calls.
Preset injects game-specific data via `DataPathResolvers`, `ConditionResolvers`, and `ExprFunctionResolvers`.

### Danmaku

First-class STG support — SoupRune's competitive advantage:

- Builtin motions: Linear, Orbital, Sine, Tween, Stationary, Aimed
- Custom motions: WASM components for exotic patterns
- Timeline-driven spawn sequences

---

## WASM Extension Model

WASM is the extension point for mod authors, not a replacement for Rust systems.

- **Use WASM for**: custom bullet behaviors, exotic spawn patterns, mod-specific logic
- **Keep in Rust for**: core motions, collision, rendering, UI layout
- **WIT interfaces**: `behavior`, `danmaku`, `spawn-pattern`, `custom-action-handler`, `mode-lifecycle`, `rule-provider`

---

## Boundary Rules

✅ Core **may**: define danmaku primitives, collision, dialogue rendering, View layout, FRE evaluation, generic
scheduling

❌ Core **must not**: import from preset, hardcode game-specific fact keys, know about specific entities (Item, Enemy),
define game state machines

---

## Directory Map

```
crates/
├── souprune/src/
│   ├── core/               # Engine infrastructure
│   │   ├── danmaku/        #   Bullet engine (privileged)
│   │   ├── dialogue/       #   Dialogue + Mortar integration
│   │   ├── view/           #   RON-driven UI
│   │   ├── collision.rs    #   SDF collision
│   │   ├── fre_bridge.rs   #   FRE ↔ ECS bridge
│   │   └── mod_system.rs   #   WASM mod loading
│   ├── preset/             # Game logic (battle, overworld, items)
│   └── app_state/          # Application state management
├── bevy_fact_rule_event/   # FRE engine (submodule)
├── bevy_mortar_bond/       # Mortar scripting (submodule)
├── bevy_alight_motion/     # Alight Motion + SDF (submodule)
├── souprune_api/           # WIT interface definitions
└── souprune_sdk/           # Rust WASM guest SDK
```
