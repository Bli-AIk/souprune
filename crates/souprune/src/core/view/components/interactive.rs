//! Interactive layer components for unified OW/Battle navigation.
//!
//! 统一 OW/Battle 导航的交互层组件。
//!
//! This module provides a unified abstraction for handling user navigation
//! across different UI contexts (Overworld menus, Battle buttons, etc.).
//!
//! 本模块提供了一个统一的抽象，用于处理不同 UI 上下文（Overworld 菜单、战斗按钮等）中的用户导航。

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[cfg(feature = "debug")]
use bevy::reflect::Reflect;

use crate::core::input::{Action, ActionRegistry, InputBehaviorConfig};

// ============================================================================
// Navigation Direction
// 导航方向
// ============================================================================

/// Cardinal directions for navigation input.
///
/// 导航输入的基本方向。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NavDirection {
    Up,
    Down,
    Left,
    Right,
}

impl NavDirection {
    /// Convert an Action to a NavDirection using the behavior config and registry.
    ///
    /// 使用行为配置和注册表将 Action 转换为 NavDirection。
    #[allow(dead_code)]
    pub fn from_action(
        action: &Action,
        registry: &ActionRegistry,
        behavior_config: &InputBehaviorConfig,
    ) -> Option<Self> {
        let up_action = behavior_config.nav_up().and_then(|name| registry.get(name));
        let down_action = behavior_config
            .nav_down()
            .and_then(|name| registry.get(name));
        let left_action = behavior_config
            .nav_left()
            .and_then(|name| registry.get(name));
        let right_action = behavior_config
            .nav_right()
            .and_then(|name| registry.get(name));

        if up_action == Some(*action) {
            Some(NavDirection::Up)
        } else if down_action == Some(*action) {
            Some(NavDirection::Down)
        } else if left_action == Some(*action) {
            Some(NavDirection::Left)
        } else if right_action == Some(*action) {
            Some(NavDirection::Right)
        } else {
            None
        }
    }

    /// Get all navigation directions.
    ///
    /// 获取所有导航方向。
    pub fn all() -> [Self; 4] {
        [Self::Up, Self::Down, Self::Left, Self::Right]
    }

    /// Get the Action for this direction using the behavior config and registry.
    /// Returns None if the corresponding action is not configured or registered.
    ///
    /// 使用行为配置和注册表获取此方向对应的 Action。
    /// 如果对应的动作未配置或未注册则返回 None。
    pub fn to_action(
        self,
        registry: &ActionRegistry,
        behavior_config: &InputBehaviorConfig,
    ) -> Option<Action> {
        let action_name = match self {
            NavDirection::Up => behavior_config.nav_up(),
            NavDirection::Down => behavior_config.nav_down(),
            NavDirection::Left => behavior_config.nav_left(),
            NavDirection::Right => behavior_config.nav_right(),
        };
        action_name.and_then(|name| registry.get(name))
    }
}

// ============================================================================
// Navigator Types
// ============================================================================

/// Navigation strategy enum that determines how directional input affects selection.
///
/// 导航策略枚举，决定方向输入如何影响选择。
///
/// This enum replaces the trait object approach to support RON serialization.
///
/// 此枚举替代 trait object 方式以支持 RON 序列化。
#[derive(Debug, Clone, Deserialize, Serialize)]
#[cfg_attr(feature = "debug", derive(Reflect))]
pub enum NavigatorType {
    /// Linear navigation for vertical/horizontal lists.
    ///
    /// 用于垂直/水平列表的线性导航。
    Linear {
        direction: LinearDirection,
        option_count: usize,
        #[serde(default)]
        looping: bool,
    },

    /// Grid navigation for 2D button layouts.
    ///
    /// 用于 2D 按钮布局的网格导航。
    Grid {
        rows: usize,
        cols: usize,
        #[serde(default)]
        looping: bool,
    },

    /// Custom navigation with explicit position mappings.
    ///
    /// 使用显式位置映射的自定义导航。
    Custom {
        /// Mapping from (current_index, direction) to next_index.
        /// Format: { "0:Left": 3, "0:Right": 1, "1:Left": 0, ... }
        #[serde(default)]
        mappings: std::collections::HashMap<String, usize>,
        option_count: usize,
    },
}

