//! Conversion from View layout style data into Taffy style data.
//!
//! 将 View 布局样式数据转换为 Taffy 样式数据。

use super::{
    SerializableAlignItems, SerializableAlignSelf, SerializableDisplay, SerializableJustifyContent,
    SerializablePositionType, SerializableRect, SerializableVal, StyleDef, StyleGap,
    UiFlexDirection, ViewOverflowAxisDef, ViewOverflowDef, ViewSizeAxisDef, ViewSizingDef,
};
use bevy::prelude::Vec2;
use taffy::geometry::Point;
use taffy::prelude::{
    AlignItems as TaffyAlignItems, Dimension, Display as TaffyDisplay,
    FlexDirection as TaffyFlexDirection, JustifyContent as TaffyJustifyContent, LengthPercentage,
    LengthPercentageAuto, Position as TaffyPosition, Rect, Size, Style,
};
use taffy::style::Overflow as TaffyOverflow;

pub(super) fn to_taffy_style(
    style: &StyleDef,
    viewport_size: Vec2,
    parent_flex_direction: UiFlexDirection,
    is_root: bool,
    natural_size: Option<Size<f32>>,
) -> Style {
    let (size, flex_grow, flex_shrink, flex_basis, align_self) = to_sizing_style(
        style,
        viewport_size,
        parent_flex_direction,
        is_root,
        natural_size,
    );
    Style {
        display: style.display.map_or(TaffyDisplay::Flex, to_taffy_display),
        position: style
            .position_type
            .map_or(TaffyPosition::Relative, to_taffy_position),
        inset: Rect {
            left: style.left.map_or(LengthPercentageAuto::auto(), |value| {
                to_length_percentage_auto(value, viewport_size)
            }),
            right: style.right.map_or(LengthPercentageAuto::auto(), |value| {
                to_length_percentage_auto(value, viewport_size)
            }),
            top: style.top.map_or(LengthPercentageAuto::auto(), |value| {
                to_length_percentage_auto(value, viewport_size)
            }),
            bottom: style.bottom.map_or(LengthPercentageAuto::auto(), |value| {
                to_length_percentage_auto(value, viewport_size)
            }),
        },
        size,
        margin: style.margin.map_or_else(auto_rect_zero, |rect| {
            to_length_percentage_auto_rect(rect, viewport_size)
        }),
        padding: style.padding.map_or_else(length_rect_zero, |rect| {
            to_length_percentage_rect(rect, viewport_size)
        }),
        border: style.border.map_or_else(length_rect_zero, |rect| {
            to_length_percentage_rect(rect, viewport_size)
        }),
        gap: style.gap.map_or_else(length_size_zero, |gap| {
            to_length_percentage_gap(gap, viewport_size)
        }),
        align_items: style.align_items.map(to_taffy_align_items),
        align_self,
        justify_content: style.justify_content.map(to_taffy_justify_content),
        flex_direction: style
            .flex_direction
            .map_or(TaffyFlexDirection::Row, to_taffy_flex_direction),
        overflow: style
            .overflow
            .map_or_else(taffy_overflow_visible, to_taffy_overflow),
        flex_basis,
        flex_grow,
        flex_shrink,
        ..Default::default()
    }
}

fn to_sizing_style(
    style: &StyleDef,
    viewport_size: Vec2,
    parent_flex_direction: UiFlexDirection,
    is_root: bool,
    natural_size: Option<Size<f32>>,
) -> (
    Size<Dimension>,
    f32,
    f32,
    Dimension,
    Option<TaffyAlignItems>,
) {
    let (width_axis, height_axis) = sizing_axes(style);
    let mut size = Size {
        width: Dimension::auto(),
        height: Dimension::auto(),
    };
    let mut flex_grow = 0.0;
    let mut flex_shrink = 1.0;
    let mut flex_basis = Dimension::auto();
    let mut align_self = style.align_self.and_then(to_taffy_align_self);
    let fit_size = natural_size.map(|size| Size {
        width: size.width + box_axis_extra(style, true, viewport_size),
        height: size.height + box_axis_extra(style, false, viewport_size),
    });

    apply_size_axis(
        width_axis,
        true,
        &mut size,
        &mut flex_grow,
        &mut flex_shrink,
        &mut flex_basis,
        &mut align_self,
        viewport_size,
        parent_flex_direction,
        is_root,
        fit_size,
    );
    apply_size_axis(
        height_axis,
        false,
        &mut size,
        &mut flex_grow,
        &mut flex_shrink,
        &mut flex_basis,
        &mut align_self,
        viewport_size,
        parent_flex_direction,
        is_root,
        fit_size,
    );
    if style.sizing.is_some()
        && !is_root
        && align_self.is_none()
        && (fit_axis_is_cross(width_axis, true, parent_flex_direction)
            || fit_axis_is_cross(height_axis, false, parent_flex_direction))
    {
        align_self = Some(TaffyAlignItems::Start);
    }

    (size, flex_grow, flex_shrink, flex_basis, align_self)
}

