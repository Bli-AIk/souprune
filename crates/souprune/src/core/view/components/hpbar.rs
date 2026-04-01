//! Dynamic UI element components.
//!
//! 动态 UI 元素组件。
//!
//! This module contains components for dynamic UI elements that need runtime updates.
//! The HP bar functionality has been migrated to the new ShaderMaterial + DynamicMaterial2d system.
//!
//! 该模块包含需要运行时更新的动态 UI 元素组件。
//! HP 条功能已迁移到新的 ShaderMaterial + DynamicMaterial2d 系统。

use bevy::prelude::*;

/// Marker component for UI elements that need dynamic updates based on player data.
/// Stores the original definition for re-evaluation.
/// 标记需要根据玩家数据动态更新的UI元素的组件。
/// 存储原始定义以便重新求值。
#[derive(Component, Clone)]
pub struct DynamicViewElement {
    pub sprite_def: Option<crate::core::view::layout::SpriteDef>,
    pub text_def: Option<crate::core::view::layout::TextDef>,
    pub view_box_def: Option<crate::core::view::layout::view_schema::ViewBoxLogicDef>,
}

/// Marker component for UI elements whose transform depends on time (@time).
/// These elements need per-frame updates for animation.
///
/// 标记 Transform 依赖时间 (@time) 的 UI 元素的组件。
/// 这些元素需要逐帧更新以实现动画。
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct TimeDependentTransform;
