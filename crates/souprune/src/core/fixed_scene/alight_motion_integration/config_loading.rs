//! # config_loading.rs
//!
//! # config_loading.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Loads the fixed-scene configuration that adapts imported Alight Motion scenes to
//! Souprune combat. It reads the project override file, parses the collision-matching regexes,
//! and stores the compiled patterns as resources for the rest of the integration layer.
//!
//! 负责加载把 Alight Motion 场景适配到 Souprune fixed-scene primitive所需的专用配置。它会读取
//! 项目里的覆盖文件，解析用于碰撞匹配的正则表达式，并把编译后的模式存成资源供整条集成链使用。

use super::{AlightMotionSceneConfig, AlightMotionScenePatterns};
use bevy::prelude::*;
use regex::Regex;
use std::path::PathBuf;

/// Load and parse Alight Motion scene config from a path.
pub(super) fn load_alight_motion_config_from_path(
    config_path: &str,
) -> (
    AlightMotionSceneConfig,
    Option<Regex>,
    Option<Regex>,
    Option<Regex>,
) {
    let mut am_config = AlightMotionSceneConfig::default();

    match std::fs::read_to_string(config_path) {
        Ok(content) => match ron::from_str::<AlightMotionSceneConfig>(&content) {
            Ok(config) => {
                am_config = config;
                info!(
                    "[AM Scene] Loaded config from {}: scale={}, offset={:?}, bullet_pattern='{}', boundary_pattern='{}', damage={}",
                    config_path,
                    am_config.scale,
                    am_config.offset,
                    am_config.bullet_pattern,
                    am_config.boundary_pattern,
                    am_config.bullet_damage
                );
            }
            Err(e) => {
                warn!(
                    "[AM Scene] Failed to parse {}: {}. Using defaults.",
                    config_path, e
                );
            }
        },
        Err(e) => {
            info!(
                "[AM Scene] Config file {} not found ({}). Using defaults: scale={}, offset={:?}",
                config_path, e, am_config.scale, am_config.offset
            );
        }
    }

    let bullet_regex = compile_regex("bullet", &am_config.bullet_pattern);
    let boundary_regex = compile_regex("boundary", &am_config.boundary_pattern);
    let hidden_regex = if am_config.hidden_pattern.is_empty() {
        None
    } else {
        compile_regex("hidden", &am_config.hidden_pattern)
    };

    (am_config, bullet_regex, boundary_regex, hidden_regex)
}

/// System to load the active mode's Alight Motion scene config.
pub(super) fn load_am_scene_config(
    mut commands: Commands,
    mut am_config: ResMut<AlightMotionSceneConfig>,
    sequence_mode: Res<crate::core::mode::SequenceMode>,
    mode_registry: Res<crate::core::mode::ModeRegistry>,
) {
    let Some(config_path) = sequence_mode
        .0
        .as_deref()
        .and_then(|mode| mode_registry.mode(mode))
        .and_then(|mode_config| mode_config.alight_motion_config.as_deref())
    else {
        let config = AlightMotionSceneConfig::default();
        let bullet_regex = compile_regex("bullet", &config.bullet_pattern);
        let boundary_regex = compile_regex("boundary", &config.boundary_pattern);
        let hidden_regex = if config.hidden_pattern.is_empty() {
            None
        } else {
            compile_regex("hidden", &config.hidden_pattern)
        };
        *am_config = config;
        commands.insert_resource(AlightMotionScenePatterns {
            bullet_regex,
            boundary_regex,
            hidden_regex,
        });
        return;
    };

    let resolved_path = resolve_config_path(config_path);
    let config_path = resolved_path.to_string_lossy();
    let (config, bullet_regex, boundary_regex, hidden_regex) =
        load_alight_motion_config_from_path(&config_path);
    *am_config = config;

    commands.insert_resource(AlightMotionScenePatterns {
        bullet_regex,
        boundary_regex,
        hidden_regex,
    });
}

fn resolve_config_path(config_path: &str) -> PathBuf {
    crate::config::resolve_path(config_path).unwrap_or_else(|| PathBuf::from(config_path))
}

fn compile_regex(label: &str, pattern: &str) -> Option<Regex> {
    match Regex::new(pattern) {
        Ok(regex) => {
            info!("[AM Scene] Compiled {label} regex: '{pattern}'");
            Some(regex)
        }
        Err(error) => {
            warn!("[AM Scene] Invalid {label} pattern '{pattern}': {error}");
            None
        }
    }
}
