//! View layout observer for debug builds.
//!
//! 调试构建下的 View 布局观察器。
//!
//! The observer inspects spawned View entities, follows their computed Taffy
//! metadata, and renders a compact overlay plus highlight for the currently
//! hovered or locked element.
//!
//! 观察器会检查已生成的 View 实体，读取其计算后的 Taffy 元数据，并为当前悬停
//! 或锁定的元素渲染紧凑面板与高亮轮廓。

use std::collections::HashMap;

use bevy::ecs::message::MessageWriter;
use bevy::prelude::*;
use bevy::window::{PrimaryWindow, Window};

use crate::core::camera::MainGameCamera;
use crate::core::view::components::{ViewElement, ViewRoot};
use crate::core::view::layout::{
    SerializableAlignItems, SerializableAlignSelf, SerializableDisplay, SerializableJustifyContent,
    SerializablePositionType, UiFlexDirection, ViewClipRect, ViewLayoutDebugMetadata,
    ViewLayoutEdges, ViewLayoutGap, ViewLayoutLengthDebug, ViewLayoutRect, ViewLayoutSizingDebug,
    ViewOverflowAxisDef, ViewOverflowDef, ViewScrollState, ViewWorld3dPlaneDef,
};
use crate::core::view::spatial::{ViewSpatialHit, ViewSpatialRoot};
use crate::extra::debug::{DebugCamera, DebugToastEvent};

const MAX_PARENT_DEPTH: usize = 64;

#[derive(Component)]
struct ViewLayoutObserverPanelRoot;

#[derive(Component)]
struct ViewLayoutObserverPanelText;

#[derive(Resource, Debug, Default)]
struct ViewLayoutObserverState {
    enabled: bool,
    always_on: bool,
    locked_entity: Option<Entity>,
}

#[derive(Debug, Clone)]
struct ViewRootObserverContext {
    entity: Entity,
    layout_path: String,
    namespace: String,
    transform: GlobalTransform,
    spatial_plane: Option<ViewWorld3dPlaneDef>,
    spatial_hit: Option<ViewSpatialHit>,
    layout_point: Option<Vec2>,
}

#[derive(Debug, Clone)]
struct ViewLayoutObserverSelection {
    entity: Entity,
    root_entity: Entity,
    root_layout_path: String,
    root_namespace: String,
    element_name: String,
    element_path: String,
    depth: usize,
    area: f32,
    rect: ViewLayoutRect,
    clip_rect: Option<ViewClipRect>,
    scroll_state: Option<ViewScrollState>,
    debug: Option<ViewLayoutDebugMetadata>,
    spatial_plane: Option<ViewWorld3dPlaneDef>,
    spatial_hit: Option<ViewSpatialHit>,
    root_transform: GlobalTransform,
}

type ViewRootObserverQuery<'w, 's> = Query<
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

type ViewLayoutElementQuery<'w, 's> = Query<
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

pub(super) fn setup_view_layout_observer_debug(app: &mut App) {
    app.init_resource::<ViewLayoutObserverState>()
        .add_systems(Startup, spawn_view_layout_observer_panel_system)
        .add_systems(
            Update,
            update_view_layout_observer_system.after(crate::core::view::ViewUpdate),
        );
}

fn spawn_view_layout_observer_panel_system(mut commands: Commands) {
    commands
        .spawn((
            ViewLayoutObserverPanelRoot,
            Node {
                display: Display::None,
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                right: Val::Px(10.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(8.0)),
                row_gap: Val::Px(4.0),
                width: Val::Px(460.0),
                ..default()
            },
            GlobalZIndex(i32::MAX - 2),
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.72)),
        ))
        .with_children(|parent| {
            parent.spawn((
                ViewLayoutObserverPanelText,
                Text::new("View Layout Observer"),
                TextFont::from_font_size(12.0),
                TextColor(Color::WHITE),
            ));
        });
}

