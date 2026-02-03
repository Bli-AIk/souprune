//! HP bar and dynamic UI element components.
//!
//! HP 条和动态 UI 元素组件。

use bevy::prelude::*;

#[cfg(feature = "debug")]
use bevy::reflect::Reflect;

use crate::core::view::layout::serde_types::DynamicColor;

/// Marker component for HP bar sprites that need custom Material2d setup.
/// Stores the original shader_params expressions for dynamic evaluation.
/// 标记组件，用于需要自定义 Material2d 设置的 HP 条精灵。
/// 存储原始的 shader_params 表达式以便动态求值。
#[derive(Component, Clone)]
pub struct HPBarSprite {
    /// Original shader_params expressions from config. Used for dynamic evaluation.
    /// Components: (hp_ratio_expr, lag_ratio_expr, half_width_expr, alpha_expr)
    /// 来自配置的原始 shader_params 表达式。用于动态求值。
    pub shader_params_expr: Option<DynamicColor>,
}

/// Marker component for UI elements that need dynamic updates based on player data.
/// Stores the original definition for re-evaluation.
/// 标记需要根据玩家数据动态更新的UI元素的组件。
/// 存储原始定义以便重新求值。
#[derive(Component, Clone)]
pub struct DynamicViewElement {
    pub sprite_def: Option<crate::core::view::layout::SpriteDef>,
    pub text_def: Option<crate::core::view::layout::TextDef>,
}

/// HP bar lag effect state.
/// Tracks delayed HP percentage for smooth decrease animation.
#[derive(Component, Debug, Clone)]
#[cfg_attr(feature = "debug", derive(Reflect))]
pub struct HPBarLag {
    pub lag_hp_ratio: f32,
    pub last_hp_ratio: f32,
    pub start_lag_ratio: f32,
    pub delay_timer: f32,
    pub anim_progress: f32,
}

impl Default for HPBarLag {
    fn default() -> Self {
        Self {
            lag_hp_ratio: 1.0,
            last_hp_ratio: 1.0,
            start_lag_ratio: 1.0,
            delay_timer: 0.0,
            anim_progress: 0.0,
        }
    }
}

impl HPBarLag {
    pub fn new(hp_ratio: f32) -> Self {
        Self {
            lag_hp_ratio: hp_ratio,
            last_hp_ratio: hp_ratio,
            start_lag_ratio: hp_ratio,
            delay_timer: 0.0,
            anim_progress: 0.5,
        }
    }
}
