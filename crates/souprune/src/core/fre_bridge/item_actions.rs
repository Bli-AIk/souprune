//! Item action handlers for FRE bridge (UseItem, CheckItem, DropItem).
//!
//! FRE 桥接的物品动作处理器（UseItem、CheckItem、DropItem）。

use bevy::prelude::*;
use bevy_fact_rule_event::{CombinedFactReader, EnumRegistry, FactReader, FactValue};

use super::eval::evaluate_local_fact_value;
use crate::core::{audio, fre_facts};

/// Resolve an index expression (e.g., "$item_selection") to a usize index.
pub(super) fn resolve_index_expr(
    index_expr: &str,
    local_facts: &bevy_fact_rule_event::FactDatabase,
    global_facts: &bevy_fact_rule_event::LayeredFactDatabase,
    enum_registry: &EnumRegistry,
) -> Option<usize> {
    use bevy_fact_rule_event::LocalFactValue;
    let combined = CombinedFactReader::new(local_facts, global_facts);
    let value = evaluate_local_fact_value(
        "_index",
        &LocalFactValue::Expr(index_expr.to_string()),
        &combined,
        enum_registry,
    );
    match value {
        FactValue::Int(i) if i >= 0 => Some(i as usize),
        _ => {
            warn!(
                "FRE Bridge: index_expr '{}' resolved to {:?}, expected non-negative Int",
                index_expr, value
            );
            None
        }
    }
}

/// Get inventory item_id at index, if valid.
fn get_inventory_item_id(
    index: usize,
    global_facts: &bevy_fact_rule_event::LayeredFactDatabase,
) -> Option<String> {
    global_facts
        .get_string_list("player:inventory")
        .and_then(|inv| inv.get(index).cloned())
}

/// Start a dialogue for an item action (OnUse/OnCheck/OnDrop).
/// Uses item's mortar file if available, otherwise falls back to defaults.
fn start_item_dialogue(
    item: &crate::core::item::Item,
    node_name: &str,
    default_node: &str,
    global_facts: &mut bevy_fact_rule_event::LayeredFactDatabase,
    dialogue_view_default: &str,
    dialogue_voice_default: &str,
    item_data: ItemDialogueData,
) {
    let (mortar_path, node) = if let Some(mortar) = &item.mortar {
        (mortar.as_str(), node_name)
    } else {
        ("items/_defaults.mortar", default_node)
    };

    start_item_dialogue_with_path(
        mortar_path,
        node,
        global_facts,
        dialogue_view_default,
        dialogue_voice_default,
        item_data,
    );
}

/// Start a dialogue with an explicit mortar path and node.
fn start_item_dialogue_with_path(
    mortar_path: &str,
    node: &str,
    global_facts: &mut bevy_fact_rule_event::LayeredFactDatabase,
    dialogue_view_default: &str,
    dialogue_voice_default: &str,
    item_data: ItemDialogueData,
) {
    info!(
        "FRE Bridge: Item dialogue — mortar: {}, node: {}",
        mortar_path, node
    );
    global_facts.set_local(
        fre_facts::DIALOGUE_PENDING_VIEW,
        FactValue::String(dialogue_view_default.to_string()),
    );
    global_facts.set_local(
        fre_facts::DIALOGUE_PENDING_MORTAR_PATH,
        FactValue::String(mortar_path.to_string()),
    );
    global_facts.set_local(
        fre_facts::DIALOGUE_PENDING_MORTAR_NODE,
        FactValue::String(node.to_string()),
    );
    global_facts.set_local(fre_facts::DIALOGUE_HAS_TYPEWRITER, FactValue::Bool(true));
    global_facts.set_local(fre_facts::DIALOGUE_HAS_FOCUS, FactValue::Bool(true));
    if !dialogue_voice_default.is_empty() {
        global_facts.set_local(
            fre_facts::DIALOGUE_VOICE,
            FactValue::String(dialogue_voice_default.to_string()),
        );
    }

    global_facts.set_local(
        fre_facts::DIALOGUE_ITEM_NAME,
        FactValue::String(item_data.locale_key),
    );
    global_facts.set_local(
        fre_facts::DIALOGUE_ITEM_DESCRIPTION,
        FactValue::String(item_data.description),
    );
    global_facts.set_local(
        fre_facts::DIALOGUE_ITEM_HEAL_AMOUNT,
        FactValue::Int(item_data.heal_amount),
    );
    global_facts.set_local(
        fre_facts::DIALOGUE_ITEM_VALUE,
        FactValue::Int(item_data.item_value),
    );

    global_facts.set_local(fre_facts::DIALOGUE_PENDING_START, FactValue::Bool(true));
}

