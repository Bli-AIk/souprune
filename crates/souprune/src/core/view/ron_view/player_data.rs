//! Player data view for accessing player facts from LayeredFactDatabase.
//!
//! 玩家数据视图，用于从 LayeredFactDatabase 访问玩家事实。

use crate::core::item::ItemId;
use bevy_fact_rule_event::{FactDatabase, FactValue, LayeredFactDatabase};

/// Helper struct to read player data from LayeredFactDatabase.
/// This provides a view into player facts for the View system.
///
/// 从 LayeredFactDatabase 读取玩家数据的辅助结构体。
/// 为 View 系统提供玩家事实的视图。
pub struct PlayerDataView<'a> {
    db: &'a LayeredFactDatabase,
    /// Optional local facts from ViewRoot (View-specific facts)
    /// 来自 ViewRoot 的可选局部事实（View 特定的事实）
    local_facts: Option<&'a FactDatabase>,
}

impl<'a> PlayerDataView<'a> {
    pub fn new(db: &'a LayeredFactDatabase) -> Self {
        Self {
            db,
            local_facts: None,
        }
    }

    /// Create a PlayerDataView with local facts from a ViewRoot.
    ///
    /// 创建一个带有来自 ViewRoot 局部事实的 PlayerDataView。
    pub fn with_local_facts(db: &'a LayeredFactDatabase, local_facts: &'a FactDatabase) -> Self {
        Self {
            db,
            local_facts: Some(local_facts),
        }
    }

    /// Get a fact value with priority: local_facts -> scene -> global.
    /// Supports `fact('key')` and `$key` syntax.
    ///
    /// 获取事实值，优先级为：local_facts -> scene -> global。
    /// 支持 `fact('key')` 和 `$key` 语法。
    pub fn get_fact(&self, key: &str) -> Option<&FactValue> {
        // First check local facts
        if let Some(local) = self.local_facts {
            if let Some(value) = local.get_by_str(key) {
                return Some(value);
            }
        }
        // Then check layered database (scene -> global)
        self.db.get_by_str(key)
    }

    /// Get a fact value as f64, with optional default.
    ///
    /// 获取事实值为 f64，带可选默认值。
    pub fn get_fact_float(&self, key: &str, default: Option<f64>) -> f64 {
        if let Some(value) = self.get_fact(key) {
            match value {
                FactValue::Float(f) => *f,
                FactValue::Int(i) => *i as f64,
                FactValue::Bool(b) => {
                    if *b {
                        1.0
                    } else {
                        0.0
                    }
                }
                FactValue::String(_) => default.unwrap_or(0.0),
            }
        } else {
            default.unwrap_or(0.0)
        }
    }

    pub fn name(&self) -> String {
        self.db
            .get_string("player_name")
            .unwrap_or("???")
            .to_string()
    }

    pub fn lv(&self) -> usize {
        self.db.get_int("player_lv").unwrap_or(1) as usize
    }

    pub fn exp(&self) -> usize {
        self.db.get_int("player_exp").unwrap_or(0) as usize
    }

    pub fn next_exp(&self) -> usize {
        self.db.get_int("player_next_exp").unwrap_or(10) as usize
    }

    pub fn hp(&self) -> usize {
        self.db.get_int("player_hp").unwrap_or(20) as usize
    }

    pub fn hp_max(&self) -> usize {
        self.db.get_int("player_hp_max").unwrap_or(20) as usize
    }

    pub fn attack(&self) -> usize {
        self.db.get_int("player_atk").unwrap_or(0) as usize
    }

    pub fn defense(&self) -> usize {
        self.db.get_int("player_def").unwrap_or(0) as usize
    }

    pub fn gold(&self) -> usize {
        self.db.get_int("player_gold").unwrap_or(0) as usize
    }

    pub fn weapon(&self) -> ItemId {
        ItemId(
            self.db
                .get_string("player_weapon")
                .unwrap_or("stick")
                .to_string(),
        )
    }

    pub fn armor(&self) -> ItemId {
        ItemId(
            self.db
                .get_string("player_armor")
                .unwrap_or("bandage")
                .to_string(),
        )
    }

    pub fn inventory(&self) -> Vec<ItemId> {
        self.db
            .get_string("player_inventory")
            .map(|s| {
                s.split(',')
                    .filter(|s| !s.is_empty())
                    .map(|s| ItemId(s.to_string()))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn inventory_capacity(&self) -> usize {
        self.db.get_int("player_inventory_capacity").unwrap_or(8) as usize
    }
}
