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

use souprune_api::Action as SemanticAction;

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

/// Result of handling an input transaction.
///
/// 处理输入事务后的结果。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InputResult {
    Ignored,
    Consumed(Vec<InputEffect>),
    PassThrough(Vec<InputEffect>),
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
