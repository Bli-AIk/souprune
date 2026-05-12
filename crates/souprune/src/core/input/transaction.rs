//! # transaction.rs
//!
//! # transaction.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Defines the shared input transaction protocol used by framework systems and
//! mod-facing bridges.
//!
//! 定义框架系统与面向模组的桥接层共用的输入事务协议。

use super::{Action, ActionRegistry, ActionStateExt, InputBehaviorConfig};
use crate::core::mode::SequenceMode;
use bevy::prelude::*;
use leafwing_input_manager::action_state::ActionState;
use souprune_api::Action as SemanticAction;

/// System set for generating input transactions from action state.
///
/// 从动作状态生成输入事务的系统集。
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct InputTransactionSet;

/// High-level input context identifier.
///
/// 高层输入上下文标识。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InputContextId {
    Overworld,
    Battle,
    Dialogue,
    View,
    Custom(String),
}

/// Target receiving an input command.
///
/// 接收输入命令的目标。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InputTarget {
    ActiveView,
    FreScope,
    Behavior(String),
}

/// Navigation direction used by input commands.
///
/// 输入命令使用的导航方向。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

/// Semantic input command after raw actions are normalized.
///
/// 原始动作被标准化之后得到的语义输入命令。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InputCommand {
    Navigate(Direction),
    Confirm,
    Cancel,
    Menu,
}

impl From<SemanticAction> for InputCommand {
    fn from(action: SemanticAction) -> Self {
        match action {
            SemanticAction::Up => Self::Navigate(Direction::Up),
            SemanticAction::Down => Self::Navigate(Direction::Down),
            SemanticAction::Left => Self::Navigate(Direction::Left),
            SemanticAction::Right => Self::Navigate(Direction::Right),
            SemanticAction::Confirm => Self::Confirm,
            SemanticAction::Cancel => Self::Cancel,
            SemanticAction::Menu => Self::Menu,
        }
    }
}

impl InputCommand {
    /// Convert a semantic action into an input command.
    ///
    /// 将语义动作转换为输入命令。
    pub fn from_action(action: SemanticAction) -> Self {
        action.into()
    }
}

/// A lightweight input effect request emitted by input handlers.
///
/// 输入处理器发出的轻量输入效果请求。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InputEffect {
    EmitEvent(String),
    OpenView(String),
    CloseView,
}

/// A single input transaction traveling through the routing layer.
///
/// 经过路由层流转的一笔输入事务。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InputEnvelope {
    pub context: InputContextId,
    pub target: InputTarget,
    pub command: InputCommand,
    pub source_action: String,
}

impl InputEnvelope {
    /// Create a new input envelope.
    ///
    /// 创建一个新的输入事务封装。
    pub fn new(
        context: InputContextId,
        target: InputTarget,
        command: InputCommand,
        source_action: impl Into<String>,
    ) -> Self {
        Self {
            context,
            target,
            command,
            source_action: source_action.into(),
        }
    }
}

/// Message emitted when a semantic input command is detected.
///
/// 检测到语义输入命令时发出的消息。
#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub struct InputEnvelopeEvent {
    pub envelope: InputEnvelope,
}

impl InputEnvelopeEvent {
    /// Create a new input envelope message.
    ///
    /// 创建新的输入事务消息。
    pub fn new(envelope: InputEnvelope) -> Self {
        Self { envelope }
    }
}

/// Result of handling an input transaction.
///
/// 处理输入事务后的结果。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InputResult {
    Ignored,
    Consumed(Vec<InputEffect>),
    PassThrough(Vec<InputEffect>),
}

fn configured_input_commands(config: &InputBehaviorConfig) -> Vec<(&str, InputCommand)> {
    let mut commands = Vec::new();

    if let Some(action) = config.nav_up() {
        commands.push((action, InputCommand::Navigate(Direction::Up)));
    }
    if let Some(action) = config.nav_down() {
        commands.push((action, InputCommand::Navigate(Direction::Down)));
    }
    if let Some(action) = config.nav_left() {
        commands.push((action, InputCommand::Navigate(Direction::Left)));
    }
    if let Some(action) = config.nav_right() {
        commands.push((action, InputCommand::Navigate(Direction::Right)));
    }
    if let Some(action) = config.ui_confirm() {
        commands.push((action, InputCommand::Confirm));
    }
    if let Some(action) = config.ui_cancel() {
        commands.push((action, InputCommand::Cancel));
    }
    if let Some(action) = config.ui_menu() {
        commands.push((action, InputCommand::Menu));
    }

    commands
}