/// Pre-computed item data for mortar dialogue variables and functions.
struct ItemDialogueData {
    locale_key: String,
    description: String,
    heal_amount: i64,
    item_value: i64,
}

/// Compute the stat value for an item (used by mortar function `get_item_value()`).
fn compute_item_value(item: &crate::core::item::Item) -> i64 {
    use crate::core::item::ItemType;
    match &item.item_type {
        ItemType::Food { effects, .. } => effects
            .iter()
            .find_map(|e| match e {
                crate::core::item::ItemEffect::Heal { amount } => Some(*amount as i64),
                _ => None,
            })
            .unwrap_or(0),
        ItemType::Weapon { damage, .. } => *damage as i64,
        ItemType::Armor { defense } => *defense as i64,
        ItemType::KeyItem => 0,
    }
}

/// Execute item effects (Heal, PlayAudio, SpawnChildItem, SetFact).
/// Returns the actual amount of HP healed (0 if no heal occurred).
fn apply_item_effects(
    item: &crate::core::item::Item,
    index: usize,
    global_facts: &mut bevy_fact_rule_event::LayeredFactDatabase,
    audio: &bevy_kira_audio::Audio,
    asset_server: &AssetServer,
) -> i64 {
    use crate::core::item::{ItemEffect, ItemType};

    let effects = match &item.item_type {
        ItemType::Food { effects, .. } => effects.as_slice(),
        _ => return 0,
    };

    let mut spawn_child: Option<String> = None;
    let mut total_healed: i64 = 0;

    for effect in effects {
        match effect {
            ItemEffect::Heal { amount } => {
                let hp = global_facts.get_int("player:hp").unwrap_or(0);
                let hp_max = global_facts.get_int("player:hp_max").unwrap_or(20);
                let new_hp = (hp + *amount as i64).min(hp_max);
                total_healed += new_hp - hp;
                info!("FRE Bridge: Heal {} → HP {}/{}", amount, new_hp, hp_max);
                global_facts.set_global("player:hp", FactValue::Int(new_hp));
            }
            ItemEffect::PlayAudio { clip_path } => {
                audio::play_sound_full_path(audio, asset_server, clip_path);
            }
            ItemEffect::SpawnChildItem { item_id } => {
                spawn_child = Some(item_id.clone());
            }
            ItemEffect::SetFact { key, value } => {
                let fact_value = crate::core::item::fact_value_from_item_fact_value(value);
                global_facts.set_global(key, fact_value);
            }
        }
    }

    // Handle inventory mutation: consume or replace with child item
    let mut inventory = global_facts
        .get_string_list("player:inventory")
        .map(|s| s.to_vec())
        .unwrap_or_default();
    if index < inventory.len() {
        if let Some(child_id) = spawn_child {
            info!(
                "FRE Bridge: SpawnChildItem '{}' at index {}",
                child_id, index
            );
            inventory[index] = child_id;
        } else if let ItemType::Food {
            consumable: true, ..
        } = &item.item_type
        {
            info!("FRE Bridge: Consuming item at index {}", index);
            inventory.remove(index);
        }
        global_facts.set_global("player:inventory", FactValue::StringList(inventory));
    }
    total_healed
}

/// Default OnUse node name for each item type.
fn default_use_node(item_type: &crate::core::item::ItemType) -> &'static str {
    use crate::core::item::ItemType;
    match item_type {
        ItemType::Food { .. } => "OnUseFoodDefault",
        ItemType::Weapon { .. } => "OnUseWeaponDefault",
        ItemType::Armor { .. } => "OnUseArmorDefault",
        ItemType::KeyItem => "OnUseKeyItemDefault",
    }
}

