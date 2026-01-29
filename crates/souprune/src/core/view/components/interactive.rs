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

use crate::core::input::Action;

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
    ///
    /// # Fields
    /// - `direction`: Primary navigation direction (Vertical or Horizontal)
    /// - `option_count`: Total number of selectable options
    /// - `looping`: Whether navigation wraps around at boundaries
    ///
    /// # 字段
    /// - `direction`: 主要导航方向（垂直或水平）
    /// - `option_count`: 可选项总数
    /// - `looping`: 是否在边界处循环
    Linear {
        direction: LinearDirection,
        option_count: usize,
        #[serde(default)]
        looping: bool,
    },

    /// Grid navigation for 2D button layouts.
    ///
    /// 用于 2D 按钮布局的网格导航。
    ///
    /// # Fields
    /// - `rows`: Number of rows in the grid
    /// - `cols`: Number of columns in the grid
    /// - `looping`: Whether navigation wraps around at boundaries
    ///
    /// # 字段
    /// - `rows`: 网格的行数
    /// - `cols`: 网格的列数
    /// - `looping`: 是否在边界处循环
    Grid {
        rows: usize,
        cols: usize,
        #[serde(default)]
        looping: bool,
    },

    /// Custom navigation with explicit position mappings.
    ///
    /// 使用显式位置映射的自定义导航。
    ///
    /// This allows non-standard navigation patterns where each position
    /// has explicit neighbors defined.
    ///
    /// 这允许非标准导航模式，其中每个位置都有显式定义的邻居。
    Custom {
        /// Mapping from (current_index, action) to next_index.
        /// Format: { "0:Left": 3, "0:Right": 1, "1:Left": 0, ... }
        ///
        /// 从 (当前索引, 动作) 到下一个索引的映射。
        /// 格式: { "0:Left": 3, "0:Right": 1, "1:Left": 0, ... }
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
    ///
    /// 使用上/下键导航。
    #[default]
    Vertical,
    /// Navigate with Left/Right keys.
    ///
    /// 使用左/右键导航。
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
    /// If None, rule always matches.
    /// Format: "index == 0", "index == 1 && !player.inventory.is_empty", etc.
    ///
    /// 可选条件表达式。
    /// 如果为 None，规则始终匹配。
    /// 格式: "index == 0", "index == 1 && !player.inventory.is_empty" 等。
    pub condition: Option<String>,

    /// Action to execute when this rule matches.
    ///
    /// 规则匹配时执行的动作。
    pub action: LayerTransitionAction,
}

/// Action to perform when a transition is triggered.
///
/// 转换触发时执行的动作。
#[derive(Debug, Clone)]
pub enum LayerTransitionAction {
    /// Switch to another layer within the same context.
    ///
    /// 切换到同一上下文中的另一个层。
    GotoLayer(String),

    /// Pop the current state (e.g., close menu, return to previous state).
    ///
    /// 弹出当前状态（例如关闭菜单、返回上一状态）。
    PopState,

    /// Push a new state onto the stack.
    ///
    /// 将新状态推入栈。
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
    /// Calculate the next index based on current selection and input action.
    ///
    /// 根据当前选择和输入动作计算下一个索引。
    ///
    /// Returns `Some(new_index)` if navigation should occur, `None` otherwise.
    ///
    /// 如果应发生导航，返回 `Some(new_index)`，否则返回 `None`。
    pub fn handle_input(&self, current: usize, action: &Action) -> Option<usize> {
        match self {
            NavigatorType::Linear {
                direction,
                option_count,
                looping,
            } => Self::handle_linear_input(current, action, *direction, *option_count, *looping),

            NavigatorType::Grid {
                rows,
                cols,
                looping,
            } => Self::handle_grid_input(current, action, *rows, *cols, *looping),

            NavigatorType::Custom {
                mappings,
                option_count: _,
            } => {
                let key = format!("{}:{:?}", current, action);
                mappings.get(&key).copied()
            }
        }
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
        action: &Action,
        direction: LinearDirection,
        option_count: usize,
        looping: bool,
    ) -> Option<usize> {
        if option_count == 0 {
            return None;
        }

        let delta: isize = match (direction, action) {
            (LinearDirection::Vertical, Action::Up) => -1,
            (LinearDirection::Vertical, Action::Down) => 1,
            (LinearDirection::Horizontal, Action::Left) => -1,
            (LinearDirection::Horizontal, Action::Right) => 1,
            _ => return None,
        };

        let max_index = option_count.saturating_sub(1);
        let new_index = current as isize + delta;

        if looping {
            // Wrap around
            if new_index < 0 {
                Some(max_index)
            } else if new_index > max_index as isize {
                Some(0)
            } else {
                Some(new_index as usize)
            }
        } else {
            // Clamp to bounds
            if new_index < 0 || new_index > max_index as isize {
                None // No change at boundary
            } else {
                Some(new_index as usize)
            }
        }
    }

