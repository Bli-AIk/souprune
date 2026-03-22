# SoupRune Code Style Guide

> This document defines the code style and documentation conventions for the SoupRune project.
> For the Simplified Chinese version, see [style_zh-hans.md](style_zh-hans.md).

---

## 1. Naming Conventions

### 1.1 General Rules

| Item                        | Case                         | Example                             |
|-----------------------------|------------------------------|-------------------------------------|
| Modules & files             | `snake_case`                 | `fre_bridge`, `chapter_schema`      |
| Types (struct, enum, trait) | `UpperCamelCase`             | `FactDatabase`, `ViewRoot`          |
| Functions & methods         | `snake_case`                 | `evaluate_conditions`, `get_by_str` |
| Constants & statics         | `SCREAMING_SNAKE_CASE`       | `MAX_ENEMIES`, `DEFAULT_SPEED`      |
| Feature flags               | `lowercase`                  | `debug`, `unsafe_gpu`               |
| Asset directories           | `lowercase_with_underscores` | `battle_sprites/`                   |
| ECS components & resources  | `UpperCamelCase`             | `PlayerHealth`, `EnumRegistry`      |

Follow the [Rust API Guidelines — Naming](https://rust-lang.github.io/api-guidelines/naming.html) as a baseline.

### 1.2 Public API: Prefer Full Words

SoupRune targets the fangame community — many contributors are hobbyists, not systems programmers. **Public API names
should use full, descriptive words.**

```rust
// ✅ Good
pub struct HealthBarSource {
    ..
}
pub fn evaluate_conditions(..) -> bool { .. }
pub enum AlightMotionEntity {..}

// ❌ Avoid
pub struct HpBarSrc {
    ..
}
pub fn eval_conds(..) -> bool { .. }
pub enum AmEntity {..}
```

Private/local code has more flexibility — short names are fine when context is clear.

### 1.3 Consistency Within a Concept

Pick **one** spelling and use it everywhere. Do not mix cases or abbreviations for the same concept:

```rust
// ❌ Inconsistent
HPBarSourceDef   // "HP" uppercase
HpSourceType     // "Hp" mixed case
DesiredHpBar     // "Hp" mixed case again

// ✅ Consistent — pick one and stick with it
HealthBarSourceDef
HealthBarSourceType
DesiredHealthBar
```

---

## 2. Abbreviations and Terminology

### 2.1 Allowed Abbreviations

These abbreviations are well-known in the Rust ecosystem and are acceptable everywhere:

| Abbreviation                     | Meaning                      |
|----------------------------------|------------------------------|
| `ctx`                            | Context                      |
| `db`                             | Database                     |
| `buf`                            | Buffer                       |
| `iter`                           | Iterator                     |
| `impl`                           | Implementation               |
| `config` / `cfg` (in attributes) | Configuration                |
| `err`                            | Error                        |
| `msg`                            | Message                      |
| `id`                             | Identifier                   |
| `idx`                            | Index (local variables only) |

### 2.2 Project-Specific Terms

These project-specific abbreviations are allowed because they are documented and consistently used:

| Term | Full Form               | Usage                                            |
|------|-------------------------|--------------------------------------------------|
| FRE  | Fact-Rule-Event         | System name, crate name (`bevy_fact_rule_event`) |
| SDF  | Signed Distance Field   | Rendering technique (from `bevy_alight_motion`)  |
| ECS  | Entity Component System | Bevy's architecture                              |
| RON  | Rusty Object Notation   | Data format                                      |

### 2.3 Prohibited in Public API

Avoid these in **public types, traits, and function names**. Use full words instead:

| Avoid                               | Use Instead                                  |
|-------------------------------------|----------------------------------------------|
| `Mgr`                               | `Manager`                                    |
| `Svc`                               | `Service`                                    |
| `Util` / `Helper`                   | Describe the actual purpose                  |
| `Misc`                              | Describe the actual purpose                  |
| `Info`                              | Be specific: `Metadata`, `Status`, `Details` |
| Single-letter prefixes (`Am`, `Hp`) | Full words (`AlightMotion`, `Health`)        |

### 2.4 Def Suffix Convention

`Def` (short for "Definition") is used throughout the codebase for **data-only structs deserialized from RON files**.
This convention is established and should be maintained:

```rust
pub struct RuleDef {
    ..
}       // deserialized from .fre.ron
pub struct ViewNodeDef {
    ..
}   // deserialized from .view.ron
pub struct EnemyDef {
    ..
}      // deserialized from enemy config
```

> ⚠️ **Important**: In RPG/fangame contexts, "def" is also commonly used as an abbreviation for "defense" (a combat
> stat). To avoid ambiguity, **always spell out "defense" in full** when referring to the combat stat. The `Def` suffix
> is
> reserved exclusively for "Definition" structs.
>
> ```rust
> // ✅ Correct
> pub struct EnemyDefence { pub value: i32 }  // "Defence" spelled out in full
>
> // ❌ Confusing
> pub struct EnemyDef { pub def: i32 }        // "def" field clashes with "Def" suffix
> ```

---

## 3. Module and Directory Organization

### 3.1 Directory Structure

```
crates/souprune/src/
├── core/           # Engine-level systems (input, view, camera, collision, ...)
├── app_state/      # Game state modules (menu, overworld, battle)
├── extra/          # Optional utilities (debug tools, format loaders)
└── lib.rs          # Plugin registration, app builder
```

### 3.2 When to Split a Module

- Once a file is getting close to **~500 total lines**, stop and ask whether it is still doing one job
- **Hard limit: 800 total lines** (enforced by `tokei_check.sh`) — files exceeding this **must** be split
- **Hard limit: 500 lines of code via `tokei`** (also enforced by `tokei_check.sh`) — dense logic files must be split even if comments or blank lines keep the total size lower
- `examples/` are **not** blocked by the 800/500 gate. They are demo binaries, not production modules. Keep them readable, but do not use examples as a place to hide product complexity
- A module has **distinct responsibilities** → split by responsibility

### 3.3 Module Style: Rust 2018+

**Do not use `mod.rs` files.** Use the Rust 2018+ module naming convention:

```
// ❌ Old style (pre-2018)
src/core/view/mod.rs

// ✅ New style (Rust 2018+)
src/core/view.rs          // module root
src/core/view/            // submodules directory
```

The only exception is `examples/` directories, where `mod.rs` is acceptable because Cargo treats `.rs` files in examples
as binaries.

This rule is enforced by `tokei_check.sh`.

### 3.4 File Naming

- One module per file (or directory)
- File name matches module name: `mod fre_bridge` → `fre_bridge.rs`, and if it has children, place them under `fre_bridge/`
- Test modules live alongside their source: `#[cfg(test)] mod tests { .. }` at the bottom of the file

### 3.5 Single Responsibility and Boundary Ownership

**Single Responsibility Principle is mandatory.**

- A module should feel like it does **one job**. If you see schema definitions, asset loading, runtime systems, and editor helpers all living together, the module is already too broad
- Plugin root files are wiring files. Use them to register plugins, systems, and resources, then move the real behavior into dedicated submodules
- If a file keeps growing because unrelated changes keep landing there, split it by responsibility before line count becomes the only warning sign

### 3.6 Layering Rules

Directory structure is not enough; dependency direction must also match the architecture.

- The directory name alone does not make something `core/`. If a module needs to know about battle, overworld, or the editor, it probably does not belong in `core/`
- A simple rule: `app_state/` may depend on `core/`, but `core/` should not reach back up into `app_state/`
- Editor crates should talk to **public engine APIs** or **schema crates**, not reach through deep internal paths just because it is convenient today
- State-specific behavior belongs in the relevant game state or a dedicated shared gameplay layer, not in generic infrastructure modules

### 3.7 Schema Source of Truth

One asset format must have exactly **one authoritative schema type**.

- Pick one schema and treat it as the source of truth
- Do not keep one version for runtime, another for linting, and a third for the editor
- Thin runtime wrappers are fine when they add `Asset`, `Reflect`, or conversion behavior; duplicated field definitions are not

### 3.8 Imports and Re-exports for Internal Code

- Make dependencies obvious when reading the file
- Avoid blanket imports in production code except for ecosystem-standard preludes such as `bevy::prelude::*`
- Avoid barrel-style re-exports (`pub use foo::*;`) inside internal modules; prefer explicit re-exports so readers can see what the module actually exposes
- Re-export entire modules only at intentional public boundaries such as a crate `prelude` or a stable top-level API

### 3.9 Compatibility and Deletion Policy

SoupRune is in active development and does **not** prioritize backward compatibility.

- Use that freedom to delete old designs instead of carrying them forever
- When you add temporary compatibility code, also write down when it dies
- If a migration is finished, remove the old path instead of leaving two systems half-alive
- If assets or rules have already moved to the new format, delete the old field names, old event names, and old bridge systems in the same phase
- "We will clean it up later" is not an acceptable reason to keep dead abstractions or parallel systems

### 3.10 Bevy Plugin Shape

The following Bevy-specific rules follow the direction of Bevy's plugin development guide and the `bevy_best_practices` project, but are written here in plain language for this repository.

If a module is called a Bevy plugin, readers should be able to treat it like the front door of one subsystem.

- A plugin file should read like wiring: register plugins, resources, assets, schedules, and state hooks, then stop
- Do not let `Plugin::build` turn into the place where real gameplay logic, parser logic, or runtime branching quietly accumulates
- In library crates, prefer exposing a named `Plugin` struct instead of only a free `plugin(app)` function. That keeps room for future configuration without breaking callers
- Third-party setup should live beside the subsystem that uses it. If a feature depends on an external Bevy crate, its configuration belongs in that subsystem's plugin, not in some distant global bootstrap

In plain terms: when someone opens a plugin file, they should quickly understand **what this subsystem registers**, not reverse-engineer **how the subsystem works**.

### 3.11 State Boundaries, Scheduling, and Cleanup

Bevy makes it easy to dump everything into `Update`. Do not do that.

- Systems in `Update` should usually be gated by `State`, `run_if`, or a clearly named `SystemSet`
- If a system should only matter while a screen, mode, or battle state is active, say that directly in scheduling instead of relying on "it probably won't do anything"
- Register `OnEnter`, `Update`, and `OnExit` for the same state near each other. A reader should be able to see setup and cleanup in one place
- Entities need an obvious cleanup story. Use `StateScoped` or an explicit cleanup marker; do not spawn long-lived scene entities and hope someone remembers to remove them later
- Top-level spawned entities should usually have a `Name`, because unnamed roots make debugging and world inspection worse for no benefit

The human rule is simple: if a state starts something, the code nearby should also make it obvious how that thing stops.

### 3.12 Events Over Tight Coupling

When two parts of the game only need to notify each other, prefer events over direct reach-through.

- Use events to pass facts, requests, and results between subsystems instead of letting one system poke deeply into another subsystem's internals
- If an event writer and reader are supposed to cooperate in the same frame, make the ordering explicit with `.before(...)`, `.after(...)`, `.chain()`, or ordered `SystemSet`s
- If a system only exists to react to an event, make that visible in scheduling with an event-based run condition instead of running it every frame for no reason
- Event names and payloads should describe domain meaning, not temporary implementation details

Put more plainly: events are the cheap and honest way to connect systems. Hidden cross-module assumptions are not.

---

## 4. Public API Design

### 4.1 Function Naming

Use **verb + noun** pattern for functions that perform actions:

```rust
pub fn evaluate_conditions(..) -> bool { .. }
pub fn register_rules(..) { .. }
pub fn spawn_view_entity(..) -> Entity { .. }
```

Use **noun** pattern for pure getters:

```rust
pub fn active_view(&self) -> Option<Entity> { .. }
pub fn fact_count(&self) -> usize { .. }
```

### 4.2 Trait Naming

- Capability traits: use adjective or `-able` suffix → `FactReader`, `Serializable`
- Behavior traits: use verb form → `Evaluate`, `Dispatch`

### 4.3 Avoid Vague Names

Do not create modules or types named `util`, `helper`, `common`, or `misc`. Name them after what they actually do.

---

## 5. Documentation Comments

### 5.1 Bilingual Documentation

All documentation must be **bilingual (English + Chinese)**. English first, then Chinese translation:

```rust
/// Evaluates all conditions in a rule against the current facts.
///
/// 根据当前 facts 评估规则中的所有条件。
pub fn evaluate_conditions(..) -> bool { .. }
```

For module-level docs:

```rust
//! # fre_bridge
//!
//! ## Module Overview
//!
//! Bridge between the game and the FRE (Fact-Rule-Event) system.
//!
//! ## 模块概述
//!
//! 游戏与 FRE（Fact-Rule-Event）系统之间的桥接层。
```

### 5.2 Doc Structure

For complex items, use this structure:

1. **One-line summary** (English)
2. **One-line summary** (Chinese)
3. Extended explanation (if needed, bilingual)
4. `# Examples` section (if applicable)

Keep docs **concise** — avoid restating what the code already makes obvious. Link to external resources rather than
explaining common concepts inline.

### 5.3 Inline Comments

Use inline comments sparingly. When used, bilingual is preferred for non-trivial logic:

```rust
// Enum resolution: Int vs String → try enum lookup
// 枚举解析：Int vs String → 尝试枚举查找
```

---

## 6. Code Style

### 6.1 Formatter

Use `rustfmt` with default settings. No `rustfmt.toml` overrides unless agreed upon by the team.

Run before every commit:

```bash
cargo fmt --all
```

### 6.2 Clippy

Clippy configuration (`clippy.toml`):

```toml
cognitive-complexity-threshold = 20
excessive-nesting-threshold = 4
too-many-lines-threshold = 120
too-many-arguments-threshold = 20
type-complexity-threshold = 1000
```

**`#[allow(clippy::...)]` is banned.** Use `#[expect(clippy::...)]` with a mandatory `// reason:` comment instead. This
rule is enforced by `tokei_check.sh`.

```rust
// ❌ Banned
#[allow(clippy::too_many_arguments)]
fn complex_function(..) { .. }

// ✅ Acceptable (only with justified reason)
#[expect(clippy::too_many_arguments)]
// reason: Bevy system parameters require all these arguments; refactoring to a SystemParam is tracked in #123
fn complex_system(..) { .. }
```

Even `#[expect]` should not be overused — it is a last resort. Prefer fixing the underlying issue:

- Excessive nesting → extract helper functions, use `let-else`, use iterator methods
- Too many arguments → use a config struct or builder pattern

### 6.3 Import Ordering

Group imports in this order, separated by blank lines:

1. `mod` declarations and `pub use` re-exports
2. Standard library (`std::`)
3. External crates (`bevy::`, `serde::`, etc.)
4. Internal crate (`crate::`, `super::`)

```rust
mod eval;
pub use eval::register_condition_evaluator_system;

use std::collections::HashMap;

use bevy::prelude::*;
use bevy_fact_rule_event::{EnumRegistry, FactReader};

use crate::core::view::components::ViewRoot;
```

### 6.4 Nesting

Maximum nesting depth is **4 levels**. Flatten with:

- `let-else` bindings
- Early returns / guard clauses
- `if-let` chains (`if let Some(x) = a && let Some(y) = b { .. }`)
- Extracting helper functions

```rust
// ❌ Too nested
if let Some(a) = get_a() {
if let Some(b) = get_b() {
if condition {
do_something(a, b);
}
}
}

// ✅ Flat
let Some(a) = get_a() else { return };
let Some(b) = get_b() else { return };
if condition {
do_something(a, b);
}
```

### 6.5 Match Arms

Use consistent formatting. Short arms can stay on one line; complex arms should use blocks:

```rust
match value {
FactValue::Int(v) => FactValue::Int( * v),
FactValue::String(v) => FactValue::String(v.clone()),
FactValue::Bool(v) => {
info ! ("Processing bool: {}", v);
FactValue::Bool( * v)
}
}
```

---

## 7. Error Handling

### 7.1 When to Use Result vs Option

- `Result<T, E>` — operation can fail with a meaningful error
- `Option<T>` — value may or may not exist (no error semantics)

### 7.2 Error Types

For crate-level errors, define a dedicated error enum:

```rust
#[derive(Debug, thiserror::Error)]
pub enum FreError {
    #[error("Unknown enum variant '{variant}' for group '{group}'")]
    UnknownEnumVariant { group: String, variant: String },
}
```

### 7.3 Panics

- **Library code**: avoid panics. Return `Result` or `Option`.
- **Application code**: panics are acceptable for truly unrecoverable states.
- **Asset loading**: use `warn!` + sensible defaults rather than panicking on malformed data.

---

## 8. Logging and Diagnostics

### 8.1 Log Levels

| Level    | Usage                                                      |
|----------|------------------------------------------------------------|
| `error!` | Unrecoverable failures that break functionality            |
| `warn!`  | Unexpected but recoverable situations                      |
| `info!`  | Significant state changes (scene transitions, asset loads) |
| `debug!` | Development diagnostics (rule evaluation details)          |
| `trace!` | Very fine-grained data (per-frame values)                  |

### 8.2 Rules

- **Never** use `println!` or `eprintln!` in library code. Use `bevy::log` macros.
- Include context in log messages: `info!("FRE Bridge: SetLocalFact({}, {:?})", key, value)`
- Gate verbose logging behind `#[cfg(feature = "debug")]` or `debug!`/`trace!` levels.

---

## 9. Testing

### 9.1 Test Location

- Unit tests: `#[cfg(test)] mod tests { .. }` at the bottom of the source file
- Integration tests: `tests/` directory at the crate root (if needed)

### 9.2 Test Naming

Use descriptive names that state the behavior being tested:

```rust
#[test]
fn enum_registry_resolves_known_variant() { .. }

#[test]
fn evaluate_conditions_returns_false_when_fact_missing() { .. }
```

### 9.3 Test Structure

- Use minimal `bevy::prelude::App` setups for ECS tests
- Prefer **deterministic assertions** — avoid frame-count dependencies
- Test one behavior per test function

### 9.4 Workspace Quality Gates

Style rules must be enforceable across the workspace, not only in the main crate.

- A rule only matters if the repository actually checks it
- Line-count checks, lint checks, and structural checks should cover all maintained first-party crates
- If a subcrate has its own quality script, the root quality entrypoint should invoke it instead of silently skipping that crate
- A rule that cannot be checked in CI should be treated as advisory, not mandatory

---

## 10. Documentation Language Policy

### 10.1 Two Canonical Documents

| File                   | Language           | Purpose             |
|------------------------|--------------------|---------------------|
| `doc/style.md`         | English            | Primary style guide |
| `doc/style_zh-hans.md` | Simplified Chinese | Chinese translation |

Both documents must maintain **identical structure** (same sections, same numbering). When updating one, update the
other.

> ⚠️ There are currently no dedicated localization maintainers. Therefore, **all documentation changes must update every
language version simultaneously**. This policy may change in the future as the team grows.

### 10.2 Code Comments and Docs

- Source code documentation (`///`, `//!`) is **bilingual** — English first, Chinese second
- Inline comments (`//`) are bilingual for non-trivial logic
- Commit messages are in **English** (conventional commit format)
- RON file comments (`//`) may be in either language

### 10.3 README

- `readme.md` — English
- `readme_zh-hans.md` — Simplified Chinese