/// Default OnCheck node name for each item type.
fn default_check_node(item_type: &crate::core::item::ItemType) -> &'static str {
    use crate::core::item::ItemType;
    match item_type {
        ItemType::Food { .. } => "OnCheckFoodDefault",
        ItemType::Weapon { .. } => "OnCheckWeaponDefault",
        ItemType::Armor { .. } => "OnCheckArmorDefault",
        ItemType::KeyItem => "OnCheckDefault",
    }
}

/// UseItem action: dispatch by item type, execute effects, prepare dialogue data.
///
/// Sets `mortar_path` and `action_param` on view local facts so the narration
/// sequence can start dialogue with proper UI state (like ACT does).
/// Does NOT start dialogue directly — the narration sequence handles that.
pub(super) fn execute_use_item(
    index_expr: &str,
    local_facts: &mut bevy_fact_rule_event::FactDatabase,
    global_facts: &mut bevy_fact_rule_event::LayeredFactDatabase,
    audio: &bevy_kira_audio::Audio,
    asset_server: &AssetServer,
    enum_registry: &EnumRegistry,
    item_registry: &crate::core::item::ItemRegistry,
) {
    use crate::core::item::ItemType;

    let Some(index) = resolve_index_expr(index_expr, local_facts, global_facts, enum_registry)
    else {
        return;
    };
    let Some(item_id) = get_inventory_item_id(index, global_facts) else {
        warn!("FRE Bridge: UseItem — no item at index {}", index);
        return;
    };
    let Some(item) = item_registry.get(&item_id) else {
        warn!("FRE Bridge: UseItem — item '{}' not in registry", item_id);
        return;
    };

    info!(
        "FRE Bridge: UseItem '{}' (type: {:?}) at index {}",
        item_id,
        std::mem::discriminant(&item.item_type),
        index
    );

    let mut actual_healed: i64 = 0;

    match &item.item_type {
        ItemType::Food { .. } => {
            actual_healed = apply_item_effects(item, index, global_facts, audio, asset_server);
        }
        ItemType::Weapon { .. } => {
            let old_weapon = global_facts
                .get_string("player:weapon")
                .unwrap_or_default()
                .to_string();
            global_facts.set_global("player:weapon", FactValue::String(item_id.clone()));
            let mut inventory = global_facts
                .get_string_list("player:inventory")
                .map(|s| s.to_vec())
                .unwrap_or_default();
            if index < inventory.len() {
                if old_weapon.is_empty() {
                    inventory.remove(index);
                } else {
                    inventory[index] = old_weapon;
                }
                global_facts.set_global("player:inventory", FactValue::StringList(inventory));
            }
            info!("FRE Bridge: Equipped weapon '{}'", item_id);
        }
        ItemType::Armor { .. } => {
            let old_armor = global_facts
                .get_string("player:armor")
                .unwrap_or_default()
                .to_string();
            global_facts.set_global("player:armor", FactValue::String(item_id.clone()));
            let mut inventory = global_facts
                .get_string_list("player:inventory")
                .map(|s| s.to_vec())
                .unwrap_or_default();
            if index < inventory.len() {
                if old_armor.is_empty() {
                    inventory.remove(index);
                } else {
                    inventory[index] = old_armor;
                }
                global_facts.set_global("player:inventory", FactValue::StringList(inventory));
            }
            info!("FRE Bridge: Equipped armor '{}'", item_id);
        }
        ItemType::KeyItem => {
            // No state change for key items
        }
    }

    // Compute mortar path and node for dialogue
    let default_node = default_use_node(&item.item_type);
    let (mortar_path, action_param) = if let Some(mortar) = &item.mortar {
        (mortar.as_str(), "OnUse")
    } else {
        ("items/_defaults.mortar", default_node)
    };

    // Set on view local facts so narration sequence can read $mortar_path / $action_param
    local_facts.set("mortar_path", FactValue::String(mortar_path.to_string()));
    local_facts.set("action_param", FactValue::String(action_param.to_string()));

    // Set item-specific data on global facts for mortar dialogue variables
    let locale_key = format!("{}:{}", item.locale.file, item.locale.name);
    set_item_dialogue_data(
        global_facts,
        ItemDialogueData {
            locale_key,
            description: item.description.clone(),
            heal_amount: actual_healed,
            item_value: compute_item_value(item),
        },
    );
}

