//! State Sprite Components and Systems
//!
//! 状态精灵组件和系统
//!
//! This module provides data-driven sprite state management.
//! Sprite textures can change based on configurable rules.
//!
//! 本模块提供数据驱动的精灵状态管理。
//! 精灵纹理可以根据配置规则变化。

use bevy::prelude::*;
use std::collections::HashMap;

#[cfg(feature = "debug")]
use bevy::reflect::Reflect;

use crate::core::view::layout::view_schema::{StateRuleDef, StateSpriteConfig, StateTriggerDef};

// ============================================================================
// Components
// 组件
// ============================================================================

/// Component that tracks state sprite configuration and current state.
///
/// 跟踪状态精灵配置和当前状态的组件。
#[derive(Component, Debug)]
#[cfg_attr(feature = "debug", derive(Reflect))]
pub struct StateSpriteState {
    /// Default texture path.
    ///
    /// 默认纹理路径。
    pub default_texture: String,

    /// Map of state names to texture paths.
    ///
    /// 状态名称到纹理路径的映射。
    pub variants: HashMap<String, String>,

    /// Rules for state evaluation (processed from config).
    /// These are evaluated in order; first matching rule wins.
    ///
    /// 状态评估规则（从配置处理）。
    /// 按顺序评估；第一个匹配的规则生效。
    pub rules: Vec<StateRule>,

    /// Currently active state (None = default).
    ///
    /// 当前激活状态（None = 默认）。
    pub current_state: Option<String>,

    /// Whether the texture needs updating.
    ///
    /// 纹理是否需要更新。
    pub dirty: bool,
}

/// A rule that maps a trigger condition to a state.
///
/// 将触发条件映射到状态的规则。
#[derive(Debug, Clone)]
#[cfg_attr(feature = "debug", derive(Reflect))]
pub struct StateRule {
    /// The trigger condition.
    ///
    /// 触发条件。
    pub trigger: StateTrigger,

    /// The state to activate when triggered.
    ///
    /// 触发时激活的状态。
    pub state: String,
}

/// Runtime representation of a state trigger.
///
/// 状态触发器的运行时表示。
///
/// TODO: Implement facts-based triggers (e.g., when fact('selection') == 0)
#[derive(Debug, Clone)]
#[cfg_attr(feature = "debug", derive(Reflect))]
pub enum StateTrigger {
    /// Placeholder for future facts-based trigger.
    /// Currently evaluates to false.
    ///
    /// 未来基于 facts 触发器的占位符。
    /// 当前评估为 false。
    FactCondition { expression: String },
}

impl StateSpriteState {
    /// Create a new StateSpriteState from configuration.
    ///
    /// 从配置创建新的 StateSpriteState。
    pub fn from_config(config: &StateSpriteConfig) -> Self {
        let mut rules = Vec::new();

        // Process explicit rules
        for rule_def in &config.rules {
            rules.push(StateRule::from_def(rule_def));
        }

        Self {
            default_texture: config.default.clone(),
            variants: config.variants.clone(),
            rules,
            current_state: None,
            dirty: true, // Initially dirty to load the default texture
        }
    }

    /// Get the texture path for the current state.
    ///
    /// 获取当前状态的纹理路径。
    pub fn current_texture_path(&self) -> &str {
        match &self.current_state {
            Some(state) => self.variants.get(state).unwrap_or(&self.default_texture),
            None => &self.default_texture,
        }
    }

    /// Update the state and mark dirty if changed.
    ///
    /// 更新状态，如果变化则标记为需要更新。
    pub fn set_state(&mut self, new_state: Option<String>) {
        if self.current_state != new_state {
            self.current_state = new_state;
            self.dirty = true;
        }
    }
}

impl StateRule {
    /// Convert from definition to runtime representation.
    ///
    /// 从定义转换为运行时表示。
    fn from_def(def: &StateRuleDef) -> Self {
        Self {
            trigger: StateTrigger::from_def(&def.trigger),
            state: def.state.clone(),
        }
    }
}

impl StateTrigger {
    /// Convert from definition to runtime representation.
    ///
    /// 从定义转换为运行时表示。
    fn from_def(def: &StateTriggerDef) -> Self {
        match def {
            // Convert old InteractiveLayerSelected to a fact condition placeholder
            StateTriggerDef::InteractiveLayerSelected { layer_id, index } => {
                StateTrigger::FactCondition {
                    expression: format!("selection == {} && layer == '{}'", index, layer_id),
                }
            }
        }
    }

    /// Evaluate the trigger. Currently returns false as placeholder.
    ///
    /// 评估触发器。当前作为占位符返回 false。
    pub fn evaluate(&self) -> bool {
        match self {
            StateTrigger::FactCondition { .. } => {
                // TODO: Implement fact-based evaluation
                false
            }
        }
    }
}

// ============================================================================
// Systems
// 系统
// ============================================================================

/// System to evaluate state sprite rules and update state.
///
/// 评估状态精灵规则并更新状态的系统。
///
/// TODO: Reimplement using facts-based triggers.
pub fn evaluate_state_sprite_rules_system(mut _state_query: Query<&mut StateSpriteState>) {
    // TODO: Implement facts-based evaluation
    // For now, this is a no-op
}

/// System to evaluate newly added state sprites.
///
/// 评估新添加的状态精灵的系统。
///
/// TODO: Reimplement using facts-based triggers.
pub fn evaluate_new_state_sprites_system(
    mut _state_query: Query<&mut StateSpriteState, Added<StateSpriteState>>,
) {
    // TODO: Implement facts-based evaluation
    // For now, this is a no-op
}

/// System to update sprite textures when state changes.
///
/// 当状态变化时更新精灵纹理的系统。
pub fn update_state_sprite_textures_system(
    asset_server: Res<AssetServer>,
    mut query: Query<(&mut StateSpriteState, &mut Sprite)>,
) {
    for (mut state, mut sprite) in query.iter_mut() {
        if state.dirty {
            // Clone the texture path to avoid borrow issues
            let texture_path = state.current_texture_path().to_string();
            sprite.image = asset_server.load(&texture_path);
            state.dirty = false;

            debug!("Updated sprite texture to: {}", texture_path);
        }
    }
}
