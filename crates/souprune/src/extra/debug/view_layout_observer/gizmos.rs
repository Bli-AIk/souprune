//! Gizmo overlay rendering for the View layout observer.
//!
//! View 布局观察器的 gizmo 覆盖层渲染。

use bevy::prelude::*;

use super::state::{
    ViewLayoutObserverMode, ViewLayoutObserverOrigin, ViewLayoutObserverSelection,
    ViewLayoutObserverSnapshot, ViewLayoutObserverState,
};
use crate::core::view::layout::{SerializableDisplay, ViewLayoutEdges, ViewLayoutRect};

#[derive(Default, Reflect, GizmoConfigGroup)]
pub(super) struct ViewLayoutObserverGizmos;

pub(super) fn setup_view_layout_observer_gizmos(app: &mut App) {
    app.init_gizmo_group::<ViewLayoutObserverGizmos>();
    if let Some(mut store) = app.world_mut().get_resource_mut::<GizmoConfigStore>() {
        let (config, _) = store.config_mut::<ViewLayoutObserverGizmos>();
        config.enabled = false;
        config.line.width = 4.0;
        config.depth_bias = -1.0;
    }
}

pub(super) fn sync_view_layout_observer_gizmos_system(
    state: Res<ViewLayoutObserverState>,
    mut store: ResMut<GizmoConfigStore>,
) {
    let (config, _) = store.config_mut::<ViewLayoutObserverGizmos>();
    config.enabled = state.overlay_active();
}

pub(super) fn draw_view_layout_observer_gizmos_system(
    state: Res<ViewLayoutObserverState>,
    snapshot: Res<ViewLayoutObserverSnapshot>,
    mut gizmos: Gizmos<ViewLayoutObserverGizmos>,
) {
    if !state.overlay_active() {
        return;
    }

    if state.mode == ViewLayoutObserverMode::All {
        for selection in &snapshot.all_selections {
            draw_observer_rect(&mut gizmos, selection, ObserverRectKind::Content, false);
        }
    }

    if let Some(selection) = snapshot.selected_selection.as_ref() {
        draw_selection_gizmos(&mut gizmos, selection, &state);
    }
}

fn draw_selection_gizmos(
    gizmos: &mut Gizmos<ViewLayoutObserverGizmos>,
    selection: &ViewLayoutObserverSelection,
    state: &ViewLayoutObserverState,
) {
    if state.show_box_model {
        draw_observer_rect(gizmos, selection, ObserverRectKind::Margin, true);
        draw_observer_rect(gizmos, selection, ObserverRectKind::Border, true);
        draw_observer_rect(gizmos, selection, ObserverRectKind::Padding, true);
        draw_observer_rect(gizmos, selection, ObserverRectKind::Content, true);
    } else {
        draw_observer_rect(gizmos, selection, ObserverRectKind::Content, true);
    }

    if state.show_flex_guides {
        draw_gap_guides(gizmos, selection);
    }

    if state.show_grid_guides {
        draw_grid_guides(gizmos, selection);
    }

    if state.show_spatial_guides && selection.spatial_plane.is_some() {
        draw_spatial_plane_guides(gizmos, selection);
    }
}

#[derive(Clone, Copy)]
enum ObserverRectKind {
    Margin,
    Border,
    Padding,
    Content,
}

fn draw_observer_rect(
    gizmos: &mut Gizmos<ViewLayoutObserverGizmos>,
    selection: &ViewLayoutObserverSelection,
    kind: ObserverRectKind,
    selected: bool,
) {
    let rect = observer_rect_for_kind(selection, kind);
    if rect.width <= 0.0 || rect.height <= 0.0 {
        return;
    }
    let color = observer_color(kind, selected);
    let offset = layout_rect_center_offset(rect, selection.rect);
    draw_layout_rect(gizmos, selection, offset, rect.width, rect.height, color);
}

