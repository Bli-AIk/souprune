use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ColliderShape {
    Circle { radius: f32 },
    Box { half_size: Vec2 },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ColliderConfig {
    pub shape: ColliderShape,
    /// Z-offset for rendering the collider visualization
    pub debug_z_offset: f32,
}

#[derive(Asset, TypePath, Debug, Clone, Deserialize, Serialize)]
pub struct BattlePlayerConfig {
    pub sprite_path: String,
    pub color: Color,
    /// Physical collider for collision detection (typically circular)
    ///
    /// 用于碰撞检测的物理碰撞体（通常为圆形）
    pub physics_collider: ColliderConfig,
    /// Trigger collider for damage detection (typically rectangular)
    ///
    /// 用于伤害检测的触发碰撞体（通常为矩形）
    pub damage_trigger: ColliderConfig,
    /// Z-axis position for the player sprite
    ///
    /// 玩家精灵的 Z 轴位置
    pub z_position: f32,
    pub default_mode_id: String,
    pub speed: f32,
    pub focus_speed_ratio: f32,
}
