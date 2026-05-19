//! Pure Taffy layout solving for view layout assets.
//!
//! 视图布局资源的纯 Taffy 布局求解。

use super::{
    RepeatDef, SerializableAlignItems, SerializableAlignSelf, SerializableDisplay,
    SerializableJustifyContent, SerializablePositionType, SerializableRect, SerializableVal,
    StyleDef, StyleGap, TextDef, UiFlexDirection, ViewLayoutAsset, ViewLayoutSlot, ViewLayoutSlots,
    ViewNodeDef, ViewOverflowAxisDef, ViewOverflowDef, ViewScrollState, ViewSizeAxisDef,
    ViewSizingDef, layout_child_path, layout_repeat_path, layout_root_path,
    serializable_vec2_to_vec2,
};
use bevy::prelude::Vec2;
use std::error::Error;
use std::fmt;
use taffy::geometry::Point;
use taffy::prelude::{
    AlignItems as TaffyAlignItems, AvailableSpace, Dimension, Display as TaffyDisplay,
    FlexDirection as TaffyFlexDirection, JustifyContent as TaffyJustifyContent, LengthPercentage,
    LengthPercentageAuto, NodeId, Position as TaffyPosition, Rect, Size, Style, TaffyTree,
};
use taffy::style::Overflow as TaffyOverflow;

/// Error returned when pure view layout solving fails.
///
/// 纯视图布局求解失败时返回的错误。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewLayoutError {
    message: String,
}

impl ViewLayoutError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for ViewLayoutError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for ViewLayoutError {}

/// Dynamic data hooks used while solving View layout.
///
/// 求解 View 布局时使用的动态数据钩子。
pub struct ViewLayoutContext<'a> {
    /// Return repeat count for a repeat definition, or `None` to keep the node unexpanded.
    ///
    /// 返回 repeat 定义的数量；返回 `None` 表示保持节点不展开。
    pub repeat_count: &'a dyn Fn(&RepeatDef) -> Option<usize>,
    /// Return the concrete item value for a repeat instance.
    ///
    /// 返回 repeat 实例的具体条目值。
    pub repeat_item: &'a dyn Fn(&RepeatDef, usize) -> Option<String>,
    /// Resolve authored text content before text measurement.
    ///
    /// 在文本测量前解析作者填写的文本内容。
    pub text_content: &'a dyn Fn(&str, Option<&ViewLayoutRepeatContext>) -> String,
}

/// Repeat variables available while measuring a repeated View node.
///
/// 测量重复 View 节点时可用的 repeat 变量。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ViewLayoutRepeatContext {
    /// Current repeat index.
    ///
    /// 当前 repeat 索引。
    pub index: usize,
    /// Optional author-named index variable.
    ///
    /// 作者可选指定的索引变量名。
    pub index_var: Option<String>,
    /// Optional author-named item variable.
    ///
    /// 作者可选指定的条目变量名。
    pub item_var: Option<String>,
    /// Concrete item value for this repeat instance.
    ///
    /// 当前 repeat 实例的具体条目值。
    pub item_value: Option<String>,
}

/// Compute stable layout slots for a view layout asset without spawning entities.
///
/// 在不生成实体的情况下，为视图布局资源计算稳定布局槽位。
pub fn compute_taffy_layout(
    asset: &ViewLayoutAsset,
    viewport_size: Vec2,
) -> Result<ViewLayoutSlots, ViewLayoutError> {
    let repeat_count = |_repeat: &RepeatDef| None;
    let repeat_item = |_repeat: &RepeatDef, _index: usize| None;
    let text_content =
        |content: &str, _repeat_ctx: Option<&ViewLayoutRepeatContext>| content.to_string();
    compute_taffy_layout_with_context(
        asset,
        viewport_size,
        ViewLayoutContext {
            repeat_count: &repeat_count,
            repeat_item: &repeat_item,
            text_content: &text_content,
        },
    )
}

/// Compute stable layout slots with dynamic repeat/text resolution.
///
/// 使用动态 repeat/text 解析计算稳定布局槽位。
pub fn compute_taffy_layout_with_context(
    asset: &ViewLayoutAsset,
    viewport_size: Vec2,
    context: ViewLayoutContext<'_>,
) -> Result<ViewLayoutSlots, ViewLayoutError> {
    let mut tree = TaffyTree::<LeafMeasureContext>::new();
    let mut nodes = Vec::new();
    let mut root_ids = Vec::new();
    for (root_idx, root) in asset.roots.iter().enumerate() {
        let ids = build_node(
            &mut tree,
            root,
            layout_root_path(root_idx, root),
            viewport_size,
            UiFlexDirection::Row,
            true,
            false,
            &context,
            None,
            &mut nodes,
        )?;
        root_ids.extend(ids);
    }

    let available = Size {
        width: AvailableSpace::Definite(viewport_size.x),
        height: AvailableSpace::Definite(viewport_size.y),
    };
    for root_id in root_ids {
        tree.compute_layout_with_measure(
            root_id,
            available,
            |known_dimensions, _available_space, _node_id, node_context, _style| {
                measure_leaf(known_dimensions, node_context)
            },
        )
        .map_err(|error| ViewLayoutError::new(format!("failed to compute layout: {error}")))?;
    }

    let mut slots = ViewLayoutSlots::new();
    for (node_id, path, name, overflow) in nodes {
        let layout = tree
            .layout(node_id)
            .map_err(|error| ViewLayoutError::new(format!("failed to read layout: {error}")))?;
        let clips_overflow = overflow_clips(overflow);
        let clip_rect = clips_overflow.then(|| {
            super::ViewClipRect::new(
                layout.location.x,
                layout.location.y,
                layout.size.width,
                layout.size.height,
            )
        });
        let scroll_state = overflow_scrolls(overflow).then_some(ViewScrollState::default());
        slots.push_with_metadata(
            ViewLayoutSlot {
                path,
                name,
                x: layout.location.x,
                y: layout.location.y,
                width: layout.size.width,
                height: layout.size.height,
            },
            clip_rect,
            scroll_state,
        );
    }

    Ok(slots)
}

