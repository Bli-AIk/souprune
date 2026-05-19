//! Fixed-scene primitive plugin, markers, and system sets.
//!
//! fixed-scene primitive 插件、标记和系统集。
//!
//! This module owns fixed-camera scene primitives: camera/input setup,
//! sequencer wiring, danmaku integration, and FRE bridge scheduling. Project
//! modes decide whether these primitives are active.
//!
//! 本模块持有固定相机场景 primitive：相机/输入初始化、sequencer 接线、
//! 弹幕集成与 FRE bridge 调度。项目 mode 决定是否启用这些 primitive。

pub mod alight_motion_integration;
pub mod danmaku;
pub mod fre;
pub mod menu_state;
mod plugin;
pub mod speech_bubble;

use bevy::prelude::*;

pub use plugin::FixedScenePlugin;
pub(crate) use plugin::{fixed_scene_scoped, on_entering_fixed_scene, on_exiting_fixed_scene};

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct FixedSceneUpdate;

/// System set for battle movement (mod behaviors).
/// Collision systems should run after this.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct FixedSceneMovementSet;

#[derive(Component)]
pub struct FixedSceneCamera;

/// Marker component for the fixed-scene input manager entity.
/// This entity holds the battle mode `ActionState<Action>`.
#[derive(Component)]
pub struct FixedSceneInputManager;
