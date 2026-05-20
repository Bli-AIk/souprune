//! Selection and hit testing for the View layout observer.
//!
//! View 布局观察器的选择与命中测试逻辑。

use std::collections::HashMap;

use bevy::prelude::*;
use bevy::window::{PrimaryWindow, Window};

use super::state::{
    MAX_PARENT_DEPTH, ViewLayoutObserverMode, ViewLayoutObserverSelection, ViewRootObserverContext,
};
use crate::core::camera::MainGameCamera;
use crate::core::view::components::{ViewElement, ViewRoot};
use crate::core::view::layout::{
    ViewClipRect, ViewLayoutDebugMetadata, ViewLayoutRect, ViewScrollState,
};
use crate::core::view::spatial::{ViewSpatialHit, ViewSpatialRoot};
use crate::extra::debug::DebugCamera;

pub(super) type ViewRootObserverQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static ViewRoot,
        &'static GlobalTransform,
        Option<&'static ViewSpatialRoot>,
        Option<&'static ViewSpatialHit>,
    ),
>;

pub(super) type ViewLayoutElementQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static ViewElement,
        &'static ViewLayoutRect,
        Option<&'static ViewLayoutDebugMetadata>,
        Option<&'static ViewClipRect>,
        Option<&'static ViewScrollState>,
    ),
>;

pub(super) fn cursor_world_2d(
    windows: &Query<&Window, With<PrimaryWindow>>,
    camera_2d: &Query<
        (&Camera, &GlobalTransform),
        (With<Camera2d>, With<MainGameCamera>, Without<DebugCamera>),
    >,
) -> Option<Vec2> {
    let cursor_position = windows.single().ok().and_then(Window::cursor_position)?;
    let (camera, camera_transform) = camera_2d.iter().find(|(camera, _)| camera.is_active)?;
    camera
        .viewport_to_world_2d(camera_transform, cursor_position)
        .ok()
}

pub(super) fn collect_hover_selection(
    cursor_world: Option<Vec2>,
    view_roots: &ViewRootObserverQuery,
    view_elements: &ViewLayoutElementQuery,
    view_root_lookup: &Query<&ViewRoot>,
    child_of_query: &Query<&ChildOf>,
    clip_rect_query: &Query<&ViewClipRect>,
) -> Option<ViewLayoutObserverSelection> {
    let root_contexts = collect_root_contexts(cursor_world, view_roots);
    let selections = view_elements.iter().filter_map(
        |(entity, element, rect, debug, clip_rect, scroll_state)| {
            let root_entity = find_view_root_entity(entity, view_root_lookup, child_of_query)?;
            let root_context = root_contexts.get(&root_entity)?;
            let layout_point = root_context.layout_point?;
            if !point_in_layout_rect(layout_point, rect)
                || !point_inside_ancestor_clips(
                    entity,
                    layout_point,
                    child_of_query,
                    clip_rect_query,
                )
            {
                return None;
            }

            Some(build_selection(
                entity,
                element,
                rect,
                debug,
                clip_rect,
                scroll_state,
                root_context,
            ))
        },
    );

    choose_best_selection(selections)
}

pub(super) fn selected_selection_for_mode(
    mode: ViewLayoutObserverMode,
    hover_selection: Option<ViewLayoutObserverSelection>,
    locked_selection: Option<ViewLayoutObserverSelection>,
) -> Option<ViewLayoutObserverSelection> {
    match mode {
        ViewLayoutObserverMode::Off => None,
        ViewLayoutObserverMode::Hover => hover_selection,
        ViewLayoutObserverMode::Locked => locked_selection.or(hover_selection),
        ViewLayoutObserverMode::All => locked_selection.or(hover_selection),
    }
}

pub(super) fn collect_locked_selection(
    locked_entity: Entity,
    view_roots: &ViewRootObserverQuery,
    view_elements: &ViewLayoutElementQuery,
    view_root_lookup: &Query<&ViewRoot>,
    child_of_query: &Query<&ChildOf>,
) -> Option<ViewLayoutObserverSelection> {
    let root_contexts = collect_root_contexts(None, view_roots);
    let (entity, element, rect, debug, clip_rect, scroll_state) =
        view_elements.get(locked_entity).ok()?;
    let root_entity = find_view_root_entity(entity, view_root_lookup, child_of_query)?;
    let root_context = root_contexts.get(&root_entity)?;
    Some(build_selection(
        entity,
        element,
        rect,
        debug,
        clip_rect,
        scroll_state,
        root_context,
    ))
}

