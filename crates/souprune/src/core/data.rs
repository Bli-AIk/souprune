//! # data.rs
//!
//! # data.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This module manages core game data such as player saves and configuration values.
//!
//! 该模块管理玩家存档及配置值等核心游戏数据。
//!
//! ## Source File Overview
//!
//! ## 源文件概述
//!
//! It defines `DataPlugin`, which initializes and manages those data-related configurations.
//!
//! 本文件定义了 `DataPlugin`，用于初始化并管理这些数据相关配置。

use crate::core::item::ItemId;
use bevy::app::{App, Plugin};
use bevy::prelude::Resource;

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

// TODO: 改为角色 Data，以支持多个角色的数据存储。
#[derive(Resource)]
pub(crate) struct PlayerData {
    pub(crate) name: String,
    pub(crate) lv: usize,
    pub(crate) exp: usize,
    pub(crate) next_exp: usize,
    pub(crate) hp: usize,
    pub(crate) hp_max: usize,
    pub(crate) attack: usize,
    pub(crate) defense: usize,
    pub(crate) gold: usize,
    pub(crate) weapon: String, // TODO: 待引入物品系统后改为物品ID
    pub(crate) armor: String,
    pub(crate) inventory: Vec<ItemId>,
    pub(crate) inventory_capacity: usize,
}

impl Default for PlayerData {
    fn default() -> Self {
        PlayerData {
            name: "Chara".to_string(),
            lv: 1,
            exp: 0,
            next_exp: 10,
            hp: 20,
            hp_max: 20,
            attack: 0,
            defense: 0,
            gold: 42,
            weapon: "stick".to_string(),
            armor: "bandage".to_string(),
            inventory: vec![
                ItemId("monster_candy".to_string()),
                ItemId("monster_candy".to_string()),
                ItemId("monster_candy".to_string()),
                ItemId("monster_candy".to_string()),
                ItemId("monster_candy".to_string()),
                ItemId("UNDEFITEM".to_string()),
            ],
            inventory_capacity: 8,
        }
    }
}
