//! Item action handlers for FRE bridge (UseItem, CheckItem, DropItem).
//!
//! FRE 桥接的物品动作处理器（UseItem、CheckItem、DropItem）。
//!
//! All item properties are read from FRE facts (injected by `inject_item_facts`)
//! — no `ItemRegistry` dependency.

use bevy::prelude::*;
use bevy_fact_rule_event::{CombinedFactReader, EnumRegistry, FactReader, FactValue};

use crate::core::fre_bridge::eval::evaluate_local_fact_value;
use crate::core::{audio, fre_facts};

// ── Item dialogue fact keys ─────────────────────────────────────────
// These are game-specific fact keys set by the preset item system
// and consumed by the dialogue system via MortarFactBindings.

/// Item locale key (e.g. "items:MONSTER_CANDY"), for mortar variable {item_name}.
pub const DIALOGUE_ITEM_NAME: &str = "dialogue:item_name";
/// Item description text, for mortar variable {item_description}.
pub const DIALOGUE_ITEM_DESCRIPTION: &str = "dialogue:item_description";
/// Actual heal amount (computed post-heal), for mortar variable {heal_amount}.
pub const DIALOGUE_ITEM_HEAL_AMOUNT: &str = "dialogue:item_heal_amount";
/// Item numeric value (Food heal / Weapon damage / Armor defense),
/// for mortar function get_item_value().
pub const DIALOGUE_ITEM_VALUE: &str = "dialogue:item_value";

/// Resolve an index expression (e.g., "$item_selection") to a usize index.
pub(crate) fn resolve_index_expr(
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

// --- Fact-based item property helpers ---

fn item_type(item_id: &str, facts: &bevy_fact_rule_event::LayeredFactDatabase) -> Option<String> {
    facts
        .get_string(&format!("items:{item_id}.type"))
        .map(|s| s.to_string())
}

fn item_mortar(item_id: &str, facts: &bevy_fact_rule_event::LayeredFactDatabase) -> Option<String> {
    facts
        .get_string(&format!("items:{item_id}.mortar"))
        .map(|s| s.to_string())
}

fn item_locale_key(
    item_id: &str,
    facts: &bevy_fact_rule_event::LayeredFactDatabase,
) -> Option<String> {
    facts
        .get_string(&format!("items:{item_id}.locale_key"))
        .map(|s| s.to_string())
}

fn item_description(item_id: &str, facts: &bevy_fact_rule_event::LayeredFactDatabase) -> String {
    facts
        .get_string(&format!("items:{item_id}.description"))
        .map(|s| s.to_string())
        .unwrap_or_default()
}

fn item_heal(item_id: &str, facts: &bevy_fact_rule_event::LayeredFactDatabase) -> Option<i64> {
    facts.get_int(&format!("items:{item_id}.heal"))
}

fn item_consumable(item_id: &str, facts: &bevy_fact_rule_event::LayeredFactDatabase) -> bool {
    facts
        .get_bool(&format!("items:{item_id}.consumable"))
        .unwrap_or(false)
}

fn item_use_audio(
    item_id: &str,
    facts: &bevy_fact_rule_event::LayeredFactDatabase,
) -> Option<String> {
    facts
        .get_string(&format!("items:{item_id}.use_audio"))
        .map(|s| s.to_string())
}

fn item_child_item(
    item_id: &str,
    facts: &bevy_fact_rule_event::LayeredFactDatabase,
) -> Option<String> {
    facts
        .get_string(&format!("items:{item_id}.child_item"))
        .map(|s| s.to_string())
}

/// Compute the stat value for an item (used by mortar function `get_item_value()`).
fn compute_item_value(
    item_id: &str,
    item_type_str: &str,
    facts: &bevy_fact_rule_event::LayeredFactDatabase,
) -> i64 {
    match item_type_str {
        "Food" => item_heal(item_id, facts).unwrap_or(0),
        "Weapon" => facts
            .get_int(&format!("items:{item_id}.damage"))
            .unwrap_or(0),
        "Armor" => facts
            .get_int(&format!("items:{item_id}.defense"))
            .unwrap_or(0),
        _ => 0,
    }
}

/// Default OnUse node name for each item type.
fn default_use_node(item_type_str: &str) -> &'static str {
    match item_type_str {
        "Food" => "OnUseFoodDefault",
        "Weapon" => "OnUseWeaponDefault",
        "Armor" => "OnUseArmorDefault",
        _ => "OnUseKeyItemDefault",
    }
}

