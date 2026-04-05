//! # ui.rs
//!
//! RON-driven view runtime.

mod camera;
pub mod components;
mod custom_sprite_material;
pub mod dynamic_material;
pub(crate) mod expr_eval;
pub mod fact_toggle_color;
pub mod layout;
mod lifecycle;
mod messages;
mod plugin;
mod procedural_textures;
pub mod reconcile;
pub mod ron_view;
pub mod sdf_shape;
pub mod sdf_view_shape;
pub mod text;
pub mod visible_when;

use bevy::prelude::*;

pub use components::box_components::ViewBox;
pub use components::text::ViewTextConfig;
pub use components::{
    ActiveView, ElementState, ViewElementHistory, ViewRoot, find_element_by_full_name,
    find_elements_by_tag,
};
pub use custom_sprite_material::PixelOutlineMaterial;
pub(crate) use layout::SdfStructureAsset;
pub use messages::{DespawnViewRequest, SpawnViewRequest};
pub use plugin::CoreViewPlugin;
pub use ron_view::RonDrivenView;

/// Universal UI update system set
///
/// 通用 UI 更新系统集
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ViewUpdate;
