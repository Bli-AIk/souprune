//! # actions.rs
//!
//! # actions.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This module defines the semantic input actions for the game.
//! Actions use internal slot-based system with configurable name mappings.
//!
//! 该模块定义了游戏的语义输入动作。
//! Actions 使用内部基于槽位的系统，并支持可配置的名称映射。

use bevy::prelude::*;
use leafwing_input_manager::prelude::*;
use std::collections::HashMap;

/// Maximum number of dynamic action slots available.
/// MOD authors can define up to this many custom actions.
///
/// 可用的最大动态动作槽位数。
/// MOD 作者最多可以定义这么多自定义动作。
pub const MAX_ACTION_SLOTS: usize = 32;

/// Action enum with enough slots for dynamic actions.
/// This is used by leafwing_input_manager for input detection.
///
/// 具有足够动态动作槽位的动作枚举。
/// 由 leafwing_input_manager 用于输入检测。
#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]
pub enum Action {
    Slot0,
    Slot1,
    Slot2,
    Slot3,
    Slot4,
    Slot5,
    Slot6,
    Slot7,
    Slot8,
    Slot9,
    Slot10,
    Slot11,
    Slot12,
    Slot13,
    Slot14,
    Slot15,
    Slot16,
    Slot17,
    Slot18,
    Slot19,
    Slot20,
    Slot21,
    Slot22,
    Slot23,
    Slot24,
    Slot25,
    Slot26,
    Slot27,
    Slot28,
    Slot29,
    Slot30,
    Slot31,
}

impl Action {
    /// Get the slot index as usize.
    ///
    /// 获取槽位索引。
    pub fn index(self) -> usize {
        match self {
            Self::Slot0 => 0,
            Self::Slot1 => 1,
            Self::Slot2 => 2,
            Self::Slot3 => 3,
            Self::Slot4 => 4,
            Self::Slot5 => 5,
            Self::Slot6 => 6,
            Self::Slot7 => 7,
            Self::Slot8 => 8,
            Self::Slot9 => 9,
            Self::Slot10 => 10,
            Self::Slot11 => 11,
            Self::Slot12 => 12,
            Self::Slot13 => 13,
            Self::Slot14 => 14,
            Self::Slot15 => 15,
            Self::Slot16 => 16,
            Self::Slot17 => 17,
            Self::Slot18 => 18,
            Self::Slot19 => 19,
            Self::Slot20 => 20,
            Self::Slot21 => 21,
            Self::Slot22 => 22,
            Self::Slot23 => 23,
            Self::Slot24 => 24,
            Self::Slot25 => 25,
            Self::Slot26 => 26,
            Self::Slot27 => 27,
            Self::Slot28 => 28,
            Self::Slot29 => 29,
            Self::Slot30 => 30,
            Self::Slot31 => 31,
        }
    }

    /// Create an action from index. Returns None if index is out of range.
    ///
    /// 从索引创建动作。如果索引超出范围则返回 None。
    pub fn from_index(index: usize) -> Option<Self> {
        Some(match index {
            0 => Self::Slot0,
            1 => Self::Slot1,
            2 => Self::Slot2,
            3 => Self::Slot3,
            4 => Self::Slot4,
            5 => Self::Slot5,
            6 => Self::Slot6,
            7 => Self::Slot7,
            8 => Self::Slot8,
            9 => Self::Slot9,
            10 => Self::Slot10,
            11 => Self::Slot11,
            12 => Self::Slot12,
            13 => Self::Slot13,
            14 => Self::Slot14,
            15 => Self::Slot15,
            16 => Self::Slot16,
            17 => Self::Slot17,
            18 => Self::Slot18,
            19 => Self::Slot19,
            20 => Self::Slot20,
            21 => Self::Slot21,
            22 => Self::Slot22,
            23 => Self::Slot23,
            24 => Self::Slot24,
            25 => Self::Slot25,
            26 => Self::Slot26,
            27 => Self::Slot27,
            28 => Self::Slot28,
            29 => Self::Slot29,
            30 => Self::Slot30,
            31 => Self::Slot31,
            _ => return None,
        })
    }

    /// Get all action slots.
    ///
    /// 获取所有动作槽位。
    pub fn all() -> [Self; MAX_ACTION_SLOTS] {
        [
            Self::Slot0,
            Self::Slot1,
            Self::Slot2,
            Self::Slot3,
            Self::Slot4,
            Self::Slot5,
            Self::Slot6,
            Self::Slot7,
            Self::Slot8,
            Self::Slot9,
            Self::Slot10,
            Self::Slot11,
            Self::Slot12,
            Self::Slot13,
            Self::Slot14,
            Self::Slot15,
            Self::Slot16,
            Self::Slot17,
            Self::Slot18,
            Self::Slot19,
            Self::Slot20,
            Self::Slot21,
            Self::Slot22,
            Self::Slot23,
            Self::Slot24,
            Self::Slot25,
            Self::Slot26,
            Self::Slot27,
            Self::Slot28,
            Self::Slot29,
            Self::Slot30,
            Self::Slot31,
        ]
    }
}

/// Registry that maps action names to internal action slots.
/// This is initialized from configuration and used to translate
/// between human-readable action names and the underlying slot system.
///
/// 将动作名称映射到内部动作槽位的注册表。
/// 从配置初始化，用于在人类可读的动作名称和底层槽位系统之间转换。
#[derive(Resource, Debug, Clone)]
pub struct ActionRegistry {
    /// Maps action name to slot
    ///
    /// 动作名称到槽位的映射
    name_to_slot: HashMap<String, Action>,

    /// Maps slot to action name
    ///
    /// 槽位到动作名称的映射
    slot_to_name: HashMap<Action, String>,

