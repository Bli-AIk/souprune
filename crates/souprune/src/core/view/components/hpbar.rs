//! HP bar and dynamic UI element components.
//!
//! HP 条和动态 UI 元素组件。
// TODO: 重构此文件。本文件包含过多的硬编码逻辑，应当通用化（如针对Sprite的Shader系统）
use bevy::prelude::*;

#[cfg(feature = "debug")]
use bevy::reflect::Reflect;

use crate::core::view::layout::serde_types::DynamicColor;

/// HP data source type - determines where HP values are read from.
/// HP 数据来源类型 - 决定从何处读取 HP 值。
#[derive(Debug, Clone, Default)]
pub enum HPSourceType {
    /// Player HP (default) - reads from player_hp and player_hp_max facts.
    /// 玩家 HP（默认）- 从 player_hp 和 player_hp_max facts 读取。
    #[default]
    Player,
    /// Enemy HP with specific index - reads from enemy_hps[index] and enemy_hp_maxs[index].
    /// 敌人 HP（指定索引）- 从 enemy_hps[index] 和 enemy_hp_maxs[index] 读取。
    Enemy { index: usize },
    /// Custom HP source using expressions.
    /// 自定义 HP 来源（使用表达式）。
    Custom {
        hp_expr: String,
        hp_max_expr: String,
    },
}

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

    /// HP data source type - determines where HP values are read from.
    /// HP 数据来源类型 - 决定从何处读取 HP 值。
    pub hp_source: HPSourceType,
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

/// Marker component for UI elements whose transform depends on time (@time).
/// These elements need per-frame updates for animation.
///
/// 标记 Transform 依赖时间 (@time) 的 UI 元素的组件。
/// 这些元素需要逐帧更新以实现动画。
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct TimeDependentTransform;

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
