//! Formatting helpers for the View layout observer.
//!
//! View 布局观察器的格式化辅助函数。

use bevy::prelude::*;

use super::state::{ViewLayoutObserverMode, ViewLayoutObserverSelection, ViewLayoutObserverState};
use crate::core::view::layout::{
    SerializableAlignItems, SerializableAlignSelf, SerializableDisplay, SerializableJustifyContent,
    SerializablePositionType, UiFlexDirection, ViewClipRect, ViewLayoutDebugMetadata,
    ViewLayoutEdges, ViewLayoutGap, ViewLayoutLengthDebug, ViewLayoutRect, ViewLayoutSizingDebug,
    ViewOverflowAxisDef, ViewOverflowDef,
};

pub(super) fn build_selection_text(
    state: &ViewLayoutObserverState,
    selection: Option<&ViewLayoutObserverSelection>,
) -> String {
    let mut lines = vec![format!(
        "State: mode={} locked={}",
        mode_label(state.mode),
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

pub(super) fn mode_label(mode: ViewLayoutObserverMode) -> &'static str {
    match mode {
        ViewLayoutObserverMode::Off => "Off",
        ViewLayoutObserverMode::Hover => "Hover",
        ViewLayoutObserverMode::Locked => "Locked",
        ViewLayoutObserverMode::All => "All",
    }
}

pub(super) fn format_layout_rect(rect: &ViewLayoutRect) -> String {
    format!(
        "x={} y={} w={} h={}",
        format_number(rect.x),
        format_number(rect.y),
        format_number(rect.width),
        format_number(rect.height)
    )
}

pub(super) fn display_label(value: SerializableDisplay) -> &'static str {
    match value {
        SerializableDisplay::Flex => "flex",
        SerializableDisplay::None => "none",
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::view::layout::{
        SerializableDisplay, SerializablePositionType, ViewLayoutEdges, ViewLayoutGap,
        ViewLayoutLengthDebug, ViewLayoutSizingDebug,
    };
    use crate::extra::debug::view_layout_observer::state::ViewLayoutObserverOrigin;

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
    fn build_selection_text_reports_selection_details() {
        let state = ViewLayoutObserverState {
            mode: ViewLayoutObserverMode::Locked,
            locked_entity: Some(Entity::from_bits(1)),
            window_entity: Some(Entity::from_bits(2)),
            camera_entity: Some(Entity::from_bits(3)),
            show_box_model: true,
            show_flex_guides: true,
            show_grid_guides: true,
            show_spatial_guides: true,
        };
        let selection = sample_selection(Entity::from_bits(7), 3, 625.0);

        let text = build_selection_text(&state, Some(&selection));

        assert!(text.contains("State: mode=Locked"));
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
