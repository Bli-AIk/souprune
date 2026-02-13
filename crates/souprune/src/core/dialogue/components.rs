//! Dialogue system components.
//!
//! 对话系统组件。

use bevy::prelude::*;

/// Dialogue controller component - manages Mortar dialogue state for an entity.
///
/// 对话控制器组件 - 管理实体的 Mortar 对话状态。
///
/// This component can be combined with [`Typewriter`] for typewriter effects,
/// but they are independent and can be used separately.
///
/// 此组件可与 [`Typewriter`] 组合使用以实现打字机效果，
/// 但它们相互独立，可以单独使用。
#[derive(Component, Debug, Clone, Default, Reflect)]
#[reflect(Component)]
pub struct MortarController {
    /// Currently bound mortar file path.
    ///
    /// 当前绑定的 mortar 文件路径。
    pub mortar_path: Option<String>,

    /// Current node name.
    ///
    /// 当前节点名。
    pub current_node: Option<String>,
}

impl MortarController {
    /// Creates a new MortarController.
    ///
    /// 创建一个新的 MortarController。
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a MortarController bound to a specific mortar file.
    ///
    /// 创建一个绑定到特定 mortar 文件的 MortarController。
    pub fn with_path(path: impl Into<String>) -> Self {
        Self {
            mortar_path: Some(path.into()),
            current_node: None,
        }
    }
}

/// Dialogue focus marker - indicates this entity responds to input.
///
/// 对话焦点标记 - 表示该实体响应输入。
///
/// Multiple entities can have [`DialogueFocus`] simultaneously
/// (e.g., split-screen dialogue). Use [`DialogueBlockingConfig`]
/// to control whether all focused typewriters must finish before advancing.
///
/// 多个实体可以同时拥有 [`DialogueFocus`]（例如分屏对话）。
/// 使用 [`DialogueBlockingConfig`] 控制是否所有焦点打字机完成后才能步进。
#[derive(Component, Debug, Clone, Copy, Default, Reflect)]
#[reflect(Component)]
pub struct DialogueFocus;
