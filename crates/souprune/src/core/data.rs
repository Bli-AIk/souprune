//! # data.rs
//!
//! # data.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This module manages core game data through the FRE (Fact-Rule-Event) system.
//! Player data is stored exclusively in the LayeredFactDatabase - there are NO
//! ECS components for player stats. This is the single source of truth.
//!
//! 该模块通过 FRE（事实-规则-事件）系统管理核心游戏数据。
//! 玩家数据完全存储在 LayeredFactDatabase 中 - 没有用于玩家属性的 ECS 组件。
//! 这是唯一的数据来源。
//!
//! ## Source File Overview
//!
//! ## 源文件概述
//!
//! It defines `DataPlugin`, which loads player facts from a .rules.ron file
//! into the global layer at startup.
//!
//! 本文件定义了 `DataPlugin`，在启动时从 .rules.ron 文件加载玩家事实到全局层。
//!
//! ## Data Flow
//!
//! ## 数据流
//!
//! 1. At startup, load `player/player_data.rules.ron` from assets
//! 2. Apply `initial_facts` to the Global layer of LayeredFactDatabase
//! 3. All systems read/write player data directly via LayeredFactDatabase
//! 4. For save/load, serialize/deserialize the facts directly
//!
//! 1. 启动时，从 assets 加载 `player/player_data.rules.ron`
//! 2. 将 `initial_facts` 应用到 LayeredFactDatabase 的全局层
//! 3. 所有系统通过 LayeredFactDatabase 直接读写玩家数据
//! 4. 存档/读档时，直接序列化/反序列化事实

use bevy::app::{App, Plugin, Startup, Update};
use bevy::asset::{AssetServer, Assets, Handle};
use bevy::prelude::{Commands, Component, Local, Name, Res, ResMut, Resource};
use bevy_fact_rule_event::{FactValue, FactValueDef, LayeredFactDatabase, RuleSetAsset};

pub(crate) struct DataPlugin;

impl Plugin for DataPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GlobalPlayerDataHandle>()
            .add_systems(
                Startup,
                (spawn_player_entity_system, load_player_data_system),
            )
            .add_systems(Update, apply_player_data_system);
    }
}

/// Marker component indicating this is the main player entity (for queries).
/// This entity has NO data components - all player data is in LayeredFactDatabase.
///
/// 标记组件，表示这是主玩家实体（用于查询）。
/// 此实体没有数据组件 - 所有玩家数据都在 LayeredFactDatabase 中。
#[derive(Component)]
pub struct MainPlayer;

/// Resource to hold the handle for the player data rules file.
///
/// 保存玩家数据规则文件句柄的资源。
#[derive(Resource, Default)]
pub struct GlobalPlayerDataHandle {
    pub handle: Option<Handle<RuleSetAsset>>,
    pub loaded: bool,
}

/// System to spawn the player entity at startup.
/// Note: This entity only has a marker component - no data components.
///
/// 在启动时生成玩家实体的系统。
/// 注意：此实体只有标记组件 - 没有数据组件。
fn spawn_player_entity_system(mut commands: Commands) {
    commands.spawn((MainPlayer, Name::new("Player")));
}

/// System to load player data from the rules file.
///
/// 从规则文件加载玩家数据的系统。
fn load_player_data_system(
    asset_server: Res<AssetServer>,
    mut player_data_handle: ResMut<GlobalPlayerDataHandle>,
) {
    if player_data_handle.handle.is_some() {
        return;
    }

    let handle: Handle<RuleSetAsset> = asset_server.load("player/player_data.rules.ron");
    player_data_handle.handle = Some(handle);

    bevy::log::info!("DataPlugin: Loading player data from player/player_data.rules.ron");
}

/// System to apply loaded player data to the global layer.
///
/// 将加载的玩家数据应用到全局层的系统。
fn apply_player_data_system(
    player_data_handle: Res<GlobalPlayerDataHandle>,
    rule_set_assets: Res<Assets<RuleSetAsset>>,
    mut layered_db: ResMut<LayeredFactDatabase>,
    mut applied: Local<bool>,
) {
    if *applied || player_data_handle.loaded {
        return;
    }

    let Some(handle) = &player_data_handle.handle else {
        return;
    };

    let Some(rule_set) = rule_set_assets.get(handle) else {
        return;
    };

    // Apply initial facts to Global layer
    for (key, value) in rule_set.get_initial_facts() {
        let fact_value: FactValue = match value {
            FactValueDef::Int(v) => FactValue::Int(*v),
            FactValueDef::Float(v) => FactValue::Float(*v),
            FactValueDef::Bool(v) => FactValue::Bool(*v),
            FactValueDef::String(v) => FactValue::String(v.clone()),
        };
        layered_db.set_global(key.as_str(), fact_value);
        bevy::log::debug!("DataPlugin: Set global fact '{}' from RON", key);
    }

    *applied = true;
    bevy::log::info!(
        "DataPlugin: Applied {} player facts to global layer",
        rule_set.get_initial_facts().len()
    );
}