/// Direction for linear navigation.
///
/// 线性导航的方向。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default)]
#[cfg_attr(feature = "debug", derive(Reflect))]
pub enum LinearDirection {
    /// Navigate with Up/Down keys.
    #[default]
    Vertical,
    /// Navigate with Left/Right keys.
    Horizontal,
}

// ============================================================================
// Layer Transition Types
// 层级转换类型
// ============================================================================

/// A transition rule with optional condition.
///
/// 带有可选条件的转换规则。
#[derive(Debug, Clone)]
pub struct LayerTransitionRule {
    /// Optional condition expression.
    pub condition: Option<String>,
    /// Action to execute when this rule matches.
    pub action: LayerTransitionAction,
}

/// Action to perform when a transition is triggered.
///
/// 转换触发时执行的动作。
#[derive(Debug, Clone)]
pub enum LayerTransitionAction {
    /// Switch to another layer within the same context.
    GotoLayer(String),
    /// Pop the current state (e.g., close menu, return to previous state).
    PopState,
    /// Push a new state onto the stack.
    PushState(String),
}

impl Default for NavigatorType {
    fn default() -> Self {
        NavigatorType::Linear {
            direction: LinearDirection::Vertical,
            option_count: 0,
            looping: false,
        }
    }
}

impl NavigatorType {
    /// Calculate the next index based on current selection and navigation direction.
    ///
    /// 根据当前选择和导航方向计算下一个索引。
    pub fn handle_direction(&self, current: usize, direction: NavDirection) -> Option<usize> {
        match self {
            NavigatorType::Linear {
                direction: linear_dir,
                option_count,
                looping,
            } => {
                Self::handle_linear_input(current, direction, *linear_dir, *option_count, *looping)
            }

            NavigatorType::Grid {
                rows,
                cols,
                looping,
            } => Self::handle_grid_input(current, direction, *rows, *cols, *looping),

            NavigatorType::Custom {
                mappings,
                option_count: _,
            } => {
                let key = format!("{}:{:?}", current, direction);
                mappings.get(&key).copied()
            }
        }
    }

    /// Calculate the next index based on current selection and input action.
    /// This version requires ActionRegistry and InputBehaviorConfig.
    ///
    /// 根据当前选择和输入动作计算下一个索引。此版本需要 ActionRegistry 和 InputBehaviorConfig。
    #[allow(dead_code)]
    pub fn handle_input(
        &self,
        current: usize,
        action: &Action,
        registry: &ActionRegistry,
        behavior_config: &InputBehaviorConfig,
    ) -> Option<usize> {
        let direction = NavDirection::from_action(action, registry, behavior_config)?;
        self.handle_direction(current, direction)
    }

    /// Get the total number of selectable options.
    ///
    /// 获取可选项总数。
    pub fn option_count(&self) -> usize {
        match self {
            NavigatorType::Linear { option_count, .. } => *option_count,
            NavigatorType::Grid { rows, cols, .. } => rows * cols,
            NavigatorType::Custom { option_count, .. } => *option_count,
        }
    }

    fn handle_linear_input(
        current: usize,
        direction: NavDirection,
        linear_direction: LinearDirection,
        option_count: usize,
        looping: bool,
    ) -> Option<usize> {
        if option_count == 0 {
            return None;
        }

        let delta: isize = match (linear_direction, direction) {
            (LinearDirection::Vertical, NavDirection::Up) => -1,
            (LinearDirection::Vertical, NavDirection::Down) => 1,
            (LinearDirection::Horizontal, NavDirection::Left) => -1,
            (LinearDirection::Horizontal, NavDirection::Right) => 1,
            _ => return None,
        };

        let max_index = option_count.saturating_sub(1);
        let new_index = current as isize + delta;

        if looping {
            if new_index < 0 {
                Some(max_index)
            } else if new_index > max_index as isize {
                Some(0)
            } else {
                Some(new_index as usize)
            }
        } else {
            if new_index < 0 || new_index > max_index as isize {
                None
            } else {
                Some(new_index as usize)
            }
        }
    }

