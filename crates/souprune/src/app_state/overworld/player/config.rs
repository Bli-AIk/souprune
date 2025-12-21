//! # config.rs
//!
//! # config.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Defines the `PlayerBehavior` resource and configuration, which dictates how the player behaves in the overworld, including movement, animation, and interaction settings.
//!
//! 定义 `PlayerBehavior` 资源和配置，规定玩家在 Overworld 中的行为，包括移动、动画和交互设置。

use crate::config;
use crate::core::basic_components::Direction;
use crate::core::character_asset::{AnimationConfigAsset, CharacterAsset};
use crate::core::input::Action;
use anyhow::{Context, Result, anyhow};
use bevy::math::Vec2;
use bevy::prelude::Resource;
use serde::Deserialize;
use std::fs;

const PLAYER_BEHAVIOR_PATH: &str = "player/player_behavior.ron";

/// Runtime configuration describing how the overworld player should behave.
///
/// 描述 Overworld 玩家行为的运行时配置资源。
#[derive(Resource, Debug, Clone)]
pub struct PlayerBehavior {
    pub spawn_position: Vec2,
    pub initial_facing: Direction,
    pub sprite_source: String,
    pub initial_state: String,
    pub initial_clip: String,
    pub collider_size: Vec2,
    pub collider_offset: Vec2,
    pub base_speed: f32,
    pub animation_config_path: String,
    pub run_action: Option<Action>,
    pub run_speed_multiplier: f32,
}

impl PlayerBehavior {
    pub fn load() -> Result<Self> {
        let config_data = PlayerBehaviorFile::read()?;

        let character_asset: CharacterAsset = load_ron_asset(&config_data.character_asset)
            .with_context(|| {
                format!(
                    "Failed to read character asset {} referenced by player behavior",
                    config_data.character_asset
                )
            })?;

        let animation_config: AnimationConfigAsset =
            load_ron_asset(&character_asset.animation_config).with_context(|| {
                format!(
                    "Failed to read animation config {} referenced by character asset",
                    character_asset.animation_config
                )
            })?;

        let sprite_source = animation_config.sprite_source.clone();

        let initial_state = config_data.initial_state.clone();
        let clip_name = resolve_initial_clip(
            &initial_state,
            &config_data.initial_facing,
            &animation_config,
        )?;

        let (run_action, run_speed_multiplier) = if let Some(run) = config_data.run.clone() {
            (Some(run.action), run.speed_multiplier)
        } else {
            (None, default_run_speed_multiplier())
        };

        Ok(Self {
            spawn_position: config_data.spawn_position.into_vec2(),
            initial_facing: config_data.initial_facing,
            sprite_source,
            initial_state,
            initial_clip: clip_name.to_string(),
            collider_size: character_asset.collider_size,
            collider_offset: character_asset.collider_offset,
            base_speed: character_asset.base_speed,
            animation_config_path: character_asset.animation_config,
            run_action,
            run_speed_multiplier,
        })
    }
}

fn resolve_initial_clip<'a>(
    state_name: &str,
    facing: &Direction,
    animation_config: &'a AnimationConfigAsset,
) -> Result<&'a str> {
    let mapping = animation_config
        .states
        .get(state_name)
        .or_else(|| animation_config.states.values().next())
        .ok_or_else(|| anyhow!("Animation config does not contain any states"))?;

    Ok(mapping.get_clip_name(facing))
}

fn load_ron_asset<T>(relative_path: &str) -> Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    let contents = read_prioritized_file(relative_path)?;
    ron::de::from_str(&contents)
        .with_context(|| format!("Failed to parse RON file: {}", relative_path))
}

fn read_prioritized_file(relative_path: &str) -> Result<String> {
    if let Some(path) = config::resolve_path(relative_path) {
        return fs::read_to_string(&path)
            .with_context(|| format!("Failed to read file {}", path.display()));
    }

    Err(anyhow!(
        "Unable to locate asset {} in project or fallback directories",
        relative_path
    ))
}

#[derive(Debug, Clone, Deserialize)]
struct PlayerBehaviorFile {
    character_asset: String,
    #[serde(default = "default_spawn_position")]
    spawn_position: Vec2Config,
    #[serde(default)]
    initial_facing: Direction,
    #[serde(default = "default_initial_state")]
    initial_state: String,
    #[serde(default)]
    run: Option<RunConfig>,
}

impl PlayerBehaviorFile {
    fn read() -> Result<Self> {
        let contents = read_prioritized_file(PLAYER_BEHAVIOR_PATH).with_context(|| {
            format!(
                "Failed to read player behavior config at {}",
                PLAYER_BEHAVIOR_PATH
            )
        })?;
        ron::de::from_str(&contents).with_context(|| {
            format!(
                "Failed to parse player behavior config at {}",
                PLAYER_BEHAVIOR_PATH
            )
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
struct RunConfig {
    action: Action,
    #[serde(default = "default_run_speed_multiplier")]
    speed_multiplier: f32,
}

fn default_initial_state() -> String {
    "Idle".to_string()
}

#[derive(Debug, Clone, Deserialize)]
struct Vec2Config {
    x: f32,
    y: f32,
}

impl Vec2Config {
    fn into_vec2(self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }
}

fn default_spawn_position() -> Vec2Config {
    Vec2Config { x: 0.0, y: 0.0 }
}

fn default_run_speed_multiplier() -> f32 {
    2.0
}
