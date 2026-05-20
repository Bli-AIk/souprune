//! Gizmo overlay rendering for the View layout observer.
//!
//! View 布局观察器的 gizmo 覆盖层渲染。

use bevy::prelude::*;

use super::state::{
    ViewLayoutObserverMode, ViewLayoutObserverSelection, ViewLayoutObserverSnapshot,
    ViewLayoutObserverState,
};
use crate::core::view::layout::{SerializableDisplay, ViewLayoutEdges, ViewLayoutRect};

pub(super) fn draw_view_layout_observer_gizmos_system(
    state: Res<ViewLayoutObserverState>,
    snapshot: Res<ViewLayoutObserverSnapshot>,
    mut gizmos: Gizmos,
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
    gizmos: &mut Gizmos,
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
    gizmos: &mut Gizmos,
    selection: &ViewLayoutObserverSelection,
    kind: ObserverRectKind,
    selected: bool,
) {
    let rect = observer_rect_for_kind(selection, kind);
    if rect.width <= 0.0 || rect.height <= 0.0 {
        return;
    }
    let color = observer_color(kind, selected);
    let z = match kind {
        ObserverRectKind::Margin => 0.08,
        ObserverRectKind::Border => 0.09,
        ObserverRectKind::Padding => 0.10,
        ObserverRectKind::Content => 0.11,
    };
    draw_layout_rect(gizmos, selection, rect, z, color);
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
    gizmos: &mut Gizmos,
    selection: &ViewLayoutObserverSelection,
    rect: ViewLayoutRect,
    z: f32,
    color: Color,
) {
    let points = [
        Vec3::new(rect.x, -rect.y, z),
        Vec3::new(rect.x + rect.width, -rect.y, z),
        Vec3::new(rect.x + rect.width, -(rect.y + rect.height), z),
        Vec3::new(rect.x, -(rect.y + rect.height), z),
    ];
    for index in 0..points.len() {
        let start = selection.root_transform.transform_point(points[index]);
        let end = selection
            .root_transform
            .transform_point(points[(index + 1) % points.len()]);
        gizmos.line(start, end, color);
    }
}

fn draw_gap_guides(gizmos: &mut Gizmos, selection: &ViewLayoutObserverSelection) {
    let Some(metadata) = selection.debug.as_ref() else {
        return;
    };
    let rect = observer_rect_for_kind(selection, ObserverRectKind::Content);
    let color = Color::srgba(0.1, 0.95, 1.0, 0.85);
    if metadata.gap.column > 0.0 {
        let center_x = rect.x + rect.width * 0.5;
        draw_layout_line(
            gizmos,
            selection,
            Vec2::new(center_x - metadata.gap.column * 0.5, rect.y),
            Vec2::new(center_x - metadata.gap.column * 0.5, rect.y + rect.height),
            0.12,
            color,
        );
        draw_layout_line(
            gizmos,
            selection,
            Vec2::new(center_x + metadata.gap.column * 0.5, rect.y),
            Vec2::new(center_x + metadata.gap.column * 0.5, rect.y + rect.height),
            0.12,
            color,
        );
    }
    if metadata.gap.row > 0.0 {
        let center_y = rect.y + rect.height * 0.5;
        draw_layout_line(
            gizmos,
            selection,
            Vec2::new(rect.x, center_y - metadata.gap.row * 0.5),
            Vec2::new(rect.x + rect.width, center_y - metadata.gap.row * 0.5),
            0.12,
            color,
        );
        draw_layout_line(
            gizmos,
            selection,
            Vec2::new(rect.x, center_y + metadata.gap.row * 0.5),
            Vec2::new(rect.x + rect.width, center_y + metadata.gap.row * 0.5),
            0.12,
            color,
        );
    }
}

fn draw_grid_guides(gizmos: &mut Gizmos, selection: &ViewLayoutObserverSelection) {
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
        draw_layout_line(
            gizmos,
            selection,
            Vec2::new(x, rect.y),
            Vec2::new(x, rect.y + rect.height),
            0.13,
            color,
        );
        let y = rect.y + rect.height * fraction;
        draw_layout_line(
            gizmos,
            selection,
            Vec2::new(rect.x, y),
            Vec2::new(rect.x + rect.width, y),
            0.13,
            color,
        );
    }
}

fn draw_spatial_plane_guides(gizmos: &mut Gizmos, selection: &ViewLayoutObserverSelection) {
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
        rect,
        0.14,
        Color::srgba(0.75, 0.55, 1.0, 0.95),
    );
}

fn draw_layout_line(
    gizmos: &mut Gizmos,
    selection: &ViewLayoutObserverSelection,
    start: Vec2,
    end: Vec2,
    z: f32,
    color: Color,
) {
    gizmos.line(
        selection
            .root_transform
            .transform_point(Vec3::new(start.x, -start.y, z)),
        selection
            .root_transform
            .transform_point(Vec3::new(end.x, -end.y, z)),
        color,
    );
}