fn update_view_layout_observer_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_2d: Query<
        (&Camera, &GlobalTransform),
        (With<Camera2d>, With<MainGameCamera>, Without<DebugCamera>),
    >,
    view_roots: ViewRootObserverQuery,
    view_elements: ViewLayoutElementQuery,
    view_root_lookup: Query<&ViewRoot>,
    child_of_query: Query<&ChildOf>,
    clip_rect_query: Query<&ViewClipRect>,
    mut state: ResMut<ViewLayoutObserverState>,
    mut panel_root: Query<&mut Node, With<ViewLayoutObserverPanelRoot>>,
    mut panel_text: Query<&mut Text, With<ViewLayoutObserverPanelText>>,
    mut toast_events: MessageWriter<DebugToastEvent>,
    mut gizmos: Gizmos,
) {
    handle_toggle_hotkeys(&keyboard, &mut state, &mut toast_events);

    let hover_selection = state.enabled.then(|| {
        collect_hover_selection(
            cursor_world_2d(&windows, &camera_2d),
            &view_roots,
            &view_elements,
            &view_root_lookup,
            &child_of_query,
            &clip_rect_query,
        )
    });
    let hover_selection = hover_selection.flatten();

    handle_lock_hotkey(
        &keyboard,
        &mut state,
        hover_selection.as_ref(),
        &mut toast_events,
    );

    let selected = if state.enabled {
        resolve_selected_selection(
            state.locked_entity,
            hover_selection,
            &view_roots,
            &view_elements,
            &view_root_lookup,
            &child_of_query,
        )
    } else {
        None
    };

    if state.locked_entity.is_some() && selected.is_none() {
        state.locked_entity = None;
    }

    sync_panel(&state, selected.as_ref(), &mut panel_root, &mut panel_text);
    if let Some(selection) = selected.as_ref() {
        draw_selection_highlight(&mut gizmos, selection, state.locked_entity.is_some());
    }
}

fn handle_toggle_hotkeys(
    keyboard: &ButtonInput<KeyCode>,
    state: &mut ViewLayoutObserverState,
    toast_events: &mut MessageWriter<DebugToastEvent>,
) {
    if keyboard.just_pressed(KeyCode::Escape) && state.locked_entity.take().is_some() {
        toast_events.write(DebugToastEvent {
            message: "View Layout Observer: lock cleared".into(),
        });
        return;
    }

    if !keyboard.just_pressed(KeyCode::F11) {
        return;
    }

    let shift = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);
    let control = keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);

    if control {
        return;
    }

    if shift {
        state.always_on = !state.always_on;
        let mode = if state.always_on { "ON" } else { "OFF" };
        toast_events.write(DebugToastEvent {
            message: format!("View Layout Observer always-on: {mode}"),
        });
        return;
    }

    state.enabled = !state.enabled;
    let mode = if state.enabled { "ON" } else { "OFF" };
    toast_events.write(DebugToastEvent {
        message: format!("View Layout Observer: {mode}"),
    });
}

fn handle_lock_hotkey(
    keyboard: &ButtonInput<KeyCode>,
    state: &mut ViewLayoutObserverState,
    hover_selection: Option<&ViewLayoutObserverSelection>,
    toast_events: &mut MessageWriter<DebugToastEvent>,
) {
    let control = keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);
    if !state.enabled || !control || !keyboard.just_pressed(KeyCode::F11) {
        return;
    }

    let Some(selection) = hover_selection else {
        if state.locked_entity.take().is_some() {
            toast_events.write(DebugToastEvent {
                message: "View Layout Observer: lock cleared".into(),
            });
        }
        return;
    };

    if state.locked_entity == Some(selection.entity) {
        state.locked_entity = None;
        toast_events.write(DebugToastEvent {
            message: "View Layout Observer: lock cleared".into(),
        });
    } else {
        state.locked_entity = Some(selection.entity);
        toast_events.write(DebugToastEvent {
            message: format!("View Layout Observer locked: {}", selection.element_name),
        });
    }
}

