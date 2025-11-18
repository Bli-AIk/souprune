//! # data.rs
//!
//! ## Module Overview
//! This module manages core functionalities related to game data,
//! primarily including player save information and other data configurations.
//!
//! ## Source File Overview
//! This file defines the `DataPlugin`,
//! which initializes and manages configurations related to game data.
//!
//! ## 模块概述
//! 该模块管理游戏数据相关的核心功能，主要包括玩家存档信息等数据配置。
//!
//! ## 源文件概述
//! 该文件定义了 `DataPlugin`，它初始化并管理游戏数据相关的配置。

use bevy::app::{App, Plugin};
use bevy::color::Srgba;
use bevy::math::Vec2;
use bevy::prelude::{Name, Resource, Transform};
use bevy_rich_text3d::{TextAlign, TextAnchor};

pub(crate) struct DataPlugin;

impl Plugin for DataPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerData>();
    }
    //TODO: 存档系统的序列化与反序列化。计划使用TOML格式进行存储。
}

/// Resource to store basic player data, such as health, attack, defense, etc.
///
/// 保存玩家基本数据的资源，例如血量、攻击、防御等。
#[derive(Resource)]
pub(crate) struct PlayerData {
    pub(crate) name: String,
    pub(crate) lv: usize,
    pub(crate) exp: usize,
    pub(crate) hp: usize,
    pub(crate) hp_max: usize,
    pub(crate) attack: usize,
    pub(crate) defense: usize,
    pub(crate) gold: usize,
}

impl Default for PlayerData {
    fn default() -> Self {
        PlayerData {
            name: "Chara".to_string(),
            lv: 1,
            exp: 0,
            hp: 20,
            hp_max: 20,
            attack: 10,
            defense: 10,
            gold: 42,
        }
    }
}