//! # item.rs
//!
//! # item.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! The `item` module implements the game's item system. It defines typed data structures
//! for items, equipment, and effects, loads them from `.items.ron` files, and maintains
//! a central `ItemRegistry` resource for querying items by ID.
//!
//! `item` 模块实现了游戏的物品系统。定义了物品、装备和效果的类型化数据结构，
//! 从 `.items.ron` 文件加载，并维护 `ItemRegistry` 资源以便通过 ID 查询物品。

use bevy::asset::LoadedFolder;
use bevy::prelude::*;
use bevy_fact_rule_event::{FactValue, LayeredFactDatabase};
use serde::{Deserialize, Serialize};
use souprune_schema::item::ItemListAsset as SchemaItemListAsset;
pub use souprune_schema::item::{Item, ItemEffect, ItemFactValue, ItemType};
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

pub struct ItemPlugin;

impl Plugin for ItemPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ItemRegistry>()
            .init_asset::<ItemListAsset>()
            .register_asset_loader(
                crate::core::ron_loader::RonAssetLoader::<ItemListAsset>::new(&["items.ron"]),
            )
            .add_systems(Startup, load_items_system)
            .add_systems(Update, sync_items_system);
    }
}

// --- Data Structures ---

#[derive(Debug, Clone, Deserialize, Serialize, Reflect, PartialEq)]
pub struct ItemId(pub String);

pub fn fact_value_from_item_fact_value(value: &ItemFactValue) -> FactValue {
    match value {
        ItemFactValue::Int(i) => FactValue::Int(*i),
        ItemFactValue::Float(f) => FactValue::Float(*f),
        ItemFactValue::Bool(b) => FactValue::Bool(*b),
        ItemFactValue::String(s) => FactValue::String(s.clone()),
    }
}

// --- Asset ---

/// A list of items loaded from a `.items.ron` file.
///
/// 从 `.items.ron` 文件加载的物品列表。
#[derive(Asset, TypePath, Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ItemListAsset(pub SchemaItemListAsset);

impl Deref for ItemListAsset {
    type Target = SchemaItemListAsset;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ItemListAsset {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl ItemListAsset {
    pub fn items(&self) -> &[Item] {
        &self.0.0
    }
}

// --- Registry & Loading Logic ---

#[derive(Resource, Default)]
pub struct ItemRegistry(pub HashMap<String, Item>);

impl ItemRegistry {
    pub fn get(&self, id: &str) -> Option<&Item> {
        self.0.get(id)
    }
}

#[derive(Resource)]
struct ItemFolderHandle(Handle<LoadedFolder>);

fn load_items_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    info!("Starting to load items from folder 'data/items'...");
    let handle = asset_server.load_folder("data/items");
    commands.insert_resource(ItemFolderHandle(handle));
}

fn sync_items_system(
    mut events: MessageReader<AssetEvent<LoadedFolder>>,
    item_folder: Option<Res<ItemFolderHandle>>,
    loaded_folders: Res<Assets<LoadedFolder>>,
    item_assets: Res<Assets<ItemListAsset>>,
    mut registry: ResMut<ItemRegistry>,
    mut global_facts: ResMut<LayeredFactDatabase>,
) {
    let Some(folder_handle) = item_folder else {
        return;
    };

    for event in events.read() {
        let AssetEvent::LoadedWithDependencies { id } = event else {
            continue;
        };
        if *id != folder_handle.0.id() {
            continue;
        }

        info!("Item folder loaded. Indexing items from .items.ron files...");

        let Some(folder) = loaded_folders.get(&folder_handle.0) else {
            continue;
        };
        for handle in &folder.handles {
            let id = handle.id().typed::<ItemListAsset>();
            let Some(item_list) = item_assets.get(id) else {
                continue;
            };
            for item in item_list.items() {
                info!("Registered Item: [{}] {}", item.id, item.locale.name);
                registry.0.insert(item.id.clone(), item.clone());
            }
        }

        inject_item_facts(&registry, &mut global_facts);

        info!(
            "Item Registry initialization complete. Total items: {}",
            registry.0.len()
        );
    }
}

/// Inject item metadata into FRE global facts so rules can query item properties.
fn inject_item_facts(registry: &ItemRegistry, facts: &mut LayeredFactDatabase) {
    for (id, item) in &registry.0 {
        let prefix = format!("items:{id}");
        let (type_name, extra_facts) = match &item.item_type {
            ItemType::Food {
                consumable,
                effects,
            } => {
                let heal = effects.iter().find_map(|e| match e {
                    ItemEffect::Heal { amount } => Some(*amount as i64),
                    _ => None,
                });
                let mut facts = vec![("consumable", FactValue::Bool(*consumable))];
                if let Some(heal) = heal {
                    facts.push(("heal", FactValue::Int(heal)));
                }
                ("Food", facts)
            }
            ItemType::Weapon { damage, .. } => {
                ("Weapon", vec![("damage", FactValue::Int(*damage as i64))])
            }
            ItemType::Armor { defense } => {
                ("Armor", vec![("defense", FactValue::Int(*defense as i64))])
            }
            ItemType::KeyItem => ("KeyItem", vec![]),
        };
        facts.set_global(
            format!("{prefix}.type"),
            FactValue::String(type_name.to_string()),
        );
        for (suffix, value) in extra_facts {
            facts.set_global(format!("{prefix}.{suffix}"), value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_item_list_from_shared_schema() {
        let ron = r#"[
            (
                id: "pie",
                locale: (
                    name: "item.pie",
                    file: "items",
                ),
                description: "A schema-backed pie.",
                mortar: Some("items/pie.mortar"),
                item_type: Food(
                    consumable: true,
                    effects: [
                        Heal(amount: 30),
                    ],
                ),
            ),
        ]"#;

        let asset: ItemListAsset = ron::from_str(ron).expect("item list");

        assert_eq!(asset.items().len(), 1);
        assert_eq!(asset.items()[0].id, "pie");
        assert!(matches!(
            asset.items()[0].item_type,
            ItemType::Food {
                consumable: true,
                ..
            }
        ));
    }

    #[test]
    fn converts_schema_item_fact_value_to_fre_value() {
        assert_eq!(
            fact_value_from_item_fact_value(&ItemFactValue::String("pie".to_string())),
            FactValue::String("pie".to_string())
        );
        assert_eq!(
            fact_value_from_item_fact_value(&ItemFactValue::Int(30)),
            FactValue::Int(30)
        );
    }
}
