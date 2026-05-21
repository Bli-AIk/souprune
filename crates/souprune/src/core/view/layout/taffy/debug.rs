//! Debug metadata extraction for Taffy View layout slots.
//!
//! Taffy View 布局槽位的调试元数据提取。

use super::style::{axis_matches_main, box_axis_extra, sizing_axes, to_layout_length};
use super::{
    SerializableDisplay, SerializablePositionType, SerializableRect, SerializableVal, StyleDef,
    StyleGap, UiFlexDirection, ViewLayoutDebugMetadata, ViewLayoutEdges, ViewLayoutGap,
    ViewLayoutLengthDebug, ViewLayoutSizingDebug, ViewNodeDef, ViewSizeAxisDef,
};
use bevy::prelude::Vec2;
use taffy::geometry::Size;

pub(super) fn build_layout_debug_metadata(
    node: &ViewNodeDef,
    path: String,
    depth: usize,
    parent_path: Option<String>,
    viewport_size: Vec2,
    parent_flex_direction: UiFlexDirection,
    is_root: bool,
    natural_size: Option<Size<f32>>,
) -> ViewLayoutDebugMetadata {
    let style = &node.style;
    ViewLayoutDebugMetadata {
        path,
        name: node.name.clone(),
        depth,
        parent_path,
        display: style.display.unwrap_or(SerializableDisplay::Flex),
        position_type: style
            .position_type
            .unwrap_or(SerializablePositionType::Relative),
        flex_direction: style.flex_direction.unwrap_or_default(),
        justify_content: style.justify_content,
        align_items: style.align_items,
        align_self: style.align_self,
        margin: debug_edges(style.margin, viewport_size),
        padding: debug_edges(style.padding, viewport_size),
        border: debug_edges(style.border, viewport_size),
        gap: debug_gap(style.gap, viewport_size),
        overflow: style.overflow,
        sizing: debug_sizing(
            style,
            viewport_size,
            parent_flex_direction,
            is_root,
            natural_size,
        ),
    }
}

fn debug_edges(rect: Option<SerializableRect>, viewport_size: Vec2) -> ViewLayoutEdges {
    rect.map_or(ViewLayoutEdges::new(0.0, 0.0, 0.0, 0.0), |rect| {
        ViewLayoutEdges::new(
            to_layout_length(rect.left, true, viewport_size),
            to_layout_length(rect.right, true, viewport_size),
            to_layout_length(rect.top, false, viewport_size),
            to_layout_length(rect.bottom, false, viewport_size),
        )
    })
}

fn debug_gap(gap: Option<StyleGap>, viewport_size: Vec2) -> ViewLayoutGap {
    gap.map_or(ViewLayoutGap::new(0.0, 0.0), |gap| {
        ViewLayoutGap::new(
            to_layout_length(gap.row, false, viewport_size),
            to_layout_length(gap.column, true, viewport_size),
        )
    })
}

fn debug_sizing(
    style: &StyleDef,
    viewport_size: Vec2,
    parent_flex_direction: UiFlexDirection,
    is_root: bool,
    natural_size: Option<Size<f32>>,
) -> ViewLayoutSizingDebug {
    let (width_axis, height_axis) = sizing_axes(style);
    let fit_size = natural_size.map(|size| Size {
        width: size.width + box_axis_extra(style, true, viewport_size),
        height: size.height + box_axis_extra(style, false, viewport_size),
    });
    let mut flex_grow = 0.0;
    let mut flex_shrink = 1.0;
    let mut flex_basis = ViewLayoutLengthDebug::Auto;

    apply_debug_flex_axis(
        width_axis,
        true,
        parent_flex_direction,
        is_root,
        &mut flex_grow,
        &mut flex_shrink,
        &mut flex_basis,
    );
    apply_debug_flex_axis(
        height_axis,
        false,
        parent_flex_direction,
        is_root,
        &mut flex_grow,
        &mut flex_shrink,
        &mut flex_basis,
    );

    ViewLayoutSizingDebug {
        width: debug_axis_length(
            width_axis,
            true,
            parent_flex_direction,
            is_root,
            fit_size.map(|size| size.width),
            viewport_size,
        ),
        height: debug_axis_length(
            height_axis,
            false,
            parent_flex_direction,
            is_root,
            fit_size.map(|size| size.height),
            viewport_size,
        ),
        flex_grow,
        flex_shrink,
        flex_basis,
    }
}

fn apply_debug_flex_axis(
    axis: ViewSizeAxisDef,
    is_width: bool,
    parent_flex_direction: UiFlexDirection,
    is_root: bool,
    flex_grow: &mut f32,
    flex_shrink: &mut f32,
    flex_basis: &mut ViewLayoutLengthDebug,
) {
    if matches!(axis, ViewSizeAxisDef::Fill)
        && !is_root
        && axis_matches_main(is_width, parent_flex_direction)
    {
        *flex_grow = 1.0;
        *flex_shrink = 1.0;
        *flex_basis = ViewLayoutLengthDebug::Px(0.0);
    }
}

fn debug_axis_length(
    axis: ViewSizeAxisDef,
    is_width: bool,
    parent_flex_direction: UiFlexDirection,
    is_root: bool,
    fit_size: Option<f32>,
    viewport_size: Vec2,
) -> ViewLayoutLengthDebug {
    match axis {
        ViewSizeAxisDef::Fit => fit_size.map_or(ViewLayoutLengthDebug::Auto, |value| {
            ViewLayoutLengthDebug::Px(value)
        }),
        ViewSizeAxisDef::Fixed(value) => debug_serializable_length(value, is_width, viewport_size),
        ViewSizeAxisDef::Fill if is_root => ViewLayoutLengthDebug::Percent(100.0),
        ViewSizeAxisDef::Fill if axis_matches_main(is_width, parent_flex_direction) => {
            ViewLayoutLengthDebug::Auto
        }
        ViewSizeAxisDef::Fill => ViewLayoutLengthDebug::Percent(100.0),
    }
}

fn debug_serializable_length(
    value: SerializableVal,
    is_width: bool,
    viewport_size: Vec2,
) -> ViewLayoutLengthDebug {
    match value {
        SerializableVal::Auto => ViewLayoutLengthDebug::Auto,
        SerializableVal::Px(value) => ViewLayoutLengthDebug::Px(value),
        SerializableVal::Percent(value) => ViewLayoutLengthDebug::Percent(value),
        SerializableVal::Vw(_) | SerializableVal::Vh(_) => {
            ViewLayoutLengthDebug::Px(to_layout_length(value, is_width, viewport_size))
        }
    }
}