fn cursor_world_2d(
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

fn collect_hover_selection(
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

fn resolve_selected_selection(
    locked_entity: Option<Entity>,
    hover_selection: Option<ViewLayoutObserverSelection>,
    view_roots: &ViewRootObserverQuery,
    view_elements: &ViewLayoutElementQuery,
    view_root_lookup: &Query<&ViewRoot>,
    child_of_query: &Query<&ChildOf>,
) -> Option<ViewLayoutObserverSelection> {
    let Some(locked_entity) = locked_entity else {
        return hover_selection;
    };

    collect_locked_selection(
        locked_entity,
        view_roots,
        view_elements,
        view_root_lookup,
        child_of_query,
    )
    .or(hover_selection)
}

fn collect_locked_selection(
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
    root_transform
        .affine()
        .inverse()
        .transform_point3(cursor_world.extend(0.0))
        .truncate()
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

fn sync_panel(
    state: &ViewLayoutObserverState,
    selection: Option<&ViewLayoutObserverSelection>,
    panel_root: &mut Query<&mut Node, With<ViewLayoutObserverPanelRoot>>,
    panel_text: &mut Query<&mut Text, With<ViewLayoutObserverPanelText>>,
) {
    let visible = state.enabled && (state.always_on || selection.is_some());
    for mut node in panel_root.iter_mut() {
        node.display = if visible {
            Display::Flex
        } else {
            Display::None
        };
    }
    if !visible {
        return;
    }
    for mut text in panel_text.iter_mut() {
        **text = build_panel_text(state, selection);
    }
}

fn build_panel_text(
    state: &ViewLayoutObserverState,
    selection: Option<&ViewLayoutObserverSelection>,
) -> String {
    let mut lines = vec![format!(
        "State: enabled={} always_on={} locked={}",
        state.enabled,
        state.always_on,
        state
            .locked_entity
            .map(|entity| format!("{entity:?}"))
            .unwrap_or_else(|| "none".to_string())
    )];

    let Some(selection) = selection else {
        lines.push("Target: none".to_string());
        return lines.join("\n");
    };

    lines.extend([
        format!(
            "Target: {} ({:?})",
            selection.element_name, selection.entity
        ),
        format!(
            "Root: {} ({:?}) ns={}",
            selection.root_layout_path, selection.root_entity, selection.root_namespace
        ),
        format!("Path: {}", selection.element_path),
        format!(
            "Depth: {} area={}",
            selection.depth,
            format_number(selection.area)
        ),
        format!("Rect: {}", format_layout_rect(&selection.rect)),
    ]);

    if let Some(clip_rect) = selection.clip_rect {
        lines.push(format!("Clip: {}", format_clip_rect(&clip_rect)));
    }
    if let Some(scroll_state) = selection.scroll_state {
        lines.push(format!(
            "Scroll: x={} y={}",
            format_number(scroll_state.offset_x),
            format_number(scroll_state.offset_y)
        ));
    }
    if let Some(metadata) = selection.debug.as_ref() {
        lines.extend(format_debug_metadata(metadata));
    } else {
        lines.push("Layout: metadata unavailable".to_string());
    }
    if let Some(plane) = selection.spatial_plane.as_ref() {
        lines.push(format!(
            "Space: 3d-plane size={}x{} ppu={} input={:?} orientation={:?} depth={:?}",
            format_number(plane.plane_size.0),
            format_number(plane.plane_size.1),
            format_number(plane.pixels_per_unit),
            plane.input,
            plane.orientation,
            plane.depth
        ));
    }
    if let Some(hit) = selection.spatial_hit {
        lines.push(format!(
            "Hit: layout=({}, {}) dist={}",
            format_number(hit.layout_position.x),
            format_number(hit.layout_position.y),
            format_number(hit.distance)
        ));
    }

    lines.join("\n")
}

fn format_debug_metadata(metadata: &ViewLayoutDebugMetadata) -> Vec<String> {
    vec![
        format!(
            "Layout: display={} pos={} dir={}",
            display_label(metadata.display),
            position_label(metadata.position_type),
            flex_direction_label(metadata.flex_direction)
        ),
        format!(
            "Flex: justify={} align_items={} align_self={}",
            justify_label(metadata.justify_content),
            align_items_label(metadata.align_items),
            align_self_label(metadata.align_self)
        ),
        format!(
            "Box: margin={} padding={} border={}",
            format_edges(&metadata.margin),
            format_edges(&metadata.padding),
            format_edges(&metadata.border)
        ),
        format!(
            "Gap: {} overflow={}",
            format_gap(&metadata.gap),
            overflow_label(metadata.overflow)
        ),
        format_sizing(&metadata.sizing),
    ]
}

fn format_sizing(sizing: &ViewLayoutSizingDebug) -> String {
    format!(
        "Sizing: w={} h={} grow={} shrink={} basis={}",
        format_length(sizing.width),
        format_length(sizing.height),
        format_number(sizing.flex_grow),
        format_number(sizing.flex_shrink),
        format_length(sizing.flex_basis)
    )
}

fn format_layout_rect(rect: &ViewLayoutRect) -> String {
    format!(
        "x={} y={} w={} h={}",
        format_number(rect.x),
        format_number(rect.y),
        format_number(rect.width),
        format_number(rect.height)
    )
}

fn format_clip_rect(rect: &ViewClipRect) -> String {
    format!(
        "x={} y={} w={} h={}",
        format_number(rect.x),
        format_number(rect.y),
        format_number(rect.width),
        format_number(rect.height)
    )
}

fn format_edges(edges: &ViewLayoutEdges) -> String {
    format!(
        "l{} r{} t{} b{}",
        format_number(edges.left),
        format_number(edges.right),
        format_number(edges.top),
        format_number(edges.bottom)
    )
}

fn format_gap(gap: &ViewLayoutGap) -> String {
    format!(
        "row={} column={}",
        format_number(gap.row),
        format_number(gap.column)
    )
}

fn format_length(value: ViewLayoutLengthDebug) -> String {
    match value {
        ViewLayoutLengthDebug::Auto => "auto".to_string(),
        ViewLayoutLengthDebug::Px(value) => format!("px({})", format_number(value)),
        ViewLayoutLengthDebug::Percent(value) => {
            format!("percent({})", format_number(value))
        }
    }
}

fn format_number(value: f32) -> String {
    if !value.is_finite() {
        return value.to_string();
    }
    if (value - value.round()).abs() < 0.001 {
        format!("{value:.0}")
    } else {
        format!("{value:.2}")
    }
}

fn display_label(value: SerializableDisplay) -> &'static str {
    match value {
        SerializableDisplay::Flex => "flex",
        SerializableDisplay::None => "none",
    }
}

fn position_label(value: SerializablePositionType) -> &'static str {
    match value {
        SerializablePositionType::Relative => "relative",
        SerializablePositionType::Absolute => "absolute",
    }
}

fn flex_direction_label(value: UiFlexDirection) -> &'static str {
    match value {
        UiFlexDirection::Row => "row",
        UiFlexDirection::Column => "column",
        UiFlexDirection::RowReverse => "row-reverse",
        UiFlexDirection::ColumnReverse => "column-reverse",
    }
}