    fn handle_grid_input(
        current: usize,
        direction: NavDirection,
        rows: usize,
        cols: usize,
        looping: bool,
    ) -> Option<usize> {
        if rows == 0 || cols == 0 {
            return None;
        }

        let row = current / cols;
        let col = current % cols;

        let (new_row, new_col) = match direction {
            NavDirection::Up => {
                if row > 0 {
                    (row - 1, col)
                } else if looping {
                    (rows - 1, col)
                } else {
                    return None;
                }
            }
            NavDirection::Down => {
                if row < rows - 1 {
                    (row + 1, col)
                } else if looping {
                    (0, col)
                } else {
                    return None;
                }
            }
            NavDirection::Left => {
                if col > 0 {
                    (row, col - 1)
                } else if looping {
                    (row, cols - 1)
                } else {
                    return None;
                }
            }
            NavDirection::Right => {
                if col < cols - 1 {
                    (row, col + 1)
                } else if looping {
                    (row, 0)
                } else {
                    return None;
                }
            }
        };

        Some(new_row * cols + new_col)
    }
}

// ============================================================================
// InteractiveLayer Component
// ============================================================================

/// Component that defines an interactive layer with navigation capabilities.
///
/// 定义具有导航功能的交互层组件。
#[derive(Component, Debug, Clone)]
#[cfg_attr(feature = "debug", derive(Reflect))]
pub struct InteractiveLayer {
    /// Unique identifier for this layer.
    pub layer_id: String,

    /// Navigation strategy for this layer.
    #[cfg_attr(feature = "debug", reflect(ignore))]
    pub navigator: NavigatorType,

    /// Current selection index.
    pub current_selection: usize,

    /// Names of selectable elements (for reactive indicator positioning and highlighting).
    pub selectable_elements: Vec<String>,

    /// Optional sound to play on navigation.
    pub sound_on_navigate: Option<String>,

    /// Whether this layer is currently active/focused.
    pub is_active: bool,

    /// Transition rules when confirm is pressed.
    #[cfg_attr(feature = "debug", reflect(ignore))]
    pub on_confirm: Vec<LayerTransitionRule>,

    /// Transition action when cancel is pressed.
    #[cfg_attr(feature = "debug", reflect(ignore))]
    pub on_cancel: Option<LayerTransitionAction>,

    /// Sound to play on confirm.
    pub sound_on_confirm: Option<String>,

    /// Sound to play on cancel.
    pub sound_on_cancel: Option<String>,
}

impl InteractiveLayer {
    /// Create a new interactive layer.
    pub fn new(layer_id: impl Into<String>, navigator: NavigatorType) -> Self {
        Self {
            layer_id: layer_id.into(),
            navigator,
            current_selection: 0,
            selectable_elements: Vec::new(),
            sound_on_navigate: None,
            is_active: false,
            on_confirm: Vec::new(),
            on_cancel: None,
            sound_on_confirm: None,
            sound_on_cancel: None,
        }
    }

    pub fn with_selectable_elements(mut self, elements: Vec<String>) -> Self {
        self.selectable_elements = elements;
        self
    }

    pub fn with_sound(mut self, sound: impl Into<String>) -> Self {
        self.sound_on_navigate = Some(sound.into());
        self
    }

    #[allow(dead_code)]
    pub fn with_active(mut self, active: bool) -> Self {
        self.is_active = active;
        self
    }

    pub fn with_on_confirm(mut self, rules: Vec<LayerTransitionRule>) -> Self {
        self.on_confirm = rules;
        self
    }

    pub fn with_on_cancel(mut self, action: LayerTransitionAction) -> Self {
        self.on_cancel = Some(action);
        self
    }

    pub fn with_sound_on_confirm(mut self, sound: impl Into<String>) -> Self {
        self.sound_on_confirm = Some(sound.into());
        self
    }

    pub fn with_sound_on_cancel(mut self, sound: impl Into<String>) -> Self {
        self.sound_on_cancel = Some(sound.into());
        self
    }

    /// Get the currently selected element name, if any.
    pub fn current_element_name(&self) -> Option<&str> {
        self.selectable_elements
            .get(self.current_selection)
            .map(|s| s.as_str())
    }

    /// Handle navigation input using NavDirection and update selection.
    /// Returns `true` if selection changed.
    pub fn navigate_direction(&mut self, direction: NavDirection) -> bool {
        if let Some(new_index) = self
            .navigator
            .handle_direction(self.current_selection, direction)
            && new_index != self.current_selection
        {
            self.current_selection = new_index;
            return true;
        }
        false
    }