    /// Next available slot for registration
    ///
    /// 下一个可用于注册的槽位
    next_slot: usize,
}

impl Default for ActionRegistry {
    fn default() -> Self {
        let mut registry = Self {
            name_to_slot: HashMap::new(),
            slot_to_name: HashMap::new(),
            next_slot: 0,
        };

        // Register default actions
        // 注册默认动作
        registry.register("Up").expect("Failed to register Up");
        registry.register("Down").expect("Failed to register Down");
        registry.register("Left").expect("Failed to register Left");
        registry
            .register("Right")
            .expect("Failed to register Right");
        registry
            .register("Confirm")
            .expect("Failed to register Confirm");
        registry
            .register("Cancel")
            .expect("Failed to register Cancel");
        registry.register("Menu").expect("Failed to register Menu");

        registry
    }
}

impl ActionRegistry {
    /// Standard action name constants
    /// 标准动作名称常量
    pub const UP: &'static str = "Up";
    pub const DOWN: &'static str = "Down";
    pub const LEFT: &'static str = "Left";
    pub const RIGHT: &'static str = "Right";
    pub const CONFIRM: &'static str = "Confirm";
    pub const CANCEL: &'static str = "Cancel";
    pub const MENU: &'static str = "Menu";

    /// Register a new action. Returns the assigned slot.
    /// Returns an error if no more slots are available or if the action is already registered.
    ///
    /// 注册新动作。返回分配的槽位。
    /// 如果没有更多可用槽位或动作已注册，则返回错误。
    pub fn register(&mut self, name: impl Into<String>) -> Result<Action, String> {
        let name = name.into();

        if self.name_to_slot.contains_key(&name) {
            return Err(format!("Action '{}' is already registered", name));
        }

        if self.next_slot >= MAX_ACTION_SLOTS {
            return Err(format!(
                "Cannot register action '{}': maximum of {} actions reached",
                name, MAX_ACTION_SLOTS
            ));
        }

        let slot = Action::from_index(self.next_slot)
            .ok_or_else(|| format!("Invalid slot index: {}", self.next_slot))?;

        self.name_to_slot.insert(name.clone(), slot);
        self.slot_to_name.insert(slot, name);
        self.next_slot += 1;

        Ok(slot)
    }

    /// Get the slot for an action name.
    ///
    /// 获取动作名称对应的槽位。
    pub fn get(&self, name: &str) -> Option<Action> {
        self.name_to_slot.get(name).copied()
    }

    /// Get the action name for a slot.
    ///
    /// 获取槽位对应的动作名称。
    #[allow(dead_code)]
    pub fn get_name(&self, slot: Action) -> Option<&str> {
        self.slot_to_name.get(&slot).map(|s| s.as_str())
    }

    /// Get all registered action names.
    ///
    /// 获取所有已注册的动作名称。
    #[allow(dead_code)]
    pub fn all_actions(&self) -> impl Iterator<Item = &str> {
        self.name_to_slot.keys().map(|s| s.as_str())
    }

    /// Check if an action is registered.
    ///
    /// 检查动作是否已注册。
    #[allow(dead_code)]
    pub fn is_registered(&self, name: &str) -> bool {
        self.name_to_slot.contains_key(name)
    }

    // Helper methods to get standard actions
    // 获取标准动作的辅助方法

    pub fn up(&self) -> Action {
        self.get(Self::UP).expect("Up action not registered")
    }

    pub fn down(&self) -> Action {
        self.get(Self::DOWN).expect("Down action not registered")
    }

    pub fn left(&self) -> Action {
        self.get(Self::LEFT).expect("Left action not registered")
    }

    pub fn right(&self) -> Action {
        self.get(Self::RIGHT).expect("Right action not registered")
    }

    pub fn confirm(&self) -> Action {
        self.get(Self::CONFIRM)
            .expect("Confirm action not registered")
    }

    pub fn cancel(&self) -> Action {
        self.get(Self::CANCEL)
            .expect("Cancel action not registered")
    }

    /// Get the Menu action slot.
    ///
    /// 获取 Menu 动作槽位。
    #[allow(dead_code)]
    pub fn menu(&self) -> Action {
        self.get(Self::MENU).expect("Menu action not registered")
    }
}

/// Extension trait for ActionState to work with named actions.
///
/// ActionState 的扩展 trait，用于处理命名动作。
#[allow(dead_code)]
pub trait ActionStateExt {
    /// Check if a named action is pressed.
    ///
    /// 检查命名动作是否按下。
    fn action_pressed(&self, registry: &ActionRegistry, action: &str) -> bool;

    /// Check if a named action was just pressed.
    ///
    /// 检查命名动作是否刚刚按下。
    fn action_just_pressed(&self, registry: &ActionRegistry, action: &str) -> bool;

    /// Check if a named action was just released.
    ///
    /// 检查命名动作是否刚刚释放。
    fn action_just_released(&self, registry: &ActionRegistry, action: &str) -> bool;
}

impl ActionStateExt for ActionState<Action> {
    fn action_pressed(&self, registry: &ActionRegistry, action: &str) -> bool {
        registry
            .get(action)
            .map(|slot| self.pressed(&slot))
            .unwrap_or(false)
    }

    fn action_just_pressed(&self, registry: &ActionRegistry, action: &str) -> bool {
        registry
            .get(action)
            .map(|slot| self.just_pressed(&slot))
            .unwrap_or(false)
    }

    fn action_just_released(&self, registry: &ActionRegistry, action: &str) -> bool {
        registry
            .get(action)
            .map(|slot| self.just_released(&slot))
            .unwrap_or(false)
    }
}
