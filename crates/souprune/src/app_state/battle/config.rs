use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Asset, TypePath, Debug, Clone, Deserialize, Serialize)]
pub struct BattlePlayerConfig {
    pub sprite_path: String,
    pub color: Color,
    pub collider_size: Vec2,
    pub default_mode_id: String,
    pub speed: f32,
    pub focus_speed_ratio: f32,
}