pub(super) fn collect_all_selections(
    view_roots: &ViewRootObserverQuery,
    view_elements: &ViewLayoutElementQuery,
    view_root_lookup: &Query<&ViewRoot>,
    child_of_query: &Query<&ChildOf>,
) -> Vec<ViewLayoutObserverSelection> {
    let root_contexts = collect_root_contexts(None, view_roots);
    let mut selections: Vec<_> = view_elements
        .iter()
        .filter_map(|(entity, element, rect, debug, clip_rect, scroll_state)| {
            let root_entity = find_view_root_entity(entity, view_root_lookup, child_of_query)?;
            let root_context = root_contexts.get(&root_entity)?;
            Some(build_selection(
                entity,
                element,
                rect,
                debug,
                clip_rect,
                scroll_state,
                root_context,
            ))
        })
        .collect();
    selections.sort_by_key(|selection| selection.depth);
    selections
}

fn collect_root_contexts(
    cursor_world: Option<Vec2>,
    view_roots: &ViewRootObserverQuery,
) -> HashMap<Entity, ViewRootObserverContext> {
    view_roots
        .iter()
        .map(
            |(entity, view_root, transform, spatial_root, spatial_hit)| {
                let layout_point = if let Some(hit) = spatial_hit {
                    Some(hit.layout_position)
                } else if spatial_root.is_none() {
                    cursor_world.map(|point| world_point_to_root_layout(point, transform))
                } else {
                    None
                };

                (
                    entity,
                    ViewRootObserverContext {
                        entity,
                        layout_path: view_root.layout_path.clone(),
                        namespace: view_root.namespace.clone(),
                        transform: *transform,
                        spatial_plane: spatial_root.map(|root| root.plane.clone()),
                        spatial_hit: spatial_hit.copied(),
                        layout_point,
                    },
                )
            },
        )
        .collect()
}

fn world_point_to_root_layout(cursor_world: Vec2, root_transform: &GlobalTransform) -> Vec2 {
    let point = root_transform
        .affine()
        .inverse()
        .transform_point3(cursor_world.extend(0.0))
        .truncate();
    Vec2::new(point.x, -point.y)
}

fn build_selection(
    entity: Entity,
    element: &ViewElement,
    rect: &ViewLayoutRect,
    debug: Option<&ViewLayoutDebugMetadata>,
    clip_rect: Option<&ViewClipRect>,
    scroll_state: Option<&ViewScrollState>,
    root_context: &ViewRootObserverContext,
) -> ViewLayoutObserverSelection {
    let depth = debug.map(|metadata| metadata.depth).unwrap_or_default();
    ViewLayoutObserverSelection {
        entity,
        root_entity: root_context.entity,
        root_layout_path: root_context.layout_path.clone(),
        root_namespace: root_context.namespace.clone(),
        element_name: element.full_name.clone(),
        element_path: debug
            .map(|metadata| metadata.path.clone())
            .unwrap_or_else(|| element.local_name.clone()),
        depth,
        area: rect.width.abs() * rect.height.abs(),
        rect: *rect,
        clip_rect: clip_rect.copied(),
        scroll_state: scroll_state.copied(),
        debug: debug.cloned(),
        spatial_plane: root_context.spatial_plane.clone(),
        spatial_hit: root_context.spatial_hit,
        root_transform: root_context.transform,
    }
}

fn find_view_root_entity(
    entity: Entity,
    view_root_lookup: &Query<&ViewRoot>,
    child_of_query: &Query<&ChildOf>,
) -> Option<Entity> {
    if view_root_lookup.get(entity).is_ok() {
        return Some(entity);
    }

    let mut current = entity;
    for _ in 0..MAX_PARENT_DEPTH {
        let parent = child_of_query.get(current).ok()?.parent();
        if view_root_lookup.get(parent).is_ok() {
            return Some(parent);
        }
        current = parent;
    }
    None
}

fn point_inside_ancestor_clips(
    entity: Entity,
    point: Vec2,
    child_of_query: &Query<&ChildOf>,
    clip_rect_query: &Query<&ViewClipRect>,
) -> bool {
    let mut current = Some(entity);
    for _ in 0..MAX_PARENT_DEPTH {
        let Some(entity) = current else {
            return true;
        };
        if let Ok(clip_rect) = clip_rect_query.get(entity)
            && !point_in_clip_rect(point, clip_rect)
        {
            return false;
        }
        current = child_of_query
            .get(entity)
            .ok()
            .map(|child_of| child_of.parent());
    }
    true
}

