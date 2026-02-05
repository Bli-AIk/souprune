//! Overworld UI components used by the overworld app state.
//!
//! 用于 overworld 应用状态的 UI 组件。
//!
//! This module has been refactored into submodules for better organization.
//!
//! 该模块已重构为子模块以更好地组织。

pub(crate) mod box_components;
pub(crate) mod camera;
pub(crate) mod hpbar;
pub(crate) mod state_sprite;
pub(crate) mod text;
pub(crate) mod view_element;

// Re-export all public types
pub(crate) use box_components::{ViewBox, ViewBoxFiller, ViewContainer};
pub(crate) use camera::{CameraAnchored, CameraAnchoredBundle, CameraAnchoredDynamic};
pub(crate) use state_sprite::StateSpriteState;
pub(crate) use text::{ViewAnimationState, ViewFont, ViewTextConfig, ViewTextTemplate};

// Public exports (used outside core::ui)
pub use hpbar::{DynamicViewElement, HPBarLag, HPBarSprite, HPSourceType, TimeDependentTransform};
pub use view_element::{
    ActiveView, ElementState, PendingViewRules, ViewElement, ViewElementHistory, ViewRoot,
    VisibleWhen, find_element_by_full_name, find_elements_by_tag,
};
