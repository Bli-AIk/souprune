//! Reactive indicator components for UI elements.
//!
//! UI 元素的响应式指示器组件。
//!
//! This module provides a generic reactive system that can respond to selection
//! changes, layer activations, and other UI events. The primary use case is as
//! a selection indicator (like the Undertale heart cursor), but the design supports
//! any reactive visual element that needs to update based on UI state changes.
//!
//! 本模块提供一个通用的响应式系统，可以响应选择变更、层激活和其他 UI 事件。
//! 主要用例是作为选择指示器（如 Undertale 的心形光标），但设计上支持
//! 任何需要根据 UI 状态变化更新的响应式视觉元素。

use bevy::prelude::*;
use std::collections::HashMap;

#[cfg(feature = "debug")]
use bevy::reflect::Reflect;

use super::layer::UILayer;
use super::visibility::UILayerVisibilityRule;

/// Marker component attached to the reactive indicator sprite entity.
///
/// 附加到响应式指示器精灵实体上的标记组件。
#[derive(Component)]
pub(crate) struct ReactiveIndicatorSprite;

/// Records which `UIBox` owns a reactive indicator sprite entity.
///
/// 记录哪个 `UIBox` 拥有响应式指示器精灵实体。
#[derive(Component, Copy, Clone)]
pub(crate) struct ReactiveIndicatorOwner(pub Entity);

/// Marker indicating the reactive indicator sprite has been spawned for this box.
///
/// 表示该 UI 框已经生成响应式指示器精灵的标记。
#[derive(Component, Copy, Clone)]
pub(crate) struct ReactiveIndicatorReady;

/// Type alias for reactive indicator visibility rules.
///
/// 响应式指示器可见性规则的类型别名。
pub(crate) type ReactiveIndicatorVisibility = UILayerVisibilityRule;

/// Defines position calculation for reactive indicators based on selection index.
///
/// 根据选择索引定义响应式指示器的位置计算方式。
#[derive(Debug, Clone)]
#[cfg_attr(feature = "debug", derive(Reflect))]
pub(crate) enum ReactivePosition {
    /// Fixed position regardless of index.
    ///
    /// 无论索引如何都使用固定位置。
    Static(Vec3),

    /// Linear interpolation: origin + step * index.
    ///
    /// 线性插值：origin + step * index。
    Linear { origin: Vec3, step: Vec3 },

    /// Custom positions for each index.
    ///
    /// 每个索引的自定义位置。
    Custom(Vec<Vec3>),
}

impl Default for ReactivePosition {
    fn default() -> Self {
        ReactivePosition::Static(Vec3::ZERO)
    }
}

impl ReactivePosition {
    #[allow(dead_code)]
    pub(crate) fn fixed(position: Vec3) -> Self {
        Self::Static(position)
    }

    #[allow(dead_code)]
    pub(crate) fn linear(origin: Vec3, step: Vec3) -> Self {
        Self::Linear { origin, step }
    }

    #[allow(dead_code)]
    pub(crate) fn custom(positions: Vec<Vec3>) -> Self {
        Self::Custom(positions)
    }

    pub(crate) fn position_for_index(&self, index: usize) -> Vec3 {
        match self {
            ReactivePosition::Static(position) => *position,
            ReactivePosition::Linear { origin, step } => *origin + *step * index as f32,
            ReactivePosition::Custom(positions) => {
                if positions.is_empty() {
                    Vec3::ZERO
                } else {
                    positions[index.min(positions.len() - 1)]
                }
            }
        }
    }
}

/// Defines placement rules for reactive indicators, including a default strategy
/// and layer-specific overrides.
///
/// 定义响应式指示器的放置规则，包括默认策略和特定层的覆盖规则。
#[derive(Debug, Clone)]
#[cfg_attr(feature = "debug", derive(Reflect))]
pub(crate) struct ReactivePlacement {
    pub(crate) default: ReactivePosition,
    pub(crate) overrides: HashMap<UILayer, ReactivePosition>,
}

impl ReactivePlacement {
    pub(crate) fn new(default: ReactivePosition) -> Self {
        Self {
            default,
            overrides: HashMap::new(),
        }
    }

    pub(crate) fn with_override(mut self, layer: UILayer, position: ReactivePosition) -> Self {
        self.overrides.insert(layer, position);
        self
    }

    pub(crate) fn get(&self, layer: &UILayer) -> &ReactivePosition {
        self.overrides.get(layer).unwrap_or(&self.default)
    }
}

impl From<ReactivePosition> for ReactivePlacement {
    fn from(position: ReactivePosition) -> Self {
        Self::new(position)
    }
}

/// A reactive indicator element that responds to UI events such as selection changes
/// and layer activations. Can be attached to any [`UIBox`].
///
/// This component is the core of the reactive UI system. While commonly used as a
/// selection cursor (like the Undertale heart), it can represent any visual element
/// that needs to react to UI state changes.
///
/// 一个响应式指示器元素，响应选择变更和层激活等 UI 事件。
/// 可附着在任意 [`UIBox`] 上。
///
/// 该组件是响应式 UI 系统的核心。虽然常用作选择光标（如 Undertale 的心形），
/// 但它可以表示任何需要响应 UI 状态变化的视觉元素。
#[derive(Component, Debug)]
#[cfg_attr(feature = "debug", derive(Reflect))]
pub(crate) struct ReactiveIndicator {
    pub(crate) sprite: Sprite,
    pub(crate) visibility: ReactiveIndicatorVisibility,
    pub(crate) placement: ReactivePlacement,
    pub(crate) transform: Transform,
    hidden: bool,
    last_index: Option<usize>,
    last_layer: Option<UILayer>,
}

impl ReactiveIndicator {
    pub(crate) fn new(
        sprite: Sprite,
        visibility: ReactiveIndicatorVisibility,
        placement: impl Into<ReactivePlacement>,
        transform: Transform,
    ) -> Self {
        Self {
            sprite,
            visibility,
            placement: placement.into(),
            transform,
            hidden: false,
            last_index: None,
            last_layer: None,
        }
    }

    pub(crate) fn sprite(&self) -> Sprite {
        self.sprite.clone()
    }

    pub(crate) fn visibility(&self) -> &ReactiveIndicatorVisibility {
        &self.visibility
    }

    pub(crate) fn desired_translation(&self, layer: &UILayer, index: usize) -> Vec3 {
        let position_rule = self.placement.get(layer);
        self.transform.translation + position_rule.position_for_index(index)
    }

    pub(crate) fn transform(&self) -> Transform {
        self.transform
    }

    pub(crate) fn translation_for_index(&mut self, layer: &UILayer, index: usize) -> Option<Vec3> {
        if self.last_index == Some(index) && self.last_layer.as_ref() == Some(layer) {
            return None;
        }

        self.last_index = Some(index);
        self.last_layer = Some(layer.clone());
        Some(self.desired_translation(layer, index))
    }

    #[allow(dead_code)]
    pub(crate) fn hide(&mut self) {
        self.hidden = true;
    }

    #[allow(dead_code)]
    pub(crate) fn show(&mut self) {
        self.hidden = false;
    }

    pub(crate) fn is_hidden(&self) -> bool {
        self.hidden
    }
}
