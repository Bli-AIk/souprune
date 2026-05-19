//!
//! # Alight Motion 动画集成模块
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This module integrates Alight Motion animations into the fixed-scene primitive.
//! It handles loading Alight Motion projects, spawning entities, and adding collision
//! components based on layer naming conventions.
//!
//! 此模块将 Alight Motion 动画集成到fixed-scene primitive中。
//! 它处理加载 Alight Motion 项目、生成实体，以及根据图层命名约定添加碰撞组件。
//!
//! ## Layer Naming Conventions / 图层命名约定
//!
//! - Layers matching `bullet_pattern` (default: `^#B`): Bullets with collision
//!   匹配 `bullet_pattern` 的图层（默认：`^#B`）：带碰撞的弹幕
//!
//! - Layers matching `boundary_pattern` (default: `^#C`): collision boundaries
//!   匹配 `boundary_pattern` 的图层（默认：`^#C`）：碰撞边界
//!
//! - If a group layer matches, all children inherit the same behavior
//!   如果编组图层匹配，所有子元素继承相同行为

mod collision;
mod config_loading;
mod markers;
mod playback;

use bevy::prelude::*;
use regex::Regex;

use crate::core::alight_motion_runtime::{
    AlightMotionPerformanceState, PlayAlightMotionPerformanceEvent,
};

/// Marker component for Alight Motion performance entities.
/// Used to identify and clean up Alight Motion-generated entities。
#[derive(Component, Debug, Clone, Default)]
pub struct AlightMotionEntity;

/// Marker for entities that should be treated as bullets (from #B group).
#[derive(Component, Debug, Clone, Default)]
pub struct AlightMotionBulletMarker;

/// Marker for entities that should be treated as collision boundaries (from #C group).
#[derive(Component, Debug, Clone, Default)]
pub struct AlightMotionBoundaryMarker;

/// Marker for entities that should be hidden (based on hidden_pattern config).
#[derive(Component, Debug, Clone, Default)]
pub struct AlightMotionHiddenMarker;

/// Configuration for Alight Motion fixed-scene integration.
#[derive(Resource, Debug, Clone, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct AlightMotionSceneConfig {
    /// Scale multiplier for Alight Motion project.
    pub scale: f32,
    /// Offset position for Alight Motion project (x, y).
    pub offset: (f32, f32),
    /// Regex pattern for bullet layers (default: "^#B").
    pub bullet_pattern: String,
    /// Regex pattern for collision boundary layers (default: "^#C").
    pub boundary_pattern: String,
    /// Regex pattern for layers that should be hidden.
    pub hidden_pattern: String,
    /// Damage dealt by bullets.
    pub bullet_damage: f32,
    /// Scale factor for bullet collision boxes relative to sprite size.
    pub collision_scale: f32,
    /// Default boundary size (width, height) when size cannot be determined from AM layer.
    #[serde(default = "default_boundary_size")]
    pub default_boundary_size: (f32, f32),
}

fn default_boundary_size() -> (f32, f32) {
    (565.0, 140.0)
}

impl Default for AlightMotionSceneConfig {
    fn default() -> Self {
        Self {
            scale: 1.0,
            offset: (0.0, 0.0),
            bullet_pattern: "^#B".to_string(),
            boundary_pattern: "^#C".to_string(),
            hidden_pattern: String::new(),
            bullet_damage: 1.0,
            collision_scale: 0.05,
            default_boundary_size: default_boundary_size(),
        }
    }
}

/// Compiled regex patterns for runtime matching.
#[derive(Resource)]
pub struct AlightMotionScenePatterns {
    pub bullet_regex: Option<Regex>,
    pub boundary_regex: Option<Regex>,
    pub hidden_regex: Option<Regex>,
}

/// Plugin for Alight Motion fixed-scene integration.
pub struct AlightMotionFixedScenePlugin;

impl Plugin for AlightMotionFixedScenePlugin {
    fn build(&self, app: &mut App) {
        let schedule = crate::game_schedule(app);
        app.init_resource::<AlightMotionPerformanceState>()
            .init_resource::<AlightMotionSceneConfig>()
            .add_message::<PlayAlightMotionPerformanceEvent>()
            .add_systems(
                schedule,
                config_loading::load_am_scene_config.run_if(super::on_entering_fixed_scene),
            )
            .add_systems(
                schedule,
                (
                    playback::handle_play_am_performance_event,
                    collision::sync_am_fit_scale_system,
                    ApplyDeferred,
                    markers::propagate_am_markers_system,
                    ApplyDeferred,
                    collision::add_am_collision_system,
                    markers::apply_am_hidden_visibility,
                    playback::check_am_performance_completion,
                )
                    .chain()
                    .in_set(crate::core::fixed_scene::FixedSceneUpdate),
            )
            .add_systems(
                schedule,
                playback::cleanup_am_entities.run_if(super::on_exiting_fixed_scene),
            );
    }
}
