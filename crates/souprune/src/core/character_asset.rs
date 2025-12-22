//! Character asset definitions for data-driven character configuration.
//!
//! 数据驱动角色配置的资产定义。

use bevy::asset::Asset;
use bevy::prelude::*;
use bevy::reflect::TypePath;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Character asset defining all properties of a character (player or NPC).
///
/// 定义角色（玩家或 NPC）所有属性的资产。
#[derive(Asset, TypePath, Debug, Clone, Serialize, Deserialize)]
pub struct CharacterAsset {
    pub name: String,
    #[serde(with = "vec2_xy")]
    pub collider_size: Vec2,
    #[serde(with = "vec2_xy")]
    pub collider_offset: Vec2,
    pub base_speed: f32,
    pub animation_config: String,
    #[serde(default)]
    pub interaction_script: Option<String>,
}

/// Animation configuration asset mapping states to animation clips.
///
/// 动画配置资产，将状态映射到动画片段。
#[derive(Asset, TypePath, Debug, Clone, Serialize, Deserialize)]
pub struct AnimationConfigAsset {
    pub sprite_source: String,
    pub states: HashMap<String, StateAnimationMapping>,
}

/// Defines how a state maps to directional animations.
///
/// 定义状态如何映射到方向动画。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StateAnimationMapping {
    Directional {
        up: String,
        down: String,
        left: String,
        right: String,
    },
    Single(String),
}

impl StateAnimationMapping {
    pub fn get_clip_name(&self, direction: &crate::core::basic_components::Direction) -> &str {
        match self {
            StateAnimationMapping::Directional {
                up,
                down,
                left,
                right,
            } => match direction {
                crate::core::basic_components::Direction::Up
                | crate::core::basic_components::Direction::UpLeft
                | crate::core::basic_components::Direction::UpRight => up,
                crate::core::basic_components::Direction::Down
                | crate::core::basic_components::Direction::DownLeft
                | crate::core::basic_components::Direction::DownRight => down,
                crate::core::basic_components::Direction::Left => left,
                crate::core::basic_components::Direction::Right => right,
            },
            StateAnimationMapping::Single(clip) => clip,
        }
    }
}

// Loaders are now handled by generic RonAssetLoader in core.rs
//
// 加载器现在由 core.rs 中的泛型 RonAssetLoader 处理

/// Component that holds the animation configuration handle.
///
/// 保存动画配置句柄的组件。
#[derive(Component)]
pub struct CharacterAnimator {
    pub config: Handle<AnimationConfigAsset>,
}

mod vec2_xy {
    use bevy::math::Vec2;
    use serde::{Deserialize, Deserializer, Serializer};

    #[derive(Deserialize)]
    struct Vec2Config {
        x: f32,
        y: f32,
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec2, D::Error>
    where
        D: Deserializer<'de>,
    {
        let helper = Vec2Config::deserialize(deserializer)?;
        Ok(Vec2::new(helper.x, helper.y))
    }

    pub fn serialize<S>(value: &Vec2, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("Vec2", 2)?;
        state.serialize_field("x", &value.x)?;
        state.serialize_field("y", &value.y)?;
        state.end()
    }
}
