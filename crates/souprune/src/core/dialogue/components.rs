//! Dialogue system components.
//!
//! 对话系统组件。

use bevy::prelude::*;

/// Named dialogue channel for an independent dialogue controller.
///
/// 独立对话控制器的命名对话通道。
#[derive(Component, Debug, Clone, PartialEq, Eq, Hash, Reflect)]
#[reflect(Component)]
pub struct DialogueChannel {
    /// Channel name.
    ///
    /// 通道名称。
    pub name: String,
}

impl DialogueChannel {
    /// Create a normalized dialogue channel.
    ///
    /// 创建规范化后的对话通道。
    pub fn new(name: impl AsRef<str>) -> Self {
        Self {
            name: crate::core::fre_facts::normalize_dialogue_channel(name.as_ref()).to_string(),
        }
    }
}

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

/// Component for typewriter voice/sound effect.
///
/// 打字机音效组件。
///
/// When attached to an entity with Typewriter, plays a sound effect
/// each time a character is displayed.
///
/// 当附加到带有 Typewriter 的实体时，每次显示字符时播放音效。
#[derive(Component, Debug, Clone, Default, Reflect)]
#[reflect(Component)]
pub struct TypewriterVoice {
    /// Path to the voice audio file (relative to assets).
    ///
    /// 音效文件路径（相对于 assets）。
    pub sound_path: String,

    /// Last observed char index for change detection.
    ///
    /// 上次观察的字符索引，用于变化检测。
    pub last_char_index: usize,
}

impl TypewriterVoice {
    /// Creates a new TypewriterVoice with the given sound path.
    ///
    /// 用给定的音效路径创建新的 TypewriterVoice。
    pub fn new(sound_path: impl Into<String>) -> Self {
        Self {
            sound_path: sound_path.into(),
            last_char_index: 0,
        }
    }
}

/// Links a `bevy_bitmap_text::TextBlock` entity to its dialogue channel.
///
/// 将 `bevy_bitmap_text::TextBlock` 实体链接到其对话通道。
///
/// Set by the dialogue-text-block bridge system when a `ViewTextTemplate`
/// references a dialogue channel fact (e.g. `{{dialogue:battle_narration:text}}`).
/// Text animation systems (shake, wave, ghost) read this to resolve the
/// `text_style` fact for the associated channel.
///
/// 当 `ViewTextTemplate` 引用对话通道 fact 时（如 `{{dialogue:battle_narration:text}}`），
/// 由对话-文本块桥接系统设置。文本动画系统（抖动、波浪、幽灵）读取此组件
/// 以解析关联通道的 `text_style` fact。
#[derive(Component, Debug, Clone, PartialEq, Eq, Hash, Reflect)]
#[reflect(Component)]
pub struct TextBlockDialogueChannel(pub String);

// Note: DialogueFocus has been removed.
// Focus control is now handled via FRE fact `dialogue:has_focus` (Bool).
// When `dialogue:has_focus` is true, all typewriters on DialogueControllerEntity
// will respond to input.
//
// 注意：DialogueFocus 已被移除。
// 焦点控制现在通过 FRE fact `dialogue:has_focus` (Bool) 处理。
// 当 `dialogue:has_focus` 为 true 时，DialogueControllerEntity 上的所有打字机
// 将响应输入。
