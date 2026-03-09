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
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::definition::LocaleInfo;

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

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub struct Item {
    pub id: String,
    #[serde(default)]
    pub locale: LocaleInfo,
    pub description: String,
    pub item_type: ItemType,
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub enum ItemType {
    Food {
        #[serde(default)]
        consumable: bool,
        #[serde(default)]
        effects: Vec<ItemEffect>,
    },
    Weapon {
        #[serde(default)]
        damage: i32,
        #[serde(default)]
        on_hit_effects: Vec<ItemEffect>,
    },
    Armor {
        #[serde(default)]
        defense: i32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
pub enum ItemEffect {
    Heal { amount: i32 },
    PlayAudio { clip_path: String },
    SpawnChildItem { item_id: String },
}

// --- Asset ---

/// A list of items loaded from a `.items.ron` file.
///
/// 从 `.items.ron` 文件加载的物品列表。
#[derive(Asset, TypePath, Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ItemListAsset(pub Vec<Item>);

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
            for item in &item_list.0 {
                info!("Registered Item: [{}] {}", item.id, item.locale.name);
                registry.0.insert(item.id.clone(), item.clone());
            }
        }
        info!(
            "Item Registry initialization complete. Total items: {}",
            registry.0.len()
        );
    }
}
