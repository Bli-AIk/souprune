# Architecture

> For the Simplified Chinese version, see [architecture_zh-hans.md](architecture_zh-hans.md).

This document describes the internal architecture of SoupRune.

---

## The Big Picture: Three Tiers

SoupRune separates concerns into three distinct layers.
Think of it as **hardware → firmware → game cartridge**:

```
┌──────────────────────────────────────────────┐
│  User Content (Mod / Game Project)            │
│  projects/<mod>/                              │
│  ── RON configs, Mortar scripts, WASM mods ── │
├──────────────────────────────────────────────┤
│  Preset Layer (Rust native)                   │
│  crates/souprune/src/preset/                  │
│  ── Battle, overworld, items, enemies ──────  │
├──────────────────────────────────────────────┤
│  Core Engine (Rust native)                    │
│  crates/souprune/src/core/                    │
│  ── FRE, View, Mortar, danmaku, collision ──  │
└──────────────────────────────────────────────┘
```

**The dependency arrow is strictly one-directional**: Core ← Preset ← User Content.
Core never imports from preset. Preset never imports from user mods.

---

## Core Engine (`core/`)

The core layer is SoupRune's "runtime" — it knows **how** things work,
but not **what** they mean. It provides danmaku motion, collision detection,
dialogue rendering, and reactive UI — but has no concept of "HP", "items",
"enemies", or "battle phases".

### FRE (Fact-Rule-Event) — The Heart

FRE is SoupRune's data-driven rule engine. It decouples data from behavior
using three simple concepts:

| Concept   | What it does                                              | Example                        |
|-----------|-----------------------------------------------------------|--------------------------------|
| **Fact**  | A global key-value store — the single source of truth     | `"player:hp" = 20`             |
| **Event** | A signal that something happened — carries no logic       | `CollisionEnter { a, b }`      |
| **Rule**  | Declarative logic that reacts to events and mutates facts | `On TakeDamage → hp -= amount` |

**How it flows**: An event fires → the FRE engine evaluates matching rules →
rules mutate facts → the View system reactively updates the UI.

This means you never write `button.set_color(gray)`. Instead, the fact that
controls the button changes, and the View reacts automatically.

### View System — Declarative UI

Views are defined in `.view_layout.ron` files — not in Rust code.
The system uses SDF rendering (via `bevy_alight_motion`) and mesh-based text
(via `bevy_rich_text3d`) to produce high-quality visuals.

Preset injects game-specific data into Views through **resolver registries**:

- `DataPathResolvers` — resolve data paths like `"player.hp"` to actual values
- `ConditionResolvers` — evaluate conditions like `"has_item('sword')"`
- `ExprFunctionResolvers` — provide custom expression functions

This means core's View system is fully generic — it doesn't know what
`"player.hp"` means until preset tells it how to resolve that path.

### Mortar VM — Dialog system virtual machine

Mortar is a bytecode VM designed for branching dialogue and scripted sequences.
It handles the inherently complex logic of dialogue trees, conditional text,
and timed event sequences — keeping this complexity out of both FRE rules and Rust code.

Mortar scripts emit abstract events; FRE captures those events to update game state.
Text content lives in Mortar; game logic lives in FRE rules.

### Danmaku — The STG Engine

SoupRune is a **RPG/STG framework at its core**. The danmaku system is not an afterthought —
it's a first-class engine feature with privileged status in `core/`.

- **Bullet lifecycle**: spawn → behavior stack → per-frame motion update → despawn
- **Builtin motions** (native Rust, zero WASM overhead):
  Linear, Orbital, Sine, Tween, Stationary, Aimed
- **Custom motions**: user-defined WASM components for exotic bullet patterns
- **Timeline performances**: RON-driven spawn sequences with configurable patterns
- **Performance**: optimized for thousands of simultaneous bullets at 60fps

### Collision System

SDF-based collision detection with `EventPhase` buffering that provides
cooldown-based deduplication. The system emits generic collision events —
it's up to the preset layer to interpret them (e.g., "collision with player = take damage").

### Mod System (WASM Runtime)

A wasmtime-based runtime that loads user-provided WASM components.
Mods can provide custom bullet behaviors, spawn patterns, action handlers,
mode lifecycle hooks, and rule providers — all through well-defined WIT interfaces.

---

## Preset Layer (`preset/`)

The preset layer transforms the generic core engine into a complete UT/DR experience.
It is written in **native Rust** (not WASM) for maximum performance and type safety.

This layer is intentionally **monolithic** — the target audience (fangame creators)
needs a complete RPG+STG toolkit, not a puzzle of optional micro-crates.

### What preset provides

