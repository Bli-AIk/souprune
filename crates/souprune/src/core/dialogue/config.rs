#![allow(deprecated)]
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
/// **DEPRECATED**: This resource is deprecated. Use FRE fact `dialogue:focus_mode` instead.
/// - `"all_finished"` - all typewriters must finish before advancing
/// - `"first_finished"` - any typewriter finished allows advancement
///
/// **已弃用**：此资源已弃用。请改用 FRE fact `dialogue:focus_mode`。
/// - `"all_finished"` - 所有打字机必须完成才能步进
/// - `"first_finished"` - 任一打字机完成即可步进
#[allow(deprecated)]
#[deprecated(since = "0.5.2", note = "Use FRE fact 'dialogue:focus_mode' instead")]
#[derive(Resource, Debug, Clone)]
pub struct DialogueBlockingConfig {
    /// If true, all focused typewriters must finish before dialogue can advance.
    /// If false, any single finished typewriter allows advancement.
    ///
    /// 若为 true，所有焦点打字机必须完成后才能步进对话。
    /// 若为 false，任一打字机完成即可步进。
    pub require_all_finished: bool,
}

#[allow(deprecated)]
impl Default for DialogueBlockingConfig {
    fn default() -> Self {
        Self {
            require_all_finished: true,
        }
    }
}

// Note: ActiveDialogueState has been removed.
// Dialogue state is now managed via FRE facts:
// - dialogue:simple_text_active (Bool) - whether simple text dialogue is active
// - dialogue:simple_text (String) - the simple text content
// - dialogue:has_typewriter (Bool) - whether typewriter is enabled
//
// 注意：ActiveDialogueState 已被移除。
// 对话状态现在通过 FRE facts 管理：
// - dialogue:simple_text_active (Bool) - 简单文本对话是否激活
// - dialogue:simple_text (String) - 简单文本内容
// - dialogue:has_typewriter (Bool) - 是否启用打字机
