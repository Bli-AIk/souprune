//! # save.rs
//!
//! # save.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This module implements the save/load system for the game. It uses TOML serialization
//! to persist game state directly from/to the LayeredFactDatabase.
//!
//! 本模块实现游戏的存档/读取系统。使用 TOML 序列化直接从/到 LayeredFactDatabase
//! 持久化游戏状态。

use bevy::prelude::*;
use bevy_fact_rule_event::LayeredFactDatabase;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

pub struct SavePlugin;

impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SaveGameEvent>()
            .add_message::<LoadGameEvent>()
            .add_message::<SaveCompleteEvent>()
            .add_message::<LoadCompleteEvent>()
            .init_resource::<SaveConfig>()
            .add_systems(Update, (handle_save_game_system, handle_load_game_system));
    }
}

// === Events ===
// === 事件 ===

/// Event to trigger a game save.
///
/// 触发游戏保存的事件。
#[derive(bevy::ecs::message::Message, Clone)]
pub struct SaveGameEvent {
    /// Save slot identifier (1-based index or custom name).
    ///
    /// 存档槽标识符（1 基索引或自定义名称）。
    pub slot: SaveSlot,
}

/// Event to trigger a game load.
///
/// 触发游戏加载的事件。
#[derive(bevy::ecs::message::Message, Clone)]
pub struct LoadGameEvent {
    /// Save slot to load from.
    ///
    /// 要加载的存档槽。
    pub slot: SaveSlot,
}

/// Event emitted when save is complete.
///
/// 存档完成时发出的事件。
#[derive(bevy::ecs::message::Message, Clone)]
pub struct SaveCompleteEvent {
    pub success: bool,
    pub slot: SaveSlot,
    pub error: Option<String>,
}

/// Event emitted when load is complete.
///
/// 加载完成时发出的事件。
#[derive(bevy::ecs::message::Message, Clone)]
pub struct LoadCompleteEvent {
    pub success: bool,
    pub slot: SaveSlot,
    pub error: Option<String>,
}

/// Save slot identifier.
///
/// 存档槽标识符。
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum SaveSlot {
    /// Numbered slot (1, 2, 3, etc.)
    ///
    /// 编号槽位（1、2、3 等）
    Numbered(u32),
    /// Named slot (for special saves like autosave)
    ///
    /// 命名槽位（用于特殊存档如自动存档）
    Named(String),
}

impl SaveSlot {
    pub fn filename(&self) -> String {
        match self {
            SaveSlot::Numbered(n) => format!("save_{}.toml", n),
            SaveSlot::Named(name) => format!("{}.toml", name),
        }
    }
}

impl Default for SaveSlot {
    fn default() -> Self {
        SaveSlot::Numbered(1)
    }
}

// === Configuration ===
// === 配置 ===

/// Configuration for the save system.
///
/// 存档系统配置。
#[derive(Resource)]
pub struct SaveConfig {
    /// Directory where save files are stored.
    ///
    /// 存档文件存储目录。
    pub save_directory: PathBuf,
    /// Maximum number of save slots.
    ///
    /// 最大存档槽数量。
    pub max_slots: u32,
}

impl Default for SaveConfig {
    fn default() -> Self {
        Self {
            save_directory: PathBuf::from("saves"),
            max_slots: 3,
        }
    }
}

// === Save Data Structures ===
// === 存档数据结构 ===

/// Marker component for entities that should be saved.
///
/// 标记组件，表示实体应被保存。
#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct Saveable;

/// The main save data structure containing all persistent game state.
/// Data is read directly from LayeredFactDatabase.
///
/// 主存档数据结构，包含所有需要持久化的游戏状态。
/// 数据直接从 LayeredFactDatabase 读取。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveData {
    /// Save file version for migration support.
    ///
    /// 存档文件版本，用于迁移支持。
    pub version: u32,
    /// Timestamp when the save was created.
    ///
    /// 存档创建时间戳。
    pub timestamp: String,
    /// Player-related save data.
    ///
    /// 玩家相关存档数据。
    pub player: PlayerSaveData,
    /// Current game progress/state.
    ///
    /// 当前游戏进度/状态。
    pub progress: ProgressSaveData,
}

impl Default for SaveData {
    fn default() -> Self {
        Self {
            version: SAVE_VERSION,
            timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            player: PlayerSaveData::default(),
            progress: ProgressSaveData::default(),
        }
    }
}