fn justify_label(value: Option<SerializableJustifyContent>) -> &'static str {
    match value {
        Some(SerializableJustifyContent::Start) => "start",
        Some(SerializableJustifyContent::End) => "end",
        Some(SerializableJustifyContent::Center) => "center",
        Some(SerializableJustifyContent::SpaceBetween) => "space-between",
        Some(SerializableJustifyContent::SpaceAround) => "space-around",
        Some(SerializableJustifyContent::SpaceEvenly) => "space-evenly",
        None => "none",
    }
}

fn align_items_label(value: Option<SerializableAlignItems>) -> &'static str {
    match value {
        Some(SerializableAlignItems::Start) => "start",
        Some(SerializableAlignItems::End) => "end",
        Some(SerializableAlignItems::Center) => "center",
        Some(SerializableAlignItems::Baseline) => "baseline",
        Some(SerializableAlignItems::Stretch) => "stretch",
        None => "none",
    }
}

fn align_self_label(value: Option<SerializableAlignSelf>) -> &'static str {
    match value {
        Some(SerializableAlignSelf::Auto) => "auto",
        Some(SerializableAlignSelf::Start) => "start",
        Some(SerializableAlignSelf::End) => "end",
        Some(SerializableAlignSelf::Center) => "center",
        Some(SerializableAlignSelf::Baseline) => "baseline",
        Some(SerializableAlignSelf::Stretch) => "stretch",
        None => "none",
    }
}

