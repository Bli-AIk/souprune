//! Battle runtime markers and system sets used by core infrastructure.
//!
//! battle 模式的运行时标记和系统集，供 core 基础设施使用。
//!
//! These are lightweight zero-logic marker types that core modules (mod_system,
//! sequencer) depend on for battle-mode scheduling. They live in core/ rather
//! than preset/ because moving them would create a circular dependency:
//! mod_system needs scheduling references, and preset depends on mod_system.
//!
//! 这些是无游戏逻辑的轻量级标记类型。Core 模块（mod_system、sequencer）
//! 依赖它们进行 battle 模式调度。它们位于 core/ 而非 preset/ 是因为移动
//! 它们会导致循环依赖：mod_system 需要调度引用，而 preset 依赖 mod_system。

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
