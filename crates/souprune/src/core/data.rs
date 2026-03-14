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
//! It defines `DataPlugin`, which loads global facts and rules from a .fre.ron file
//! (configured in mod.toml as `global_rules`) into the global layer at startup.
//!
//! 本文件定义了 `DataPlugin`，在启动时从 .fre.ron 文件
//! （在 mod.toml 中配置为 `global_rules`）加载全局事实和规则到全局层。
//!
//! ## Data Flow
//!
//! ## 数据流
//!
//! 1. At startup, read `global_rules` path from config (e.g., "global.fre.ron")
//! 2. Load the FreAsset from that path
//! 3. Apply `facts` to the Global layer of LayeredFactDatabase
//! 4. Register `rules` with scope: Global to LayeredRuleRegistry
//! 5. All systems read/write player data directly via LayeredFactDatabase
//! 6. For save/load, serialize/deserialize the facts directly
//!
//! 1. 启动时，从配置读取 `global_rules` 路径（如 "global.fre.ron"）
//! 2. 从该路径加载 FreAsset
//! 3. 将 `facts` 应用到 LayeredFactDatabase 的全局层
//! 4. 将 `rules` 以 scope: Global 注册到 LayeredRuleRegistry
//! 5. 所有系统通过 LayeredFactDatabase 直接读写玩家数据
//! 6. 存档/读档时，直接序列化/反序列化事实

use bevy::app::{App, Plugin, Startup};
use bevy::asset::{AssetServer, Assets, Handle};
use bevy::prelude::{Commands, Component, Local, Name, Res, ResMut, Resource};
use bevy_fact_rule_event::{FreAsset, LayeredFactDatabase, LayeredRuleRegistry, RuleScope};

pub(crate) struct DataPlugin;

impl Plugin for DataPlugin {
    fn build(&self, app: &mut App) {
        let schedule = crate::game_schedule(app);
        app.init_resource::<GlobalRulesHandle>()
            .add_systems(
                Startup,
                (spawn_player_entity_system, load_global_rules_system),
            )
            .add_systems(schedule, apply_global_rules_system);
    }
}

/// Marker component indicating this is the main player entity (for queries).
/// This entity has NO data components - all player data is in LayeredFactDatabase.
///
/// 标记组件，表示这是主玩家实体（用于查询）。
/// 此实体没有数据组件 - 所有玩家数据都在 LayeredFactDatabase 中。
#[derive(Component)]
pub struct MainPlayer;

/// Resource to hold the handle for the global rules file.
///
/// 保存全局规则文件句柄的资源。
#[derive(Resource, Default)]
pub struct GlobalRulesHandle {
    pub handle: Option<Handle<FreAsset>>,
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

/// System to load global rules from the configured path.
/// The path is read from `config.game.global_rules` (set in mod.toml).
///
/// 从配置的路径加载全局规则的系统。
/// 路径从 `config.game.global_rules`（在 mod.toml 中设置）读取。
fn load_global_rules_system(
    asset_server: Res<AssetServer>,
    config: Res<crate::config::SoupruneConfig>,
    mut global_rules_handle: ResMut<GlobalRulesHandle>,
) {
    if global_rules_handle.handle.is_some() {
        return;
    }

    if config.game.global_rules.is_empty() {
        bevy::log::warn!(
            "DataPlugin: No global_rules path configured in mod.toml. Player data will not be initialized."
        );
        return;
    }

    // Clone the path to avoid lifetime issues with asset_server.load()
    let path: String = config.game.global_rules.clone();
    let handle: Handle<FreAsset> = asset_server.load(path);
    global_rules_handle.handle = Some(handle);

    bevy::log::info!(
        "DataPlugin: Loading global rules from '{}'",
        config.game.global_rules
    );
}

/// System to apply loaded global rules to the global layer.
/// This runs once after the asset is loaded.
/// Now also registers rules with Global scope.
///
/// 将加载的全局规则应用到全局层的系统。
/// 在资产加载后运行一次。
/// 现在同时注册 Global 作用域的规则。
fn apply_global_rules_system(
    global_rules_handle: Res<GlobalRulesHandle>,
    fre_assets: Res<Assets<FreAsset>>,
    mut layered_db: ResMut<LayeredFactDatabase>,
    mut registry: ResMut<LayeredRuleRegistry>,
    mut applied: Local<bool>,
    mut enum_registry: ResMut<bevy_fact_rule_event::EnumRegistry>,
) {
    if *applied || global_rules_handle.loaded {
        return;
    }

    let Some(handle) = &global_rules_handle.handle else {
        return;
    };

    let Some(fre_asset) = fre_assets.get(handle) else {
        return;
    };

    // Register enums from this asset
    enum_registry.register_from_asset(fre_asset);

    // Apply facts to Global layer (these are game-wide persistent facts)
    for (key, value) in fre_asset.resolve_facts(&enum_registry) {
        layered_db.set_global(key.as_str(), value);
        bevy::log::debug!("DataPlugin: Set global fact '{}' from FRE file", key);
    }

    // Register rules with Global scope
    // Rules from global.fre.ron should declare scope: Global, but we force Global here
    // to ensure backwards compatibility
    let rule_defs = fre_asset.get_rule_defs();
    for (idx, rule_def) in rule_defs.iter().enumerate() {
        let rule = rule_def.to_rule_with_index(idx, RuleScope::Global);
        bevy::log::debug!(
            "DataPlugin: Registering global rule '{}' from FRE file",
            rule.id
        );
        registry.register(rule);
    }

    *applied = true;
    bevy::log::info!(
        "DataPlugin: Applied {} global facts and {} global rules from FRE file",
        fre_asset.get_facts().len(),
        rule_defs.len()
    );
}
