//! # item.rs
//!
//! ItemListAsset schema types for `.items.ron` files.
//! Mirrors `souprune::core::item` without Bevy dependency.
//!
//! `.items.ron` 文件的物品列表 Schema 类型。

use crate::enemy::LocaleInfo;
use serde::{Deserialize, Serialize};

/// A list of items — top-level `.items.ron` schema.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ItemListAsset(pub Vec<Item>);

/// A single item definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub id: String,
    #[serde(default)]
    pub locale: LocaleInfo,
    pub description: String,
    /// Optional Mortar script for conditional text.
    #[serde(default)]
    pub mortar: Option<String>,
    pub item_type: ItemType,
}

/// Item type with variant-specific data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ItemType {
    Food {
        #[serde(default = "default_true")]
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
    /// Key item — non-consumable, non-droppable.
    KeyItem,
}

fn default_true() -> bool {
    true
}

/// Item effect triggered on use or hit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ItemEffect {
    Heal {
        amount: i32,
    },
    PlayAudio {
        clip_path: String,
    },
    SpawnChildItem {
        item_id: String,
    },
    /// Set an FRE fact (generic extension point).
    SetFact {
        key: String,
        value: ItemFactValue,
    },
}

/// Lightweight fact value for item effects (independent of FRE crate).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ItemFactValue {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
}
