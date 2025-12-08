//! Character asset definitions for data-driven character configuration.
//!
//! 数据驱动角色配置的资产定义。

use bevy::asset::io::Reader;
use bevy::asset::{Asset, AssetLoader, LoadContext};
use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::tasks::ConditionalSendFuture;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Character asset defining all properties of a character (player or NPC).
///
/// 定义角色（玩家或 NPC）所有属性的资产。
#[derive(Asset, TypePath, Debug, Clone, Serialize, Deserialize)]
pub struct CharacterAsset {
    pub name: String,
    pub collider_size: Vec2,
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

/// Asset loader for `.char.ron` files.
///
/// `.char.ron` 文件的资产加载器。
#[derive(Default)]
pub struct CharacterAssetLoader;

impl AssetLoader for CharacterAssetLoader {
    type Asset = CharacterAsset;
    type Settings = ();
    type Error = anyhow::Error;

    fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> impl ConditionalSendFuture<Output = Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            let asset = ron::de::from_bytes::<CharacterAsset>(&bytes)?;
            Ok(asset)
        })
    }

    fn extensions(&self) -> &[&str] {
        &["char.ron"]
    }
}

/// Asset loader for `.anim.ron` files.
///
/// `.anim.ron` 文件的资产加载器。
#[derive(Default)]
pub struct AnimationConfigAssetLoader;

impl AssetLoader for AnimationConfigAssetLoader {
    type Asset = AnimationConfigAsset;
    type Settings = ();
    type Error = anyhow::Error;

    fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> impl ConditionalSendFuture<Output=Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            let asset = ron::de::from_bytes::<AnimationConfigAsset>(&bytes)?;
            Ok(asset)
        })
    }

    fn extensions(&self) -> &[&str] {
        &["anim.ron"]
    }
}

/// Component that holds the animation configuration handle.
///
/// 保存动画配置句柄的组件。
#[derive(Component)]
pub struct CharacterAnimator {
    pub config: Handle<AnimationConfigAsset>,
}
