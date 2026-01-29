//! Text-related UI components.
//!
//! 文本相关的 UI 组件。

use bevy::color::Srgba;
use bevy::prelude::*;
use bevy_rich_text3d::{TextAlign, TextAnchor};

#[cfg(feature = "debug")]
use bevy::reflect::Reflect;

/// Component that records the UI layer and the current selection index within that layer.
///
/// Access pattern:
/// - Fields are private to enforce read-only access from outside code in the crate.
/// - Use the provided getters to read `layer`, `index` and `max_index`.
/// - Use `set_layer` and `set_index` to change state in a controlled way (clamps and resets index as needed).
///
/// 记录 UI 层以及该层内当前选中项索引的组件。
///
/// 访问约定：
/// - 字段为私有以在 crate 范围内强制读取访问。
/// - 使用提供的 getter 来读取 `layer`、`index` 和 `max_index`。
/// - 使用 `set_layer` 和 `set_index` 以受控方式修改状态（会进行夹住或重置索引）。
#[derive(Component, Debug, Clone)]
#[cfg_attr(feature = "debug", derive(Reflect))]
pub(crate) struct UIAnimationState {
    pub(crate) state_name: String,
}

/// Font configuration for UI text
///
/// UI 文本的字体配置
#[derive(Debug, Clone)]
#[cfg_attr(feature = "debug", derive(Reflect))]
pub(crate) enum UIFont {
    DeterminationMono,
    DeterminationSans,
    Hud,
    BattleHud,
}

impl UIFont {
    /// Get font name and default size
    ///
    /// 获取字体名称和默认大小
    pub(crate) fn font_name(&self) -> &'static str {
        match self {
            UIFont::DeterminationMono => "Determination Mono SimSun",
            UIFont::DeterminationSans => "Determination Sans SimSun",
            UIFont::Hud => "Crypt of Tomorrow Fusion",
            UIFont::BattleHud => "Mars Needs Cunnilingus",
        }
    }

    /// Get default rendering size (for texture atlas)
    ///
    /// 获取默认渲染大小（用于纹理图集）
    pub(crate) fn default_size(&self) -> f32 {
        128.
    }
}

/// Configuration for a single text element
///
/// 单个文本元素的配置
#[derive(Debug, Clone)]
#[cfg_attr(feature = "debug", derive(Reflect))]
pub(crate) struct UITextConfig {
    pub(crate) name: Name,
    pub(crate) content: String,
    pub(crate) template: Option<String>,
    pub(crate) font: UIFont,
    pub(crate) world_scale: Vec2,
    pub(crate) color: Srgba,
    pub(crate) transform: Transform,
    pub(crate) align: TextAlign,
    pub(crate) anchor: TextAnchor,
    pub(crate) line_height: f32,
}

impl Default for UITextConfig {
    fn default() -> Self {
        Self {
            name: Name::new("Text"),
            content: "Text".to_string(),
            template: None,
            font: UIFont::DeterminationMono,
            world_scale: Vec2::splat(13.),
            color: Srgba::WHITE,
            transform: Transform::default(),
            align: TextAlign::Left,
            anchor: TextAnchor::BOTTOM_RIGHT,
            line_height: 1.0,
        }
    }
}

/// Stores the original template string for dynamic text updates.
///
/// 存储原始模板字符串以用于动态文本更新。
#[derive(Component, Debug, Clone)]
#[cfg_attr(feature = "debug", derive(Reflect))]
pub(crate) struct UITextTemplate(pub(crate) String);
