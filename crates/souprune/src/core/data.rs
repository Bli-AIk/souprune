//! # data.rs
//!
//! # data.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This module manages core game data through the FRE (Fact-Rule-Event) system.
//! Player data is stored in the LayeredFactDatabase and synced with ECS components.
//!
//! 该模块通过 FRE（事实-规则-事件）系统管理核心游戏数据。
//! 玩家数据存储在 LayeredFactDatabase 中并与 ECS 组件同步。
//!
//! ## Source File Overview
//!
//! ## 源文件概述
//!
//! It defines `DataPlugin`, which initializes player facts in the global layer
//! and provides bidirectional sync between FRE facts and ECS components.
//!
//! 本文件定义了 `DataPlugin`，用于在全局层初始化玩家事实，
//! 并提供 FRE 事实与 ECS 组件之间的双向同步。

use crate::core::item::ItemId;
use crate::core::player_components::{
    Equipment, Gold, Health, Inventory, Level, PlayerBundle, PlayerName, Stats,
};
use bevy::app::{App, Plugin, Startup, Update};
use bevy::prelude::{
    Added, Changed, Commands, Component, IntoScheduleConfigs, Name, Or, Query, Res, ResMut, With,
};
use bevy_fact_rule_event::LayeredFactDatabase;

pub(crate) struct DataPlugin;

impl Plugin for DataPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            (spawn_player_entity_system, init_player_facts_system),
        )
        .add_systems(
            Update,
            (
                sync_ecs_to_fre_system.run_if(player_ecs_changed),
                sync_fre_to_ecs_system,
            ),
        );
    }
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

/// System to initialize player facts in the global layer at startup.
///
/// 在启动时在全局层初始化玩家事实的系统。
fn init_player_facts_system(mut layered_db: ResMut<LayeredFactDatabase>) {
    // Initialize all player facts in the global layer with default values
    // These will be overwritten by the first ECS sync
    layered_db.set_global("player_name", "Chara".to_string());
    layered_db.set_global("player_lv", 1i64);
    layered_db.set_global("player_exp", 0i64);
    layered_db.set_global("player_next_exp", 10i64);
    layered_db.set_global("player_hp", 20i64);
    layered_db.set_global("player_hp_max", 20i64);
    layered_db.set_global("player_atk", 0i64);
    layered_db.set_global("player_def", 0i64);
    layered_db.set_global("player_gold", 42i64);
    layered_db.set_global("player_weapon", "stick".to_string());
    layered_db.set_global("player_armor", "bandage".to_string());
    // Inventory as comma-separated string for now
    layered_db.set_global(
        "player_inventory",
        "monster_candy,monster_candy,monster_candy,monster_candy,monster_candy,UNDEFITEM"
            .to_string(),
    );
    layered_db.set_global("player_inventory_capacity", 8i64);

    bevy::log::info!("DataPlugin: Initialized player facts in global layer");
}

