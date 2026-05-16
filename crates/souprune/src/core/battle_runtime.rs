//! Battle mode runtime plugin, markers, and system sets.
//!
//! battle 模式运行时插件、标记和系统集。
//!
//! This module owns the generic battle-mode host runtime: camera/input setup,
//! sequencer wiring, danmaku integration, and FRE bridge scheduling. Project
//! project-specific battle area commands, player spawning, item use, and
//! enemy-turn selection live in project WASM runtimes instead.
//!
//! 本模块持有通用 battle 模式宿主运行时：相机/输入初始化、sequencer 接线、
//! 弹幕集成与 FRE bridge 调度。项目特化的战斗区域命令、战斗玩家生成、物品使用、
//! 敌人回合选择位于 project WASM runtime 中。

pub mod alight_motion_integration;
pub mod danmaku;
pub mod fre;
pub mod menu_state;
mod plugin;
pub mod speech_bubble;

use bevy::prelude::*;

pub use plugin::BattlePlugin;
pub(crate) use plugin::{battle_scoped, on_entering_battle, on_exiting_battle};

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
