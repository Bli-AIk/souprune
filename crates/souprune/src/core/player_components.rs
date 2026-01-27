//! # player_components.rs
//!
//! # player_components.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This module defines ECS components for player data, replacing the monolithic `PlayerData` resource
//! with fine-grained components for better parallelism and extensibility.
//!
//! 本模块定义玩家数据的 ECS 组件，用细粒度组件替代单一的 `PlayerData` 资源，
//! 以提高并行性和可扩展性。

use bevy::prelude::*;

use crate::core::item::ItemId;

/// Marker component to identify the player entity.
///
/// 标记组件，用于识别玩家实体。
#[derive(Component, Default)]
pub struct Player;

/// Component storing the player's display name.
///
/// 存储玩家显示名称的组件。
#[derive(Component, Clone, Debug, Reflect)]
pub struct PlayerName(pub String);

impl Default for PlayerName {
    fn default() -> Self {
        Self("Chara".to_string())
    }
}

/// Component storing level progression data.
///
/// 存储等级进度数据的组件。
#[derive(Component, Clone, Debug, Default, Reflect)]
pub struct Level {
    /// Current level.
    ///
    /// 当前等级。
    pub lv: usize,
    /// Current experience points.
    ///
    /// 当前经验值。
    pub exp: usize,
    /// Experience required for next level.
    ///
    /// 升级所需经验值。
    pub next_exp: usize,
}

impl Level {
    pub fn new(lv: usize, exp: usize, next_exp: usize) -> Self {
        Self { lv, exp, next_exp }
    }

    /// Check if ready to level up.
    ///
    /// 检查是否可以升级。
    pub fn can_level_up(&self) -> bool {
        self.exp >= self.next_exp
    }
}

/// Component storing health data.
///
/// 存储生命值数据的组件。
#[derive(Component, Clone, Debug, Default, Reflect)]
pub struct Health {
    /// Current health points.
    ///
    /// 当前生命值。
    pub current: usize,
    /// Maximum health points.
    ///
    /// 最大生命值。
    pub max: usize,
}

impl Health {
    pub fn new(current: usize, max: usize) -> Self {
        Self { current, max }
    }

    /// Check if the entity is alive.
    ///
    /// 检查实体是否存活。
    pub fn is_alive(&self) -> bool {
        self.current > 0
    }

    /// Get health as a percentage (0.0 to 1.0).
    ///
    /// 获取生命值百分比（0.0 到 1.0）。
    pub fn percentage(&self) -> f32 {
        if self.max == 0 {
            0.0
        } else {
            self.current as f32 / self.max as f32
        }
    }

    /// Apply damage, returning actual damage dealt.
    ///
    /// 应用伤害，返回实际造成的伤害。
    pub fn take_damage(&mut self, amount: usize) -> usize {
        let actual = amount.min(self.current);
        self.current = self.current.saturating_sub(amount);
        actual
    }

    /// Heal, returning actual amount healed.
    ///
    /// 治疗，返回实际治疗量。
    pub fn heal(&mut self, amount: usize) -> usize {
        let headroom = self.max.saturating_sub(self.current);
        let actual = amount.min(headroom);
        self.current += actual;
        actual
    }
}

/// Component storing combat stats.
///
/// 存储战斗属性的组件。
#[derive(Component, Clone, Debug, Default, Reflect)]
pub struct Stats {
    /// Attack power.
    ///
    /// 攻击力。
    pub attack: usize,
    /// Defense power.
    ///
    /// 防御力。
    pub defense: usize,
}

impl Stats {
    pub fn new(attack: usize, defense: usize) -> Self {
        Self { attack, defense }
    }
}

/// Component storing gold/currency.
///
/// 存储金币/货币的组件。
#[derive(Component, Clone, Debug, Default, Reflect)]
pub struct Gold(pub usize);

impl Gold {
    pub fn new(amount: usize) -> Self {
        Self(amount)
    }

    /// Add gold, returning new total.
    ///
    /// 增加金币，返回新的总额。
    pub fn add(&mut self, amount: usize) -> usize {
        self.0 = self.0.saturating_add(amount);
        self.0
    }

    /// Spend gold if affordable, returning true if successful.
    ///
    /// 如果足够则花费金币，成功返回 true。
    pub fn spend(&mut self, amount: usize) -> bool {
        if self.0 >= amount {
            self.0 -= amount;
            true
        } else {
            false
        }
    }
}

/// Component storing equipped items.
///
/// 存储装备物品的组件。
#[derive(Component, Clone, Debug, Reflect)]
pub struct Equipment {
    /// Currently equipped weapon.
    ///
    /// 当前装备的武器。
    pub weapon: ItemId,
    /// Currently equipped armor.
    ///
    /// 当前装备的护甲。
    pub armor: ItemId,
}

impl Default for Equipment {
    fn default() -> Self {
        Self {
            weapon: ItemId("stick".to_string()),
            armor: ItemId("bandage".to_string()),
        }
    }
}

/// Component storing the player's inventory.
///
/// 存储玩家物品栏的组件。
#[derive(Component, Clone, Debug, Reflect)]
pub struct Inventory {
    /// Items in the inventory.
    ///
    /// 物品栏中的物品。
    pub items: Vec<ItemId>,
    /// Maximum inventory capacity.
    ///
    /// 物品栏最大容量。
    pub capacity: usize,
}

impl Default for Inventory {
    fn default() -> Self {
        Self {
            items: vec![
                ItemId("monster_candy".to_string()),
                ItemId("monster_candy".to_string()),
                ItemId("monster_candy".to_string()),
                ItemId("monster_candy".to_string()),
                ItemId("monster_candy".to_string()),
                ItemId("UNDEFITEM".to_string()),
            ],
            capacity: 8,
        }
    }
}

impl Inventory {
    /// Check if inventory has space.
    ///
    /// 检查物品栏是否有空间。
    pub fn has_space(&self) -> bool {
        self.items.len() < self.capacity
    }

    /// Add an item if there's space.
    ///
    /// 如果有空间则添加物品。
    pub fn add_item(&mut self, item: ItemId) -> bool {
        if self.has_space() {
            self.items.push(item);
            true
        } else {
            false
        }
    }

    /// Remove an item by index.
    ///
    /// 按索引移除物品。
    pub fn remove_item(&mut self, index: usize) -> Option<ItemId> {
        if index < self.items.len() {
            Some(self.items.remove(index))
        } else {
            None
        }
    }

    /// Get the number of items.
    ///
    /// 获取物品数量。
    pub fn count(&self) -> usize {
        self.items.len()
    }
}

/// Bundle for spawning a complete player entity with all components.
///
/// 用于生成包含所有组件的完整玩家实体的 Bundle。
#[derive(Bundle, Default)]
pub struct PlayerBundle {
    pub player: Player,
    pub name: PlayerName,
    pub level: Level,
    pub health: Health,
    pub stats: Stats,
    pub gold: Gold,
    pub equipment: Equipment,
    pub inventory: Inventory,
}

impl PlayerBundle {
    /// Create a new player bundle with default Undertale-style starting values.
    ///
    /// 创建具有默认 Undertale 风格初始值的新玩家 Bundle。
    pub fn new() -> Self {
        Self {
            player: Player,
            name: PlayerName::default(),
            level: Level::new(1, 0, 10),
            health: Health::new(20, 20),
            stats: Stats::new(0, 0),
            gold: Gold::new(42),
            equipment: Equipment::default(),
            inventory: Inventory::default(),
        }
    }
}