/// Default OnCheck node name for each item type.
fn default_check_node(item_type_str: &str) -> &'static str {
    match item_type_str {
        "Food" => "OnCheckFoodDefault",
        "Weapon" => "OnCheckWeaponDefault",
        "Armor" => "OnCheckArmorDefault",
        _ => "OnCheckDefault",
    }
}

/// Pre-computed item data for mortar dialogue variables and functions.
struct ItemDialogueData {
    locale_key: String,
    description: String,
    heal_amount: i64,
    item_value: i64,
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
    global_facts.set_local(
        fre_facts::dialogue_channel_key(fre_facts::DIALOGUE_DEFAULT_CHANNEL, "has_typewriter"),
        FactValue::Bool(true),
    );
    global_facts.set_local(
        fre_facts::dialogue_channel_key(fre_facts::DIALOGUE_DEFAULT_CHANNEL, "has_focus"),
        FactValue::Bool(true),
    );
    if !dialogue_voice_default.is_empty() {
        global_facts.set_local(
            fre_facts::DIALOGUE_VOICE,
            FactValue::String(dialogue_voice_default.to_string()),
        );
        global_facts.set_local(
            fre_facts::dialogue_channel_key(fre_facts::DIALOGUE_DEFAULT_CHANNEL, "voice"),
            FactValue::String(dialogue_voice_default.to_string()),
        );
    }

    set_item_dialogue_data(global_facts, item_data);

    global_facts.set_local(fre_facts::DIALOGUE_PENDING_START, FactValue::Bool(true));
}

/// Set item-specific data on global facts for mortar dialogue variables.
fn set_item_dialogue_data(
    global_facts: &mut bevy_fact_rule_event::LayeredFactDatabase,
    item_data: ItemDialogueData,
) {
    global_facts.set_local(DIALOGUE_ITEM_NAME, FactValue::String(item_data.locale_key));
    global_facts.set_local(
        DIALOGUE_ITEM_DESCRIPTION,
        FactValue::String(item_data.description),
    );
    global_facts.set_local(
        DIALOGUE_ITEM_HEAL_AMOUNT,
        FactValue::Int(item_data.heal_amount),
    );
    global_facts.set_local(DIALOGUE_ITEM_VALUE, FactValue::Int(item_data.item_value));
}

/// Execute item effects for a Food item (heal, audio, child spawn).
/// Returns the actual amount of HP healed.
fn apply_food_effects(
    item_id: &str,
    index: usize,
    global_facts: &mut bevy_fact_rule_event::LayeredFactDatabase,
    audio: &bevy_kira_audio::Audio,
    asset_server: &AssetServer,
    audio_cache: &mut crate::core::audio::AudioSourceCache,
) -> i64 {
    let mut total_healed: i64 = 0;

    if let Some(heal_amount) = item_heal(item_id, global_facts) {
        let hp = global_facts.get_int("player:hp").unwrap_or(0);
        let hp_max = global_facts.get_int("player:hp_max").unwrap_or(20);
        let new_hp = (hp + heal_amount).min(hp_max);
        total_healed = new_hp - hp;
        info!(
            "FRE Bridge: Heal {} → HP {}/{}",
            heal_amount, new_hp, hp_max
        );
        global_facts.set_global("player:hp", FactValue::Int(new_hp));
    }

    if let Some(clip_path) = item_use_audio(item_id, global_facts) {
        audio::play_sound_full_path(audio, asset_server, audio_cache, &clip_path);
    }

    // Handle inventory mutation: consume or replace with child item
    let child_item = item_child_item(item_id, global_facts);
    let consumable = item_consumable(item_id, global_facts);

    let mut inventory = global_facts
        .get_string_list("player:inventory")
        .map(|s| s.to_vec())
        .unwrap_or_default();
    if index < inventory.len() {
        if let Some(child_id) = child_item {
            info!(
                "FRE Bridge: SpawnChildItem '{}' at index {}",
                child_id, index
            );
            inventory[index] = child_id;
        } else if consumable {
            info!("FRE Bridge: Consuming item at index {}", index);
            inventory.remove(index);
        }
        global_facts.set_global("player:inventory", FactValue::StringList(inventory));
    }
    total_healed
}

