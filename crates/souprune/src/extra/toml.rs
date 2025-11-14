//! # toml.rs
//!
//! ## Module Overview
//! This module provides functionalities for loading and parsing TOML configuration files as assets in Bevy.
//!
//! ## Source File Overview
//! This file defines the `TomlPlugin`, `TomlAsset`, and `TomlAssetLoader` for handling TOML-based game configurations,
//! including modules, animations, and sprites.
//!
//! ## 模块概述
//! 该模块提供了在 Bevy 中加载和解析 TOML 配置文件作为资产的功能。
//!
//! ## 源文件概述
//! 该文件定义了 `TomlPlugin`、`TomlAsset` 和 `TomlAssetLoader`，用于处理基于 TOML 的游戏配置，
//! 包括模块、动画和精灵。

use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, LoadContext};
use bevy::prelude::*;
use bevy::tasks::ConditionalSendFuture;
use serde::Deserialize;

pub(crate) mod config;

pub struct TomlPlugin;

impl Plugin for TomlPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<TomlAsset>()
            .init_asset_loader::<TomlAssetLoader>();
    }
}

#[derive(Asset, TypePath, Debug)]
pub struct TomlAsset {
    pub config: TomlConfig,
}

#[derive(Debug, Deserialize)]
pub struct TomlConfig {
    #[serde(default)]
    pub module: Vec<ModuleConfig>,
    #[serde(default)]
    pub animations: Vec<AnimationConfig>,
    #[serde(default)]
    pub sprites: Vec<SpriteConfig>,
}

#[derive(Debug, Deserialize)]
pub struct ModuleConfig {
    #[serde(default)]
    pub loaded_before: Vec<String>,
}

fn default_loop() -> bool {
    true
}

fn default_frame_duration() -> f32 {
    0.15
}

#[derive(Debug, Deserialize, Clone)]
pub struct AnimationConfig {
    pub name: String,
    pub path: String,
    #[serde(default)]
    pub flip_x: bool,
    #[serde(default)]
    pub flip_y: bool,
    #[serde(default = "default_loop")]
    pub looping: bool,
    #[serde(default = "default_frame_duration")]
    pub frame_duration: f32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SpriteConfig {
    pub name: String,
    pub path: String,
    #[serde(default)]
    pub flip_x: bool,
    #[serde(default)]
    pub flip_y: bool,
}

#[derive(Default)]
pub struct TomlAssetLoader;

impl AssetLoader for TomlAssetLoader {
    type Asset = TomlAsset;
    type Settings = ();
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> impl ConditionalSendFuture<Output = Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;

            let config: TomlConfig = toml::from_str(std::str::from_utf8(&bytes)?)?;

            info!(
                "Successfully loaded toml file: {:?} (animations: {}, sprites: {})",
                load_context.path(),
                config.animations.len(),
                config.sprites.len()
            );

            Ok(TomlAsset { config })
        })
    }

    fn extensions(&self) -> &[&str] {
        &["toml"]
    }
}