//! Character asset definitions for data-driven character configuration.
//!
//! 数据驱动角色配置的资源定义。

use bevy::asset::Asset;
use bevy::prelude::*;
use bevy::reflect::TypePath;
use serde::{Deserialize, Serialize};
use souprune_schema::character::{
    AnimationConfigAsset as SchemaAnimationConfigAsset, CharacterAsset as SchemaCharacterAsset,
    Vec2XY,
};
pub use souprune_schema::character::{AnimationEntry, StateAnimationMapping};
use std::ops::{Deref, DerefMut};

/// Character asset runtime wrapper.
///
/// `.character.ron` 的权威结构在 `souprune_schema::character`。
/// 这里仅保留 Bevy 资源包装和运行时辅助方法。
#[derive(Asset, TypePath, Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CharacterAsset(pub SchemaCharacterAsset);

impl Deref for CharacterAsset {
    type Target = SchemaCharacterAsset;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for CharacterAsset {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl CharacterAsset {
    pub fn collider_size_vec2(&self) -> Vec2 {
        vec2_xy_to_vec2(&self.0.collider_size)
    }

    pub fn collider_offset_vec2(&self) -> Vec2 {
        vec2_xy_to_vec2(&self.0.collider_offset)
    }
}

/// Animation configuration asset runtime wrapper.
///
/// `.animation_config.ron` 的实际字段由共享 schema crate 维护。
#[derive(Asset, TypePath, Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AnimationConfigAsset(pub SchemaAnimationConfigAsset);

impl Deref for AnimationConfigAsset {
    type Target = SchemaAnimationConfigAsset;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for AnimationConfigAsset {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub fn state_animation_entry<'a>(
    mapping: &'a StateAnimationMapping,
    direction: &crate::core::basic_components::Direction,
) -> &'a AnimationEntry {
    match mapping {
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
        StateAnimationMapping::Single(entry) => entry,
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

fn vec2_xy_to_vec2(value: &Vec2XY) -> Vec2 {
    Vec2::new(value.x, value.y)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::basic_components::Direction;

    #[test]
    fn parses_character_asset_and_converts_colliders_to_vec2() {
        let ron = r#"(
            name: "hero",
            collider_size: (x: 12.0, y: 20.0),
            collider_offset: (x: 1.0, y: -2.0),
            base_speed: 96.0,
            animation_config: "characters/hero_anim.animation_config.ron",
            interaction_script: Some("scripts/hero_interact.mortar"),
        )"#;

        let asset: CharacterAsset = ron::from_str(ron).expect("character asset");

        assert_eq!(asset.name, "hero");
        assert_eq!(asset.collider_size_vec2(), Vec2::new(12.0, 20.0));
        assert_eq!(asset.collider_offset_vec2(), Vec2::new(1.0, -2.0));
        assert_eq!(
            asset.animation_config,
            "characters/hero_anim.animation_config.ron"
        );
    }

    #[test]
    fn resolves_directional_animation_entry() {
        let mapping = StateAnimationMapping::Directional {
            up: AnimationEntry::Path("walk_up".to_string()),
            down: AnimationEntry::Path("walk_down".to_string()),
            left: AnimationEntry::Path("walk_left".to_string()),
            right: AnimationEntry::Path("walk_right".to_string()),
        };

        assert_eq!(
            state_animation_entry(&mapping, &Direction::UpLeft).path(),
            "walk_up"
        );
        assert_eq!(
            state_animation_entry(&mapping, &Direction::Right).path(),
            "walk_right"
        );
    }
}
