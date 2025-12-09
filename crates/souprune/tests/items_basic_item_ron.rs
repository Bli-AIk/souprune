//! Item list RON tests for the sample backpack data.
//!
//! 示例背包物品列表 RON 测试。

#[path = "test_support.rs"]
mod test_support;

use std::collections::HashSet;

use souprune::{Item, ItemEffect, ItemRegistry, ItemType};

/// Ensure the item array parses correctly.
///
/// 确保物品数组可以正确解析。
#[test]
fn basic_item_asset_deserializes() {
    let items: Vec<Item> = test_support::parse_project_ron("items/basic.item.ron");
    assert!(!items.is_empty(), "items list should not be empty");
    assert!(
        items.iter().any(|item| item.id == "monster_candy"),
        "monster_candy must exist for the tutorial inventory"
    );
}

/// Check that every SpawnChildItem effect references an item defined in the same file.
///
/// 检查每一个 SpawnChildItem 效果都引用了同一文件中定义的物品。
#[test]
fn basic_item_spawn_child_references_exist() {
    let items: Vec<Item> = test_support::parse_project_ron("items/basic.item.ron");
    let ids: HashSet<&str> = items.iter().map(|item| item.id.as_str()).collect();

    for item in &items {
        if let ItemType::Food { effects, .. } = &item.item_type {
            for effect in effects {
                if let ItemEffect::SpawnChildItem { item_id } = effect {
                    assert!(
                        ids.contains(item_id.as_str()),
                        "SpawnChildItem reference {} should exist",
                        item_id
                    );
                }
            }
        }
    }
}

/// Rehearse registry logic by inserting items and inspecting key gameplay stats.
///
/// 通过将物品插入注册表并检查关键属性来预演逻辑。
#[test]
fn basic_item_registry_behaves() {
    let items: Vec<Item> = test_support::parse_project_ron("items/basic.item.ron");
    let mut registry = ItemRegistry::default();
    for item in items {
        registry.0.insert(item.id.clone(), item);
    }

    let candy = registry
        .get("monster_candy")
        .expect("monster_candy should be registered");
    if let ItemType::Food {
        consumable,
        effects,
    } = &candy.item_type
    {
        assert!(*consumable);
        assert!(
            effects
                .iter()
                .any(|effect| matches!(effect, ItemEffect::Heal { amount } if *amount == 10)),
            "monster_candy should heal 10 HP"
        );
    } else {
        panic!("monster_candy must be a Food item");
    }

    let glove = registry
        .get("tough_glove")
        .expect("tough_glove should be registered");
    if let ItemType::Weapon { damage, .. } = &glove.item_type {
        assert!(*damage >= 0);
    } else {
        panic!("tough_glove must be a Weapon");
    }
}