/// UseItem action: dispatch by item type, execute effects, prepare dialogue data.
///
/// When `start_dialogue` is true, starts dialogue through global facts directly
/// (for contexts like overworld backpack where no narration sequence runs).
/// When false (default), only sets local_facts for the narration sequence to read.
pub(crate) fn execute_use_item(
    index_expr: &str,
    local_facts: &mut bevy_fact_rule_event::FactDatabase,
    global_facts: &mut bevy_fact_rule_event::LayeredFactDatabase,
    audio: &bevy_kira_audio::Audio,
    asset_server: &AssetServer,
    audio_cache: &mut crate::core::audio::AudioSourceCache,
    enum_registry: &EnumRegistry,
    start_dialogue: bool,
    dialogue_view_default: &str,
    dialogue_voice_default: &str,
) {
    let Some(index) = resolve_index_expr(index_expr, local_facts, global_facts, enum_registry)
    else {
        return;
    };
    let Some(item_id) = get_inventory_item_id(index, global_facts) else {
        warn!("FRE Bridge: UseItem — no item at index {}", index);
        return;
    };
    let Some(type_str) = item_type(&item_id, global_facts) else {
        warn!("FRE Bridge: UseItem — item '{}' has no type fact", item_id);
        return;
    };

    info!(
        "FRE Bridge: UseItem '{}' (type: {}) at index {}",
        item_id, type_str, index
    );

    let mut actual_healed: i64 = 0;

    match type_str.as_str() {
        "Food" => {
            actual_healed = apply_food_effects(
                &item_id,
                index,
                global_facts,
                audio,
                asset_server,
                audio_cache,
            );
        }
        "Weapon" => {
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
        "Armor" => {
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
        _ => {
            // KeyItem or unknown — no state change
        }
    }

    // Compute mortar path and node for dialogue
    let mortar = item_mortar(&item_id, global_facts);
    let default_node = default_use_node(&type_str);
    let (mortar_path, action_param) = if let Some(ref mortar) = mortar {
        (mortar.as_str(), "OnUse")
    } else {
        ("items/_defaults.mortar", default_node)
    };

    // Set on view local facts (used by battle narration sequence)
    local_facts.set("mortar_path", FactValue::String(mortar_path.to_string()));
    local_facts.set("action_param", FactValue::String(action_param.to_string()));

    let locale_key = item_locale_key(&item_id, global_facts).unwrap_or_default();
    let description = item_description(&item_id, global_facts);
    let item_data = ItemDialogueData {
        locale_key,
        description,
        heal_amount: actual_healed,
        item_value: compute_item_value(&item_id, &type_str, global_facts),
    };

    if start_dialogue {
        start_item_dialogue_with_path(
            mortar_path,
            action_param,
            global_facts,
            dialogue_view_default,
            dialogue_voice_default,
            item_data,
        );
    } else {
        set_item_dialogue_data(global_facts, item_data);
    }
}

/// CheckItem action: start dialogue with OnCheck node, no state change.
pub(crate) fn execute_check_item(
    index_expr: &str,
    local_facts: &bevy_fact_rule_event::FactDatabase,
    global_facts: &mut bevy_fact_rule_event::LayeredFactDatabase,
    enum_registry: &EnumRegistry,
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
    let Some(type_str) = item_type(&item_id, global_facts) else {
        warn!(
            "FRE Bridge: CheckItem — item '{}' has no type fact",
            item_id
        );
        return;
    };

    info!("FRE Bridge: CheckItem '{}'", item_id);
    let default_node = default_check_node(&type_str);
    let mortar = item_mortar(&item_id, global_facts);
    let (mortar_path, node) = if let Some(ref mortar) = mortar {
        (mortar.as_str(), "OnCheck")
    } else {
        ("items/_defaults.mortar", default_node)
    };

    let locale_key = item_locale_key(&item_id, global_facts).unwrap_or_default();
    let description = item_description(&item_id, global_facts);
    start_item_dialogue_with_path(
        mortar_path,
        node,
        global_facts,
        dialogue_view_default,
        dialogue_voice_default,
        ItemDialogueData {
            locale_key,
            description,
            heal_amount: 0,
            item_value: compute_item_value(&item_id, &type_str, global_facts),
        },
    );
}

/// DropItem action: remove from inventory and start OnDrop dialogue.
/// KeyItem is non-droppable — this action is silently ignored.
pub(crate) fn execute_drop_item(
    index_expr: &str,
    local_facts: &bevy_fact_rule_event::FactDatabase,
    global_facts: &mut bevy_fact_rule_event::LayeredFactDatabase,
    enum_registry: &EnumRegistry,
    dialogue_view_default: &str,
    dialogue_voice_default: &str,
) {
    let Some(index) = resolve_index_expr(index_expr, local_facts, global_facts, enum_registry)
    else {
        return;
    };
    let Some(item_id) = get_inventory_item_id(index, global_facts) else {
        warn!("FRE Bridge: DropItem — no item at index {}", index);
        return;
    };
    let Some(type_str) = item_type(&item_id, global_facts) else {
        warn!("FRE Bridge: DropItem — item '{}' has no type fact", item_id);
        return;
    };

    if type_str == "KeyItem" {
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

    let mortar = item_mortar(&item_id, global_facts);
    let (mortar_path, node) = if let Some(ref mortar) = mortar {
        (mortar.as_str(), "OnDrop")
    } else {
        ("items/_defaults.mortar", "OnDropDefault")
    };

    let locale_key = item_locale_key(&item_id, global_facts).unwrap_or_default();
    let description = item_description(&item_id, global_facts);
    start_item_dialogue_with_path(
        mortar_path,
        node,
        global_facts,
        dialogue_view_default,
        dialogue_voice_default,
        ItemDialogueData {
            locale_key,
            description,
            heal_amount: 0,
            item_value: compute_item_value(&item_id, &type_str, global_facts),
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_fact_rule_event::LayeredFactDatabase;

    #[test]
    fn item_dialogue_start_refreshes_default_channel_focus() {
        let mut facts = LayeredFactDatabase::new();
        let dialogue_view = "overworld/view/dialogue.view.ron";
        facts.set_local(
            fre_facts::dialogue_channel_key(fre_facts::DIALOGUE_DEFAULT_CHANNEL, "has_focus"),
            FactValue::Bool(false),
        );

        start_item_dialogue_with_path(
            "items/monster_candy.mortar",
            "OnUse",
            &mut facts,
            dialogue_view,
            "assets/audios/voice/voice_monster.wav",
            ItemDialogueData {
                locale_key: "items:MONSTER_CANDY".to_string(),
                description: "Monster Candy".to_string(),
                heal_amount: 0,
                item_value: 10,
            },
        );

        assert_eq!(
            facts.get_bool(&fre_facts::dialogue_channel_key(
                fre_facts::DIALOGUE_DEFAULT_CHANNEL,
                "has_focus"
            )),
            Some(true)
        );
        assert_eq!(
            facts.get_string(fre_facts::DIALOGUE_PENDING_VIEW),
            Some(dialogue_view)
        );
        assert_eq!(
            facts.get_bool(fre_facts::DIALOGUE_PENDING_START),
            Some(true)
        );
    }
}
