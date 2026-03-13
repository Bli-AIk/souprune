//! # character.rs
//!
//! CharacterAsset and AnimationConfigAsset schema types for `.character.ron` files.
//! Mirrors `souprune::core::character_asset` without Bevy dependency.
//!
//! `.character.ron` 文件的角色资产 Schema 类型。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Vec2 serialized as `{ x: f32, y: f32 }` (matches `#[serde(with = "vec2_xy")]`).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Vec2XY {
    pub x: f32,
    pub y: f32,
}

/// Character asset — one form of `.character.ron` schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterAsset {
    pub name: String,
    pub collider_size: Vec2XY,
    pub collider_offset: Vec2XY,
    pub base_speed: f32,
    pub animation_config: String,
    #[serde(default)]
    pub interaction_script: Option<String>,
}

/// Defines how a state maps to directional animations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StateAnimationMapping {
    Directional {
        up: String,
        down: String,
        left: String,
        right: String,
    },
    Single(String),
}

/// Animation configuration asset — another form of `.character.ron` schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationConfigAsset {
    pub sprite_source: String,
    pub states: HashMap<String, StateAnimationMapping>,
}
