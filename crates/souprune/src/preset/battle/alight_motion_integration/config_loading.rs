//! # config_loading.rs
//!
//! # config_loading.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Loads the battle-specific configuration that adapts imported Alight Motion scenes to
//! Souprune combat. It reads the project override file, parses the collision-matching regexes,
//! and stores the compiled patterns as resources for the rest of the integration layer.
//!
//! 负责加载把 Alight Motion 场景适配到 Souprune 战斗系统所需的专用配置。它会读取
//! 项目里的覆盖文件，解析用于碰撞匹配的正则表达式，并把编译后的模式存成资源供整条集成链使用。

use super::{AlightMotionBattleConfig, AlightMotionBattlePatterns};
use bevy::prelude::*;
use regex::Regex;

/// Load and parse Alight Motion battle config from a path.
pub(super) fn load_alight_motion_config_from_path(
    config_path: &str,
) -> (
    AlightMotionBattleConfig,
    Option<Regex>,
    Option<Regex>,
    Option<Regex>,
) {
    let mut am_config = AlightMotionBattleConfig::default();

    match std::fs::read_to_string(config_path) {
        Ok(content) => match ron::from_str::<AlightMotionBattleConfig>(&content) {
            Ok(config) => {
                am_config = config;
                info!(
                    "[AM Battle] Loaded config from {}: scale={}, offset={:?}, bullet_pattern='{}', battle_box_pattern='{}', damage={}",
                    config_path,
                    am_config.scale,
                    am_config.offset,
                    am_config.bullet_pattern,
                    am_config.battle_box_pattern,
                    am_config.bullet_damage
                );
            }
            Err(e) => {
                warn!(
                    "[AM Battle] Failed to parse {}: {}. Using defaults.",
                    config_path, e
                );
            }
        },
        Err(e) => {
            info!(
                "[AM Battle] Config file {} not found ({}). Using defaults: scale={}, offset={:?}",
                config_path, e, am_config.scale, am_config.offset
            );
        }
    }

    let bullet_regex = compile_regex("bullet", &am_config.bullet_pattern);
    let battle_box_regex = compile_regex("battle_box", &am_config.battle_box_pattern);
    let hidden_regex = if am_config.hidden_pattern.is_empty() {
        None
    } else {
        compile_regex("hidden", &am_config.hidden_pattern)
    };

    (am_config, bullet_regex, battle_box_regex, hidden_regex)
}

/// System to load Alight Motion battle config from the mod's battle directory.
pub(super) fn load_am_battle_config(
    mut commands: Commands,
    mut am_config: ResMut<AlightMotionBattleConfig>,
    project_config: Res<crate::config::SoupruneConfig>,
) {
    let config_path = format!(
        "{}/{}/states/battle/alight_motion_config.ron",
        crate::config::get_projects_base_path().display(),
        project_config.project.mod_name
    );

    let (config, bullet_regex, battle_box_regex, hidden_regex) =
        load_alight_motion_config_from_path(&config_path);
    *am_config = config;

    commands.insert_resource(AlightMotionBattlePatterns {
        bullet_regex,
        battle_box_regex,
        hidden_regex,
    });
}

fn compile_regex(label: &str, pattern: &str) -> Option<Regex> {
    match Regex::new(pattern) {
        Ok(regex) => {
            info!("[AM Battle] Compiled {label} regex: '{pattern}'");
            Some(regex)
        }
        Err(error) => {
            warn!("[AM Battle] Invalid {label} pattern '{pattern}': {error}");
            None
        }
    }
}