pub(super) fn sizing_axes(style: &StyleDef) -> (ViewSizeAxisDef, ViewSizeAxisDef) {
    let (mut width, mut height) = match style.sizing {
        Some(ViewSizingDef::Fit) | None => (ViewSizeAxisDef::Fit, ViewSizeAxisDef::Fit),
        Some(ViewSizingDef::Fill) => (ViewSizeAxisDef::Fill, ViewSizeAxisDef::Fill),
        Some(ViewSizingDef::Fixed { width, height }) => (
            ViewSizeAxisDef::Fixed(width),
            ViewSizeAxisDef::Fixed(height),
        ),
        Some(ViewSizingDef::Axes { width, height }) => (width, height),
    };
    if let Some(value) = style.width {
        width = ViewSizeAxisDef::Fixed(value);
    }
    if let Some(value) = style.height {
        height = ViewSizeAxisDef::Fixed(value);
    }
    (width, height)
}

pub(super) fn box_axis_extra(style: &StyleDef, is_width: bool, viewport_size: Vec2) -> f32 {
    spacing_axis_extra(style.padding, is_width, viewport_size)
        + spacing_axis_extra(style.border, is_width, viewport_size)
}

fn spacing_axis_extra(rect: Option<SerializableRect>, is_width: bool, viewport_size: Vec2) -> f32 {
    rect.map_or(0.0, |rect| {
        let (start, end) = if is_width {
            (rect.left, rect.right)
        } else {
            (rect.top, rect.bottom)
        };
        to_layout_length(start, is_width, viewport_size)
            + to_layout_length(end, is_width, viewport_size)
    })
}

pub(super) fn to_layout_length(value: SerializableVal, is_width: bool, viewport_size: Vec2) -> f32 {
    match value {
        SerializableVal::Auto => 0.0,
        SerializableVal::Px(value) => value,
        SerializableVal::Percent(value) => {
            let basis = if is_width {
                viewport_size.x
            } else {
                viewport_size.y
            };
            basis * value / 100.0
        }
        SerializableVal::Vw(value) => viewport_size.x * value / 100.0,
        SerializableVal::Vh(value) => viewport_size.y * value / 100.0,
    }
}

fn apply_size_axis(
    axis: ViewSizeAxisDef,
    is_width: bool,
    size: &mut Size<Dimension>,
    flex_grow: &mut f32,
    flex_shrink: &mut f32,
    flex_basis: &mut Dimension,
    align_self: &mut Option<TaffyAlignItems>,
    viewport_size: Vec2,
    parent_flex_direction: UiFlexDirection,
    is_root: bool,
    fit_size: Option<Size<f32>>,
) {
    match axis {
        ViewSizeAxisDef::Fit => {
            let value = fit_size.map_or(Dimension::auto(), |size| {
                Dimension::length(if is_width { size.width } else { size.height })
            });
            set_size_axis(size, is_width, value);
        }
        ViewSizeAxisDef::Fixed(value) => {
            set_size_axis(size, is_width, to_dimension(value, viewport_size));
        }
        ViewSizeAxisDef::Fill if is_root => {
            set_size_axis(size, is_width, Dimension::percent(1.0));
        }
        ViewSizeAxisDef::Fill if axis_matches_main(is_width, parent_flex_direction) => {
            *flex_grow = 1.0;
            *flex_shrink = 1.0;
            *flex_basis = Dimension::length(0.0);
            set_size_axis(size, is_width, Dimension::auto());
        }
        ViewSizeAxisDef::Fill => {
            set_size_axis(size, is_width, Dimension::percent(1.0));
            if align_self.is_none() {
                *align_self = Some(TaffyAlignItems::Stretch);
            }
        }
    }
}

fn set_size_axis(size: &mut Size<Dimension>, is_width: bool, value: Dimension) {
    if is_width {
        size.width = value;
    } else {
        size.height = value;
    }
}

pub(super) fn axis_matches_main(is_width: bool, parent_flex_direction: UiFlexDirection) -> bool {
    matches!(
        (is_width, parent_flex_direction),
        (true, UiFlexDirection::Row | UiFlexDirection::RowReverse)
            | (
                false,
                UiFlexDirection::Column | UiFlexDirection::ColumnReverse
            )
    )
}

