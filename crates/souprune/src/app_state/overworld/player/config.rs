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
use crate::core::character_asset::{
    AnimationConfigAsset, CharacterAsset, state_animation_clip_name,
};
use anyhow::{Context, Result, anyhow};
use bevy::math::Vec2;
use bevy::prelude::Resource;
use serde::Deserialize;
use std::fs;

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
    /// Run action name (for looking up via ActionRegistry)
    pub run_action: Option<String>,
    pub run_speed_multiplier: f32,
    /// Player invincibility configuration
    pub invincibility: InvincibilityConfig,
}

impl PlayerBehavior {
    pub fn load() -> Result<Self> {
        // Get path from global config
        let global_config = config::load_config();
        let behavior_path = &global_config.game.player_behavior_path;

        if behavior_path.is_empty() {
            return Err(anyhow!("Player behavior path is empty"));
        }

        let config_data = PlayerBehaviorFile::read(behavior_path)?;

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

        let invincibility = config_data
            .invincibility
            .map(InvincibilityConfig::from)
            .unwrap_or_default();

        Ok(Self {
            spawn_position: config_data.spawn_position.into_vec2(),
            initial_facing: config_data.initial_facing,
            sprite_source,
            initial_state,
            initial_clip: clip_name.to_string(),
            collider_size: character_asset.collider_size_vec2(),
            collider_offset: character_asset.collider_offset_vec2(),
            base_speed: character_asset.base_speed,
            animation_config_path: character_asset.animation_config.clone(),
            run_action,
            run_speed_multiplier,
            invincibility,
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

    Ok(state_animation_clip_name(mapping, facing))
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
    /// Player invincibility configuration for chase/battle damage
    #[serde(default)]
    invincibility: Option<InvincibilityFileConfig>,
}

impl PlayerBehaviorFile {
    fn read(path: &str) -> Result<Self> {
        let contents = read_prioritized_file(path)
            .with_context(|| format!("Failed to read player behavior config at {}", path))?;
        ron::de::from_str(&contents)
            .with_context(|| format!("Failed to parse player behavior config at {}", path))
    }
}

#[derive(Debug, Clone, Deserialize)]
struct RunConfig {
    /// Action name as a string (e.g., "Cancel" for shift key)
    action: String,
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

/// File-level invincibility configuration (loaded from RON).
///
/// 文件级无敌配置（从 RON 加载）。
#[derive(Debug, Clone, Deserialize)]
struct InvincibilityFileConfig {
    /// Duration of invincibility in seconds after taking damage
    #[serde(default = "default_invincibility_duration")]
    duration: f32,
    /// Interval for heart color flash during invincibility (in seconds)
    #[serde(default = "default_flash_interval")]
    flash_interval: f32,
    /// Normal heart color (hex format, e.g., "#FF0000")
    #[serde(default = "default_normal_color")]
    normal_color: String,
    /// Flash heart color (hex format, e.g., "#800000")
    #[serde(default = "default_flash_color")]
    flash_color: String,
}

impl Default for InvincibilityFileConfig {
    fn default() -> Self {
        Self {
            duration: default_invincibility_duration(),
            flash_interval: default_flash_interval(),
            normal_color: default_normal_color(),
            flash_color: default_flash_color(),
        }
    }
}

fn default_invincibility_duration() -> f32 {
    1.0
}

fn default_flash_interval() -> f32 {
    0.1
}

fn default_normal_color() -> String {
    "#FF0000".to_string()
}

fn default_flash_color() -> String {
    "#800000".to_string()
}

/// Runtime invincibility configuration.
///
/// 运行时无敌配置。
#[derive(Debug, Clone)]
pub struct InvincibilityConfig {
    /// Duration of invincibility in seconds after taking damage
    pub duration: f32,
    /// Interval for heart color flash during invincibility (in seconds)
    pub flash_interval: f32,
    /// Normal heart color (hex format)
    pub normal_color: String,
    /// Flash heart color (hex format)
    pub flash_color: String,
}

impl Default for InvincibilityConfig {
    fn default() -> Self {
        Self {
            duration: default_invincibility_duration(),
            flash_interval: default_flash_interval(),
            normal_color: default_normal_color(),
            flash_color: default_flash_color(),
        }
    }
}

impl From<InvincibilityFileConfig> for InvincibilityConfig {
    fn from(file_config: InvincibilityFileConfig) -> Self {
        Self {
            duration: file_config.duration,
            flash_interval: file_config.flash_interval,
            normal_color: file_config.normal_color,
            flash_color: file_config.flash_color,
        }
    }
}
