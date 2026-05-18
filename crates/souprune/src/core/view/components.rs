//! top-down UI components used by the top_down app state.
//!
//! 用于 top_down 应用状态的 UI 组件。
//!
//! This module has been refactored into submodules for better organization.
//!
//! 该模块已重构为子模块以更好地组织。

pub mod box_components;
pub mod camera;
pub(crate) mod hpbar;
pub mod shader_material;
pub(crate) mod state_sprite;
pub mod text;
pub(crate) mod view_element;

// Re-export all public types
pub(crate) use box_components::{ViewBox, ViewBoxAnchor, ViewBoxFiller, ViewContainer};
pub use shader_material::ShaderMaterial;
pub(crate) use state_sprite::StateSpriteState;
pub(crate) use text::{
    ViewAnimationState, ViewFont, ViewTextAnimationStyle, ViewTextConfig, ViewTextTemplate,
};

// Public exports (used outside core::ui)
pub use hpbar::{DynamicViewElement, TimeDependentTransform};
pub use view_element::{
    ActiveView, ElementState, LocalState, PendingViewData, PendingViewRules, ViewElement,
    ViewElementHistory, ViewFocusScope, ViewFocusStack, ViewNodeTags, ViewRoot, VisibleWhen,
    find_element_by_full_name, find_elements_by_tag,
};