/// Set item-specific data on global facts for mortar dialogue variables.
///
/// Does NOT start dialogue — only prepares `dialogue:item_*` facts so the
/// mortar script can access item name, heal amount, etc.
fn set_item_dialogue_data(
    global_facts: &mut bevy_fact_rule_event::LayeredFactDatabase,
    item_data: ItemDialogueData,
) {
    global_facts.set_local(
        fre_facts::DIALOGUE_ITEM_NAME,
        FactValue::String(item_data.locale_key),
    );
    global_facts.set_local(
        fre_facts::DIALOGUE_ITEM_DESCRIPTION,
        FactValue::String(item_data.description),
    );
    global_facts.set_local(
        fre_facts::DIALOGUE_ITEM_HEAL_AMOUNT,
        FactValue::Int(item_data.heal_amount),
    );
    global_facts.set_local(
        fre_facts::DIALOGUE_ITEM_VALUE,
        FactValue::Int(item_data.item_value),
    );
}

/// CheckItem action: start dialogue with OnCheck node, no state change.
pub(super) fn execute_check_item(
    index_expr: &str,
    local_facts: &bevy_fact_rule_event::FactDatabase,
    global_facts: &mut bevy_fact_rule_event::LayeredFactDatabase,
    enum_registry: &EnumRegistry,
    item_registry: &crate::core::item::ItemRegistry,
    dialogue_view_default: &str,
    dialogue_voice_default: &str,
) {
    let Some(index) = resolve_index_expr(index_expr, local_facts, global_facts, enum_registry)
    else {
        return;
    };
    let Some(item_id) = get_inventory_item_id(index, global_facts) else {
        warn!("FRE Bridge: CheckItem — no item at index {}", index);
        return;
    };
    let Some(item) = item_registry.get(&item_id) else {
        warn!("FRE Bridge: CheckItem — item '{}' not in registry", item_id);
        return;
    };

    info!("FRE Bridge: CheckItem '{}'", item_id);
    let default_node = default_check_node(&item.item_type);
    let locale_key = format!("{}:{}", item.locale.file, item.locale.name);
    start_item_dialogue(
        item,
        "OnCheck",
        default_node,
        global_facts,
        dialogue_view_default,
        dialogue_voice_default,
        ItemDialogueData {
            locale_key,
            description: item.description.clone(),
            heal_amount: 0,
            item_value: compute_item_value(item),
        },
    );
}

/// DropItem action: remove from inventory and start OnDrop dialogue.
/// KeyItem is non-droppable — this action is silently ignored.
pub(super) fn execute_drop_item(
    index_expr: &str,
    local_facts: &bevy_fact_rule_event::FactDatabase,
    global_facts: &mut bevy_fact_rule_event::LayeredFactDatabase,
    enum_registry: &EnumRegistry,
    item_registry: &crate::core::item::ItemRegistry,
    dialogue_view_default: &str,
    dialogue_voice_default: &str,
) {
    use crate::core::item::ItemType;

    let Some(index) = resolve_index_expr(index_expr, local_facts, global_facts, enum_registry)
    else {
        return;
    };
    let Some(item_id) = get_inventory_item_id(index, global_facts) else {
        warn!("FRE Bridge: DropItem — no item at index {}", index);
        return;
    };
    let Some(item) = item_registry.get(&item_id) else {
        warn!("FRE Bridge: DropItem — item '{}' not in registry", item_id);
        return;
    };

    if matches!(item.item_type, ItemType::KeyItem) {
        info!(
            "FRE Bridge: DropItem — '{}' is a KeyItem (non-droppable), ignoring",
            item_id
        );
        return;
    }

    info!("FRE Bridge: DropItem '{}' at index {}", item_id, index);
    let mut inventory = global_facts
        .get_string_list("player:inventory")
        .map(|s| s.to_vec())
        .unwrap_or_default();
    if index < inventory.len() {
        inventory.remove(index);
        global_facts.set_global("player:inventory", FactValue::StringList(inventory));
    }

    let locale_key = format!("{}:{}", item.locale.file, item.locale.name);
    start_item_dialogue(
        item,
        "OnDrop",
        "OnDropDefault",
        global_facts,
        dialogue_view_default,
        dialogue_voice_default,
        ItemDialogueData {
            locale_key,
            description: item.description.clone(),
            heal_amount: 0,
            item_value: compute_item_value(item),
        },
    );
}
