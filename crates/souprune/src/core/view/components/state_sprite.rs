//! State Sprite Components and Systems
//!
//! 状态精灵组件和系统
//!
//! This module provides data-driven sprite state management.
//! Sprite textures can change based on configurable rules like
//! interactive layer selection.
//!
//! 本模块提供数据驱动的精灵状态管理。
//! 精灵纹理可以根据配置规则（如交互层选中状态）变化。

use bevy::prelude::*;
use std::collections::HashMap;

#[cfg(feature = "debug")]
use bevy::reflect::Reflect;

use super::interactive::InteractiveLayer;
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
    /// 当前激活的状态（None = 默认）。
    pub current_state: Option<String>,

    /// Whether the sprite texture needs to be updated.
    ///
    /// 精灵纹理是否需要更新。
    pub dirty: bool,
}

/// Runtime representation of a state rule.
///
/// 状态规则的运行时表示。
#[derive(Debug, Clone)]
#[cfg_attr(feature = "debug", derive(Reflect))]
pub struct StateRule {
    /// The trigger condition.
    ///
    /// 触发条件。
    pub trigger: StateTrigger,

    /// The state to switch to when triggered.
    ///
    /// 触发时要切换到的状态。
    pub state: String,
}

/// Runtime representation of a state trigger.
///
/// 状态触发器的运行时表示。
#[derive(Debug, Clone)]
#[cfg_attr(feature = "debug", derive(Reflect))]
pub enum StateTrigger {
    /// Triggered when this element is selected in an interactive layer.
    ///
    /// 当此元素在交互层中被选中时触发。
    InteractiveLayerSelected { layer_id: String, index: usize },
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

        // Process sugar syntax: subscribe_selection
        if let Some((layer_id, index)) = &config.subscribe_selection {
            rules.push(StateRule {
                trigger: StateTrigger::InteractiveLayerSelected {
                    layer_id: layer_id.clone(),
                    index: *index,
                },
                state: "selected".to_string(),
            });
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
            StateTriggerDef::InteractiveLayerSelected { layer_id, index } => {
                StateTrigger::InteractiveLayerSelected {
                    layer_id: layer_id.clone(),
                    index: *index,
                }
            }
        }
    }

    /// Evaluate the trigger against the current interactive layer state.
    ///
    /// 根据当前交互层状态评估触发器。
    pub fn evaluate(
        &self,
        layer_id: &str,
        layer_is_active: bool,
        current_selection: usize,
    ) -> bool {
        match self {
            StateTrigger::InteractiveLayerSelected {
                layer_id: trigger_layer_id,
                index,
            } => layer_id == trigger_layer_id && layer_is_active && current_selection == *index,
        }
    }
}

// ============================================================================
// Systems
// 系统
// ============================================================================

/// Marker resource to track if initial evaluation has happened.
///
/// 标记资源，用于跟踪初始评估是否已完成。
#[derive(Resource, Default)]
pub struct StateSpriteInitialized(pub bool);

/// System to evaluate state sprite rules and update state.
///
/// 评估状态精灵规则并更新状态的系统。
///
/// This system runs whenever an InteractiveLayer changes or when a new StateSpriteState is added.
/// It evaluates all StateSpriteState components and updates their state based on the configured rules.
///
/// 此系统在 InteractiveLayer 变化时或添加新的 StateSpriteState 时运行。
/// 它评估所有 StateSpriteState 组件并根据配置的规则更新其状态。
pub fn evaluate_state_sprite_rules_system(
    layer_query: Query<&InteractiveLayer, Changed<InteractiveLayer>>,
    mut state_query: Query<&mut StateSpriteState>,
) {
    // Early exit if no layers changed
    if layer_query.is_empty() {
        return;
    }

    // Collect all changed layer states
    // 收集所有变化的层状态
    let layers: Vec<&InteractiveLayer> = layer_query.iter().collect();

    // Evaluate each state sprite
    for mut state in state_query.iter_mut() {
        let mut new_state: Option<String> = None;

        // Evaluate rules in order; first matching rule wins
        'rule_loop: for rule in &state.rules {
            for layer in &layers {
                if rule
                    .trigger
                    .evaluate(&layer.layer_id, layer.is_active, layer.current_selection)
                {
                    new_state = Some(rule.state.clone());
                    break 'rule_loop;
                }
            }
        }

        state.set_state(new_state);
    }
}

/// System to evaluate newly added state sprites against all interactive layers.
///
/// 针对所有交互层评估新添加的状态精灵的系统。
///
/// This handles the case when a state sprite is spawned and needs immediate evaluation.
///
/// 处理状态精灵生成后需要立即评估的情况。
pub fn evaluate_new_state_sprites_system(
    layer_query: Query<&InteractiveLayer>,
    mut state_query: Query<&mut StateSpriteState, Added<StateSpriteState>>,
) {
    // Early exit if no new state sprites
    if state_query.is_empty() {
        return;
    }

    // Collect all layer states
    let layers: Vec<&InteractiveLayer> = layer_query.iter().collect();

    // Evaluate each new state sprite
    for mut state in state_query.iter_mut() {
        let mut new_state: Option<String> = None;

        // Evaluate rules in order; first matching rule wins
        'rule_loop: for rule in &state.rules {
            for layer in &layers {
                if rule
                    .trigger
                    .evaluate(&layer.layer_id, layer.is_active, layer.current_selection)
                {
                    new_state = Some(rule.state.clone());
                    break 'rule_loop;
                }
            }
        }

        state.set_state(new_state);
    }
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
        }
    }
}
