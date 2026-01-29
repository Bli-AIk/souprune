//! Visibility rules for UI layers.
//!
//! UI 层的可见性规则。

#[cfg(feature = "debug")]
use bevy::reflect::Reflect;

use super::layer::UILayer;

/// Controls the visibility behavior of components relative to the active [`UILayer`].
///
/// 控制组件相对于当前激活 [`UILayer`] 的可见性表现。
#[derive(Debug, Clone)]
#[cfg_attr(feature = "debug", derive(Reflect))]
#[derive(Default)]
pub(crate) enum UILayerVisibilityRule {
    #[default]
    Always,
    AlwaysHidden,
    OnlyIn(Vec<UILayer>),
    Except(Vec<UILayer>),
}

impl UILayerVisibilityRule {
    pub(crate) fn is_visible_for(&self, layer: &UILayer) -> bool {
        match self {
            UILayerVisibilityRule::Always => true,
            UILayerVisibilityRule::AlwaysHidden => false,
            UILayerVisibilityRule::OnlyIn(layers) => layers.iter().any(|l| l == layer),
            UILayerVisibilityRule::Except(layers) => layers.iter().all(|l| l != layer),
        }
    }
}
