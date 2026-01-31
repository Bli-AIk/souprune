//! Visibility rules for UI layers.
//!
//! UI 层的可见性规则。

#[cfg(feature = "debug")]
use bevy::reflect::Reflect;

use super::layer::ViewLayer;

/// Controls the visibility behavior of components relative to the active [`ViewLayer`].
///
/// 控制组件相对于当前激活 [`ViewLayer`] 的可见性表现。
#[derive(Debug, Clone)]
#[cfg_attr(feature = "debug", derive(Reflect))]
#[derive(Default)]
pub(crate) enum ViewLayerVisibilityRule {
    #[default]
    Always,
    AlwaysHidden,
    OnlyIn(Vec<ViewLayer>),
    Except(Vec<ViewLayer>),
}

impl ViewLayerVisibilityRule {
    pub(crate) fn is_visible_for(&self, layer: &ViewLayer) -> bool {
        match self {
            ViewLayerVisibilityRule::Always => true,
            ViewLayerVisibilityRule::AlwaysHidden => false,
            ViewLayerVisibilityRule::OnlyIn(layers) => layers.iter().any(|l| l == layer),
            ViewLayerVisibilityRule::Except(layers) => layers.iter().all(|l| l != layer),
        }
    }
}