/// Current save file version.
///
/// 当前存档文件版本。
pub const SAVE_VERSION: u32 = 1;

/// Player-specific save data, read from LayeredFactDatabase.
///
/// 玩家特定存档数据，从 LayeredFactDatabase 读取。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlayerSaveData {
    /// Player name.
    ///
    /// 玩家名称。
    pub name: String,
    /// Level data.
    ///
    /// 等级数据。
    pub level: LevelData,
    /// Health data.
    ///
    /// 生命值数据。
    pub health: HealthData,
    /// Combat stats.
    ///
    /// 战斗属性。
    pub stats: StatsData,
    /// Gold amount.
    ///
    /// 金币数量。
    pub gold: usize,
    /// Equipment.
    ///
    /// 装备。
    pub equipment: EquipmentData,
    /// Inventory items.
    ///
    /// 物品栏。
    pub inventory: InventoryData,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LevelData {
    pub lv: usize,
    pub exp: usize,
    pub next_exp: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HealthData {
    pub current: usize,
    pub max: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StatsData {
    pub attack: usize,
    pub defense: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EquipmentData {
    pub weapon: String,
    pub armor: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InventoryData {
    pub items: Vec<String>,
    pub capacity: usize,
}

/// Game progress data (map location, flags, etc.).
///
/// 游戏进度数据（地图位置、标记等）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressSaveData {
    /// Current map path.
    ///
    /// 当前地图路径。
    pub current_map: String,
    /// Player position on the map.
    ///
    /// 玩家在地图上的位置。
    pub position: (f32, f32),
    /// Game flags (story progress, choices, etc.).
    ///
    /// 游戏标记（剧情进度、选择等）。
    pub flags: std::collections::HashMap<String, bool>,
}

impl Default for ProgressSaveData {
    fn default() -> Self {
        Self {
            current_map: "overworld/levels/ruins/ruins_3.tmx".to_string(),
            position: (0.0, 0.0),
            flags: std::collections::HashMap::new(),
        }
    }
}

// === Systems ===
// === 系统 ===

/// System to handle save game events.
/// Reads player data directly from LayeredFactDatabase.
///
/// 处理保存游戏事件的系统。
/// 直接从 LayeredFactDatabase 读取玩家数据。
fn handle_save_game_system(
    mut events: MessageReader<SaveGameEvent>,
    mut complete_events: MessageWriter<SaveCompleteEvent>,
    save_config: Res<SaveConfig>,
    layered_db: Res<LayeredFactDatabase>,
    player_query: Query<&Transform, With<crate::core::data::MainPlayer>>,
    souprune_config: Res<crate::config::SoupruneConfig>,
) {
    for event in events.read() {
        let result = save_game(
            &save_config,
            &layered_db,
            &player_query,
            &souprune_config.game,
            &event.slot,
        );

        match result {
            Ok(()) => {
                info!("Game saved to slot {:?}", event.slot);
                complete_events.write(SaveCompleteEvent {
                    success: true,
                    slot: event.slot.clone(),
                    error: None,
                });
            }
            Err(e) => {
                error!("Failed to save game: {}", e);
                complete_events.write(SaveCompleteEvent {
                    success: false,
                    slot: event.slot.clone(),
                    error: Some(e.to_string()),
                });
            }
        }
    }
}

/// System to handle load game events.
/// Writes player data directly to LayeredFactDatabase.
///
/// 处理加载游戏事件的系统。
/// 直接将玩家数据写入 LayeredFactDatabase。
fn handle_load_game_system(
    mut events: MessageReader<LoadGameEvent>,
    mut complete_events: MessageWriter<LoadCompleteEvent>,
    save_config: Res<SaveConfig>,
    mut layered_db: ResMut<LayeredFactDatabase>,
    mut player_query: Query<&mut Transform, With<crate::core::data::MainPlayer>>,
) {
    for event in events.read() {
        let result = load_game(
            &save_config,
            &mut layered_db,
            &mut player_query,
            &event.slot,
        );

        match result {
            Ok(()) => {
                info!("Game loaded from slot {:?}", event.slot);
                complete_events.write(LoadCompleteEvent {
                    success: true,
                    slot: event.slot.clone(),
                    error: None,
                });
            }
            Err(e) => {
                error!("Failed to load game: {}", e);
                complete_events.write(LoadCompleteEvent {
                    success: false,
                    slot: event.slot.clone(),
                    error: Some(e.to_string()),
                });
            }
        }
    }
}

// === Core Save/Load Functions ===
// === 核心存档/读取函数 ===

/// Save the game to the specified slot.
/// Reads all player data from LayeredFactDatabase.
///
/// 将游戏保存到指定槽位。
/// 从 LayeredFactDatabase 读取所有玩家数据。
fn save_game(
    config: &SaveConfig,
    layered_db: &LayeredFactDatabase,
    player_query: &Query<&Transform, With<crate::core::data::MainPlayer>>,
    game_config: &crate::config::GameConfig,
    slot: &SaveSlot,
) -> anyhow::Result<()> {
    // Ensure save directory exists
    fs::create_dir_all(&config.save_directory)?;

    // Get player position
    let position = player_query
        .single()
        .map(|t| (t.translation.x, t.translation.y))
        .unwrap_or((0.0, 0.0));

    // Read player data from FRE
    let name = layered_db
        .get_string("player_name")
        .unwrap_or("Unknown")
        .to_string();
    let lv = layered_db.get_int_or("player_lv", 1) as usize;
    let exp = layered_db.get_int_or("player_exp", 0) as usize;
    let next_exp = layered_db.get_int_or("player_next_exp", 10) as usize;
    let hp = layered_db.get_int_or("player_hp", 20) as usize;
    let hp_max = layered_db.get_int_or("player_hp_max", 20) as usize;
    let atk = layered_db.get_int_or("player_atk", 0) as usize;
    let def = layered_db.get_int_or("player_def", 0) as usize;
    let gold = layered_db.get_int_or("player_gold", 0) as usize;
    let weapon = layered_db
        .get_string("player_weapon")
        .unwrap_or("stick")
        .to_string();
    let armor = layered_db
        .get_string("player_armor")
        .unwrap_or("bandage")
        .to_string();
    let inventory_str = layered_db
        .get_string("player_inventory")
        .unwrap_or("")
        .to_string();
    let inventory_items: Vec<String> = inventory_str
        .split(',')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();
    let inventory_capacity = layered_db.get_int_or("player_inventory_capacity", 8) as usize;

    let save_data = SaveData {
        version: SAVE_VERSION,
        timestamp: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        player: PlayerSaveData {
            name,
            level: LevelData { lv, exp, next_exp },
            health: HealthData {
                current: hp,
                max: hp_max,
            },
            stats: StatsData {
                attack: atk,
                defense: def,
            },
            gold,
            equipment: EquipmentData { weapon, armor },
            inventory: InventoryData {
                items: inventory_items,
                capacity: inventory_capacity,
            },
        },
        progress: ProgressSaveData {
            current_map: game_config.initial_map_path.clone(),
            position,
            flags: std::collections::HashMap::new(),
        },
    };

    // Serialize to TOML
    let toml_string = toml::to_string_pretty(&save_data)?;

    // Write to file
    let save_path = config.save_directory.join(slot.filename());
    let mut file = fs::File::create(&save_path)?;
    file.write_all(toml_string.as_bytes())?;

    info!("Saved game to: {:?}", save_path);
    Ok(())
}

/// Load the game from the specified slot.
/// Writes all player data to LayeredFactDatabase.
///
/// 从指定槽位加载游戏。
/// 将所有玩家数据写入 LayeredFactDatabase。
fn load_game(
    config: &SaveConfig,
    layered_db: &mut LayeredFactDatabase,
    player_query: &mut Query<&mut Transform, With<crate::core::data::MainPlayer>>,
    slot: &SaveSlot,
) -> anyhow::Result<()> {
    let save_path = config.save_directory.join(slot.filename());

    if !save_path.exists() {
        anyhow::bail!("Save file does not exist: {:?}", save_path);
    }

    // Read and parse save file
    let contents = fs::read_to_string(&save_path)?;
    let save_data: SaveData = toml::from_str(&contents)?;

    // Check version compatibility
    if save_data.version > SAVE_VERSION {
        anyhow::bail!(
            "Save file version {} is newer than supported version {}",
            save_data.version,
            SAVE_VERSION
        );
    }

    let player_data = &save_data.player;

    // Write player data to FRE global layer
    layered_db.set_global("player_name", player_data.name.clone());
    layered_db.set_global("player_lv", player_data.level.lv as i64);
    layered_db.set_global("player_exp", player_data.level.exp as i64);
    layered_db.set_global("player_next_exp", player_data.level.next_exp as i64);
    layered_db.set_global("player_hp", player_data.health.current as i64);
    layered_db.set_global("player_hp_max", player_data.health.max as i64);
    layered_db.set_global("player_atk", player_data.stats.attack as i64);
    layered_db.set_global("player_def", player_data.stats.defense as i64);
    layered_db.set_global("player_gold", player_data.gold as i64);
    layered_db.set_global("player_weapon", player_data.equipment.weapon.clone());
    layered_db.set_global("player_armor", player_data.equipment.armor.clone());
    layered_db.set_global("player_inventory", player_data.inventory.items.join(","));
    layered_db.set_global(
        "player_inventory_capacity",
        player_data.inventory.capacity as i64,
    );

    // Update player position
    if let Ok(mut transform) = player_query.single_mut() {
        transform.translation.x = save_data.progress.position.0;
        transform.translation.y = save_data.progress.position.1;
    }

    info!("Loaded game from: {:?}", save_path);
    Ok(())
}

// === Utility Functions ===
// === 实用函数 ===

/// Get the save file path for a slot.
///
/// 获取槽位的存档文件路径。
pub fn get_save_path(config: &SaveConfig, slot: &SaveSlot) -> PathBuf {
    config.save_directory.join(slot.filename())
}

/// Check if a save file exists for a slot.
///
/// 检查槽位是否存在存档文件。
pub fn save_exists(config: &SaveConfig, slot: &SaveSlot) -> bool {
    get_save_path(config, slot).exists()
}

/// List all existing save slots.
///
/// 列出所有存在的存档槽。
pub fn list_saves(config: &SaveConfig) -> Vec<SaveSlot> {
    let mut saves = Vec::new();

    if !config.save_directory.exists() {
        return saves;
    }

    if let Ok(entries) = fs::read_dir(&config.save_directory) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(name) = path.file_stem().and_then(|s| s.to_str())
                && let Some(ext) = path.extension().and_then(|s| s.to_str())
                && ext == "toml"
            {
                if let Some(num_str) = name.strip_prefix("save_")
                    && let Ok(num) = num_str.parse::<u32>()
                {
                    saves.push(SaveSlot::Numbered(num));
                    continue;
                }
                saves.push(SaveSlot::Named(name.to_string()));
            }
        }
    }

    saves.sort_by(|a, b| match (a, b) {
        (SaveSlot::Numbered(n1), SaveSlot::Numbered(n2)) => n1.cmp(n2),
        (SaveSlot::Numbered(_), SaveSlot::Named(_)) => std::cmp::Ordering::Less,
        (SaveSlot::Named(_), SaveSlot::Numbered(_)) => std::cmp::Ordering::Greater,
        (SaveSlot::Named(s1), SaveSlot::Named(s2)) => s1.cmp(s2),
    });

    saves
}

/// Delete a save file.
///
/// 删除存档文件。
pub fn delete_save(config: &SaveConfig, slot: &SaveSlot) -> anyhow::Result<()> {
    let path = get_save_path(config, slot);
    if path.exists() {
        fs::remove_file(&path)?;
        info!("Deleted save: {:?}", path);
    }
    Ok(())
}

/// Read save metadata without loading the full game.
///
/// 读取存档元数据而不加载完整游戏。
pub fn read_save_metadata(config: &SaveConfig, slot: &SaveSlot) -> anyhow::Result<SaveMetadata> {
    let path = get_save_path(config, slot);
    let contents = fs::read_to_string(&path)?;
    let save_data: SaveData = toml::from_str(&contents)?;

    Ok(SaveMetadata {
        slot: slot.clone(),
        version: save_data.version,
        timestamp: save_data.timestamp,
        player_name: save_data.player.name,
        player_level: save_data.player.level.lv,
        current_map: save_data.progress.current_map,
    })
}

/// Metadata about a save file for display purposes.
///
/// 用于显示的存档文件元数据。
#[derive(Debug, Clone)]
pub struct SaveMetadata {
    pub slot: SaveSlot,
    pub version: u32,
    pub timestamp: String,
    pub player_name: String,
    pub player_level: usize,
    pub current_map: String,
}
