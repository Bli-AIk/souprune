//! # battle.rs
//!
//! BattlePlayerConfig schema types for `.battle_player.ron` files.
//! Mirrors `souprune::app_state::battle::player_config_schema` without Bevy dependency.
//!
//! `.battle_player.ron` 文件的战斗玩家配置 Schema 类型。

use crate::bevy_types::BevyColor;
use serde::{Deserialize, Serialize};

/// Collider shape for battle entities.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum BattleColliderShape {
    Circle { radius: f32 },
    Box { half_size: (f32, f32) },
}

/// Collider configuration with debug visualization offset.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ColliderConfig {
    pub shape: BattleColliderShape,
    pub debug_z_offset: f32,
}

/// Invincibility configuration for battle damage behavior.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BattleInvincibilityConfig {
    #[serde(default = "default_invincibility_duration")]
    pub duration: f32,
    #[serde(default = "default_flash_interval")]
    pub flash_interval: f32,
    #[serde(default = "default_normal_color")]
    pub normal_color: BevyColor,
    #[serde(default = "default_flash_color")]
    pub flash_color: BevyColor,
    #[serde(default)]
    pub damage_sound: Option<String>,
}

impl Default for BattleInvincibilityConfig {
    fn default() -> Self {
        Self {
            duration: default_invincibility_duration(),
            flash_interval: default_flash_interval(),
            normal_color: default_normal_color(),
            flash_color: default_flash_color(),
            damage_sound: None,
        }
    }
}

/// Battle player configuration — top-level `.battle_player.ron` schema.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BattlePlayerConfig {
    pub sprite_path: String,
    pub color: BevyColor,
    pub physics_collider: ColliderConfig,
    pub damage_trigger: ColliderConfig,
    pub z_position: f32,
    pub default_mode_id: String,
    pub speed: f32,
    pub focus_speed_ratio: f32,
    #[serde(default)]
    pub invincibility: BattleInvincibilityConfig,
}

// ============================================================================
// Default helpers
// ============================================================================

fn default_invincibility_duration() -> f32 {
    1.0
}

fn default_flash_interval() -> f32 {
    0.25
}

fn default_normal_color() -> BevyColor {
    BevyColor::Srgba(crate::bevy_types::SrgbaColor {
        red: 1.0,
        green: 0.0,
        blue: 0.0,
        alpha: 1.0,
    })
}

fn default_flash_color() -> BevyColor {
    BevyColor::Srgba(crate::bevy_types::SrgbaColor {
        red: 0.5,
        green: 0.0,
        blue: 0.0,
        alpha: 1.0,
    })
}
