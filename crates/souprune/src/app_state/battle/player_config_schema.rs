//! # player_config_schema.rs
//!
//! # player_config_schema.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Defines the schema for Battle Player configuration in RON files.
//! Maps to `battle_player.ron`.
//!
//! 定义 Battle Player 在 RON 文件中的配置 Schema。
//! 映射到 `battle_player.ron`。

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

/// Configuration for player invincibility behavior after taking damage.
///
/// 玩家受伤后的无敌行为配置。
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InvincibilityConfig {
    /// Duration of invincibility in seconds after taking damage
    ///
    /// 受伤后无敌持续时间（秒）
    #[serde(default = "default_invincibility_duration")]
    pub duration: f32,

    /// Interval for heart color flash during invincibility (in seconds)
    ///
    /// 无敌期间心形颜色闪烁间隔（秒）
    #[serde(default = "default_flash_interval")]
    pub flash_interval: f32,

    /// Normal heart color (pure red by default)
    ///
    /// 正常心形颜色（默认纯红色）
    #[serde(default = "default_normal_color")]
    pub normal_color: Color,

    /// Flash heart color (dark red by default)
    ///
    /// 闪烁心形颜色（默认暗红色）
    #[serde(default = "default_flash_color")]
    pub flash_color: Color,

    /// Sound to play when taking damage (full asset path)
    ///
    /// 受伤时播放的音效（完整资源路径）
    #[serde(default)]
    pub damage_sound: Option<String>,
}

fn default_invincibility_duration() -> f32 {
    1.0
}
fn default_flash_interval() -> f32 {
    0.25
}
fn default_box_id() -> String {
    "main".to_string()
}
fn default_normal_color() -> Color {
    Color::srgb(1.0, 0.0, 0.0)
}
fn default_flash_color() -> Color {
    Color::srgb(0.5, 0.0, 0.0)
}

impl Default for InvincibilityConfig {
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
    /// ID of the battle box this player is initially bound to.
    /// Defaults to "main".
    ///
    /// 此玩家初始绑定的战斗框 ID。
    /// 默认为 "main"。
    #[serde(default = "default_box_id")]
    pub default_box: String,
    /// Invincibility configuration for damage behavior.
    /// If not specified, defaults are used.
    ///
    /// 伤害行为的无敌配置。
    /// 如果未指定，则使用默认值。
    #[serde(default)]
    pub invincibility: InvincibilityConfig,
}
