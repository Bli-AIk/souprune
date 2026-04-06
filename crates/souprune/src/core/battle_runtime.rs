//! Battle runtime markers and system sets used by core infrastructure.
//!
//! battle 模式的运行时标记和系统集，供 core 基础设施使用。
//!
//! These are lightweight marker components and system sets with no game logic.
//! Core modules (mod_system, sequencer) depend on these for scheduling.
//!
//! 这些是无游戏逻辑的轻量级标记组件和系统集。
//! Core 模块（mod_system、sequencer）依赖它们进行调度。

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