    fn handle_grid_input(
        current: usize,
        action: &Action,
        rows: usize,
        cols: usize,
        looping: bool,
    ) -> Option<usize> {
        if rows == 0 || cols == 0 {
            return None;
        }

        let row = current / cols;
        let col = current % cols;

        let (new_row, new_col) = match action {
            Action::Up => {
                if row > 0 {
                    (row - 1, col)
                } else if looping {
                    (rows - 1, col)
                } else {
                    return None;
                }
            }
            Action::Down => {
                if row < rows - 1 {
                    (row + 1, col)
                } else if looping {
                    (0, col)
                } else {
                    return None;
                }
            }
            Action::Left => {
                if col > 0 {
                    (row, col - 1)
                } else if looping {
                    (row, cols - 1)
                } else {
                    return None;
                }
            }
            Action::Right => {
                if col < cols - 1 {
                    (row, col + 1)
                } else if looping {
                    (row, 0)
                } else {
                    return None;
                }
            }
            _ => return None,
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
///
/// This unifies the navigation logic for both Overworld menus and Battle UI.
///
/// 这统一了 Overworld 菜单和 Battle UI 的导航逻辑。
#[derive(Component, Debug, Clone)]
#[cfg_attr(feature = "debug", derive(Reflect))]
pub struct InteractiveLayer {
    /// Unique identifier for this layer.
    ///
    /// 此层的唯一标识符。
    pub layer_id: String,

    /// Navigation strategy for this layer.
    ///
    /// 此层的导航策略。
    #[cfg_attr(feature = "debug", reflect(ignore))]
    pub navigator: NavigatorType,

    /// Current selection index.
    ///
    /// 当前选择索引。
    pub current_selection: usize,

    /// Names of selectable elements (for reactive indicator positioning and highlighting).
    ///
    /// 可选元素的名称（用于响应式指示器定位和高亮）。
    ///
    /// The index in this list corresponds to the selection index.
    ///
    /// 此列表中的索引对应于选择索引。
    pub selectable_elements: Vec<String>,

    /// Optional sound to play on navigation.
    ///
    /// 导航时播放的可选声音。
    pub sound_on_navigate: Option<String>,

    /// Whether this layer is currently active/focused.
    ///
    /// 此层是否当前活跃/聚焦。
    pub is_active: bool,

    /// Transition rules when confirm is pressed.
    ///
    /// 确认时的转换规则。
    ///
    /// Rules are evaluated in order; first matching rule is executed.
    ///
    /// 规则按顺序求值；执行第一个匹配的规则。
    #[cfg_attr(feature = "debug", reflect(ignore))]
    pub on_confirm: Vec<LayerTransitionRule>,

    /// Transition action when cancel is pressed.
    ///
    /// 取消时的转换动作。
    #[cfg_attr(feature = "debug", reflect(ignore))]
    pub on_cancel: Option<LayerTransitionAction>,

    /// Sound to play on confirm.
    ///
    /// 确认时播放的声音。
    pub sound_on_confirm: Option<String>,

    /// Sound to play on cancel.
    ///
    /// 取消时播放的声音。
    pub sound_on_cancel: Option<String>,
}

impl InteractiveLayer {
    /// Create a new interactive layer.
    ///
    /// 创建新的交互层。
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

    /// Set the selectable elements.
    ///
    /// 设置可选元素。
    pub fn with_selectable_elements(mut self, elements: Vec<String>) -> Self {
        self.selectable_elements = elements;
        self
    }

    /// Set the navigation sound.
    ///
    /// 设置导航声音。
    pub fn with_sound(mut self, sound: impl Into<String>) -> Self {
        self.sound_on_navigate = Some(sound.into());
        self
    }

    /// Set active state.
    ///
    /// 设置活跃状态。
    pub fn with_active(mut self, active: bool) -> Self {
        self.is_active = active;
        self
    }

    /// Set confirm transition rules.
    ///
    /// 设置确认转换规则。
    pub fn with_on_confirm(mut self, rules: Vec<LayerTransitionRule>) -> Self {
        self.on_confirm = rules;
        self
    }

    /// Set cancel transition action.
    ///
    /// 设置取消转换动作。
    pub fn with_on_cancel(mut self, action: LayerTransitionAction) -> Self {
        self.on_cancel = Some(action);
        self
    }

    /// Set confirm sound.
    ///
    /// 设置确认声音。
    pub fn with_sound_on_confirm(mut self, sound: impl Into<String>) -> Self {
        self.sound_on_confirm = Some(sound.into());
        self
    }

    /// Set cancel sound.
    ///
    /// 设置取消声音。
    pub fn with_sound_on_cancel(mut self, sound: impl Into<String>) -> Self {
        self.sound_on_cancel = Some(sound.into());
        self
    }

    /// Get the currently selected element name, if any.
    ///
    /// 获取当前选中的元素名称（如有）。
    pub fn current_element_name(&self) -> Option<&str> {
        self.selectable_elements
            .get(self.current_selection)
            .map(|s| s.as_str())
    }

    /// Handle navigation input and update selection.
    ///
    /// 处理导航输入并更新选择。
    ///
    /// Returns `true` if selection changed.
    ///
    /// 如果选择发生变化，返回 `true`。
    pub fn navigate(&mut self, action: &Action) -> bool {
        if let Some(new_index) = self.navigator.handle_input(self.current_selection, action)
            && new_index != self.current_selection {
                self.current_selection = new_index;
                return true;
            }
        false
    }

    /// Set selection to a specific index, clamping to valid range.
    ///
    /// 设置选择为特定索引，限制在有效范围内。
    pub fn set_selection(&mut self, index: usize) {
        let max = self.navigator.option_count().saturating_sub(1);
        self.current_selection = index.min(max);
    }
}

// ============================================================================
// RON Definition Types (for ViewLayoutAsset)
// ============================================================================

/// RON definition for an interactive layer.
///
/// 交互层的 RON 定义。
#[derive(Debug, Clone, Deserialize)]
pub struct InteractiveLayerDef {
    /// Navigation strategy type.
    ///
    /// 导航策略类型。
    pub navigator_type: NavigatorTypeDef,

    /// Names of selectable elements.
    ///
    /// 可选元素的名称。
    #[serde(default)]
    pub selectable_elements: Vec<String>,

    /// Sound to play on navigation.
    ///
    /// 导航时播放的声音。
    #[serde(default)]
    pub sound_on_navigate: Option<String>,

    /// Transition rules when confirm is pressed.
    ///
    /// 确认时的转换规则。
    #[serde(default)]
    pub on_confirm: Option<Vec<LayerTransitionRuleDef>>,

    /// Transition action when cancel is pressed.
    ///
    /// 取消时的转换动作。
    #[serde(default)]
    pub on_cancel: Option<LayerTransitionActionDef>,

    /// Sound to play on confirm.
    ///
    /// 确认时播放的声音。
    #[serde(default)]
    pub sound_on_confirm: Option<String>,

    /// Sound to play on cancel.
    ///
    /// 取消时播放的声音。
    #[serde(default)]
    pub sound_on_cancel: Option<String>,
}

/// RON definition for a layer transition rule.
///
/// 层级转换规则的 RON 定义。
#[derive(Debug, Clone, Deserialize)]
pub struct LayerTransitionRuleDef {
    /// Optional condition expression.
    ///
    /// 可选条件表达式。
    #[serde(default)]
    pub condition: Option<String>,

    /// Action to execute when rule matches.
    ///
    /// 规则匹配时执行的动作。
    pub action: LayerTransitionActionDef,
}

/// RON definition for a layer transition action.
///
/// 层级转换动作的 RON 定义。
#[derive(Debug, Clone, Deserialize)]
pub enum LayerTransitionActionDef {
    /// Switch to another layer.
    GotoLayer(String),
    /// Pop the current state.
    PopState,
    /// Push a new state.
    PushState(String),
}

impl LayerTransitionRuleDef {
    /// Convert to runtime LayerTransitionRule.
    ///
    /// 转换为运行时 LayerTransitionRule。
    pub fn into_rule(self) -> LayerTransitionRule {
        LayerTransitionRule {
            condition: self.condition,
            action: self.action.into_action(),
        }
    }
}

impl LayerTransitionActionDef {
    /// Convert to runtime LayerTransitionAction.
    ///
    /// 转换为运行时 LayerTransitionAction。
    pub fn into_action(self) -> LayerTransitionAction {
        match self {
            LayerTransitionActionDef::GotoLayer(layer) => LayerTransitionAction::GotoLayer(layer),
            LayerTransitionActionDef::PopState => LayerTransitionAction::PopState,
            LayerTransitionActionDef::PushState(state) => LayerTransitionAction::PushState(state),
        }
    }
}

/// Option count definition supporting static values or dynamic expressions.
///
/// 支持静态值或动态表达式的选项数量定义。
///
/// # Examples
/// ```ron
/// // Static value
/// option_count: 4,
///
/// // Dynamic expression
/// option_count: "inventory.len()",
/// option_count: "min(inventory.len(), 8)",
/// ```
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum OptionCountDef {
    /// A static numeric value.
    ///
    /// 静态数值。
    Static(usize),
    /// A dynamic expression to evaluate at runtime.
    ///
    /// 运行时求值的动态表达式。
    Dynamic(String),
}

impl OptionCountDef {
    /// Evaluate the option count, returning a static value or evaluating the expression.
    ///
    /// 求值选项数量，返回静态值或求值表达式。
    pub fn evaluate(&self, player_data: &crate::core::data::PlayerData) -> usize {
        match self {
            OptionCountDef::Static(value) => *value,
            OptionCountDef::Dynamic(expr) => evaluate_option_count_expression(expr, player_data),
        }
    }
}

/// Evaluate an option count expression.
///
/// 求值选项数量表达式。
///
/// Supported expressions:
/// - `inventory.len()` - Number of items in inventory
/// - `inventory_capacity` - Maximum inventory size
/// - `min(a, b)` - Minimum of two values
/// - `max(a, b)` - Maximum of two values
/// - Numeric literals
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
///
/// 导航器类型的 RON 定义。
#[derive(Debug, Clone, Deserialize)]
pub enum NavigatorTypeDef {
    /// Linear navigation.
    ///
    /// 线性导航。
    Linear {
        #[serde(default)]
        direction: LinearDirection,
        option_count: OptionCountDef,
        #[serde(default)]
        looping: bool,
    },

    /// Grid navigation.
    ///
    /// 网格导航。
    Grid {
        rows: usize,
        cols: usize,
        #[serde(default)]
        looping: bool,
    },
}

impl NavigatorTypeDef {
    /// Convert to runtime NavigatorType with expression evaluation.
    ///
    /// 使用表达式求值转换为运行时 NavigatorType。
    pub fn into_navigator(self, player_data: &crate::core::data::PlayerData) -> NavigatorType {
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
    /// Build an InteractiveLayer component from this definition.
    ///
    /// 从此定义构建 InteractiveLayer 组件。
    ///
    /// # Arguments
    /// - `layer_id`: Unique identifier for this layer
    /// - `player_data`: Player data for evaluating dynamic expressions
    pub fn build(
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

        // Add transition rules
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
// 等待交互组件
// ============================================================================

/// Component marking an entity that is waiting for player interaction.
///
/// 标记正在等待玩家交互的实体的组件。
///
/// When attached to an InteractiveLayer entity, the layer becomes the active
/// interaction target. The interaction is considered complete when the player
/// presses the confirm button.
///
/// 当附加到 InteractiveLayer 实体时，该层成为活跃的交互目标。
/// 当玩家按下确认按钮时，交互被视为完成。
///
/// # Battle Integration / 战斗集成
///
/// In battle, this is used with the `AwaitInteraction` Chapter to block
/// the sequencer until the player makes a selection.
///
/// 在战斗中，这与 `AwaitInteraction` Chapter 一起使用，
/// 阻塞 sequencer 直到玩家做出选择。
#[derive(Component, Debug, Clone, Default)]
#[cfg_attr(feature = "debug", derive(Reflect))]
pub struct AwaitingInteraction {
    /// The Chapter entity that is waiting for this interaction to complete.
    ///
    /// 正在等待此交互完成的 Chapter 实体。
    ///
    /// When interaction is confirmed, this entity will be marked with `ChapterFinished`.
    ///
    /// 当交互确认时，此实体将被标记为 `ChapterFinished`。
    pub chapter_entity: Option<Entity>,
}

/// Result of an interaction completion.
///
/// 交互完成的结果。
///
/// This is stored for the Chapter to retrieve the player's selection.
///
/// 这被存储以供 Chapter 检索玩家的选择。
#[derive(Component, Debug, Clone)]
#[cfg_attr(feature = "debug", derive(Reflect))]
pub struct InteractionResult {
    /// The selected index (0-based).
    ///
    /// 选中的索引（从 0 开始）。
    pub selected_index: usize,

    /// The name of the selected element, if available.
    ///
    /// 选中元素的名称（如有）。
    #[cfg_attr(feature = "debug", reflect(ignore))]
    pub selected_element: Option<String>,

    /// The layer ID where the selection was made.
    ///
    /// 做出选择的层 ID。
    #[cfg_attr(feature = "debug", reflect(ignore))]
    pub layer_id: String,
}

impl InteractionResult {
    /// Create a new interaction result.
    ///
    /// 创建新的交互结果。
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
// Selection Changed Event
// ============================================================================

/// Message fired when an interactive layer's selection changes.
///
/// 当交互层的选择发生变化时触发的消息。
#[derive(bevy::ecs::message::Message, Debug, Clone)]
pub struct SelectionChangedEvent {
    /// The layer ID that changed.
    ///
    /// 发生变化的层 ID。
    pub layer_id: String,

    /// Previous selection index.
    ///
    /// 之前的选择索引。
    pub previous_index: usize,

    /// New selection index.
    ///
    /// 新的选择索引。
    pub new_index: usize,

    /// Entity of the layer.
    ///
    /// 层的实体。
    pub entity: Entity,
}

/// Message fired when an interactive layer receives a confirm action.
///
/// 当交互层收到确认动作时触发的消息。
#[derive(bevy::ecs::message::Message, Debug, Clone)]
pub struct SelectionConfirmedEvent {
    /// The layer ID.
    ///
    /// 层 ID。
    pub layer_id: String,

    /// Selected index.
    ///
    /// 选中的索引。
    pub selected_index: usize,

    /// Name of selected element, if available.
    ///
    /// 选中元素的名称（如有）。
    pub selected_element: Option<String>,

    /// Entity of the layer.
    ///
    /// 层的实体。
    pub entity: Entity,
}

/// Message fired when an interactive layer receives a cancel action.
///
/// 当交互层收到取消动作时触发的消息。
#[derive(bevy::ecs::message::Message, Debug, Clone)]
pub struct SelectionCancelledEvent {
    /// The layer ID.
    ///
    /// 层 ID。
    pub layer_id: String,

    /// Entity of the layer.
    ///
    /// 层的实体。
    pub entity: Entity,
}

/// Message fired when an interactive layer becomes active.
///
/// 当交互层变为活跃状态时触发的消息。
#[derive(bevy::ecs::message::Message, Debug, Clone)]
pub struct LayerActivatedEvent {
    /// The layer ID.
    ///
    /// 层 ID。
    pub layer_id: String,

    /// Current selection index when activated.
    ///
    /// 激活时的当前选择索引。
    pub current_selection: usize,

    /// Entity of the layer.
    ///
    /// 层的实体。
    pub entity: Entity,
}

/// Message fired when an interactive layer becomes inactive.
///
/// 当交互层变为非活跃状态时触发的消息。
#[derive(bevy::ecs::message::Message, Debug, Clone)]
pub struct LayerDeactivatedEvent {
    /// The layer ID.
    ///
    /// 层 ID。
    pub layer_id: String,

    /// Entity of the layer.
    ///
    /// 层的实体。
    pub entity: Entity,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_vertical_navigation() {
        let nav = NavigatorType::Linear {
            direction: LinearDirection::Vertical,
            option_count: 3,
            looping: false,
        };

        // Start at 0, go down
        assert_eq!(nav.handle_input(0, &Action::Down), Some(1));
        assert_eq!(nav.handle_input(1, &Action::Down), Some(2));
        assert_eq!(nav.handle_input(2, &Action::Down), None); // At boundary

        // Go up
        assert_eq!(nav.handle_input(2, &Action::Up), Some(1));
        assert_eq!(nav.handle_input(0, &Action::Up), None); // At boundary

        // Left/Right should not work in vertical mode
        assert_eq!(nav.handle_input(1, &Action::Left), None);
        assert_eq!(nav.handle_input(1, &Action::Right), None);
    }

    #[test]
    fn test_linear_looping() {
        let nav = NavigatorType::Linear {
            direction: LinearDirection::Vertical,
            option_count: 3,
            looping: true,
        };

        // Loop from end to start
        assert_eq!(nav.handle_input(2, &Action::Down), Some(0));
        // Loop from start to end
        assert_eq!(nav.handle_input(0, &Action::Up), Some(2));
    }

    #[test]
    fn test_grid_navigation() {
        // 2x2 grid:
        // 0 1
        // 2 3
        let nav = NavigatorType::Grid {
            rows: 2,
            cols: 2,
            looping: false,
        };

        // Right
        assert_eq!(nav.handle_input(0, &Action::Right), Some(1));
        assert_eq!(nav.handle_input(1, &Action::Right), None); // At boundary

        // Down
        assert_eq!(nav.handle_input(0, &Action::Down), Some(2));
        assert_eq!(nav.handle_input(2, &Action::Down), None); // At boundary

        // Left
        assert_eq!(nav.handle_input(1, &Action::Left), Some(0));
        assert_eq!(nav.handle_input(0, &Action::Left), None); // At boundary

        // Up
        assert_eq!(nav.handle_input(2, &Action::Up), Some(0));
        assert_eq!(nav.handle_input(0, &Action::Up), None); // At boundary
    }

    #[test]
    fn test_grid_looping() {
        let nav = NavigatorType::Grid {
            rows: 2,
            cols: 2,
            looping: true,
        };

        // Loop right
        assert_eq!(nav.handle_input(1, &Action::Right), Some(0));
        // Loop left
        assert_eq!(nav.handle_input(0, &Action::Left), Some(1));
        // Loop down
        assert_eq!(nav.handle_input(2, &Action::Down), Some(0));
        // Loop up
        assert_eq!(nav.handle_input(0, &Action::Up), Some(2));
    }

    #[test]
    fn test_interactive_layer_navigate() {
        let mut layer = InteractiveLayer::new(
            "test",
            NavigatorType::Linear {
                direction: LinearDirection::Vertical,
                option_count: 3,
                looping: false,
            },
        );

        assert_eq!(layer.current_selection, 0);
        assert!(layer.navigate(&Action::Down));
        assert_eq!(layer.current_selection, 1);
        assert!(!layer.navigate(&Action::Left)); // No change
        assert_eq!(layer.current_selection, 1);
    }
}