fn observer_rect_for_kind(
    selection: &ViewLayoutObserverSelection,
    kind: ObserverRectKind,
) -> ViewLayoutRect {
    let rect = selection.rect;
    let Some(metadata) = selection.debug.as_ref() else {
        return rect;
    };
    match kind {
        ObserverRectKind::Margin => expand_rect(rect, metadata.margin),
        ObserverRectKind::Border => rect,
        ObserverRectKind::Padding => inset_rect(rect, metadata.border),
        ObserverRectKind::Content => {
            inset_rect(inset_rect(rect, metadata.border), metadata.padding)
        }
    }
}

fn observer_color(kind: ObserverRectKind, selected: bool) -> Color {
    let alpha = if selected { 1.0 } else { 0.45 };
    match kind {
        ObserverRectKind::Margin => Color::srgba(1.0, 0.28, 0.28, alpha),
        ObserverRectKind::Border => Color::srgba(1.0, 0.82, 0.18, alpha),
        ObserverRectKind::Padding => Color::srgba(0.2, 0.95, 0.45, alpha),
        ObserverRectKind::Content => Color::srgba(0.18, 0.78, 1.0, alpha),
    }
}

fn expand_rect(rect: ViewLayoutRect, edges: ViewLayoutEdges) -> ViewLayoutRect {
    ViewLayoutRect {
        x: rect.x - edges.left,
        y: rect.y - edges.top,
        width: (rect.width + edges.left + edges.right).max(0.0),
        height: (rect.height + edges.top + edges.bottom).max(0.0),
    }
}

fn inset_rect(rect: ViewLayoutRect, edges: ViewLayoutEdges) -> ViewLayoutRect {
    ViewLayoutRect {
        x: rect.x + edges.left,
        y: rect.y + edges.top,
        width: (rect.width - edges.left - edges.right).max(0.0),
        height: (rect.height - edges.top - edges.bottom).max(0.0),
    }
}

fn draw_layout_rect(
    gizmos: &mut Gizmos<ViewLayoutObserverGizmos>,
    selection: &ViewLayoutObserverSelection,
    center_offset: Vec2,
    width: f32,
    height: f32,
    color: Color,
) {
    if selection.spatial_plane.is_some() {
        draw_layout_rect_3d(gizmos, selection, center_offset, width, height, color);
    } else {
        draw_layout_rect_2d(gizmos, selection, center_offset, width, height, color);
    }
}

fn draw_layout_rect_2d(
    gizmos: &mut Gizmos<ViewLayoutObserverGizmos>,
    selection: &ViewLayoutObserverSelection,
    center_offset: Vec2,
    width: f32,
    height: f32,
    color: Color,
) {
    let position = selection.element_transform.translation().truncate()
        + layout_origin_to_center(selection)
        + center_offset;
    gizmos.rect_2d(
        Isometry2d::from_translation(position),
        Vec2::new(width, height),
        color,
    );
}

fn draw_layout_rect_3d(
    gizmos: &mut Gizmos<ViewLayoutObserverGizmos>,
    selection: &ViewLayoutObserverSelection,
    center_offset: Vec2,
    width: f32,
    height: f32,
    color: Color,
) {
    let pixels_per_unit = selection
        .spatial_plane
        .as_ref()
        .map(|plane| valid_pixels_per_unit(plane.pixels_per_unit))
        .unwrap_or(1.0);
    let half_width = width / pixels_per_unit * 0.5;
    let half_height = height / pixels_per_unit * 0.5;
    let center = (layout_origin_to_center(selection) + center_offset) / pixels_per_unit;
    let corners = [
        Vec3::new(center.x - half_width, center.y + half_height, 0.06),
        Vec3::new(center.x + half_width, center.y + half_height, 0.06),
        Vec3::new(center.x + half_width, center.y - half_height, 0.06),
        Vec3::new(center.x - half_width, center.y - half_height, 0.06),
    ];
    for index in 0..corners.len() {
        let start = selection.element_transform.transform_point(corners[index]);
        let end = selection
            .element_transform
            .transform_point(corners[(index + 1) % corners.len()]);
        gizmos.line(start, end, color);
    }
}

