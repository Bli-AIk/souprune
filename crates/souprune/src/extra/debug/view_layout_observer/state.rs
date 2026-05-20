//! Shared state for the View layout observer.
//!
//! View 布局观察器的共享状态。

use bevy::prelude::*;

use crate::core::view::layout::{
    ViewClipRect, ViewLayoutDebugMetadata, ViewLayoutRect, ViewScrollState, ViewWorld3dPlaneDef,
};
use crate::core::view::spatial::ViewSpatialHit;

pub(super) const MAX_PARENT_DEPTH: usize = 64;

#[derive(Resource, Debug, Default, Clone)]
pub(super) struct ViewLayoutObserverState {
    pub(super) mode: ViewLayoutObserverMode,
    pub(super) locked_entity: Option<Entity>,
    pub(super) window_entity: Option<Entity>,
    pub(super) camera_entity: Option<Entity>,
    pub(super) show_box_model: bool,
    pub(super) show_flex_guides: bool,
    pub(super) show_grid_guides: bool,
    pub(super) show_spatial_guides: bool,
}

impl ViewLayoutObserverState {
    pub(super) fn overlay_active(&self) -> bool {
        self.window_entity.is_some() && self.mode != ViewLayoutObserverMode::Off
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(super) enum ViewLayoutObserverMode {
    #[default]
    Off,
    Hover,
    Locked,
    All,
}

#[derive(Debug, Clone)]
pub(super) struct ViewRootObserverContext {
    pub(super) entity: Entity,
    pub(super) layout_path: String,
    pub(super) namespace: String,
    pub(super) transform: GlobalTransform,
    pub(super) spatial_plane: Option<ViewWorld3dPlaneDef>,
    pub(super) spatial_hit: Option<ViewSpatialHit>,
    pub(super) layout_point: Option<Vec2>,
}

#[derive(Debug, Clone)]
pub(super) struct ViewLayoutObserverSelection {
    pub(super) entity: Entity,
    pub(super) root_entity: Entity,
    pub(super) root_layout_path: String,
    pub(super) root_namespace: String,
    pub(super) element_name: String,
    pub(super) element_path: String,
    pub(super) depth: usize,
    pub(super) area: f32,
    pub(super) rect: ViewLayoutRect,
    pub(super) clip_rect: Option<ViewClipRect>,
    pub(super) scroll_state: Option<ViewScrollState>,
    pub(super) debug: Option<ViewLayoutDebugMetadata>,
    pub(super) spatial_plane: Option<ViewWorld3dPlaneDef>,
    pub(super) spatial_hit: Option<ViewSpatialHit>,
    pub(super) root_transform: GlobalTransform,
}

#[derive(Resource, Debug, Default, Clone)]
pub(super) struct ViewLayoutObserverSnapshot {
    pub(super) hover_selection: Option<ViewLayoutObserverSelection>,
    pub(super) selected_selection: Option<ViewLayoutObserverSelection>,
    pub(super) all_selections: Vec<ViewLayoutObserverSelection>,
}