    /// Handle navigation input using Action and ActionRegistry.
    /// Returns `true` if selection changed.
    #[allow(dead_code)]
    pub fn navigate(
        &mut self,
        action: &Action,
        registry: &ActionRegistry,
        behavior_config: &InputBehaviorConfig,
    ) -> bool {
        if let Some(new_index) =
            self.navigator
                .handle_input(self.current_selection, action, registry, behavior_config)
            && new_index != self.current_selection
        {
            self.current_selection = new_index;
            return true;
        }
        false
    }

    /// Set selection to a specific index, clamping to valid range.
    pub fn set_selection(&mut self, index: usize) {
        let max = self.navigator.option_count().saturating_sub(1);
        self.current_selection = index.min(max);
    }
}

// ============================================================================
// RON Definition Types (for ViewLayoutAsset)
// ============================================================================

/// RON definition for an interactive layer.
#[derive(Debug, Clone, Deserialize)]
pub struct InteractiveLayerDef {
    pub navigator_type: NavigatorTypeDef,
    #[serde(default)]
    pub selectable_elements: Vec<String>,
    #[serde(default)]
    pub sound_on_navigate: Option<String>,
    #[serde(default)]
    pub on_confirm: Option<Vec<LayerTransitionRuleDef>>,
    #[serde(default)]
    pub on_cancel: Option<LayerTransitionActionDef>,
    #[serde(default)]
    pub sound_on_confirm: Option<String>,
    #[serde(default)]
    pub sound_on_cancel: Option<String>,
}

/// RON definition for a layer transition rule.
#[derive(Debug, Clone, Deserialize)]
pub struct LayerTransitionRuleDef {
    #[serde(default)]
    pub condition: Option<String>,
    pub action: LayerTransitionActionDef,
}

/// RON definition for a layer transition action.
#[derive(Debug, Clone, Deserialize)]
pub enum LayerTransitionActionDef {
    GotoLayer(String),
    PopState,
    PushState(String),
}

impl LayerTransitionRuleDef {
    pub fn into_rule(self) -> LayerTransitionRule {
        LayerTransitionRule {
            condition: self.condition,
            action: self.action.into_action(),
        }
    }
}

impl LayerTransitionActionDef {
    pub fn into_action(self) -> LayerTransitionAction {
        match self {
            LayerTransitionActionDef::GotoLayer(layer) => LayerTransitionAction::GotoLayer(layer),
            LayerTransitionActionDef::PopState => LayerTransitionAction::PopState,
            LayerTransitionActionDef::PushState(state) => LayerTransitionAction::PushState(state),
        }
    }
}

/// Option count definition supporting static values or dynamic expressions.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum OptionCountDef {
    Static(usize),
    Dynamic(String),
}

impl OptionCountDef {
    pub(crate) fn evaluate(&self, player_data: &crate::core::data::PlayerData) -> usize {
        match self {
            OptionCountDef::Static(value) => *value,
            OptionCountDef::Dynamic(expr) => evaluate_option_count_expression(expr, player_data),
        }
    }
}

fn evaluate_option_count_expression(
    expr: &str,
    player_data: &crate::core::data::PlayerData,
) -> usize {
    let expr = expr.trim();

    if expr == "inventory.len()" {
        return player_data.inventory.len();
    }

    if expr == "inventory_capacity" {
        return player_data.inventory_capacity;
    }

    if expr.starts_with("min(") && expr.ends_with(")") {
        let inner = &expr[4..expr.len() - 1];
        let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
        if parts.len() == 2 {
            let a = evaluate_option_count_expression(parts[0], player_data);
            let b = evaluate_option_count_expression(parts[1], player_data);
            return a.min(b);
        }
    }

    if expr.starts_with("max(") && expr.ends_with(")") {
        let inner = &expr[4..expr.len() - 1];
        let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
        if parts.len() == 2 {
            let a = evaluate_option_count_expression(parts[0], player_data);
            let b = evaluate_option_count_expression(parts[1], player_data);
            return a.max(b);
        }
    }

    if let Ok(value) = expr.parse::<usize>() {
        return value;
    }

    warn!(
        "Unable to evaluate option count expression: {}, defaulting to 1",
        expr
    );
    1
}