fn layout_origin_to_center(selection: &ViewLayoutObserverSelection) -> Vec2 {
    match selection.origin {
        ViewLayoutObserverOrigin::Center => Vec2::ZERO,
        ViewLayoutObserverOrigin::TopLeft => {
            Vec2::new(selection.rect.width * 0.5, -selection.rect.height * 0.5)
        }
    }
}

fn layout_rect_center_offset(rect: ViewLayoutRect, base: ViewLayoutRect) -> Vec2 {
    Vec2::new(
        rect.x - base.x + rect.width * 0.5 - base.width * 0.5,
        -(rect.y - base.y + rect.height * 0.5 - base.height * 0.5),
    )
}

fn valid_pixels_per_unit(value: f32) -> f32 {
    if value.is_finite() && value > 0.0 {
        value
    } else {
        1.0
    }
}

fn draw_gap_guides(
    gizmos: &mut Gizmos<ViewLayoutObserverGizmos>,
    selection: &ViewLayoutObserverSelection,
) {
    let Some(metadata) = selection.debug.as_ref() else {
        return;
    };
    let rect = observer_rect_for_kind(selection, ObserverRectKind::Content);
    let color = Color::srgba(0.1, 0.95, 1.0, 0.85);
    if metadata.gap.column > 0.0 {
        let center_x = rect.x + rect.width * 0.5;
        draw_vertical_layout_line(
            gizmos,
            selection,
            Vec2::new(
                center_x - metadata.gap.column * 0.5,
                rect.y + rect.height * 0.5,
            ),
            rect.height,
            color,
        );
        draw_vertical_layout_line(
            gizmos,
            selection,
            Vec2::new(
                center_x + metadata.gap.column * 0.5,
                rect.y + rect.height * 0.5,
            ),
            rect.height,
            color,
        );
    }
    if metadata.gap.row > 0.0 {
        let center_y = rect.y + rect.height * 0.5;
        draw_horizontal_layout_line(
            gizmos,
            selection,
            Vec2::new(rect.x + rect.width * 0.5, center_y - metadata.gap.row * 0.5),
            rect.width,
            color,
        );
        draw_horizontal_layout_line(
            gizmos,
            selection,
            Vec2::new(rect.x + rect.width * 0.5, center_y + metadata.gap.row * 0.5),
            rect.width,
            color,
        );
    }
}

fn draw_grid_guides(
    gizmos: &mut Gizmos<ViewLayoutObserverGizmos>,
    selection: &ViewLayoutObserverSelection,
) {
    let Some(metadata) = selection.debug.as_ref() else {
        return;
    };
    if metadata.display == SerializableDisplay::None {
        return;
    }

    let rect = observer_rect_for_kind(selection, ObserverRectKind::Content);
    let color = Color::srgba(0.45, 0.62, 1.0, 0.6);
    for fraction in [0.25, 0.5, 0.75] {
        let x = rect.x + rect.width * fraction;
        draw_vertical_layout_line(
            gizmos,
            selection,
            Vec2::new(x, rect.y + rect.height * 0.5),
            rect.height,
            color,
        );
        let y = rect.y + rect.height * fraction;
        draw_horizontal_layout_line(
            gizmos,
            selection,
            Vec2::new(rect.x + rect.width * 0.5, y),
            rect.width,
            color,
        );
    }
}

fn draw_spatial_plane_guides(
    gizmos: &mut Gizmos<ViewLayoutObserverGizmos>,
    selection: &ViewLayoutObserverSelection,
) {
    let Some(plane) = selection.spatial_plane.as_ref() else {
        return;
    };
    let width = plane.plane_size.0;
    let height = plane.plane_size.1;
    let rect = ViewLayoutRect {
        x: -width * 0.5,
        y: -height * 0.5,
        width,
        height,
    };
    draw_layout_rect(
        gizmos,
        selection,
        Vec2::ZERO,
        rect.width,
        rect.height,
        Color::srgba(0.75, 0.55, 1.0, 0.95),
    );
}

