//! Overworld UI components used by the overworld app state.
//!
//! 用于 overworld 应用状态的 UI 组件。
//!
//! This module has been refactored into submodules for better organization.
//! All original exports are preserved for backwards compatibility.
//!
//! 该模块已重构为子模块以更好地组织。
//! 所有原始导出都保留以保持向后兼容性。

pub(crate) mod box_components;
pub(crate) mod camera;
pub(crate) mod cursor;
pub(crate) mod hpbar;
pub(crate) mod interactive;
pub(crate) mod layer;
pub(crate) mod navigation;
pub(crate) mod text;
pub(crate) mod view_element;
pub(crate) mod visibility;

// Re-export all public types
pub(crate) use box_components::{
    UIBox, UIBoxFiller, UIBoxVisibility, UIContainer, UIContainerVisibility,
};
pub(crate) use camera::{CameraAnchored, CameraAnchoredBundle, CameraAnchoredDynamic};
pub(crate) use cursor::{
    BoxCursor, BoxCursorOwner, BoxCursorPlacement, BoxCursorPosition, BoxCursorReady,
    BoxCursorSprite, BoxCursorVisibility,
};
pub(crate) use interactive::{
    AwaitingInteraction, InteractionResult, InteractiveLayer, InteractiveLayerDef,
    LayerTransitionAction, LayerTransitionRule, LinearDirection, NavigatorType, NavigatorTypeDef,
    SelectionCancelledEvent, SelectionChangedEvent, SelectionConfirmedEvent,
};
pub(crate) use layer::UILayer;
pub(crate) use navigation::{
    IndexBound, LayerTransitions, TransitionAction, TransitionRule, UILayerNavigationConfig,
    UILayerNavigationRule, UILayerTransitionConfig,
};
// Note: RonUI is deprecated, use InteractiveLayer for new code.
// Export for backward compatibility with chase.rs and battle/sequencer.rs
// 注意：RonUI 已弃用，新代码请使用 InteractiveLayer。
// 为了与 chase.rs 和 battle/sequencer.rs 保持向后兼容而导出
pub(crate) use text::{RonUI, UIAnimationState, UIFont, UITextConfig, UITextTemplate};
pub(crate) use visibility::UILayerVisibilityRule;

// Public exports (used outside core::ui)
pub use hpbar::{DynamicUIElement, HPBarLag, HPBarSprite};
pub use view_element::{
    ElementState, ViewElement, ViewElementHistory, ViewRoot, find_element_by_full_name,
    find_elements_by_tag,
};