- **Battle system**: turn-based state machine, HP/damage, enemy AI, battle box
- **Overworld**: player controller, NPC interaction, tilemap, area triggers, chase sequences
- **Item system**: `ItemRegistry`, item effects (heal, equip, audio), FRE fact injection
- **Enemy system**: `EnemyRegistry`, enemy data, encounter configuration
- **FRE integration**: game-specific action handlers and rule definitions
- **View integration**: DataPath/Condition/ExprFunction resolvers for reactive UI
- **Dialogue integration**: `MortarFactBindings` for injecting game data into dialogue variables

### How preset talks to core

Preset communicates with core exclusively through standard Bevy and FRE mechanisms:

1. **Bevy ECS**: Components, Resources, Events, Systems, Plugins
2. **FRE**: Rules, Facts, Events, Action handlers
3. **Resolver registries**: Dynamic registration of data resolvers
4. **ViewActionExtensions**: Extensible action dispatch for View events
5. **MortarFactBindings**: Dynamic Mortar function/variable bindings

---

## User Content Layer (`projects/`)

This is where game creators work. Content is authored entirely through data and scripts:

| Format        | Purpose              | Example                                                           |
|---------------|----------------------|-------------------------------------------------------------------|
| **RON**       | Structured game data | Item definitions, enemy stats, View layouts, danmaku performances |
| **Mortar**    | Scripted sequences   | Dialogue trees, cutscenes, event chains                           |
| **FRE rules** | Game logic           | State transitions, conditional behaviors, damage formulas         |
| **WASM**      | Custom code          | Exotic bullet patterns, special boss mechanics                    |
| **Assets**    | Media files          | Sprites, audio, tilemaps, Alight Motion projects                  |

---

## WASM Extension Model

WASM is the **extension point for mod authors** — not a replacement for Rust.

| Use WASM for            | Keep in Rust           |
|-------------------------|------------------------|
| Custom bullet behaviors | Core motion primitives |
| Exotic spawn patterns   | Collision detection    |
| Mod-specific game logic | Rendering & UI layout  |
| Special boss mechanics  | FRE rule evaluation    |

**WIT interfaces** define the contract between engine and mods:
`behavior`, `danmaku`, `spawn-pattern`, `custom-action-handler`,
`mode-lifecycle`, `rule-provider`

**Performance note**: WASM has serialization overhead at the boundary.
Hot-path code (bullet updates × thousands of bullets × 60fps) should stay in Rust.

---

## Boundary Rules

These are the architectural invariants that keep SoupRune maintainable:

| ✅ Core may                              | ❌ Core must not                         |
|-----------------------------------------|-----------------------------------------|
| Define danmaku motion primitives        | Import from `preset/`                   |
| Define collision shapes and events      | Hardcode game-specific fact keys        |
| Define dialogue rendering and Mortar VM | Know about Items, Enemies, or BattleBox |
| Define View layout and reactive updates | Define game state machines              |
| Define FRE rule evaluation              | Register game-specific Mortar functions |
| Define generic scheduling primitives    | Contain UT/DR-specific vocabulary       |

---

## Crate Map

```
crates/
├── souprune/                     # Main framework crate
│   └── src/
│       ├── core/                 # Tier 1: Engine infrastructure
│       │   ├── danmaku/          #   ★ STG bullet engine (privileged)
│       │   ├── dialogue/         #   Dialogue UI & Mortar integration
│       │   ├── view/             #   RON-driven declarative UI
│       │   │   └── ron_view/
│       │   │       └── player_data.rs  # Resolver registries
│       │   ├── collision.rs      #   SDF collision detection
│       │   ├── fre_bridge.rs     #   FRE ↔ ECS bridge
│       │   ├── fre_facts.rs      #   Core fact key constants
│       │   ├── mod_system.rs     #   WASM mod loading & registries
│       │   └── sequencer.rs      #   Chapter-based game flow
│       ├── preset/               # Tier 2: UT/DR game logic
│       │   ├── battle/           #   Battle state machine
│       │   ├── overworld/        #   Overworld exploration
│       │   ├── item.rs           #   Item registry & data
│       │   ├── item_actions.rs   #   Item FRE action handlers
│       │   └── enemy.rs          #   Enemy registry & data
│       └── app_state/            # Application state management
│
├── bevy_fact_rule_event/         # FRE engine (git submodule)
├── bevy_mortar_bond/             # Mortar scripting (git submodule)
├── bevy_ecs_typewriter/          # Typewriter text effect (git submodule)
├── bevy_alight_motion/           # Alight Motion + SDF rendering (git submodule)
│
├── souprune_api/                 # WIT interface definitions
├── souprune_sdk/                 # Rust WASM guest SDK
├── souprune_mod_test/            # Example WASM mod
└── souprune_mock_host/           # Standalone WASM test host
```