fn build_node(
    tree: &mut TaffyTree<LeafMeasureContext>,
    node: &ViewNodeDef,
    path: String,
    viewport_size: Vec2,
    parent_flex_direction: UiFlexDirection,
    is_root: bool,
    ancestor_hidden: bool,
    context: &ViewLayoutContext<'_>,
    repeat_ctx: Option<&ViewLayoutRepeatContext>,
    nodes: &mut Vec<(NodeId, String, String, Option<ViewOverflowDef>)>,
) -> Result<Vec<NodeId>, ViewLayoutError> {
    if let Some(repeat) = &node.repeat
        && let Some(count) = (context.repeat_count)(repeat)
    {
        let mut repeated = Vec::with_capacity(count);
        for index in 0..count {
            let instance_ctx = ViewLayoutRepeatContext {
                index,
                index_var: repeat.index_var.clone(),
                item_var: Some(
                    repeat
                        .item_var
                        .clone()
                        .unwrap_or_else(|| "item".to_string()),
                ),
                item_value: (context.repeat_item)(repeat, index),
            };
            repeated.push(build_single_node(
                tree,
                node,
                layout_repeat_path(&path, index),
                viewport_size,
                parent_flex_direction,
                is_root,
                ancestor_hidden,
                context,
                Some(&instance_ctx),
                nodes,
            )?);
        }
        return Ok(repeated);
    }

    Ok(vec![build_single_node(
        tree,
        node,
        path,
        viewport_size,
        parent_flex_direction,
        is_root,
        ancestor_hidden,
        context,
        repeat_ctx,
        nodes,
    )?])
}

