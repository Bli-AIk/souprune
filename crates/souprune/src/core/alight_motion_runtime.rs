//! Shared Alight Motion runtime state and events.
//!
//! 供 core sequencer 与 battle 集成共同使用的 Alight Motion 运行时类型。

use bevy::prelude::*;

/// Resource to track active Alight Motion performance state.
#[derive(Resource, Default)]
pub struct AlightMotionPerformanceState {
    pub is_playing: bool,
    pub total_duration_ms: f32,
    pub project_entity: Option<Entity>,
    pub final_scale: f32,
}

/// Event to request starting an Alight Motion performance.
#[derive(bevy::ecs::message::Message, Debug, Clone)]
pub struct PlayAlightMotionPerformanceEvent {
    pub amproj_path: String,
    pub alight_motion_config_path: Option<String>,
    pub wait_for_completion: bool,
}

impl PlayAlightMotionPerformanceEvent {
    pub fn new(amproj_path: String) -> Self {
        Self {
            amproj_path,
            alight_motion_config_path: None,
            wait_for_completion: true,
        }
    }

    pub fn with_config(amproj_path: String, alight_motion_config_path: Option<String>) -> Self {
        Self {
            amproj_path,
            alight_motion_config_path,
            wait_for_completion: true,
        }
    }
}
