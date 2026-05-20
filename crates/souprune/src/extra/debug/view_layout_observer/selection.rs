//! Selection and hit testing for the View layout observer.
//!
//! View 布局观察器的选择与命中测试逻辑。

use std::collections::HashMap;

use bevy::prelude::*;
use bevy::window::{PrimaryWindow, Window};

use super::state::{
    MAX_PARENT_DEPTH, ViewLayoutObserverMode, ViewLayoutObserverOrigin,
    ViewLayoutObserverSelection, ViewRootObserverContext,
};
use crate::core::camera::MainGameCamera;
use crate::core::view::components::{ViewContainer, ViewElement, ViewRoot};
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
        &'static GlobalTransform,
        &'static InheritedVisibility,
        Option<&'static ViewContainer>,
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
    include_hidden: bool,
    view_roots: &ViewRootObserverQuery,
    view_elements: &ViewLayoutElementQuery,
    view_root_lookup: &Query<&ViewRoot>,
    child_of_query: &Query<&ChildOf>,
) -> Option<ViewLayoutObserverSelection> {
    let root_contexts = collect_root_contexts(cursor_world, view_roots);
    let selections = view_elements.iter().filter_map(
        |(
            entity,
            element,
            rect,
            transform,
            inherited_visibility,
            container,
            debug,
            clip_rect,
            scroll_state,
        )| {
            let root_entity = find_view_root_entity(entity, view_root_lookup, child_of_query)?;
            let root_context = root_contexts.get(&root_entity)?;
            let origin = observer_origin(container);
            let is_hidden = !inherited_visibility.get();
            if is_hidden && !include_hidden {
                return None;
            }
            if !cursor_hits_layout_rect(root_context, transform, rect, rect, origin)
                || !point_inside_ancestor_clips(entity, root_context, view_elements, child_of_query)
            {
                return None;
            }

            Some(build_selection(
                entity,
                element,
                rect,
                transform,
                inherited_visibility,
                container,
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
    let (
        entity,
        element,
        rect,
        transform,
        inherited_visibility,
        container,
        debug,
        clip_rect,
        scroll_state,
    ) = view_elements.get(locked_entity).ok()?;
    let root_entity = find_view_root_entity(entity, view_root_lookup, child_of_query)?;
    let root_context = root_contexts.get(&root_entity)?;
    Some(build_selection(
        entity,
        element,
        rect,
        transform,
        inherited_visibility,
        container,
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
        .filter_map(
            |(
                entity,
                element,
                rect,
                transform,
                inherited_visibility,
                container,
                debug,
                clip_rect,
                scroll_state,
            )| {
                let root_entity = find_view_root_entity(entity, view_root_lookup, child_of_query)?;
                let root_context = root_contexts.get(&root_entity)?;
                Some(build_selection(
                    entity,
                    element,
                    rect,
                    transform,
                    inherited_visibility,
                    container,
                    debug,
                    clip_rect,
                    scroll_state,
                    root_context,
                ))
            },
        )
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
            |(entity, view_root, _transform, spatial_root, spatial_hit)| {
                let cursor_world = if spatial_root.is_none() {
                    cursor_world
                } else {
                    None
                };

                (
                    entity,
                    ViewRootObserverContext {
                        entity,
                        layout_path: view_root.layout_path.clone(),
                        namespace: view_root.namespace.clone(),
                        spatial_plane: spatial_root.map(|root| root.plane.clone()),
                        spatial_hit: spatial_hit.copied(),
                        cursor_world,
                    },
                )
            },
        )
        .collect()
}

fn build_selection(
    entity: Entity,
    element: &ViewElement,
    rect: &ViewLayoutRect,
    transform: &GlobalTransform,
    inherited_visibility: &InheritedVisibility,
    container: Option<&ViewContainer>,
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
        element_transform: *transform,
        origin: observer_origin(container),
        is_hidden: !inherited_visibility.get(),
        clip_rect: clip_rect.copied(),
        scroll_state: scroll_state.copied(),
        debug: debug.cloned(),
        spatial_plane: root_context.spatial_plane.clone(),
        spatial_hit: root_context.spatial_hit,
    }
}

fn observer_origin(container: Option<&ViewContainer>) -> ViewLayoutObserverOrigin {
    if container.is_some() {
        ViewLayoutObserverOrigin::TopLeft
    } else {
        ViewLayoutObserverOrigin::Center
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
    root_context: &ViewRootObserverContext,
    view_elements: &ViewLayoutElementQuery,
    child_of_query: &Query<&ChildOf>,
) -> bool {
    let mut current = Some(entity);
    for _ in 0..MAX_PARENT_DEPTH {
        let Some(entity) = current else {
            return true;
        };
        if let Ok((_, _, rect, transform, _, container, _, Some(clip_rect), _)) =
            view_elements.get(entity)
            && !cursor_hits_layout_rect(
                root_context,
                transform,
                rect,
                &layout_rect_from_clip_rect(*clip_rect),
                observer_origin(container),
            )
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

fn cursor_hits_layout_rect(
    root_context: &ViewRootObserverContext,
    transform: &GlobalTransform,
    base_rect: &ViewLayoutRect,
    target_rect: &ViewLayoutRect,
    origin: ViewLayoutObserverOrigin,
) -> bool {
    let Some(point) = cursor_layout_point(root_context, transform, base_rect, origin) else {
        return false;
    };
    point_in_local_layout_rect(point, *target_rect, *base_rect)
}

fn cursor_layout_point(
    root_context: &ViewRootObserverContext,
    transform: &GlobalTransform,
    base_rect: &ViewLayoutRect,
    origin: ViewLayoutObserverOrigin,
) -> Option<Vec2> {
    if root_context.spatial_plane.is_some() {
        let hit = root_context.spatial_hit?;
        let local = transform
            .affine()
            .inverse()
            .transform_point3(hit.world_position);
        let pixels_per_unit = root_context
            .spatial_plane
            .as_ref()
            .map(|plane| valid_pixels_per_unit(plane.pixels_per_unit))
            .unwrap_or(1.0);
        return Some(local_point_to_layout_point(
            local.truncate(),
            *base_rect,
            origin,
            pixels_per_unit,
        ));
    }

    let cursor_world = root_context.cursor_world?;
    let local = cursor_world - transform.translation().truncate();
    Some(local_point_to_layout_point(local, *base_rect, origin, 1.0))
}

fn local_point_to_layout_point(
    local: Vec2,
    base_rect: ViewLayoutRect,
    origin: ViewLayoutObserverOrigin,
    pixels_per_unit: f32,
) -> Vec2 {
    let local = local * pixels_per_unit;
    match origin {
        ViewLayoutObserverOrigin::Center => Vec2::new(
            local.x + base_rect.width * 0.5,
            -local.y + base_rect.height * 0.5,
        ),
        ViewLayoutObserverOrigin::TopLeft => Vec2::new(local.x, -local.y),
    }
}

fn point_in_local_layout_rect(
    point: Vec2,
    target_rect: ViewLayoutRect,
    base_rect: ViewLayoutRect,
) -> bool {
    let min = Vec2::new(target_rect.x - base_rect.x, target_rect.y - base_rect.y);
    let max = min + Vec2::new(target_rect.width, target_rect.height);
    point.x >= min.x && point.x <= max.x && point.y >= min.y && point.y <= max.y
}

fn layout_rect_from_clip_rect(rect: ViewClipRect) -> ViewLayoutRect {
    ViewLayoutRect {
        x: rect.x,
        y: rect.y,
        width: rect.width,
        height: rect.height,
    }
}

fn valid_pixels_per_unit(value: f32) -> f32 {
    if value.is_finite() && value > 0.0 {
        value
    } else {
        1.0
    }
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
            element_transform: GlobalTransform::IDENTITY,
            origin: ViewLayoutObserverOrigin::Center,
            is_hidden: false,
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
    fn centered_origin_cursor_maps_to_local_layout_box() {
        let point = local_point_to_layout_point(
            Vec2::ZERO,
            ViewLayoutRect {
                x: 12.0,
                y: 24.0,
                width: 200.0,
                height: 100.0,
            },
            ViewLayoutObserverOrigin::Center,
            1.0,
        );

        assert_eq!(point, Vec2::new(100.0, 50.0));
    }

    #[test]
    fn top_left_origin_cursor_uses_container_local_top_left() {
        let point = local_point_to_layout_point(
            Vec2::new(40.0, -35.0),
            ViewLayoutRect {
                x: 12.0,
                y: 24.0,
                width: 200.0,
                height: 100.0,
            },
            ViewLayoutObserverOrigin::TopLeft,
            1.0,
        );

        assert_eq!(point, Vec2::new(40.0, 35.0));
    }

    #[test]
    fn local_layout_rect_comparison_is_relative_to_element_rect() {
        let base = ViewLayoutRect {
            x: 100.0,
            y: 40.0,
            width: 200.0,
            height: 100.0,
        };
        let clipped = ViewLayoutRect {
            x: 120.0,
            y: 50.0,
            width: 60.0,
            height: 30.0,
        };

        assert!(point_in_local_layout_rect(
            Vec2::new(40.0, 20.0),
            clipped,
            base
        ));
        assert!(!point_in_local_layout_rect(
            Vec2::new(10.0, 20.0),
            clipped,
            base
        ));
    }

    #[test]
    fn spatial_cursor_point_scales_plane_units_to_layout_pixels() {
        let point = local_point_to_layout_point(
            Vec2::new(0.25, -0.1),
            ViewLayoutRect {
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 80.0,
            },
            ViewLayoutObserverOrigin::Center,
            100.0,
        );

        assert_eq!(point, Vec2::new(75.0, 50.0));
    }

    #[test]
    fn observer_origin_uses_top_left_for_pure_containers_only() {
        let container = ViewContainer;

        assert_eq!(
            observer_origin(Some(&container)),
            ViewLayoutObserverOrigin::TopLeft
        );
        assert_eq!(observer_origin(None), ViewLayoutObserverOrigin::Center);
    }
}