/// Filter type for detecting changes to any player ECS component.
///
/// 用于检测任何玩家 ECS 组件变化的过滤器类型。
type PlayerEcsChangedFilter = (
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

/// Run condition: check if any player ECS component has changed.
///
/// 运行条件：检查是否有任何玩家 ECS 组件发生变化。
fn player_ecs_changed(query: Query<(), PlayerEcsChangedFilter>) -> bool {
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

/// System to sync ECS components to FRE facts when ECS changes.
///
/// 当 ECS 组件变化时同步到 FRE 事实的系统。
fn sync_ecs_to_fre_system(
    query: PlayerComponentsQuery,
    mut layered_db: ResMut<LayeredFactDatabase>,
) {
    let Ok((name, level, health, stats, gold, equipment, inventory)) = query.single() else {
        return;
    };

    // Sync all player data to global facts
    layered_db.set_global("player_name", name.0.clone());
    layered_db.set_global("player_lv", level.lv as i64);
    layered_db.set_global("player_exp", level.exp as i64);
    layered_db.set_global("player_next_exp", level.next_exp as i64);
    layered_db.set_global("player_hp", health.current as i64);
    layered_db.set_global("player_hp_max", health.max as i64);
    layered_db.set_global("player_atk", stats.attack as i64);
    layered_db.set_global("player_def", stats.defense as i64);
    layered_db.set_global("player_gold", gold.0 as i64);
    layered_db.set_global("player_weapon", equipment.weapon.0.clone());
    layered_db.set_global("player_armor", equipment.armor.0.clone());

    // Serialize inventory as comma-separated string
    let inventory_str = inventory
        .items
        .iter()
        .map(|item| item.0.as_str())
        .collect::<Vec<_>>()
        .join(",");
    layered_db.set_global("player_inventory", inventory_str);
    layered_db.set_global("player_inventory_capacity", inventory.capacity as i64);

    bevy::log::trace!("DataPlugin: Synced ECS to FRE facts");
}

/// Query type for writing to player components.
///
/// 用于写入玩家组件的查询类型。
type PlayerComponentsMutQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut PlayerName,
        &'static mut Level,
        &'static mut Health,
        &'static mut Stats,
        &'static mut Gold,
        &'static mut Equipment,
        &'static mut Inventory,
    ),
    With<MainPlayer>,
>;

/// System to sync FRE facts to ECS components when facts change.
/// This is a pull-based approach that checks if facts differ from ECS.
///
/// 当 FRE 事实变化时同步到 ECS 组件的系统。
/// 这是一种拉取式方法，检查事实是否与 ECS 不同。
fn sync_fre_to_ecs_system(
    mut query: PlayerComponentsMutQuery,
    layered_db: Res<LayeredFactDatabase>,
) {
    use bevy::prelude::DetectChanges;

    // Only run if the database is changed
    if !layered_db.is_changed() {
        return;
    }

    let Ok((mut name, mut level, mut health, mut stats, mut gold, mut equipment, mut inventory)) =
        query.single_mut()
    else {
        return;
    };

    // Sync from facts to ECS (only if different to avoid triggering Changed)
    if let Some(fact_name) = layered_db.get_string("player_name") {
        if name.0 != fact_name {
            name.0 = fact_name.to_string();
        }
    }

    if let Some(fact_lv) = layered_db.get_int("player_lv") {
        let lv = fact_lv as usize;
        if level.lv != lv {
            level.lv = lv;
        }
    }

    if let Some(fact_exp) = layered_db.get_int("player_exp") {
        let exp = fact_exp as usize;
        if level.exp != exp {
            level.exp = exp;
        }
    }

    if let Some(fact_next_exp) = layered_db.get_int("player_next_exp") {
        let next_exp = fact_next_exp as usize;
        if level.next_exp != next_exp {
            level.next_exp = next_exp;
        }
    }

    if let Some(fact_hp) = layered_db.get_int("player_hp") {
        let hp = fact_hp as usize;
        if health.current != hp {
            health.current = hp;
        }
    }

    if let Some(fact_hp_max) = layered_db.get_int("player_hp_max") {
        let hp_max = fact_hp_max as usize;
        if health.max != hp_max {
            health.max = hp_max;
        }
    }

    if let Some(fact_atk) = layered_db.get_int("player_atk") {
        let atk = fact_atk as usize;
        if stats.attack != atk {
            stats.attack = atk;
        }
    }

    if let Some(fact_def) = layered_db.get_int("player_def") {
        let def = fact_def as usize;
        if stats.defense != def {
            stats.defense = def;
        }
    }

    if let Some(fact_gold) = layered_db.get_int("player_gold") {
        let g = fact_gold as usize;
        if gold.0 != g {
            gold.0 = g;
        }
    }

    if let Some(fact_weapon) = layered_db.get_string("player_weapon") {
        if equipment.weapon.0 != fact_weapon {
            equipment.weapon = ItemId(fact_weapon.to_string());
        }
    }

    if let Some(fact_armor) = layered_db.get_string("player_armor") {
        if equipment.armor.0 != fact_armor {
            equipment.armor = ItemId(fact_armor.to_string());
        }
    }

    // Deserialize inventory from comma-separated string
    if let Some(fact_inventory) = layered_db.get_string("player_inventory") {
        let items: Vec<ItemId> = fact_inventory
            .split(',')
            .filter(|s| !s.is_empty())
            .map(|s| ItemId(s.to_string()))
            .collect();
        if inventory.items != items {
            inventory.items = items;
        }
    }

    if let Some(fact_capacity) = layered_db.get_int("player_inventory_capacity") {
        let cap = fact_capacity as usize;
        if inventory.capacity != cap {
            inventory.capacity = cap;
        }
    }
}