fn draw_vertical_layout_line(
    gizmos: &mut Gizmos<ViewLayoutObserverGizmos>,
    selection: &ViewLayoutObserverSelection,
    layout_center: Vec2,
    height: f32,
    color: Color,
) {
    draw_layout_axis_line(
        gizmos,
        selection,
        layout_center,
        Vec2::new(0.0, height),
        color,
    );
}

fn draw_horizontal_layout_line(
    gizmos: &mut Gizmos<ViewLayoutObserverGizmos>,
    selection: &ViewLayoutObserverSelection,
    layout_center: Vec2,
    width: f32,
    color: Color,
) {
    draw_layout_axis_line(
        gizmos,
        selection,
        layout_center,
        Vec2::new(width, 0.0),
        color,
    );
}

fn draw_layout_axis_line(
    gizmos: &mut Gizmos<ViewLayoutObserverGizmos>,
    selection: &ViewLayoutObserverSelection,
    layout_center: Vec2,
    size: Vec2,
    color: Color,
) {
    let center_offset = Vec2::new(
        layout_center.x - selection.rect.x - selection.rect.width * 0.5,
        -(layout_center.y - selection.rect.y - selection.rect.height * 0.5),
    );
    if selection.spatial_plane.is_some() {
        let pixels_per_unit = selection
            .spatial_plane
            .as_ref()
            .map(|plane| valid_pixels_per_unit(plane.pixels_per_unit))
            .unwrap_or(1.0);
        let center = (layout_origin_to_center(selection) + center_offset) / pixels_per_unit;
        let half = Vec2::new(size.x * 0.5, -size.y * 0.5) / pixels_per_unit;
        let start = selection.element_transform.transform_point(Vec3::new(
            center.x - half.x,
            center.y - half.y,
            0.07,
        ));
        let end = selection.element_transform.transform_point(Vec3::new(
            center.x + half.x,
            center.y + half.y,
            0.07,
        ));
        gizmos.line(start, end, color);
        return;
    }

    let center = selection.element_transform.translation().truncate()
        + layout_origin_to_center(selection)
        + center_offset;
    let half = Vec2::new(size.x * 0.5, -size.y * 0.5);
    gizmos.line_2d(center - half, center + half, color);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extra::debug::view_layout_observer::state::ViewLayoutObserverOrigin;

    fn selection_with_origin(origin: ViewLayoutObserverOrigin) -> ViewLayoutObserverSelection {
        ViewLayoutObserverSelection {
            entity: Entity::from_bits(1),
            root_entity: Entity::from_bits(2),
            root_layout_path: "view/test.view.ron".to_string(),
            root_namespace: "test".to_string(),
            element_name: "test::Node".to_string(),
            element_path: "0:Node".to_string(),
            depth: 0,
            area: 20_000.0,
            rect: ViewLayoutRect {
                x: 0.0,
                y: 0.0,
                width: 200.0,
                height: 100.0,
            },
            element_transform: GlobalTransform::IDENTITY,
            origin,
            clip_rect: None,
            scroll_state: None,
            debug: None,
            spatial_plane: None,
            spatial_hit: None,
        }
    }

    #[test]
    fn top_left_origin_offsets_from_layout_box_not_drawn_rect() {
        let selection = selection_with_origin(ViewLayoutObserverOrigin::TopLeft);
        let content = ViewLayoutRect {
            x: 10.0,
            y: 20.0,
            width: 60.0,
            height: 30.0,
        };

        let center = layout_origin_to_center(&selection)
            + layout_rect_center_offset(content, selection.rect);

        assert_eq!(center, Vec2::new(40.0, -35.0));
    }

    #[test]
    fn centered_origin_uses_drawn_rect_offset_only() {
        let selection = selection_with_origin(ViewLayoutObserverOrigin::Center);
        let content = ViewLayoutRect {
            x: 10.0,
            y: 20.0,
            width: 60.0,
            height: 30.0,
        };

        let center = layout_origin_to_center(&selection)
            + layout_rect_center_offset(content, selection.rect);

        assert_eq!(center, Vec2::new(-60.0, 15.0));
    }
}
