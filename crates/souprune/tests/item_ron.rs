//! Item `.item.ron` asset tests.
//!
//! `.item.ron` 物品资产测试。

#[path = "test_support.rs"]
mod test_support;

use std::collections::{HashMap, HashSet};

use souprune::{Item, ItemEffect, ItemRegistry, ItemType};

const ITEM_DIR: &str = "items";
const ITEM_SUFFIX: &str = ".item.ron";

fn item_files() -> Vec<String> {
    let files = test_support::list_project_files_with_suffix(ITEM_DIR, ITEM_SUFFIX);
    assert!(
        !files.is_empty(),
        "No .item.ron files found under projects/example_mod/items"
    );
    files
}

fn load_all_items() -> HashMap<String, Vec<Item>> {
    let mut map = HashMap::new();
    for relative in item_files() {
        let items: Vec<Item> = test_support::parse_project_ron(&relative);
        assert!(
            !items.is_empty(),
            "{} should contain at least one item definition",
            relative
        );
        map.insert(relative, items);
    }
    map
}

/// Ensure every `.item.ron` file parses and items have ids/names.
///
/// 确保每个 `.item.ron` 文件都能解析，并且物品拥有 ID 与名称。
#[test]
fn item_assets_deserialize() {
    for (relative, items) in load_all_items() {
        for item in items {
            assert!(
                !item.id.is_empty(),
                "item id should not be empty in {}",
                relative
            );
            assert!(
                !item.locate_name.is_empty(),
                "item locate_name should not be empty in {}",
                relative
            );
        }
    }
}

/// Check `SpawnChildItem` references point to existing IDs in the same file.
///
/// 检查 `SpawnChildItem` 引用的 ID 在同一文件内存在。
#[test]
fn item_spawn_child_references_exist() {
    for (relative, items) in load_all_items() {
        let ids: HashSet<&str> = items.iter().map(|item| item.id.as_str()).collect();
        for item in &items {
            if let ItemType::Food { effects, .. } = &item.item_type {
                for effect in effects {
                    if let ItemEffect::SpawnChildItem { item_id } = effect {
                        assert!(
                            ids.contains(item_id.as_str()),
                            "SpawnChildItem {} referenced by {} in {} should exist",
                            item_id,
                            item.id,
                            relative
                        );
                    }
                }
            }
        }
    }
}

/// Rehearse registry logic by inserting all items and verifying lookups.
///
/// 通过插入所有物品并验证查询来预演注册表逻辑。
#[test]
fn item_registry_roundtrip() {
    let mut registry = ItemRegistry::default();
    for (_, items) in load_all_items() {
        for item in items {
            registry.0.insert(item.id.clone(), item);
        }
    }
    assert!(
        !registry.0.is_empty(),
        "ItemRegistry should contain entries after loading"
    );
    for (id, item) in &registry.0 {
        let looked_up = registry
            .get(id)
            .unwrap_or_else(|| panic!("Failed to retrieve {} from registry", id));
        assert_eq!(
            looked_up.locate_name, item.locate_name,
            "registry returned a different item for id {id}"
        );
    }
}
