//! Dialogue system configuration resources.
//!
//! 对话系统配置资源。

use bevy::prelude::*;

/// Configuration for dialogue input events.
///
/// 对话输入事件配置。
///
/// Maps FRE event names to dialogue actions. All input handling
/// goes through FRE rules - no hardcoded keys.
///
/// 将 FRE 事件名映射到对话动作。所有输入处理通过 FRE 规则完成，
/// 无硬编码按键。
#[derive(Resource, Debug, Clone)]
pub struct DialogueInputConfig {
    /// FRE event name for advancing dialogue (e.g., "dialogue:advance").
    ///
    /// 步进对话的 FRE 事件名（如 "dialogue:advance"）。
    pub advance_event: String,

    /// FRE event name for skipping typewriter (e.g., "dialogue:skip_typewriter").
    ///
    /// 跳过打字机的 FRE 事件名（如 "dialogue:skip_typewriter"）。
    pub skip_typewriter_event: String,
}

impl Default for DialogueInputConfig {
    fn default() -> Self {
        Self {
            advance_event: "dialogue_advance".to_string(),
            skip_typewriter_event: "dialogue_skip_typewriter".to_string(),
        }
    }
}

/// Configuration for dialogue blocking behavior with multiple focuses.
///
/// 多焦点场景下的对话阻塞行为配置。
///
/// When multiple entities have [`DialogueFocus`], this controls
/// whether all typewriters must finish before advancing.
///
/// 当多个实体拥有 [`DialogueFocus`] 时，此配置控制是否所有打字机
/// 都必须完成后才能步进。
#[derive(Resource, Debug, Clone)]
pub struct DialogueBlockingConfig {
    /// If true, all focused typewriters must finish before dialogue can advance.
    /// If false, any single finished typewriter allows advancement.
    ///
    /// 若为 true，所有焦点打字机必须完成后才能步进对话。
    /// 若为 false，任一打字机完成即可步进。
    pub require_all_finished: bool,
}

impl Default for DialogueBlockingConfig {
    fn default() -> Self {
        Self {
            require_all_finished: true,
        }
    }
}

/// Runtime state for the current active dialogue.
///
/// 当前活动对话的运行时状态。
///
/// This resource is set when a dialogue starts and cleared when it ends.
/// It stores the component configuration for the current dialogue.
///
/// 此资源在对话启动时设置，结束时清除。
/// 它存储当前对话的组件配置。
#[derive(Resource, Debug, Clone, Default)]
pub struct ActiveDialogueState {
    /// Whether the current dialogue uses typewriter effect.
    ///
    /// 当前对话是否使用打字机效果。
    pub has_typewriter: bool,

    /// Whether the current dialogue uses Mortar controller.
    ///
    /// 当前对话是否使用 Mortar 控制器。
    pub has_mortar: bool,

    /// Simple text content for non-Mortar dialogue.
    ///
    /// 非 Mortar 对话的简单文本内容。
    pub simple_text: Option<String>,
}