fn context_from_mode(mode: Option<&SequenceMode>) -> InputContextId {
    match mode.and_then(|mode| mode.0.as_deref()) {
        Some("overworld") => InputContextId::Overworld,
        Some("battle") => InputContextId::Battle,
        Some("dialogue") => InputContextId::Dialogue,
        Some("view") => InputContextId::View,
        Some(other) => InputContextId::Custom(other.to_string()),
        None => InputContextId::Custom("unknown".to_string()),
    }
}

fn collect_input_envelopes(
    action_state: &ActionState<Action>,
    registry: &ActionRegistry,
    behavior_config: &InputBehaviorConfig,
    context: InputContextId,
    target: InputTarget,
) -> Vec<InputEnvelope> {
    configured_input_commands(behavior_config)
        .into_iter()
        .filter_map(|(source_action, command)| {
            action_state
                .action_just_pressed(registry, source_action)
                .then(|| {
                    InputEnvelope::new(
                        context.clone(),
                        target.clone(),
                        command,
                        source_action.to_string(),
                    )
                })
        })
        .collect()
}

/// Emit input envelope messages from the current action state.
///
/// 从当前动作状态发出输入事务消息。
pub fn emit_input_envelopes_system(
    registry: Res<ActionRegistry>,
    behavior_config: Res<InputBehaviorConfig>,
    mode: Option<Res<SequenceMode>>,
    query: Query<&ActionState<Action>>,
    mut writer: MessageWriter<InputEnvelopeEvent>,
) {
    let Some(action_state) = query.iter().next() else {
        return;
    };

    let context = context_from_mode(mode.as_deref());
    let target = InputTarget::FreScope;

    for envelope in collect_input_envelopes(
        action_state,
        &registry,
        &behavior_config,
        context.clone(),
        target.clone(),
    ) {
        writer.write(InputEnvelopeEvent::new(envelope));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::input::{NavigationConfig, UIConfig};
    use bevy::platform::time::Instant;

    #[test]
    fn semantic_actions_map_to_input_commands() {
        let cases = [
            (SemanticAction::Up, InputCommand::Navigate(Direction::Up)),
            (
                SemanticAction::Down,
                InputCommand::Navigate(Direction::Down),
            ),
            (
                SemanticAction::Left,
                InputCommand::Navigate(Direction::Left),
            ),
            (
                SemanticAction::Right,
                InputCommand::Navigate(Direction::Right),
            ),
            (SemanticAction::Confirm, InputCommand::Confirm),
            (SemanticAction::Cancel, InputCommand::Cancel),
            (SemanticAction::Menu, InputCommand::Menu),
        ];

        for (action, expected) in cases {
            assert_eq!(InputCommand::from_action(action), expected);
            assert_eq!(InputCommand::from(action), expected);
        }
    }

    #[test]
    fn envelope_keeps_source_action_and_route() {
        let envelope = InputEnvelope::new(
            InputContextId::Battle,
            InputTarget::ActiveView,
            InputCommand::Confirm,
            "Confirm",
        );

        assert!(matches!(envelope.context, InputContextId::Battle));
        assert!(matches!(envelope.target, InputTarget::ActiveView));
        assert!(matches!(envelope.command, InputCommand::Confirm));
        assert_eq!(envelope.source_action, "Confirm");
    }

    fn test_registry_and_config(
        action_name: &str,
    ) -> (ActionRegistry, InputBehaviorConfig, Action) {
        let mut registry = ActionRegistry::new();
        let action = registry
            .register(action_name)
            .expect("test action should register");
        let config = InputBehaviorConfig {
            navigation: NavigationConfig {
                down: Some(action_name.to_string()),
                ..Default::default()
            },
            ui: UIConfig::default(),
        };

        (registry, config, action)
    }

    #[test]
    fn just_pressed_down_action_generates_navigate_down_envelope() {
        let (registry, config, action) = test_registry_and_config("MoveDown");
        let mut action_state = ActionState::<Action>::default();
        action_state.press(&action);

        let envelopes = collect_input_envelopes(
            &action_state,
            &registry,
            &config,
            InputContextId::Battle,
            InputTarget::FreScope,
        );

        assert_eq!(envelopes.len(), 1);
        assert_eq!(
            envelopes[0].command,
            InputCommand::Navigate(Direction::Down)
        );
        assert_eq!(envelopes[0].source_action, "MoveDown");
    }

    #[test]
    fn held_down_action_does_not_repeat_discrete_envelope() {
        let (registry, config, action) = test_registry_and_config("MoveDown");
        let mut action_state = ActionState::<Action>::default();
        action_state.press(&action);

        let now = Instant::now();
        action_state.tick(now, now);

        let envelopes = collect_input_envelopes(
            &action_state,
            &registry,
            &config,
            InputContextId::Battle,
            InputTarget::FreScope,
        );

        assert!(action_state.pressed(&action));
        assert!(envelopes.is_empty());
    }
}