fn point_in_layout_rect(point: Vec2, rect: &ViewLayoutRect) -> bool {
    point.x >= rect.x
        && point.x <= rect.x + rect.width
        && point.y >= rect.y
        && point.y <= rect.y + rect.height
}

fn point_in_clip_rect(point: Vec2, rect: &ViewClipRect) -> bool {
    point.x >= rect.x
        && point.x <= rect.x + rect.width
        && point.y >= rect.y
        && point.y <= rect.y + rect.height
}

fn choose_best_selection(
    selections: impl IntoIterator<Item = ViewLayoutObserverSelection>,
) -> Option<ViewLayoutObserverSelection> {
    selections.into_iter().reduce(|best, candidate| {
        if selection_is_preferred(&candidate, &best) {
            candidate
        } else {
            best
        }
    })
}

fn selection_is_preferred(
    candidate: &ViewLayoutObserverSelection,
    best: &ViewLayoutObserverSelection,
) -> bool {
    candidate.depth > best.depth || (candidate.depth == best.depth && candidate.area < best.area)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::view::layout::{
        SerializableAlignItems, SerializableAlignSelf, SerializableDisplay,
        SerializableJustifyContent, SerializablePositionType, UiFlexDirection, ViewLayoutEdges,
        ViewLayoutGap, ViewLayoutLengthDebug, ViewLayoutSizingDebug,
    };

    fn sample_selection(entity: Entity, depth: usize, area: f32) -> ViewLayoutObserverSelection {
        let width = area.sqrt();
        ViewLayoutObserverSelection {
            entity,
            root_entity: Entity::from_bits(99),
            root_layout_path: "view/demo.view.ron".to_string(),
            root_namespace: "view_demo".to_string(),
            element_name: "demo::Element".to_string(),
            element_path: format!("0:Root/{depth}:Node"),
            depth,
            area,
            rect: ViewLayoutRect {
                x: 12.0,
                y: 24.0,
                width,
                height: width,
            },
            clip_rect: None,
            scroll_state: None,
            debug: Some(ViewLayoutDebugMetadata {
                path: format!("0:Root/{depth}:Node"),
                name: "Node".to_string(),
                depth,
                parent_path: Some("0:Root".to_string()),
                display: SerializableDisplay::Flex,
                position_type: SerializablePositionType::Relative,
                flex_direction: UiFlexDirection::Row,
                justify_content: Some(SerializableJustifyContent::Center),
                align_items: Some(SerializableAlignItems::Center),
                align_self: Some(SerializableAlignSelf::Auto),
                margin: ViewLayoutEdges::new(1.0, 2.0, 3.0, 4.0),
                padding: ViewLayoutEdges::new(5.0, 6.0, 7.0, 8.0),
                border: ViewLayoutEdges::new(9.0, 10.0, 11.0, 12.0),
                gap: ViewLayoutGap::new(13.0, 14.0),
                overflow: None,
                sizing: ViewLayoutSizingDebug {
                    width: ViewLayoutLengthDebug::Px(10.0),
                    height: ViewLayoutLengthDebug::Px(10.0),
                    flex_grow: 1.0,
                    flex_shrink: 1.0,
                    flex_basis: ViewLayoutLengthDebug::Px(0.0),
                },
            }),
            spatial_plane: None,
            spatial_hit: None,
            root_transform: GlobalTransform::IDENTITY,
        }
    }

    #[test]
    fn choose_best_selection_prefers_deeper_candidate() {
        let shallow = sample_selection(Entity::from_bits(1), 1, 400.0);
        let deep = sample_selection(Entity::from_bits(2), 4, 400.0);

        let chosen = choose_best_selection([shallow, deep]).expect("selection");

        assert_eq!(chosen.entity, Entity::from_bits(2));
    }

    #[test]
    fn selected_selection_for_mode_uses_locked_selection_when_locked() {
        let hover = sample_selection(Entity::from_bits(1), 1, 400.0);
        let locked = sample_selection(Entity::from_bits(2), 2, 300.0);

        let selected =
            selected_selection_for_mode(ViewLayoutObserverMode::Locked, Some(hover), Some(locked))
                .expect("selection");

        assert_eq!(selected.entity, Entity::from_bits(2));
    }

    #[test]
    fn world_point_to_root_layout_flips_world_y_into_layout_y() {
        let layout_point =
            world_point_to_root_layout(Vec2::new(24.0, -18.0), &GlobalTransform::IDENTITY);

        assert_eq!(layout_point, Vec2::new(24.0, 18.0));
    }
}
