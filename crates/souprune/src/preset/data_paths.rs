//! Game-specific data path resolvers and condition resolvers.
//!
//! 游戏特定的数据路径解析器和条件解析器。
//!
//! These register computed data paths (e.g. "player.total_attack")
//! and view conditions (e.g. "player.hp.is_low") that depend on
//! game-specific knowledge like equipment, inventory, and stats.

use crate::core::view::ron_view::parsing::{
    ConditionResolvers, DataPathResolvers, ExprFunctionResolvers,
};
use bevy_fact_rule_event::{FactDatabase, FactValue, LayeredFactDatabase};

/// Helper: read string fact from layered DB with optional local override.
fn get_string(db: &LayeredFactDatabase, local: Option<&FactDatabase>, key: &str) -> String {
    if let Some(local) = local {
        if let Some(FactValue::String(s)) = local.get_by_str(key) {
            return s.clone();
        }
    }
    match db.get_by_str(key) {
        Some(FactValue::String(s)) => s.clone(),
        Some(FactValue::Int(i)) => i.to_string(),
        _ => String::new(),
    }
}

/// Helper: read int fact from layered DB with optional local override.
fn get_int(db: &LayeredFactDatabase, local: Option<&FactDatabase>, key: &str) -> i64 {
    if let Some(local) = local {
        if let Some(FactValue::Int(i)) = local.get_by_str(key) {
            return *i;
        }
    }
    match db.get_by_str(key) {
        Some(FactValue::Int(i)) => *i,
        Some(FactValue::Float(f)) => *f as i64,
        _ => 0,
    }
}

/// Helper: resolve an item's display name via FRE facts and mortar string table.
fn resolve_item_name(
    item_id: &str,
    db: &LayeredFactDatabase,
    mortar_strings: &crate::extra::mortar::MortarStringTable,
) -> Option<String> {
    let locale_key = match db.get_by_str(&format!("items:{item_id}.locale_key")) {
        Some(FactValue::String(s)) => s.clone(),
        _ => return None,
    };
    Some(mortar_strings.resolve(&locale_key).to_string())
}

/// Register all UT/DR-specific computed data path resolvers.
pub fn register_data_path_resolvers(resolvers: &mut DataPathResolvers) {
    // player.inventory: format inventory item names
    resolvers.register("player.inventory", |db, local, mortar_strings| {
        let inventory = match db.get_by_str("player:inventory") {
            Some(FactValue::StringList(list)) => list.clone(),
            _ => return String::new(),
        };
        let capacity = get_int(db, local, "player:inventory_capacity").max(1) as usize;
        inventory
            .iter()
            .take(capacity)
            .map(|item_id| {
                resolve_item_name(item_id, db, mortar_strings)
                    .unwrap_or_else(|| format!("UNDEFINED ({item_id})"))
            })
            .collect::<Vec<_>>()
            .join("\n")
    });

    // player.weapon: display name of equipped weapon
    resolvers.register("player.weapon", |db, local, mortar_strings| {
        let weapon = get_string(db, local, "player:weapon");
        resolve_item_name(&weapon, db, mortar_strings).unwrap_or(weapon)
    });

    // player.weapon_atk: weapon's damage stat
    resolvers.register("player.weapon_atk", |db, local, _| {
        let weapon = get_string(db, local, "player:weapon");
        match db.get_by_str(&format!("items:{weapon}.damage")) {
            Some(FactValue::Int(i)) => i.to_string(),
            _ => "0".to_string(),
        }
    });

    // player.total_attack: base attack + weapon damage
    resolvers.register("player.total_attack", |db, local, _| {
        let weapon = get_string(db, local, "player:weapon");
        let weapon_atk = match db.get_by_str(&format!("items:{weapon}.damage")) {
            Some(FactValue::Int(i)) => *i,
            _ => 0,
        };
        (get_int(db, local, "player:attack") + weapon_atk).to_string()
    });

    // player.armor: display name of equipped armor
    resolvers.register("player.armor", |db, local, mortar_strings| {
        let armor = get_string(db, local, "player:armor");
        resolve_item_name(&armor, db, mortar_strings).unwrap_or(armor)
    });

    // player.armor_def: armor's defense stat
    resolvers.register("player.armor_def", |db, local, _| {
        let armor = get_string(db, local, "player:armor");
        match db.get_by_str(&format!("items:{armor}.defense")) {
            Some(FactValue::Int(i)) => i.to_string(),
            _ => "0".to_string(),
        }
    });

    // player.total_defense: base defense + armor defense
    resolvers.register("player.total_defense", |db, local, _| {
        let armor = get_string(db, local, "player:armor");
        let armor_def = match db.get_by_str(&format!("items:{armor}.defense")) {
            Some(FactValue::Int(i)) => *i,
            _ => 0,
        };
        (get_int(db, local, "player:defense") + armor_def).to_string()
    });
}

/// Register all UT/DR-specific condition resolvers.
pub fn register_condition_resolvers(resolvers: &mut ConditionResolvers) {
    resolvers.register("player.inventory.is_empty", |db, _local| {
        match db.get_by_str("player:inventory") {
            Some(FactValue::StringList(list)) => list.is_empty(),
            _ => true,
        }
    });

    resolvers.register("player.inventory.is_not_empty", |db, _local| {
        match db.get_by_str("player:inventory") {
            Some(FactValue::StringList(list)) => !list.is_empty(),
            _ => false,
        }
    });

    resolvers.register("player.hp.is_low", |db, local| {
        let hp = get_int(db, local, "player:hp");
        let hp_max = get_int(db, local, "player:hp_max").max(1);
        hp < hp_max / 4
    });

    resolvers.register("player.hp.is_critical", |db, local| {
        let hp = get_int(db, local, "player:hp");
        hp <= 1
    });

    resolvers.register("player.gold.is_zero", |db, local| {
        get_int(db, local, "player:gold") == 0
    });
}

/// Register expression functions for fasteval expressions (e.g. `inventory_is_empty()`).
///
/// 注册 fasteval 表达式函数（如 `inventory_is_empty()`）。
pub fn register_expr_function_resolvers(resolvers: &mut ExprFunctionResolvers) {
    resolvers.register("inventory_is_empty", |db, _local| {
        match db.get_by_str("player:inventory") {
            Some(FactValue::StringList(list)) => {
                if list.is_empty() {
                    1.0
                } else {
                    0.0
                }
            }
            _ => 1.0,
        }
    });
}
