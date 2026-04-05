//! Shared battle runtime markers and system sets.
//!
//! battle 逻辑和 core 基础设施都会用到的运行时标签与系统集。
//!
//! **为何保留在 core/**:
//! - `core/mod_system.rs` 依赖 `BattleInputManager` 和 `BattleMovementSet`
//!   来调度 mod 行为系统的运行顺序。
//! - 移至 preset 会导致 core ↔ preset 循环依赖。
//! - 这些类型是轻量级标记/系统集，无游戏逻辑，留在 core 是合理的。

use bevy::prelude::*;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct BattleUpdate;

/// System set for battle movement (mod behaviors).
/// Collision systems should run after this.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct BattleMovementSet;

#[derive(Component)]
pub struct BattleCamera;

/// Marker component for the battle input manager entity.
/// This entity holds the battle mode `ActionState<Action>`.
#[derive(Component)]
pub struct BattleInputManager;
