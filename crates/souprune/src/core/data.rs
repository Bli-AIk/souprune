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
//! The plugin uses an ECS-based approach with fine-grained components and also maintains
//! a `PlayerData` resource for convenient global access.
//!
//! 本文件定义了 `DataPlugin`，用于初始化并管理这些数据相关配置。
//! 该插件采用基于 ECS 的方式使用细粒度组件，同时维护一个 `PlayerData` 资源以便全局访问。

use crate::core::item::ItemId;
use crate::core::player_components::{
    Equipment, Gold, Health, Inventory, Level, PlayerBundle, PlayerName, Stats,
};
use bevy::app::{App, Plugin, Startup, Update};
use bevy::prelude::{
    Added, Changed, Commands, Component, IntoScheduleConfigs, Name, Or, Query, ResMut, Resource,
    With,
};

pub(crate) struct DataPlugin;

impl Plugin for DataPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerData>()
            .add_systems(Startup, spawn_player_entity_system)
            .add_systems(
                Update,
                sync_player_entity_to_resource_system.run_if(player_entity_changed),
            );
    }
    //TODO: 存档系统的序列化与反序列化。计划使用TOML格式进行存储。
}

/// Marker component indicating this is the main player entity (for queries).
///
/// 标记组件，表示这是主玩家实体（用于查询）。
#[derive(Component)]
pub struct MainPlayer;

/// System to spawn the player entity at startup with all ECS components.
///
/// 在启动时生成带有所有 ECS 组件的玩家实体的系统。
fn spawn_player_entity_system(mut commands: Commands) {
    commands.spawn((PlayerBundle::new(), MainPlayer, Name::new("Player")));
}

/// Filter type for detecting changes to any player component.
///
/// 用于检测任何玩家组件变化的过滤器类型。
type PlayerChangedFilter = (
    With<MainPlayer>,
    Or<(
        Changed<PlayerName>,
        Changed<Level>,
        Changed<Health>,
        Changed<Stats>,
        Changed<Gold>,
        Changed<Equipment>,
        Changed<Inventory>,
        Added<MainPlayer>,
    )>,
);

/// Run condition: check if any player component has changed.
///
/// 运行条件：检查是否有任何玩家组件发生变化。
fn player_entity_changed(query: Query<(), PlayerChangedFilter>) -> bool {
    !query.is_empty()
}

/// Query type for reading all player components.
///
/// 用于读取所有玩家组件的查询类型。
type PlayerComponentsQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static PlayerName,
        &'static Level,
        &'static Health,
        &'static Stats,
        &'static Gold,
        &'static Equipment,
        &'static Inventory,
    ),
    With<MainPlayer>,
>;

/// System to sync player entity components to the PlayerData resource.
///
/// 将玩家实体组件同步到 PlayerData 资源的系统。
fn sync_player_entity_to_resource_system(
    query: PlayerComponentsQuery,
    mut player_data: ResMut<PlayerData>,
) {
    let Ok((name, level, health, stats, gold, equipment, inventory)) = query.single() else {
        return;
    };

    player_data.name = name.0.clone();
    player_data.lv = level.lv;
    player_data.exp = level.exp;
    player_data.next_exp = level.next_exp;
    player_data.hp = health.current;
    player_data.hp_max = health.max;
    player_data.attack = stats.attack;
    player_data.defense = stats.defense;
    player_data.gold = gold.0;
    player_data.weapon = equipment.weapon.clone();
    player_data.armor = equipment.armor.clone();
    player_data.inventory = inventory.items.clone();
    player_data.inventory_capacity = inventory.capacity;
}

/// Resource to store basic player data, such as health, attack, defense, etc.
///
/// 保存玩家基本数据的资源，例如血量、攻击、防御等。
///
/// This resource provides convenient global access to player data.
/// For direct ECS queries, use `Query<(&Health, &Stats, ...), With<MainPlayer>>` instead.
///
/// 此资源提供对玩家数据的便捷全局访问。
/// 如需直接 ECS 查询，请使用 `Query<(&Health, &Stats, ...), With<MainPlayer>>`。
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
    pub(crate) weapon: ItemId,
    pub(crate) armor: ItemId,
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
            weapon: ItemId("stick".to_string()),
            armor: ItemId("bandage".to_string()),
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