fn fit_axis_is_cross(
    axis: ViewSizeAxisDef,
    is_width: bool,
    parent_flex_direction: UiFlexDirection,
) -> bool {
    matches!(axis, ViewSizeAxisDef::Fit) && !axis_matches_main(is_width, parent_flex_direction)
}

fn to_length_percentage_auto_rect(
    rect: SerializableRect,
    viewport_size: Vec2,
) -> Rect<LengthPercentageAuto> {
    Rect {
        left: to_length_percentage_auto(rect.left, viewport_size),
        right: to_length_percentage_auto(rect.right, viewport_size),
        top: to_length_percentage_auto(rect.top, viewport_size),
        bottom: to_length_percentage_auto(rect.bottom, viewport_size),
    }
}

fn auto_rect_zero() -> Rect<LengthPercentageAuto> {
    Rect {
        left: LengthPercentageAuto::length(0.0),
        right: LengthPercentageAuto::length(0.0),
        top: LengthPercentageAuto::length(0.0),
        bottom: LengthPercentageAuto::length(0.0),
    }
}

fn length_rect_zero() -> Rect<LengthPercentage> {
    Rect {
        left: LengthPercentage::length(0.0),
        right: LengthPercentage::length(0.0),
        top: LengthPercentage::length(0.0),
        bottom: LengthPercentage::length(0.0),
    }
}

fn length_size_zero() -> Size<LengthPercentage> {
    Size {
        width: LengthPercentage::length(0.0),
        height: LengthPercentage::length(0.0),
    }
}

fn to_length_percentage_rect(
    rect: SerializableRect,
    viewport_size: Vec2,
) -> Rect<LengthPercentage> {
    Rect {
        left: to_length_percentage(rect.left, viewport_size),
        right: to_length_percentage(rect.right, viewport_size),
        top: to_length_percentage(rect.top, viewport_size),
        bottom: to_length_percentage(rect.bottom, viewport_size),
    }
}

fn to_length_percentage_gap(gap: StyleGap, viewport_size: Vec2) -> Size<LengthPercentage> {
    Size {
        width: to_length_percentage(gap.column, viewport_size),
        height: to_length_percentage(gap.row, viewport_size),
    }
}

fn to_dimension(value: SerializableVal, viewport_size: Vec2) -> Dimension {
    match value {
        SerializableVal::Auto => Dimension::auto(),
        SerializableVal::Px(value) => Dimension::length(value),
        SerializableVal::Percent(value) => Dimension::percent(value / 100.0),
        SerializableVal::Vw(value) => Dimension::length(viewport_size.x * value / 100.0),
        SerializableVal::Vh(value) => Dimension::length(viewport_size.y * value / 100.0),
    }
}

fn to_length_percentage_auto(value: SerializableVal, viewport_size: Vec2) -> LengthPercentageAuto {
    match value {
        SerializableVal::Auto => LengthPercentageAuto::auto(),
        SerializableVal::Px(value) => LengthPercentageAuto::length(value),
        SerializableVal::Percent(value) => LengthPercentageAuto::percent(value / 100.0),
        SerializableVal::Vw(value) => LengthPercentageAuto::length(viewport_size.x * value / 100.0),
        SerializableVal::Vh(value) => LengthPercentageAuto::length(viewport_size.y * value / 100.0),
    }
}

fn to_length_percentage(value: SerializableVal, viewport_size: Vec2) -> LengthPercentage {
    match value {
        SerializableVal::Auto => LengthPercentage::length(0.0),
        SerializableVal::Px(value) => LengthPercentage::length(value),
        SerializableVal::Percent(value) => LengthPercentage::percent(value / 100.0),
        SerializableVal::Vw(value) => LengthPercentage::length(viewport_size.x * value / 100.0),
        SerializableVal::Vh(value) => LengthPercentage::length(viewport_size.y * value / 100.0),
    }
}

fn to_taffy_display(display: SerializableDisplay) -> TaffyDisplay {
    match display {
        SerializableDisplay::Flex => TaffyDisplay::Flex,
        SerializableDisplay::None => TaffyDisplay::None,
    }
}

fn to_taffy_position(position: SerializablePositionType) -> TaffyPosition {
    match position {
        SerializablePositionType::Relative => TaffyPosition::Relative,
        SerializablePositionType::Absolute => TaffyPosition::Absolute,
    }
}

