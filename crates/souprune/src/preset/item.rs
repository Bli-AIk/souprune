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

/// Inject item metadata into FRE global facts so rules and View templates can
/// query item properties without depending on `ItemRegistry`.
fn inject_item_facts(registry: &ItemRegistry, facts: &mut LayeredFactDatabase) {
    for (id, item) in &registry.0 {
        let prefix = format!("items:{id}");

        // Locale key for mortar string resolution (e.g. "items:item.pie")
        let locale_key = format!("{}:{}", item.locale.file, item.locale.name);
        facts.set_global(
            format!("{prefix}.locale_key"),
            FactValue::String(locale_key),
        );

        // Description text
        facts.set_global(
            format!("{prefix}.description"),
            FactValue::String(item.description.clone()),
        );

        // Mortar file path (full path with .mortar extension)
        if let Some(mortar) = &item.mortar {
            facts.set_global(
                format!("{prefix}.mortar"),
                FactValue::String(mortar.clone()),
            );
            let ns = mortar.strip_suffix(".mortar").unwrap_or(mortar);
            facts.set_global(
                format!("{prefix}.mortar_ns"),
                FactValue::String(ns.to_string()),
            );
        }

        match &item.item_type {
            ItemType::Food {
                consumable,
                effects,
            } => {
                facts.set_global(
                    format!("{prefix}.type"),
                    FactValue::String("Food".to_string()),
                );
                facts.set_global(format!("{prefix}.consumable"), FactValue::Bool(*consumable));
                for effect in effects {
                    match effect {
                        ItemEffect::Heal { amount } => {
                            facts.set_global(
                                format!("{prefix}.heal"),
                                FactValue::Int(*amount as i64),
                            );
                        }
                        ItemEffect::PlayAudio { clip_path } => {
                            facts.set_global(
                                format!("{prefix}.use_audio"),
                                FactValue::String(clip_path.clone()),
                            );
                        }
                        ItemEffect::SpawnChildItem { item_id } => {
                            facts.set_global(
                                format!("{prefix}.child_item"),
                                FactValue::String(item_id.clone()),
                            );
                        }
                        ItemEffect::SetFact { key, value } => {
                            // SetFact effects are rare; apply them at load time.
                            facts.set_global(key, fact_value_from_item_fact_value(value));
                        }
                    }
                }
            }
            ItemType::Weapon { damage, .. } => {
                facts.set_global(
                    format!("{prefix}.type"),
                    FactValue::String("Weapon".to_string()),
                );
                facts.set_global(format!("{prefix}.damage"), FactValue::Int(*damage as i64));
            }
            ItemType::Armor { defense } => {
                facts.set_global(
                    format!("{prefix}.type"),
                    FactValue::String("Armor".to_string()),
                );
                facts.set_global(format!("{prefix}.defense"), FactValue::Int(*defense as i64));
            }
            ItemType::KeyItem => {
                facts.set_global(
                    format!("{prefix}.type"),
                    FactValue::String("KeyItem".to_string()),
                );
            }
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