/// RON definition for navigator type.
#[derive(Debug, Clone, Deserialize)]
pub enum NavigatorTypeDef {
    Linear {
        #[serde(default)]
        direction: LinearDirection,
        option_count: OptionCountDef,
        #[serde(default)]
        looping: bool,
    },
    Grid {
        rows: usize,
        cols: usize,
        #[serde(default)]
        looping: bool,
    },
}

impl NavigatorTypeDef {
    pub(crate) fn into_navigator(
        self,
        player_data: &crate::core::data::PlayerData,
    ) -> NavigatorType {
        match self {
            NavigatorTypeDef::Linear {
                direction,
                option_count,
                looping,
            } => NavigatorType::Linear {
                direction,
                option_count: option_count.evaluate(player_data),
                looping,
            },
            NavigatorTypeDef::Grid {
                rows,
                cols,
                looping,
            } => NavigatorType::Grid {
                rows,
                cols,
                looping,
            },
        }
    }
}

impl InteractiveLayerDef {
    pub(crate) fn build(
        &self,
        layer_id: impl Into<String>,
        player_data: &crate::core::data::PlayerData,
    ) -> InteractiveLayer {
        let mut layer = InteractiveLayer::new(
            layer_id,
            self.navigator_type.clone().into_navigator(player_data),
        )
        .with_selectable_elements(self.selectable_elements.clone());

        if let Some(sound) = &self.sound_on_navigate {
            layer = layer.with_sound(sound.clone());
        }

        if let Some(on_confirm) = &self.on_confirm {
            layer =
                layer.with_on_confirm(on_confirm.iter().map(|r| r.clone().into_rule()).collect());
        }

        if let Some(on_cancel) = &self.on_cancel {
            layer = layer.with_on_cancel(on_cancel.clone().into_action());
        }

        if let Some(sound) = &self.sound_on_confirm {
            layer = layer.with_sound_on_confirm(sound.clone());
        }

        if let Some(sound) = &self.sound_on_cancel {
            layer = layer.with_sound_on_cancel(sound.clone());
        }

        layer
    }
}

// ============================================================================
// AwaitingInteraction Component
// ============================================================================

/// Component marking an entity that is waiting for player interaction.
#[derive(Component, Debug, Clone, Default)]
#[cfg_attr(feature = "debug", derive(Reflect))]
pub struct AwaitingInteraction {
    /// The Chapter entity that is waiting for this interaction to complete.
    pub chapter_entity: Option<Entity>,
}

/// Result of an interaction completion.
#[derive(Component, Debug, Clone)]
#[cfg_attr(feature = "debug", derive(Reflect))]
pub struct InteractionResult {
    pub selected_index: usize,
    #[cfg_attr(feature = "debug", reflect(ignore))]
    pub selected_element: Option<String>,
    #[cfg_attr(feature = "debug", reflect(ignore))]
    pub layer_id: String,
}

impl InteractionResult {
    pub fn new(
        selected_index: usize,
        selected_element: Option<String>,
        layer_id: impl Into<String>,
    ) -> Self {
        Self {
            selected_index,
            selected_element,
            layer_id: layer_id.into(),
        }
    }
}

// ============================================================================
// Selection Events
// ============================================================================

/// Message fired when an interactive layer's selection changes.
#[derive(bevy::ecs::message::Message, Debug, Clone)]
pub struct SelectionChangedEvent {
    pub layer_id: String,
    pub previous_index: usize,
    pub new_index: usize,
    pub entity: Entity,
}

/// Message fired when an interactive layer receives a confirm action.
#[derive(bevy::ecs::message::Message, Debug, Clone)]
pub struct SelectionConfirmedEvent {
    pub layer_id: String,
    pub selected_index: usize,
    pub selected_element: Option<String>,
    pub entity: Entity,
}

/// Message fired when an interactive layer receives a cancel action.
#[derive(bevy::ecs::message::Message, Debug, Clone)]
pub struct SelectionCancelledEvent {
    pub layer_id: String,
    pub entity: Entity,
}

/// Message fired when an interactive layer becomes active.
#[derive(bevy::ecs::message::Message, Debug, Clone)]
pub struct LayerActivatedEvent {
    pub layer_id: String,
    pub current_selection: usize,
    pub entity: Entity,
}

/// Message fired when an interactive layer becomes inactive.
#[derive(bevy::ecs::message::Message, Debug, Clone)]
pub struct LayerDeactivatedEvent {
    pub layer_id: String,
    pub entity: Entity,
}
