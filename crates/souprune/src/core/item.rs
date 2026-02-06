//! # item.rs
//!
//! # item.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! The `item` module implements the game's item system. It defines data structures for items,
//! equipment, and effects, loads item data from `.fre.ron` files, and maintains a central
//! `ItemRegistry` resource for querying items by ID.
//!
//! `item` 模块实现了游戏的物品系统。它定义了物品、装备和效果的数据结构，
//! 从 `.fre.ron` 文件加载物品数据，并维护了一个核心的 `ItemRegistry` 资源，以便通过 ID 查询物品。

use bevy::asset::LoadedFolder;
use bevy::prelude::*;
use bevy_fact_rule_event::FreAsset;
use serde::Deserialize;
use std::collections::HashMap;

pub struct ItemPlugin;

impl Plugin for ItemPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ItemRegistry>()
            .add_systems(Startup, load_items_system)
            .add_systems(Update, sync_items_system);
    }
}

// --- Data Structures ---
//
// --- 数据结构 ---

#[derive(Debug, Clone, Deserialize, Reflect, PartialEq)]
pub struct ItemId(pub String);

#[derive(Debug, Clone, Reflect)]
pub struct Item {
    pub id: String,
    pub locate_name: String,
    pub locate_file: String,
    pub description: String,
    pub item_type: ItemType,
}

#[derive(Debug, Clone, Reflect)]
pub enum ItemType {
    Food {
        consumable: bool,
        effects: Vec<ItemEffect>,
    },
    Weapon {
        damage: i32,
        on_hit_effects: Vec<ItemEffect>,
    },
    Armor {
        defense: i32,
    },
}

#[derive(Debug, Clone, Reflect)]
pub enum ItemEffect {
    Heal { amount: i32 },
    PlayAudio { clip_path: String },
    SpawnChildItem { item_id: String },
}

// --- Registry & Loading Logic ---
//
// --- 注册表与加载逻辑 ---

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
    // Start loading the "shared/items" folder.
    // The MultiSourceAssetReader configured in main.rs will route this to
    // projects/<active_project>/shared/items/ automatically.
    //
    // 开始加载 "shared/items" 文件夹。
    // main.rs 中配置的 MultiSourceAssetReader 会自动将其路由到
    // projects/<active_project>/shared/items/ 目录。

    info!("Starting to load items from folder 'items'...");
    let handle = asset_server.load_folder("shared/items");
    commands.insert_resource(ItemFolderHandle(handle));
}

fn sync_items_system(
    mut events: MessageReader<AssetEvent<LoadedFolder>>,
    item_folder: Option<Res<ItemFolderHandle>>,
    loaded_folders: Res<Assets<LoadedFolder>>,
    fre_assets: Res<Assets<FreAsset>>,
    mut registry: ResMut<ItemRegistry>,
) {
    let Some(folder_handle) = item_folder else {
        return;
    };

    for event in events.read() {
        // Wait until the folder is fully loaded (LoadedWithDependencies ensures children are ready)
        //
        // 等待直到文件夹完全加载（LoadedWithDependencies 确保子项已准备好）
        if let AssetEvent::LoadedWithDependencies { id } = event
            && *id == folder_handle.0.id()
        {
            info!("Item folder loaded. Indexing items from FRE files...");

            if let Some(folder) = loaded_folders.get(&folder_handle.0) {
                for handle in &folder.handles {
                    // Try to interpret handle as FreAsset
                    // 尝试将句柄解释为 FreAsset
                    let id = handle.id().typed::<FreAsset>();

                    if let Some(fre_asset) = fre_assets.get(id) {
                        // Parse items from FRE facts
                        // 从 FRE facts 解析物品
                        parse_items_from_fre(fre_asset, &mut registry);
                    }
                }
            }
            info!(
                "Item Registry initialization complete. Total items: {}",
                registry.0.len()
            );
        }
    }
}

/// Parse items from FRE asset facts into ItemRegistry
///
/// 从 FRE 资产 facts 解析物品到 ItemRegistry
fn parse_items_from_fre(fre_asset: &FreAsset, registry: &mut ItemRegistry) {
    use bevy_fact_rule_event::FactValueDef;

    // Get item registry list
    // 获取物品注册表列表
    let item_ids: Vec<String> =
        if let Some(FactValueDef::StringList(ids)) = fre_asset.facts.get("items.registry") {
            ids.clone()
        } else {
            warn!("No items.registry found in FRE file");
            return;
        };

    for item_id in item_ids {
        // Helper to get string fact
        // 获取字符串 fact 的辅助函数
        let get_str = |suffix: &str| -> String {
            let key = format!("items.{}.{}", item_id, suffix);
            match fre_asset.facts.get(&key) {
                Some(FactValueDef::String(s)) => s.clone(),
                _ => String::new(),
            }
        };

        // Helper to get int fact
        // 获取整数 fact 的辅助函数
        let get_int = |suffix: &str| -> i32 {
            let key = format!("items.{}.{}", item_id, suffix);
            match fre_asset.facts.get(&key) {
                Some(FactValueDef::Int(i)) => *i as i32,
                _ => 0,
            }
        };

        // Helper to get bool fact
        // 获取布尔 fact 的辅助函数
        let get_bool = |suffix: &str| -> bool {
            let key = format!("items.{}.{}", item_id, suffix);
            match fre_asset.facts.get(&key) {
                Some(FactValueDef::Bool(b)) => *b,
                _ => false,
            }
        };

        let locate_name = get_str("locate_name");
        let locate_file = get_str("locate_file");
        let description = get_str("description");
        let item_type_str = get_str("type");

        let item_type = match item_type_str.as_str() {
            "Food" => {
                let mut effects = Vec::new();

                // Check for heal effect
                let heal = get_int("heal");
                if heal > 0 {
                    effects.push(ItemEffect::Heal { amount: heal });
                }

                // Check for audio effect
                let audio = get_str("audio");
                if !audio.is_empty() {
                    effects.push(ItemEffect::PlayAudio { clip_path: audio });
                }

                // Check for spawn child effect
                let spawn_child = get_str("spawn_child");
                if !spawn_child.is_empty() {
                    effects.push(ItemEffect::SpawnChildItem {
                        item_id: spawn_child,
                    });
                }

                ItemType::Food {
                    consumable: get_bool("consumable"),
                    effects,
                }
            }
            "Weapon" => {
                let mut on_hit_effects = Vec::new();

                // Check for audio effect
                let audio = get_str("audio");
                if !audio.is_empty() {
                    on_hit_effects.push(ItemEffect::PlayAudio { clip_path: audio });
                }

                ItemType::Weapon {
                    damage: get_int("damage"),
                    on_hit_effects,
                }
            }
            "Armor" => ItemType::Armor {
                defense: get_int("defense"),
            },
            _ => {
                warn!(
                    "Unknown item type '{}' for item '{}'",
                    item_type_str, item_id
                );
                continue;
            }
        };

        let item = Item {
            id: item_id.clone(),
            locate_name,
            locate_file,
            description,
            item_type,
        };

        info!("Registered Item: [{}] {}", item.id, item.locate_name);
        registry.0.insert(item_id, item);
    }
}