fn to_taffy_flex_direction(direction: UiFlexDirection) -> TaffyFlexDirection {
    match direction {
        UiFlexDirection::Row => TaffyFlexDirection::Row,
        UiFlexDirection::Column => TaffyFlexDirection::Column,
        UiFlexDirection::RowReverse => TaffyFlexDirection::RowReverse,
        UiFlexDirection::ColumnReverse => TaffyFlexDirection::ColumnReverse,
    }
}

fn to_taffy_justify_content(justify_content: SerializableJustifyContent) -> TaffyJustifyContent {
    match justify_content {
        SerializableJustifyContent::Start => TaffyJustifyContent::Start,
        SerializableJustifyContent::End => TaffyJustifyContent::End,
        SerializableJustifyContent::Center => TaffyJustifyContent::Center,
        SerializableJustifyContent::SpaceBetween => TaffyJustifyContent::SpaceBetween,
        SerializableJustifyContent::SpaceAround => TaffyJustifyContent::SpaceAround,
        SerializableJustifyContent::SpaceEvenly => TaffyJustifyContent::SpaceEvenly,
    }
}

fn to_taffy_align_items(align_items: SerializableAlignItems) -> TaffyAlignItems {
    match align_items {
        SerializableAlignItems::Start => TaffyAlignItems::Start,
        SerializableAlignItems::End => TaffyAlignItems::End,
        SerializableAlignItems::Center => TaffyAlignItems::Center,
        SerializableAlignItems::Baseline => TaffyAlignItems::Baseline,
        SerializableAlignItems::Stretch => TaffyAlignItems::Stretch,
    }
}

fn to_taffy_align_self(align_self: SerializableAlignSelf) -> Option<TaffyAlignItems> {
    match align_self {
        SerializableAlignSelf::Auto => None,
        SerializableAlignSelf::Start => Some(TaffyAlignItems::Start),
        SerializableAlignSelf::End => Some(TaffyAlignItems::End),
        SerializableAlignSelf::Center => Some(TaffyAlignItems::Center),
        SerializableAlignSelf::Baseline => Some(TaffyAlignItems::Baseline),
        SerializableAlignSelf::Stretch => Some(TaffyAlignItems::Stretch),
    }
}

fn taffy_overflow_visible() -> Point<TaffyOverflow> {
    Point {
        x: TaffyOverflow::Visible,
        y: TaffyOverflow::Visible,
    }
}

fn to_taffy_overflow(overflow: ViewOverflowDef) -> Point<TaffyOverflow> {
    let (horizontal, vertical) = overflow_axes(overflow);
    Point {
        x: to_taffy_overflow_axis(horizontal),
        y: to_taffy_overflow_axis(vertical),
    }
}

fn to_taffy_overflow_axis(axis: ViewOverflowAxisDef) -> TaffyOverflow {
    match axis {
        ViewOverflowAxisDef::Visible => TaffyOverflow::Visible,
        ViewOverflowAxisDef::Hidden => TaffyOverflow::Hidden,
        ViewOverflowAxisDef::Scroll => TaffyOverflow::Scroll,
    }
}

fn overflow_axes(overflow: ViewOverflowDef) -> (ViewOverflowAxisDef, ViewOverflowAxisDef) {
    match overflow {
        ViewOverflowDef::Visible => (ViewOverflowAxisDef::Visible, ViewOverflowAxisDef::Visible),
        ViewOverflowDef::Hidden => (ViewOverflowAxisDef::Hidden, ViewOverflowAxisDef::Hidden),
        ViewOverflowDef::Scroll => (ViewOverflowAxisDef::Scroll, ViewOverflowAxisDef::Scroll),
        ViewOverflowDef::Axes {
            horizontal,
            vertical,
        } => (horizontal, vertical),
    }
}

pub(super) fn overflow_clips(overflow: Option<ViewOverflowDef>) -> bool {
    overflow.is_some_and(|overflow| {
        let (horizontal, vertical) = overflow_axes(overflow);
        matches!(
            horizontal,
            ViewOverflowAxisDef::Hidden | ViewOverflowAxisDef::Scroll
        ) || matches!(
            vertical,
            ViewOverflowAxisDef::Hidden | ViewOverflowAxisDef::Scroll
        )
    })
}

pub(super) fn overflow_scrolls(overflow: Option<ViewOverflowDef>) -> bool {
    overflow.is_some_and(|overflow| {
        let (horizontal, vertical) = overflow_axes(overflow);
        matches!(horizontal, ViewOverflowAxisDef::Scroll)
            || matches!(vertical, ViewOverflowAxisDef::Scroll)
    })
}
