//! Pure Taffy layout solving for view layout assets.
//!
//! 视图布局资源的纯 Taffy 布局求解。

use super::{
    RepeatDef, SerializableAlignItems, SerializableAlignSelf, SerializableDisplay,
    SerializableJustifyContent, SerializablePositionType, SerializableRect, SerializableVal,
    StyleDef, StyleGap, TextDef, UiFlexDirection, ViewLayoutAsset, ViewLayoutDebugMetadata,
    ViewLayoutEdges, ViewLayoutGap, ViewLayoutLengthDebug, ViewLayoutSizingDebug, ViewLayoutSlot,
    ViewLayoutSlots, ViewNodeDef, ViewOverflowAxisDef, ViewOverflowDef, ViewScrollState,
    ViewSizeAxisDef, ViewSizingDef, layout_child_path, layout_repeat_path, layout_root_path,
    serializable_vec2_to_vec2,
};
use bevy::prelude::Vec2;
use std::error::Error;
use std::fmt;
use taffy::prelude::{AvailableSpace, NodeId, Size, TaffyTree};

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

#[derive(Debug)]
struct LayoutNodeRecord {
    node_id: NodeId,
    path: String,
    debug_metadata: ViewLayoutDebugMetadata,
    overflow: Option<ViewOverflowDef>,
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
    let mut nodes = Vec::<LayoutNodeRecord>::new();
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
            0,
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
    for record in nodes {
        let layout = tree
            .layout(record.node_id)
            .map_err(|error| ViewLayoutError::new(format!("failed to read layout: {error}")))?;
        let LayoutNodeRecord {
            path,
            debug_metadata,
            overflow,
            ..
        } = record;
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
        slots.push_with_debug_metadata(
            ViewLayoutSlot {
                path,
                name: debug_metadata.name.clone(),
                x: layout.location.x,
                y: layout.location.y,
                width: layout.size.width,
                height: layout.size.height,
            },
            debug_metadata,
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
    depth: usize,
    parent_path: Option<String>,
    nodes: &mut Vec<LayoutNodeRecord>,
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
                depth,
                parent_path.clone(),
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
        depth,
        parent_path,
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
    depth: usize,
    parent_path: Option<String>,
    nodes: &mut Vec<LayoutNodeRecord>,
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
                depth + 1,
                Some(path.clone()),
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
        let debug_metadata = build_layout_debug_metadata(
            node,
            path.clone(),
            depth,
            parent_path,
            viewport_size,
            parent_flex_direction,
            is_root,
            natural_size,
        );
        nodes.push(LayoutNodeRecord {
            node_id,
            path,
            debug_metadata,
            overflow: node.style.overflow,
        });
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

mod debug;
mod style;

use debug::build_layout_debug_metadata;
use style::{overflow_clips, overflow_scrolls, to_taffy_style};

#[cfg(test)]
mod tests;
