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

fn default_frame_duration() -> f32 {
    0.15
}

fn default_looping() -> bool {
    true
}

/// A single animation entry — either a plain path or a full descriptor.
///
/// In RON:
/// - `"characters/frisk/walk/down"` → simple path, no flip, defaults
/// - `(path: "characters/frisk/walk/side", flip_x: true)` → with flip
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AnimationEntry {
    /// Just a path (relative to module texture root).
    Path(String),
    /// Full descriptor with path + optional flip / timing overrides.
    Full {
        path: String,
        #[serde(default)]
        flip_x: bool,
        #[serde(default)]
        flip_y: bool,
        #[serde(default)]
        frame_duration: Option<f32>,
        #[serde(default)]
        looping: Option<bool>,
    },
}

impl AnimationEntry {
    pub fn path(&self) -> &str {
        match self {
            Self::Path(p) => p,
            Self::Full { path, .. } => path,
        }
    }

    pub fn flip_x(&self) -> bool {
        match self {
            Self::Path(_) => false,
            Self::Full { flip_x, .. } => *flip_x,
        }
    }

    pub fn flip_y(&self) -> bool {
        match self {
            Self::Path(_) => false,
            Self::Full { flip_y, .. } => *flip_y,
        }
    }

    pub fn frame_duration_override(&self) -> Option<f32> {
        match self {
            Self::Path(_) => None,
            Self::Full { frame_duration, .. } => *frame_duration,
        }
    }

    pub fn looping_override(&self) -> Option<bool> {
        match self {
            Self::Path(_) => None,
            Self::Full { looping, .. } => *looping,
        }
    }
}

/// Defines how a state maps to directional animations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StateAnimationMapping {
    Directional {
        up: AnimationEntry,
        down: AnimationEntry,
        left: AnimationEntry,
        right: AnimationEntry,
    },
    Single(AnimationEntry),
}

/// Animation configuration asset — another form of `.character.ron` schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationConfigAsset {
    pub sprite_source: String,
    #[serde(default = "default_frame_duration")]
    pub default_frame_duration: f32,
    #[serde(default = "default_looping")]
    pub default_looping: bool,
    pub states: HashMap<String, StateAnimationMapping>,
}
