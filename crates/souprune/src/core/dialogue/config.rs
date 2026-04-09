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

// Note: ActiveDialogueState has been removed.
// Dialogue state is now managed via FRE facts:
// - dialogue:active (Bool) - whether dialogue is active
// - dialogue:has_typewriter (Bool) - whether typewriter is enabled
// - dialogue:has_mortar (Bool) - whether mortar VM is driving the dialogue
//
// 注意：ActiveDialogueState 已被移除。
// 对话状态现在通过 FRE facts 管理：
// - dialogue:active (Bool) - 对话是否激活
// - dialogue:has_typewriter (Bool) - 是否启用打字机
// - dialogue:has_mortar (Bool) - 是否由 Mortar VM 驱动对话