fn build_single_node(
    tree: &mut TaffyTree<LeafMeasureContext>,
    node: &ViewNodeDef,
    path: String,
    viewport_size: Vec2,
    parent_flex_direction: UiFlexDirection,
    is_root: bool,
    ancestor_hidden: bool,
    context: &ViewLayoutContext<'_>,
    repeat_ctx: Option<&ViewLayoutRepeatContext>,
    nodes: &mut Vec<(NodeId, String, String, Option<ViewOverflowDef>)>,
) -> Result<NodeId, ViewLayoutError> {
    let hidden = ancestor_hidden || matches!(node.style.display, Some(SerializableDisplay::None));
    let node_flex_direction = node.style.flex_direction.unwrap_or_default();
    let children = node
        .children
        .iter()
        .enumerate()
        .map(|(child_idx, child)| {
            build_node(
                tree,
                child,
                layout_child_path(&path, child_idx, child),
                viewport_size,
                node_flex_direction,
                false,
                hidden,
                context,
                repeat_ctx,
                nodes,
            )
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    let natural_size = natural_measure_size(node, context, repeat_ctx);
    let style = to_taffy_style(
        &node.style,
        viewport_size,
        parent_flex_direction,
        is_root,
        natural_size,
    );
    let node_id = if children.is_empty() {
        if let Some(context) = natural_size.map(|size| LeafMeasureContext { size }) {
            tree.new_leaf_with_context(style, context)
        } else {
            tree.new_leaf(style)
        }
    } else {
        tree.new_with_children(style, &children)
    }
    .map_err(|error| ViewLayoutError::new(format!("failed to create layout node: {error}")))?;
    if !hidden {
        nodes.push((node_id, path, node.name.clone(), node.style.overflow));
    }
    Ok(node_id)
}

#[derive(Clone, Debug)]
struct LeafMeasureContext {
    size: Size<f32>,
}

fn natural_measure_size(
    node: &ViewNodeDef,
    context: &ViewLayoutContext<'_>,
    repeat_ctx: Option<&ViewLayoutRepeatContext>,
) -> Option<Size<f32>> {
    node.view_box
        .as_ref()
        .map(|view_box| Size {
            width: view_box.width,
            height: view_box.height,
        })
        .or_else(|| {
            (!node.texts.is_empty()).then(|| measure_texts(&node.texts, context, repeat_ctx))
        })
}

fn measure_leaf(
    known_dimensions: Size<Option<f32>>,
    node_context: Option<&mut LeafMeasureContext>,
) -> Size<f32> {
    let measured = node_context.map_or(
        Size {
            width: 0.0,
            height: 0.0,
        },
        |context| context.size,
    );
    Size {
        width: known_dimensions.width.unwrap_or(measured.width),
        height: known_dimensions.height.unwrap_or(measured.height),
    }
}

fn measure_texts(
    texts: &[TextDef],
    context: &ViewLayoutContext<'_>,
    repeat_ctx: Option<&ViewLayoutRepeatContext>,
) -> Size<f32> {
    texts.iter().fold(
        Size {
            width: 0.0,
            height: 0.0,
        },
        |mut total, text| {
            let size = measure_text(text, context, repeat_ctx);
            total.width = total.width.max(size.width);
            total.height += size.height;
            total
        },
    )
}

fn measure_text(
    text: &TextDef,
    context: &ViewLayoutContext<'_>,
    repeat_ctx: Option<&ViewLayoutRepeatContext>,
) -> Size<f32> {
    let raw_content = text.content.as_deref().unwrap_or("");
    let content = (context.text_content)(raw_content, repeat_ctx);
    let view_font: crate::core::view::components::ViewFont = text.font.clone().into();
    let font_size = view_font.default_size();
    let world_scale = serializable_vec2_to_vec2(&text.world_scale);
    let scale = world_scale.x.abs() / font_size;
    let line_height = text.line_height.unwrap_or(1.0);
    let char_spacing = text.char_spacing.unwrap_or(0.0);
    let word_spacing = text.word_spacing.unwrap_or(0.0);
    let lines = content.split('\n').collect::<Vec<_>>();
    let line_count = lines.len() as f32;
    let glyph_width = font_size * 0.5;
    let width = lines
        .iter()
        .map(|line| measure_text_line(line, glyph_width, char_spacing, word_spacing) * scale)
        .fold(0.0, f32::max);
    Size {
        width,
        height: line_count * font_size * line_height * scale,
    }
}

/// Apply repeat variables to authored text before regular template resolution.
///
/// 在常规模板解析前，把 repeat 变量应用到作者文本中。
pub fn apply_layout_repeat_context_to_text(
    content: &str,
    repeat_ctx: Option<&ViewLayoutRepeatContext>,
) -> String {
    let Some(repeat_ctx) = repeat_ctx else {
        return content.to_string();
    };

    let mut result = replace_at_variable(content, "i", &repeat_ctx.index.to_string());
    result = replace_at_variable(&result, "index", &repeat_ctx.index.to_string());
    if let Some(index_var) = &repeat_ctx.index_var {
        result = replace_at_variable(&result, index_var, &repeat_ctx.index.to_string());
    }
    if let (Some(item_var), Some(item_value)) = (&repeat_ctx.item_var, &repeat_ctx.item_value) {
        result = replace_at_variable(&result, item_var, item_value);
    }
    result
}

fn replace_at_variable(input: &str, name: &str, value: &str) -> String {
    let token = format!("@{name}");
    let mut output = String::with_capacity(input.len());
    let mut rest = input;

    while let Some(pos) = rest.find(&token) {
        let after_token = &rest[pos + token.len()..];
        output.push_str(&rest[..pos]);
        if after_token
            .chars()
            .next()
            .is_none_or(|ch| !ch.is_ascii_alphanumeric() && ch != '_')
        {
            output.push_str(value);
        } else {
            output.push_str(&token);
        }
        rest = after_token;
    }

    output.push_str(rest);
    output
}

fn measure_text_line(line: &str, glyph_width: f32, char_spacing: f32, word_spacing: f32) -> f32 {
    let char_count = line.chars().count();
    if char_count == 0 {
        return 0.0;
    }
    let word_gap_count = line.chars().filter(|value| value.is_whitespace()).count();
    char_count as f32 * (glyph_width + char_spacing) + word_gap_count as f32 * word_spacing
}

fn to_taffy_style(
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

fn sizing_axes(style: &StyleDef) -> (ViewSizeAxisDef, ViewSizeAxisDef) {
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

fn box_axis_extra(style: &StyleDef, is_width: bool, viewport_size: Vec2) -> f32 {
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

fn to_layout_length(value: SerializableVal, is_width: bool, viewport_size: Vec2) -> f32 {
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

fn axis_matches_main(is_width: bool, parent_flex_direction: UiFlexDirection) -> bool {
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

fn overflow_clips(overflow: Option<ViewOverflowDef>) -> bool {
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

fn overflow_scrolls(overflow: Option<ViewOverflowDef>) -> bool {
    overflow.is_some_and(|overflow| {
        let (horizontal, vertical) = overflow_axes(overflow);
        matches!(horizontal, ViewOverflowAxisDef::Scroll)
            || matches!(vertical, ViewOverflowAxisDef::Scroll)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::view::layout::{
        SerializableDisplay, SerializablePositionType, SerializableTransform, SerializableVal,
        StyleDef, StyleGap, UiFlexDirection, ViewFocusPolicyDef, ViewNodeDef, ViewOverflowAxisDef,
        ViewOverflowDef, ViewScrollState, ViewSizeAxisDef, ViewSizingDef, ViewSpaceDef,
        ViewSpatialAnchorDef, ViewSpatialInputDef, ViewSpatialOrientationDef,
    };

    fn asset(root: ViewNodeDef) -> ViewLayoutAsset {
        ViewLayoutAsset {
            roots: vec![root],
            requires: Vec::new(),
            facts: None,
            space: None,
            coordinate_system: Default::default(),
            coordinate_space: None,
        }
    }

    fn node(name: &str, style: StyleDef, children: Vec<ViewNodeDef>) -> ViewNodeDef {
        ViewNodeDef {
            name: name.to_string(),
            tags: Vec::new(),
            style,
            transform: None,
            focus_policy: None,
            visible_when: None,
            background_color: None,
            border_color: None,
            image: None,
            sprite: None,
            state_sprite: None,
            texts: Vec::new(),
            view_box: None,
            children,
            repeat: None,
        }
    }

    fn find_node<'a>(nodes: &'a [ViewNodeDef], name: &str) -> Option<&'a ViewNodeDef> {
        nodes.iter().find_map(|node| {
            if node.name == name {
                Some(node)
            } else {
                find_node(&node.children, name)
            }
        })
    }

    fn assert_close(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() < 0.01,
            "expected {actual} to be close to {expected}"
        );
    }

    #[test]
    fn row_flex_centers_children_with_gap() {
        let child_style = StyleDef {
            width: Some(SerializableVal::Px(100.0)),
            height: Some(SerializableVal::Px(40.0)),
            ..Default::default()
        };
        let root = node(
            "root",
            StyleDef {
                width: Some(SerializableVal::Px(640.0)),
                height: Some(SerializableVal::Px(480.0)),
                flex_direction: Some(UiFlexDirection::Row),
                justify_content: Some(
                    crate::core::view::layout::SerializableJustifyContent::Center,
                ),
                gap: Some(StyleGap {
                    row: SerializableVal::Px(0.0),
                    column: SerializableVal::Px(20.0),
                }),
                ..Default::default()
            },
            vec![
                node("first", child_style.clone(), Vec::new()),
                node("second", child_style, Vec::new()),
            ],
        );

        let slots = compute_taffy_layout(&asset(root), Vec2::new(640.0, 480.0)).unwrap();

        assert_close(slots.get("0:root/0:first").expect("first slot").x, 210.0);
        assert_close(slots.get("0:root/1:second").expect("second slot").x, 330.0);
    }

    #[test]
    fn absolute_child_uses_parent_inset_and_does_not_participate_in_sibling_flex() {
        let sized_child = StyleDef {
            width: Some(SerializableVal::Px(100.0)),
            height: Some(SerializableVal::Px(40.0)),
            ..Default::default()
        };
        let absolute_child = StyleDef {
            width: Some(SerializableVal::Px(50.0)),
            height: Some(SerializableVal::Px(30.0)),
            left: Some(SerializableVal::Px(25.0)),
            top: Some(SerializableVal::Px(35.0)),
            position_type: Some(SerializablePositionType::Absolute),
            ..Default::default()
        };
        let root = node(
            "root",
            StyleDef {
                width: Some(SerializableVal::Px(640.0)),
                height: Some(SerializableVal::Px(480.0)),
                flex_direction: Some(UiFlexDirection::Row),
                gap: Some(StyleGap {
                    row: SerializableVal::Px(0.0),
                    column: SerializableVal::Px(20.0),
                }),
                ..Default::default()
            },
            vec![
                node("first", sized_child.clone(), Vec::new()),
                node("absolute", absolute_child, Vec::new()),
                node("second", sized_child, Vec::new()),
            ],
        );

        let slots = compute_taffy_layout(&asset(root), Vec2::new(640.0, 480.0)).unwrap();

        let absolute = slots.get("0:root/1:absolute").expect("absolute slot");
        assert_close(absolute.x, 25.0);
        assert_close(absolute.y, 35.0);
        assert_close(slots.get("0:root/0:first").expect("first slot").x, 0.0);
        assert_close(slots.get("0:root/2:second").expect("second slot").x, 120.0);
    }

    #[test]
    fn display_none_node_is_absent_from_slots_and_flex_flow() {
        let visible_child = StyleDef {
            width: Some(SerializableVal::Px(100.0)),
            height: Some(SerializableVal::Px(40.0)),
            ..Default::default()
        };
        let hidden_child = StyleDef {
            width: Some(SerializableVal::Px(100.0)),
            height: Some(SerializableVal::Px(40.0)),
            display: Some(SerializableDisplay::None),
            ..Default::default()
        };
        let root = node(
            "root",
            StyleDef {
                width: Some(SerializableVal::Px(640.0)),
                height: Some(SerializableVal::Px(480.0)),
                flex_direction: Some(UiFlexDirection::Row),
                gap: Some(StyleGap {
                    row: SerializableVal::Px(0.0),
                    column: SerializableVal::Px(20.0),
                }),
                ..Default::default()
            },
            vec![
                node("first", visible_child.clone(), Vec::new()),
                node("hidden", hidden_child, Vec::new()),
                node("second", visible_child, Vec::new()),
            ],
        );

        let slots = compute_taffy_layout(&asset(root), Vec2::new(640.0, 480.0)).unwrap();

        assert!(slots.get("0:root/1:hidden").is_none());
        assert_close(slots.get("0:root/0:first").expect("first slot").x, 0.0);
        assert_close(slots.get("0:root/2:second").expect("second slot").x, 120.0);
    }

    #[test]
    fn overflow_hidden_and_scroll_create_slot_metadata() {
        let root = node(
            "root",
            StyleDef {
                width: Some(SerializableVal::Px(640.0)),
                height: Some(SerializableVal::Px(480.0)),
                flex_direction: Some(UiFlexDirection::Row),
                ..Default::default()
            },
            vec![
                node(
                    "hidden",
                    StyleDef {
                        width: Some(SerializableVal::Px(160.0)),
                        height: Some(SerializableVal::Px(80.0)),
                        overflow: Some(ViewOverflowDef::Hidden),
                        ..Default::default()
                    },
                    Vec::new(),
                ),
                node(
                    "scroll",
                    StyleDef {
                        width: Some(SerializableVal::Px(120.0)),
                        height: Some(SerializableVal::Px(60.0)),
                        overflow: Some(ViewOverflowDef::Axes {
                            horizontal: ViewOverflowAxisDef::Visible,
                            vertical: ViewOverflowAxisDef::Scroll,
                        }),
                        ..Default::default()
                    },
                    Vec::new(),
                ),
            ],
        );

        let slots = compute_taffy_layout(&asset(root), Vec2::new(640.0, 480.0)).unwrap();

        let hidden = slots.get("0:root/0:hidden").expect("hidden slot");
        let hidden_clip = slots
            .clip_rect("0:root/0:hidden")
            .expect("hidden clip rect");
        assert_close(hidden_clip.x, hidden.x);
        assert_close(hidden_clip.y, hidden.y);
        assert_close(hidden_clip.width, 160.0);
        assert_close(hidden_clip.height, 80.0);
        assert!(slots.scroll_state("0:root/0:hidden").is_none());

        let scroll = slots.get("0:root/1:scroll").expect("scroll slot");
        let scroll_clip = slots
            .clip_rect("0:root/1:scroll")
            .expect("scroll clip rect");
        assert_close(scroll_clip.x, scroll.x);
        assert_close(scroll_clip.y, scroll.y);
        assert_close(scroll_clip.width, 120.0);
        assert_close(scroll_clip.height, 60.0);
        assert_eq!(
            slots.scroll_state("0:root/1:scroll"),
            Some(&ViewScrollState::default())
        );
    }

    #[test]
    fn explicit_transform_is_not_applied_to_solver_output() {
        let mut child = node(
            "child",
            StyleDef {
                width: Some(SerializableVal::Px(100.0)),
                height: Some(SerializableVal::Px(40.0)),
                ..Default::default()
            },
            Vec::new(),
        );
        child.transform = Some(SerializableTransform {
            translation: Some((
                crate::core::sequencer::chapter_schema::Value::Static(1000.0),
                crate::core::sequencer::chapter_schema::Value::Static(2000.0),
                crate::core::sequencer::chapter_schema::Value::Static(0.0),
            )),
            rotation: None,
            scale: None,
        });
        let root = node(
            "root",
            StyleDef {
                width: Some(SerializableVal::Px(640.0)),
                height: Some(SerializableVal::Px(480.0)),
                ..Default::default()
            },
            vec![child],
        );

        let slots = compute_taffy_layout(&asset(root), Vec2::new(640.0, 480.0)).unwrap();

        let child = slots.get("0:root/0:child").expect("child slot");
        assert_close(child.x, 0.0);
        assert_close(child.y, 0.0);
    }

    #[test]
    fn sibling_index_keeps_duplicate_names_distinct() {
        let child_style = StyleDef {
            width: Some(SerializableVal::Px(100.0)),
            height: Some(SerializableVal::Px(40.0)),
            ..Default::default()
        };
        let root = node(
            "root",
            StyleDef {
                width: Some(SerializableVal::Px(640.0)),
                height: Some(SerializableVal::Px(480.0)),
                flex_direction: Some(UiFlexDirection::Row),
                gap: Some(StyleGap {
                    row: SerializableVal::Px(0.0),
                    column: SerializableVal::Px(20.0),
                }),
                ..Default::default()
            },
            vec![
                node("dup", child_style.clone(), Vec::new()),
                node("dup", child_style, Vec::new()),
            ],
        );

        let slots = compute_taffy_layout(&asset(root), Vec2::new(640.0, 480.0)).unwrap();

        assert_close(slots.get("0:root/0:dup").expect("first duplicate").x, 0.0);
        assert_close(
            slots.get("0:root/1:dup").expect("second duplicate").x,
            120.0,
        );
    }

    #[test]
    fn manual_acceptance_view_asset_parses_and_solves() {
        let mut asset: ViewLayoutAsset = ron::from_str(include_str!(
            "../../../../examples/assets/view/taffy_minimal.view.ron"
        ))
        .expect("manual acceptance view should parse");
        asset.apply_coordinate_system();

        let slots = compute_taffy_layout(&asset, Vec2::new(640.0, 480.0)).unwrap();

        let centered = slots
            .get("0:TaffyMinimalRoot/0:CenteredElement")
            .expect("centered element slot");
        assert_close(centered.x, 240.0);
        assert_close(centered.width, 160.0);
        let button_row = slots
            .get("0:TaffyMinimalRoot/1:ButtonRow")
            .expect("button row slot");
        assert!(button_row.width > 360.0);
        let fit_probe = slots
            .get("0:TaffyMinimalRoot/4:MeasuredViewBox")
            .expect("fit probe slot");
        assert_close(fit_probe.width, 134.0);
        assert_close(fit_probe.height, 54.0);
        assert!(
            slots
                .get("0:TaffyMinimalRoot/3:HiddenDisplayNoneProbe")
                .is_none()
        );
        let hidden_leaf = slots
            .iter()
            .find(|slot| slot.name == "Stage4HiddenLeafPanel")
            .expect("stage 4 hidden leaf panel slot");
        assert!(slots.clip_rect(&hidden_leaf.path).is_some());
        assert!(slots.scroll_state(&hidden_leaf.path).is_none());
        let scroll_viewport = slots
            .iter()
            .find(|slot| slot.name == "Stage4ScrollViewport")
            .expect("stage 4 scroll viewport slot");
        assert!(slots.clip_rect(&scroll_viewport.path).is_some());
        assert_eq!(
            slots.scroll_state(&scroll_viewport.path),
            Some(&ViewScrollState::default())
        );
        let stage4_region = find_node(&asset.roots, "Stage4AcceptanceRegion")
            .expect("stage 4 acceptance region node");
        assert!(matches!(
            stage4_region.focus_policy,
            Some(ViewFocusPolicyDef::Scope)
        ));
    }

    #[test]
    fn fit_view_box_leaf_uses_view_box_measurement() {
        let mut measured = node(
            "measured",
            StyleDef {
                sizing: Some(ViewSizingDef::Fit),
                ..Default::default()
            },
            Vec::new(),
        );
        measured.view_box = Some(crate::core::view::layout::ViewBoxLogicDef {
            width: 180.0,
            height: 64.0,
            border_width: 0.0,
            offset: (
                crate::core::sequencer::chapter_schema::Value::Static(0.0),
                crate::core::sequencer::chapter_schema::Value::Static(0.0),
                crate::core::sequencer::chapter_schema::Value::Static(0.0),
            ),
            fill_shader: None,
            structure_file: None,
            fill_color: None,
        });
        let root = node(
            "root",
            StyleDef {
                width: Some(SerializableVal::Px(640.0)),
                height: Some(SerializableVal::Px(480.0)),
                ..Default::default()
            },
            vec![measured],
        );

        let slots = compute_taffy_layout(&asset(root), Vec2::new(640.0, 480.0)).unwrap();

        let measured = slots.get("0:root/0:measured").expect("measured slot");
        assert_close(measured.width, 180.0);
        assert_close(measured.height, 64.0);
    }

    #[test]
    fn fit_view_box_adds_padding_and_border_to_measured_content() {
        let mut measured = node(
            "measured",
            StyleDef {
                sizing: Some(ViewSizingDef::Fit),
                padding: Some(crate::core::view::layout::SerializableRect {
                    left: SerializableVal::Px(10.0),
                    right: SerializableVal::Px(10.0),
                    top: SerializableVal::Px(3.0),
                    bottom: SerializableVal::Px(7.0),
                }),
                border: Some(crate::core::view::layout::SerializableRect {
                    left: SerializableVal::Px(1.0),
                    right: SerializableVal::Px(2.0),
                    top: SerializableVal::Px(4.0),
                    bottom: SerializableVal::Px(6.0),
                }),
                ..Default::default()
            },
            Vec::new(),
        );
        measured.view_box = Some(crate::core::view::layout::ViewBoxLogicDef {
            width: 100.0,
            height: 50.0,
            border_width: 0.0,
            offset: (
                crate::core::sequencer::chapter_schema::Value::Static(0.0),
                crate::core::sequencer::chapter_schema::Value::Static(0.0),
                crate::core::sequencer::chapter_schema::Value::Static(0.0),
            ),
            fill_shader: None,
            structure_file: None,
            fill_color: None,
        });
        let root = node(
            "root",
            StyleDef {
                width: Some(SerializableVal::Px(640.0)),
                height: Some(SerializableVal::Px(480.0)),
                ..Default::default()
            },
            vec![measured],
        );

        let slots = compute_taffy_layout(&asset(root), Vec2::new(640.0, 480.0)).unwrap();

        let measured = slots.get("0:root/0:measured").expect("measured slot");
        assert_close(measured.width, 123.0);
        assert_close(measured.height, 70.0);
    }

    #[test]
    fn fit_view_box_with_children_uses_view_box_measurement() {
        let mut measured = node(
            "measured",
            StyleDef {
                sizing: Some(ViewSizingDef::Fit),
                flex_direction: Some(UiFlexDirection::Row),
                ..Default::default()
            },
            vec![node(
                "child",
                StyleDef {
                    width: Some(SerializableVal::Px(25.0)),
                    height: Some(SerializableVal::Px(12.0)),
                    ..Default::default()
                },
                Vec::new(),
            )],
        );
        measured.view_box = Some(crate::core::view::layout::ViewBoxLogicDef {
            width: 180.0,
            height: 64.0,
            border_width: 0.0,
            offset: (
                crate::core::sequencer::chapter_schema::Value::Static(0.0),
                crate::core::sequencer::chapter_schema::Value::Static(0.0),
                crate::core::sequencer::chapter_schema::Value::Static(0.0),
            ),
            fill_shader: None,
            structure_file: None,
            fill_color: None,
        });
        let root = node(
            "root",
            StyleDef {
                width: Some(SerializableVal::Px(640.0)),
                height: Some(SerializableVal::Px(480.0)),
                ..Default::default()
            },
            vec![measured],
        );

        let slots = compute_taffy_layout(&asset(root), Vec2::new(640.0, 480.0)).unwrap();

        let measured = slots.get("0:root/0:measured").expect("measured slot");
        assert_close(measured.width, 180.0);
        assert_close(measured.height, 64.0);
    }

    #[test]
    fn fit_text_leaf_uses_conservative_text_measurement() {
        let mut measured = node(
            "text",
            StyleDef {
                sizing: Some(ViewSizingDef::Fit),
                ..Default::default()
            },
            Vec::new(),
        );
        measured.texts.push(crate::core::view::layout::TextDef {
            id: "label".to_string(),
            content: Some("AB\nC".to_string()),
            font: "DTM-Mono".to_string(),
            align: None,
            anchor: None,
            world_scale: (
                crate::core::sequencer::chapter_schema::Value::Static(1.0),
                crate::core::sequencer::chapter_schema::Value::Static(1.0),
            ),
            color: (
                crate::core::sequencer::chapter_schema::Value::Static(1.0),
                crate::core::sequencer::chapter_schema::Value::Static(1.0),
                crate::core::sequencer::chapter_schema::Value::Static(1.0),
                crate::core::sequencer::chapter_schema::Value::Static(1.0),
            ),
            transform: SerializableTransform::default(),
            line_height: Some(1.0),
            char_spacing: Some(0.0),
            word_spacing: Some(0.0),
            text_style: None,
            conditional_style: None,
            visible_when: None,
        });
        let root = node(
            "root",
            StyleDef {
                width: Some(SerializableVal::Px(640.0)),
                height: Some(SerializableVal::Px(480.0)),
                ..Default::default()
            },
            vec![measured],
        );

        let slots = compute_taffy_layout(&asset(root), Vec2::new(640.0, 480.0)).unwrap();

        let measured = slots.get("0:root/0:text").expect("text slot");
        assert_close(measured.width, 1.0);
        assert_close(measured.height, 2.0);
    }

    #[test]
    fn fit_text_measurement_preserves_negative_spacing() {
        let mut measured = node(
            "text",
            StyleDef {
                sizing: Some(ViewSizingDef::Fit),
                ..Default::default()
            },
            Vec::new(),
        );
        measured.texts.push(crate::core::view::layout::TextDef {
            id: "label".to_string(),
            content: Some("A B".to_string()),
            font: "DTM-Mono".to_string(),
            align: None,
            anchor: None,
            world_scale: (
                crate::core::sequencer::chapter_schema::Value::Static(128.0),
                crate::core::sequencer::chapter_schema::Value::Static(128.0),
            ),
            color: (
                crate::core::sequencer::chapter_schema::Value::Static(1.0),
                crate::core::sequencer::chapter_schema::Value::Static(1.0),
                crate::core::sequencer::chapter_schema::Value::Static(1.0),
                crate::core::sequencer::chapter_schema::Value::Static(1.0),
            ),
            transform: SerializableTransform::default(),
            line_height: Some(1.0),
            char_spacing: Some(-4.0),
            word_spacing: Some(-8.0),
            text_style: None,
            conditional_style: None,
            visible_when: None,
        });
        let root = node(
            "root",
            StyleDef {
                width: Some(SerializableVal::Px(640.0)),
                height: Some(SerializableVal::Px(480.0)),
                ..Default::default()
            },
            vec![measured],
        );

        let slots = compute_taffy_layout(&asset(root), Vec2::new(640.0, 480.0)).unwrap();

        let measured = slots.get("0:root/0:text").expect("text slot");
        assert_close(measured.width, 172.0);
    }

    #[test]
    fn fit_text_measurement_applies_spacing_after_every_glyph_and_keeps_trailing_line() {
        let mut measured = node(
            "text",
            StyleDef {
                sizing: Some(ViewSizingDef::Fit),
                ..Default::default()
            },
            Vec::new(),
        );
        measured.texts.push(crate::core::view::layout::TextDef {
            id: "label".to_string(),
            content: Some("AB\n".to_string()),
            font: "DTM-Mono".to_string(),
            align: None,
            anchor: None,
            world_scale: (
                crate::core::sequencer::chapter_schema::Value::Static(128.0),
                crate::core::sequencer::chapter_schema::Value::Static(128.0),
            ),
            color: (
                crate::core::sequencer::chapter_schema::Value::Static(1.0),
                crate::core::sequencer::chapter_schema::Value::Static(1.0),
                crate::core::sequencer::chapter_schema::Value::Static(1.0),
                crate::core::sequencer::chapter_schema::Value::Static(1.0),
            ),
            transform: SerializableTransform::default(),
            line_height: Some(1.0),
            char_spacing: Some(16.0),
            word_spacing: Some(0.0),
            text_style: None,
            conditional_style: None,
            visible_when: None,
        });
        let root = node(
            "root",
            StyleDef {
                width: Some(SerializableVal::Px(640.0)),
                height: Some(SerializableVal::Px(480.0)),
                ..Default::default()
            },
            vec![measured],
        );

        let slots = compute_taffy_layout(&asset(root), Vec2::new(640.0, 480.0)).unwrap();

        let measured = slots.get("0:root/0:text").expect("text slot");
        assert_close(measured.width, 160.0);
        assert_close(measured.height, 256.0);
    }

    #[test]
    fn fixed_sizing_sets_dimensions_without_width_height_fields() {
        let root = node(
            "root",
            StyleDef {
                sizing: Some(ViewSizingDef::Fixed {
                    width: SerializableVal::Px(320.0),
                    height: SerializableVal::Px(120.0),
                }),
                ..Default::default()
            },
            Vec::new(),
        );

        let slots = compute_taffy_layout(&asset(root), Vec2::new(640.0, 480.0)).unwrap();

        let root = slots.get("0:root").expect("root slot");
        assert_close(root.width, 320.0);
        assert_close(root.height, 120.0);
    }

    #[test]
    fn fill_sizing_takes_remaining_main_axis_space() {
        let fixed = node(
            "fixed",
            StyleDef {
                sizing: Some(ViewSizingDef::Fixed {
                    width: SerializableVal::Px(100.0),
                    height: SerializableVal::Px(40.0),
                }),
                ..Default::default()
            },
            Vec::new(),
        );
        let fill = node(
            "fill",
            StyleDef {
                sizing: Some(ViewSizingDef::Axes {
                    width: ViewSizeAxisDef::Fill,
                    height: ViewSizeAxisDef::Fixed(SerializableVal::Px(40.0)),
                }),
                ..Default::default()
            },
            Vec::new(),
        );
        let root = node(
            "root",
            StyleDef {
                width: Some(SerializableVal::Px(640.0)),
                height: Some(SerializableVal::Px(480.0)),
                flex_direction: Some(UiFlexDirection::Row),
                gap: Some(StyleGap {
                    row: SerializableVal::Px(0.0),
                    column: SerializableVal::Px(20.0),
                }),
                ..Default::default()
            },
            vec![fixed, fill],
        );

        let slots = compute_taffy_layout(&asset(root), Vec2::new(640.0, 480.0)).unwrap();

        let fill = slots.get("0:root/1:fill").expect("fill slot");
        assert_close(fill.width, 520.0);
    }

    #[test]
    fn spatial_plane_view_asset_parses_and_solves() {
        let layout: ViewLayoutAsset = ron::from_str(include_str!(
            "../../../../examples/assets/view/spatial_plane.view.ron"
        ))
        .expect("spatial plane example should parse");

        let Some(ViewSpaceDef::World3dPlane(plane)) = &layout.space else {
            panic!("spatial plane example should declare World3dPlane space");
        };
        assert!(plane.rotation_degrees.is_some());
        assert!(matches!(
            plane.anchor,
            ViewSpatialAnchorDef::Named(ref name) if name == "SpatialAnchor"
        ));
        assert!(matches!(
            plane.orientation,
            ViewSpatialOrientationDef::Fixed
        ));
        assert!(matches!(plane.input, ViewSpatialInputDef::PlaneRay));

        let slots = compute_taffy_layout(&layout, Vec2::new(360.0, 220.0))
            .expect("spatial plane example should solve layout");

        assert!(slots.get("0:SpatialPanel").is_some());
        assert!(slots.get("0:SpatialPanel/0:SpatialRow").is_some());
        assert!(
            slots
                .get("0:SpatialPanel/0:SpatialRow/0:SpatialRowItemA")
                .is_some()
        );
        assert!(
            slots
                .get("0:SpatialPanel/1:SpatialAbsoluteMarker")
                .is_some()
        );
    }
}
