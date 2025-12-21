//! # item.rs
//!
//! # item.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! The `item` module implements the game's item system. It defines data structures for items, equipment, and effects, provides an asset loader for parsing `.item.ron` files, and maintains a central `ItemRegistry` resource for querying items by ID.
//!
//! `item` 模块实现了游戏的物品系统。它定义了物品、装备和效果的数据结构，提供了用于解析 `.item.ron` 文件的资产加载器，并维护了一个核心的 `ItemRegistry` 资源，以便通过 ID 查询物品。

use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, LoadContext, LoadedFolder};
use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::tasks::ConditionalSendFuture;
use serde::Deserialize;
use std::collections::HashMap;

pub struct ItemPlugin;

impl Plugin for ItemPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<ItemAsset>()
            .init_asset_loader::<ItemAssetLoader>()
            .init_resource::<ItemRegistry>()
            .add_systems(Startup, load_items_system)
            .add_systems(Update, sync_items_system);
    }
}

// --- Data Structures ---
//
// --- 数据结构 ---

#[derive(Asset, TypePath, Debug)]
pub struct ItemAsset(pub Vec<Item>);

#[derive(Debug, Clone, Deserialize, Reflect, PartialEq)]
pub struct ItemId(pub String);

#[derive(Debug, Clone, Deserialize, Reflect)]
pub struct Item {
    pub id: String,
    pub locate_name: String,
    pub locate_file: String,
    pub description: String,
    pub item_type: ItemType,
}

#[derive(Debug, Clone, Deserialize, Reflect)]
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

#[derive(Debug, Clone, Deserialize, Reflect)]
pub enum ItemEffect {
    Heal { amount: i32 },
    PlayAudio { clip_path: String },
    SpawnChildItem { item_id: String },
}

// --- Asset Loader ---
//
// --- 资产加载器 ---

#[derive(Default)]
pub struct ItemAssetLoader;

impl AssetLoader for ItemAssetLoader {
    type Asset = ItemAsset;
    type Settings = ();
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext,
    ) -> impl ConditionalSendFuture<Output = Result<Self::Asset, Self::Error>> {
        Box::pin(async move {
            let mut bytes = Vec::new();
            reader.read_to_end(&mut bytes).await?;
            // ron::de::from_bytes handles the decoding
            //
            // ron::de::from_bytes 处理解码
            let items: Vec<Item> = ron::de::from_bytes(&bytes)?;

            // Log for debugging
            //
            // 调试日志
            // info!("Loaded {} items from ron file", items.len());

            Ok(ItemAsset(items))
        })
    }

    fn extensions(&self) -> &[&str] {
        &["item.ron"]
    }
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
    // Start loading the "items" folder.
    // The MultiSourceAssetReader configured in main.rs will route this to
    // projects/<active_project>/items/ automatically.
    //
    // 开始加载 "items" 文件夹。
    // main.rs 中配置的 MultiSourceAssetReader 会自动将其路由到
    // projects/<active_project>/items/ 目录。

    info!("Starting to load items from folder 'items'...");
    let handle = asset_server.load_folder("items");
    commands.insert_resource(ItemFolderHandle(handle));
}

fn sync_items_system(
    mut events: MessageReader<AssetEvent<LoadedFolder>>,
    item_folder: Option<Res<ItemFolderHandle>>,
    loaded_folders: Res<Assets<LoadedFolder>>,
    item_assets: Res<Assets<ItemAsset>>,
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
            info!("Item folder loaded. Indexing items...");

            if let Some(folder) = loaded_folders.get(&folder_handle.0) {
                for handle in &folder.handles {
                    // Try to cast the untyped handle to an ItemAsset handle
                    // The ID matches, so we can try to look it up in the ItemAsset storage
                    // We interpret the untyped ID as an ItemAsset ID.
                    //
                    // 尝试将未类型化的句柄转换为 ItemAsset 句柄
                    // ID 匹配，因此我们可以尝试在 ItemAsset 存储中查找它
                    // 我们将未类型化的 ID 解释为 ItemAsset ID。
                    let id = handle.id().typed::<ItemAsset>();

                    if let Some(asset) = item_assets.get(id) {
                        for item in &asset.0 {
                            info!("Registered Item: [{}] {}", item.id, item.locate_name);
                            registry.0.insert(item.id.clone(), item.clone());
                        }
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
