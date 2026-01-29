//! Cursor components for UI boxes.
//!
//! UI 框的光标组件。

use bevy::prelude::*;
use std::collections::HashMap;

#[cfg(feature = "debug")]
use bevy::reflect::Reflect;

use super::layer::UILayer;
use super::visibility::UILayerVisibilityRule;

/// Marker component attached to the cursor sprite entity spawned under a UI box.
///
/// 标记生成在 UI 框下方的光标精灵实体。
#[derive(Component)]
pub(crate) struct BoxCursorSprite;

/// Records which `UIBox` owns a cursor sprite entity.
///
/// 记录哪个 `UIBox` 拥有光标精灵实体。
#[derive(Component, Copy, Clone)]
pub(crate) struct BoxCursorOwner(pub Entity);

/// Marker indicating the cursor sprite has been spawned for this box.
///
/// 表示该 UI 框已经生成光标精灵的标记。
#[derive(Component, Copy, Clone)]
pub(crate) struct BoxCursorReady;

/// Type alias for cursor visibility rules.
///
/// 光标可见性规则的类型别名。
pub(crate) type BoxCursorVisibility = UILayerVisibilityRule;

/// Helper that turns an index into a translation offset for the cursor sprite.
///
/// 将索引转换为光标精灵位移的辅助类型。
#[derive(Debug, Clone)]
#[cfg_attr(feature = "debug", derive(Reflect))]
pub(crate) enum BoxCursorPosition {
    Static(Vec3),
    Linear { origin: Vec3, step: Vec3 },
    Custom(Vec<Vec3>),
}

impl Default for BoxCursorPosition {
    fn default() -> Self {
        BoxCursorPosition::Static(Vec3::ZERO)
    }
}

impl BoxCursorPosition {
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
            BoxCursorPosition::Static(position) => *position,
            BoxCursorPosition::Linear { origin, step } => *origin + *step * index as f32,
            BoxCursorPosition::Custom(positions) => {
                if positions.is_empty() {
                    Vec3::ZERO
                } else {
                    positions[index.min(positions.len() - 1)]
                }
            }
        }
    }
}

/// Defines cursor placement rules, including a default strategy and layer-specific overrides.
///
/// 定义光标放置规则，包括默认策略和特定层的覆盖规则。
#[derive(Debug, Clone)]
#[cfg_attr(feature = "debug", derive(Reflect))]
pub(crate) struct BoxCursorPlacement {
    pub(crate) default: BoxCursorPosition,
    pub(crate) overrides: HashMap<UILayer, BoxCursorPosition>,
}

impl BoxCursorPlacement {
    pub(crate) fn new(default: BoxCursorPosition) -> Self {
        Self {
            default,
            overrides: HashMap::new(),
        }
    }

    pub(crate) fn with_override(mut self, layer: UILayer, position: BoxCursorPosition) -> Self {
        self.overrides.insert(layer, position);
        self
    }

    pub(crate) fn get(&self, layer: &UILayer) -> &BoxCursorPosition {
        self.overrides.get(layer).unwrap_or(&self.default)
    }
}

impl From<BoxCursorPosition> for BoxCursorPlacement {
    fn from(position: BoxCursorPosition) -> Self {
        Self::new(position)
    }
}

/// Configurable cursor that can be attached to any [`UIBox`].
///
/// 可附着在任意 [`UIBox`] 上的可配置光标。
#[derive(Component, Debug)]
#[cfg_attr(feature = "debug", derive(Reflect))]
pub(crate) struct BoxCursor {
    pub(crate) sprite: Sprite,
    pub(crate) visibility: BoxCursorVisibility,
    pub(crate) placement: BoxCursorPlacement,
    pub(crate) transform: Transform,
    hidden: bool,
    last_index: Option<usize>,
    last_layer: Option<UILayer>,
}

impl BoxCursor {
    pub(crate) fn new(
        sprite: Sprite,
        visibility: BoxCursorVisibility,
        placement: impl Into<BoxCursorPlacement>,
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

    pub(crate) fn visibility(&self) -> &BoxCursorVisibility {
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
