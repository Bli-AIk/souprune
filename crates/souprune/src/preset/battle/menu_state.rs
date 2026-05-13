//! Battle menu derived state ownership.
//!
//! battle 菜单派生状态的所有权边界。
//!
//! This module owns the runtime-only values that are derived from battle menu
//! selection and player inventory. FRE rules still own explicit menu commands
//! such as changing `depth`, `menu_context`, or `item_selection`; this module
//! only publishes the read-only display facts consumed by the View.
//!
//! 本模块拥有从 battle 菜单选择和玩家背包派生出的运行时值。FRE 规则仍然拥有
//! `depth`、`menu_context`、`item_selection` 等显式菜单命令；本模块只发布
//! 供 View 读取的展示用 facts。

use bevy::prelude::*;
use bevy_fact_rule_event::{FactReader, FactValue, LayeredFactDatabase};

use crate::core::view::ViewRoot;
use crate::core::view::components::ActiveView;
use crate::extra::mortar::MortarStringTable;

const ITEM_PAGE_SIZE: i64 = 4;
const ITEM_MODE_DEPTH: i64 = 1;
const ITEM_MENU_CONTEXT: i64 = 2;

/// Tracks the previous item-state projection to avoid unnecessary display-list rewrites.
///
/// 跟踪上一次物品状态投影，避免不必要地重写展示列表。
#[derive(Resource, Default)]
pub struct BattleMenuStateTracker {
    last_in_item_mode: bool,
    last_inventory: Vec<String>,
}

/// Synchronize battle menu derived state for the active View.
///
/// 同步激活 View 的 battle 菜单派生状态。
pub fn sync_battle_menu_state_system(
    mut tracker: ResMut<BattleMenuStateTracker>,
    mut view_roots: Query<&mut ViewRoot, With<ActiveView>>,
    layered_db: Res<LayeredFactDatabase>,
    mortar_strings: Res<MortarStringTable>,
) {
    let Ok(mut view_root) = view_roots.single_mut() else {
        return;
    };

    sync_battle_menu_state_for_view(&mut tracker, &mut view_root, &layered_db, &mortar_strings);
}

pub(crate) fn sync_battle_menu_state_for_view(
    tracker: &mut BattleMenuStateTracker,
    view_root: &mut ViewRoot,
    layered_db: &LayeredFactDatabase,
    mortar_strings: &MortarStringTable,
) {
    let depth = view_root.local_state().get_int("depth").unwrap_or(0);
    let menu_context = view_root.local_state().get_int("menu_context").unwrap_or(0);
    let in_item_mode = depth == ITEM_MODE_DEPTH && menu_context == ITEM_MENU_CONTEXT;

    if !in_item_mode {
        tracker.last_in_item_mode = false;
        return;
    }

    let inventory = layered_db
        .get_string_list("player:inventory")
        .map(|v| v.to_vec())
        .unwrap_or_default();

    if !tracker.last_in_item_mode || tracker.last_inventory != inventory {
        let display_names: Vec<String> = inventory
            .iter()
            .map(|item_id| resolve_item_display_name(item_id, layered_db, mortar_strings))
            .collect();

        view_root.set_local_value("item_display_names", FactValue::StringList(display_names));
        view_root.set_local_value("item_count", FactValue::Int(inventory.len() as i64));
    }

    let clamped_selection = clamped_item_selection(
        view_root
            .local_state()
            .get_int("item_selection")
            .unwrap_or(0),
        inventory.len(),
    );
    view_root.set_local_value("item_selection", FactValue::Int(clamped_selection));

    let page_count = item_page_count(inventory.len());
    let page = clamped_selection / ITEM_PAGE_SIZE + 1;
    view_root.set_local_value("item_page", FactValue::Int(page));
    view_root.set_local_value("item_page_count", FactValue::Int(page_count));

    tracker.last_in_item_mode = true;
    tracker.last_inventory = inventory;
}

fn clamped_item_selection(selection: i64, item_count: usize) -> i64 {
    let max_selection = item_count.saturating_sub(1) as i64;
    selection.clamp(0, max_selection)
}

fn item_page_count(item_count: usize) -> i64 {
    ((item_count as i64 + ITEM_PAGE_SIZE - 1) / ITEM_PAGE_SIZE).max(1)
}

/// Resolve an item's display name for the battle menu.
///
/// 解析物品在 battle 菜单中的显示名称。
fn resolve_item_display_name(
    item_id: &str,
    global_facts: &LayeredFactDatabase,
    mortar_strings: &MortarStringTable,
) -> String {
    if let Some(mortar_ns) = global_facts.get_string(&format!("items:{item_id}.mortar_ns")) {
        let battle_key = format!("{mortar_ns}:battle_name");
        if let Some(name) = mortar_strings.get(&battle_key) {
            return name.to_string();
        }
    }

    if let Some(locale_key) = global_facts.get_string(&format!("items:{item_id}.locale_key")) {
        return mortar_strings.resolve(locale_key).to_string();
    }

    format!("??? ({item_id})")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::view::ViewRoot;

    fn item_mode_view(item_selection: i64) -> ViewRoot {
        let mut view_root = ViewRoot::new("battle/view/undertale.view.ron".to_string());
        view_root.set_local_value("depth", FactValue::Int(ITEM_MODE_DEPTH));
        view_root.set_local_value("menu_context", FactValue::Int(ITEM_MENU_CONTEXT));
        view_root.set_local_value("item_selection", FactValue::Int(item_selection));
        view_root
    }

    #[test]
    fn clamps_stale_item_selection_to_last_inventory_item() {
        let mut tracker = BattleMenuStateTracker::default();
        let mut view_root = item_mode_view(8);
        let mut layered_db = LayeredFactDatabase::new();
        layered_db.set_global(
            "player:inventory",
            FactValue::StringList(vec![
                "monster_candy".to_string(),
                "dry_noodles".to_string(),
                "stick".to_string(),
                "bandage".to_string(),
                "cell_phone".to_string(),
            ]),
        );
        layered_db.set_global(
            "items:cell_phone.locale_key",
            FactValue::String("items:item.cell_phone".to_string()),
        );
        let mortar_strings = MortarStringTable::default();

        sync_battle_menu_state_for_view(&mut tracker, &mut view_root, &layered_db, &mortar_strings);

        assert_eq!(view_root.local_state().get_int("item_selection"), Some(4));
        assert_eq!(view_root.local_state().get_int("item_page"), Some(2));
        assert_eq!(view_root.local_state().get_int("item_page_count"), Some(2));
        assert_eq!(view_root.local_state().get_int("item_count"), Some(5));
    }

    #[test]
    fn keeps_empty_inventory_on_page_one() {
        let mut tracker = BattleMenuStateTracker::default();
        let mut view_root = item_mode_view(3);
        let mut layered_db = LayeredFactDatabase::new();
        layered_db.set_global("player:inventory", FactValue::StringList(Vec::new()));
        let mortar_strings = MortarStringTable::default();

        sync_battle_menu_state_for_view(&mut tracker, &mut view_root, &layered_db, &mortar_strings);

        assert_eq!(view_root.local_state().get_int("item_selection"), Some(0));
        assert_eq!(view_root.local_state().get_int("item_page"), Some(1));
        assert_eq!(view_root.local_state().get_int("item_page_count"), Some(1));
        assert_eq!(view_root.local_state().get_int("item_count"), Some(0));
    }
}