fn overflow_label(value: Option<ViewOverflowDef>) -> String {
    match value {
        Some(ViewOverflowDef::Visible) => "visible".to_string(),
        Some(ViewOverflowDef::Hidden) => "hidden".to_string(),
        Some(ViewOverflowDef::Scroll) => "scroll".to_string(),
        Some(ViewOverflowDef::Axes {
            horizontal,
            vertical,
        }) => format!(
            "axes(h={}, v={})",
            overflow_axis_label(horizontal),
            overflow_axis_label(vertical)
        ),
        None => "none".to_string(),
    }
}

fn overflow_axis_label(value: ViewOverflowAxisDef) -> &'static str {
    match value {
        ViewOverflowAxisDef::Visible => "visible",
        ViewOverflowAxisDef::Hidden => "hidden",
        ViewOverflowAxisDef::Scroll => "scroll",
    }
}

fn draw_selection_highlight(
    gizmos: &mut Gizmos,
    selection: &ViewLayoutObserverSelection,
    locked: bool,
) {
    let color = if locked {
        Color::srgb(0.1, 0.9, 1.0)
    } else {
        Color::srgb(1.0, 0.85, 0.18)
    };
    let rect = selection.rect;
    let points = [
        Vec3::new(rect.x, -rect.y, 0.03),
        Vec3::new(rect.x + rect.width, -rect.y, 0.03),
        Vec3::new(rect.x + rect.width, -(rect.y + rect.height), 0.03),
        Vec3::new(rect.x, -(rect.y + rect.height), 0.03),
    ];
    for index in 0..points.len() {
        let start = selection.root_transform.transform_point(points[index]);
        let end = selection
            .root_transform
            .transform_point(points[(index + 1) % points.len()]);
        gizmos.line(start, end, color);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_selection(entity: Entity, depth: usize, area: f32) -> ViewLayoutObserverSelection {
        let width = area.sqrt();
        let rect = ViewLayoutRect {
            x: 12.0,
            y: 24.0,
            width,
            height: width,
        };
        ViewLayoutObserverSelection {
            entity,
            root_entity: Entity::from_bits(99),
            root_layout_path: "view/demo.view.ron".to_string(),
            root_namespace: "view_demo".to_string(),
            element_name: "demo::Element".to_string(),
            element_path: format!("0:Root/{depth}:Node"),
            depth,
            area,
            rect,
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
    fn build_panel_text_reports_selection_details() {
        let state = ViewLayoutObserverState {
            enabled: true,
            always_on: true,
            locked_entity: Some(Entity::from_bits(1)),
        };
        let selection = sample_selection(Entity::from_bits(7), 3, 625.0);

        let text = build_panel_text(&state, Some(&selection));

        assert!(text.contains("State: enabled=true always_on=true"));
        assert!(text.contains("Target: demo::Element"));
        assert!(text.contains("Path: 0:Root/3:Node"));
        assert!(text.contains("Root: view/demo.view.ron"));
        assert!(text.contains("Rect: x=12"));
        assert!(text.contains("Layout: display=flex pos=relative dir=row"));
        assert!(text.contains("Sizing: w=px(10) h=px(10) grow=1 shrink=1 basis=px(0)"));
    }

    #[test]
    fn format_gap_uses_row_and_column_labels() {
        let gap = ViewLayoutGap::new(12.0, 18.5);

        assert_eq!(format_gap(&gap), "row=12 column=18.50");
    }
}
