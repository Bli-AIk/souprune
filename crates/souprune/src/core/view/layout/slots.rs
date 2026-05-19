//! Taffy layout slot storage.
//!
//! Taffy 布局槽位存储。

use std::collections::HashMap;

use bevy::prelude::Component;
#[cfg(feature = "debug")]
use bevy::reflect::Reflect;

use super::{
    SerializableAlignItems, SerializableAlignSelf, SerializableDisplay, SerializableJustifyContent,
    SerializablePositionType, UiFlexDirection, ViewNodeDef, ViewOverflowDef,
};

/// A computed layout rectangle for one stable view node path.
///
/// 一个稳定视图节点路径对应的计算布局矩形。
#[derive(Debug, Clone, PartialEq)]
pub struct ViewLayoutSlot {
    /// Stable path from the asset root to this node.
    ///
    /// 从资源根节点到当前节点的稳定路径。
    pub path: String,
    /// Node name from the view layout asset.
    ///
    /// 视图布局资源中的节点名称。
    pub name: String,
    /// Computed left coordinate in pixels.
    ///
    /// 计算得到的左侧像素坐标。
    pub x: f32,
    /// Computed top coordinate in pixels.
    ///
    /// 计算得到的顶部像素坐标。
    pub y: f32,
    /// Computed width in pixels.
    ///
    /// 计算得到的像素宽度。
    pub width: f32,
    /// Computed height in pixels.
    ///
    /// 计算得到的像素高度。
    pub height: f32,
}

/// Computed layout edge values in pixels.
///
/// 以像素表示的计算布局边缘值。
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "debug", derive(Reflect))]
pub struct ViewLayoutEdges {
    /// Left edge in pixels.
    ///
    /// 左边缘像素值。
    pub left: f32,
    /// Right edge in pixels.
    ///
    /// 右边缘像素值。
    pub right: f32,
    /// Top edge in pixels.
    ///
    /// 上边缘像素值。
    pub top: f32,
    /// Bottom edge in pixels.
    ///
    /// 下边缘像素值。
    pub bottom: f32,
}

impl ViewLayoutEdges {
    /// Create pixel edge values.
    ///
    /// 创建像素边缘值。
    pub const fn new(left: f32, right: f32, top: f32, bottom: f32) -> Self {
        Self {
            left,
            right,
            top,
            bottom,
        }
    }
}

/// Computed layout gap values in pixels.
///
/// 以像素表示的计算布局间距值。
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "debug", derive(Reflect))]
pub struct ViewLayoutGap {
    /// Row-axis gap in pixels.
    ///
    /// 行轴间距像素值。
    pub row: f32,
    /// Column-axis gap in pixels.
    ///
    /// 列轴间距像素值。
    pub column: f32,
}

impl ViewLayoutGap {
    /// Create pixel gap values.
    ///
    /// 创建像素间距值。
    pub const fn new(row: f32, column: f32) -> Self {
        Self { row, column }
    }
}

/// Computed sizing semantics for a View node.
///
/// View 节点的计算尺寸语义。
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "debug", derive(Reflect))]
pub enum ViewLayoutLengthDebug {
    /// Automatic size.
    ///
    /// 自动尺寸。
    Auto,
    /// Fixed pixel size.
    ///
    /// 固定像素尺寸。
    Px(f32),
    /// Percentage size.
    ///
    /// 百分比尺寸。
    Percent(f32),
}

/// Computed size and flex information for a View node.
///
/// View 节点的计算尺寸与 Flex 信息。
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "debug", derive(Reflect))]
pub struct ViewLayoutSizingDebug {
    /// Width sizing semantic.
    ///
    /// 宽度尺寸语义。
    pub width: ViewLayoutLengthDebug,
    /// Height sizing semantic.
    ///
    /// 高度尺寸语义。
    pub height: ViewLayoutLengthDebug,
    /// Flex grow value.
    ///
    /// Flex grow 值。
    pub flex_grow: f32,
    /// Flex shrink value.
    ///
    /// Flex shrink 值。
    pub flex_shrink: f32,
    /// Flex basis semantic.
    ///
    /// Flex basis 语义。
    pub flex_basis: ViewLayoutLengthDebug,
}

/// Computed layout debug metadata for a stable View node path.
///
/// 稳定 View 节点路径对应的计算布局调试元数据。
#[derive(Component, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "debug", derive(Reflect))]
pub struct ViewLayoutDebugMetadata {
    /// Stable path from the asset root to this node.
    ///
    /// 从资源根节点到当前节点的稳定路径。
    pub path: String,
    /// Node name from the view layout asset.
    ///
    /// 视图布局资源中的节点名称。
    pub name: String,
    /// Tree depth in the authored layout.
    ///
    /// 作者布局中的树深度。
    pub depth: usize,
    /// Stable path of the parent node, if present.
    ///
    /// 父节点的稳定路径（如果存在）。
    pub parent_path: Option<String>,
    /// Layout display kind.
    ///
    /// 布局显示类型。
    pub display: SerializableDisplay,
    /// Layout position type.
    ///
    /// 布局定位类型。
    pub position_type: SerializablePositionType,
    /// Flex direction for this node.
    ///
    /// 当前节点的 Flex 方向。
    pub flex_direction: UiFlexDirection,
    /// Justification mode for child distribution.
    ///
    /// 子项分布的对齐模式。
    pub justify_content: Option<SerializableJustifyContent>,
    /// Cross-axis alignment for children.
    ///
    /// 子项的交叉轴对齐。
    pub align_items: Option<SerializableAlignItems>,
    /// Cross-axis alignment override for this node.
    ///
    /// 当前节点的交叉轴对齐覆盖。
    pub align_self: Option<SerializableAlignSelf>,
    /// Computed margin edges in pixels.
    ///
    /// 计算得到的 margin 像素边缘。
    pub margin: ViewLayoutEdges,
    /// Computed padding edges in pixels.
    ///
    /// 计算得到的 padding 像素边缘。
    pub padding: ViewLayoutEdges,
    /// Computed border edges in pixels.
    ///
    /// 计算得到的 border 像素边缘。
    pub border: ViewLayoutEdges,
    /// Computed gap values in pixels.
    ///
    /// 计算得到的 gap 像素值。
    pub gap: ViewLayoutGap,
    /// Authored overflow behavior, if any.
    ///
    /// 作者填写的 overflow 行为（如果有）。
    pub overflow: Option<ViewOverflowDef>,
    /// Computed sizing semantics.
    ///
    /// 计算得到的尺寸语义。
    pub sizing: ViewLayoutSizingDebug,
}

/// Runtime layout rectangle stored on a spawned View element.
///
/// 存储在已生成 View 元素上的运行时布局矩形。
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct ViewLayoutRect {
    /// Computed left coordinate in pixels.
    ///
    /// 计算得到的左侧像素坐标。
    pub x: f32,
    /// Computed top coordinate in pixels.
    ///
    /// 计算得到的顶部像素坐标。
    pub y: f32,
    /// Computed width in pixels.
    ///
    /// 计算得到的像素宽度。
    pub width: f32,
    /// Computed height in pixels.
    ///
    /// 计算得到的像素高度。
    pub height: f32,
}

/// Runtime clipping rectangle for a View node.
///
/// View 节点的运行时裁剪矩形。
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct ViewClipRect {
    /// Clip left coordinate in pixels.
    ///
    /// 裁剪矩形左侧像素坐标。
    pub x: f32,
    /// Clip top coordinate in pixels.
    ///
    /// 裁剪矩形顶部像素坐标。
    pub y: f32,
    /// Clip width in pixels.
    ///
    /// 裁剪矩形像素宽度。
    pub width: f32,
    /// Clip height in pixels.
    ///
    /// 裁剪矩形像素高度。
    pub height: f32,
}

/// Runtime scroll state for a View node.
///
/// View 节点的运行时滚动状态。
#[derive(Component, Debug, Default, Clone, Copy, PartialEq)]
pub struct ViewScrollState {
    /// Horizontal scroll offset in pixels.
    ///
    /// 水平方向滚动偏移像素值。
    pub offset_x: f32,
    /// Vertical scroll offset in pixels.
    ///
    /// 垂直方向滚动偏移像素值。
    pub offset_y: f32,
}

impl ViewClipRect {
    /// Create a clip rectangle from computed bounds.
    ///
    /// 根据计算边界创建裁剪矩形。
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

impl From<&ViewLayoutSlot> for ViewLayoutRect {
    fn from(slot: &ViewLayoutSlot) -> Self {
        Self {
            x: slot.x,
            y: slot.y,
            width: slot.width,
            height: slot.height,
        }
    }
}

impl From<&ViewLayoutSlot> for ViewClipRect {
    fn from(slot: &ViewLayoutSlot) -> Self {
        Self::new(slot.x, slot.y, slot.width, slot.height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn view_layout_slots_expose_debug_metadata() {
        let mut slots = ViewLayoutSlots::new();
        slots.push_with_debug_metadata(
            ViewLayoutSlot {
                path: "0:root".to_string(),
                name: "root".to_string(),
                x: 10.0,
                y: 20.0,
                width: 30.0,
                height: 40.0,
            },
            ViewLayoutDebugMetadata {
                path: "0:root".to_string(),
                name: "root".to_string(),
                depth: 0,
                parent_path: None,
                display: SerializableDisplay::Flex,
                position_type: SerializablePositionType::Relative,
                flex_direction: UiFlexDirection::Row,
                justify_content: None,
                align_items: None,
                align_self: None,
                margin: ViewLayoutEdges::new(1.0, 2.0, 3.0, 4.0),
                padding: ViewLayoutEdges::new(5.0, 6.0, 7.0, 8.0),
                border: ViewLayoutEdges::new(9.0, 10.0, 11.0, 12.0),
                gap: ViewLayoutGap::new(13.0, 14.0),
                overflow: None,
                sizing: ViewLayoutSizingDebug {
                    width: ViewLayoutLengthDebug::Auto,
                    height: ViewLayoutLengthDebug::Auto,
                    flex_grow: 1.0,
                    flex_shrink: 0.0,
                    flex_basis: ViewLayoutLengthDebug::Px(0.0),
                },
            },
            None,
            None,
        );

        let debug = slots.debug_metadata("0:root").expect("debug metadata");
        assert_eq!(debug.depth, 0);
        assert_eq!(debug.parent_path, None);
        assert_eq!(debug.margin.left, 1.0);
    }
}

/// Computed layout slots indexed by stable node path.
///
/// 按稳定节点路径索引的计算布局槽位集合。
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ViewLayoutSlots {
    slots: Vec<ViewLayoutSlot>,
    by_path: HashMap<String, usize>,
    debug_metadata: HashMap<String, ViewLayoutDebugMetadata>,
    clip_rects: HashMap<String, ViewClipRect>,
    scroll_states: HashMap<String, ViewScrollState>,
}

impl ViewLayoutSlots {
    /// Create an empty slot collection.
    ///
    /// 创建空槽位集合。
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a computed slot.
    ///
    /// 插入一个计算后的槽位。
    pub fn push(&mut self, slot: ViewLayoutSlot) {
        self.by_path.insert(slot.path.clone(), self.slots.len());
        self.slots.push(slot);
    }

    /// Insert a computed slot with debug metadata.
    ///
    /// 插入带调试元数据的计算槽位。
    pub fn push_with_debug_metadata(
        &mut self,
        slot: ViewLayoutSlot,
        debug_metadata: ViewLayoutDebugMetadata,
        clip_rect: Option<ViewClipRect>,
        scroll_state: Option<ViewScrollState>,
    ) {
        self.debug_metadata
            .insert(slot.path.clone(), debug_metadata);
        self.push_with_metadata(slot, clip_rect, scroll_state);
    }

    /// Insert a computed slot with overflow metadata.
    ///
    /// 插入带 overflow 元数据的计算槽位。
    pub fn push_with_metadata(
        &mut self,
        slot: ViewLayoutSlot,
        clip_rect: Option<ViewClipRect>,
        scroll_state: Option<ViewScrollState>,
    ) {
        if let Some(clip_rect) = clip_rect {
            self.clip_rects.insert(slot.path.clone(), clip_rect);
        }
        if let Some(scroll_state) = scroll_state {
            self.scroll_states.insert(slot.path.clone(), scroll_state);
        }
        self.push(slot);
    }

    /// Return a slot by its stable path.
    ///
    /// 根据稳定路径返回槽位。
    pub fn get(&self, path: &str) -> Option<&ViewLayoutSlot> {
        self.by_path.get(path).and_then(|idx| self.slots.get(*idx))
    }

    /// Return clip metadata by stable path.
    ///
    /// 根据稳定路径返回裁剪元数据。
    pub fn clip_rect(&self, path: &str) -> Option<&ViewClipRect> {
        self.clip_rects.get(path)
    }

    /// Return debug metadata by stable path.
    ///
    /// 根据稳定路径返回调试元数据。
    pub fn debug_metadata(&self, path: &str) -> Option<&ViewLayoutDebugMetadata> {
        self.debug_metadata.get(path)
    }

    /// Return scroll metadata by stable path.
    ///
    /// 根据稳定路径返回滚动元数据。
    pub fn scroll_state(&self, path: &str) -> Option<&ViewScrollState> {
        self.scroll_states.get(path)
    }

    /// Return all slots in traversal order.
    ///
    /// 按遍历顺序返回所有槽位。
    pub fn iter(&self) -> impl Iterator<Item = &ViewLayoutSlot> {
        self.slots.iter()
    }

    /// Return all debug metadata in traversal order.
    ///
    /// 按遍历顺序返回所有调试元数据。
    pub fn debug_iter(&self) -> impl Iterator<Item = &ViewLayoutDebugMetadata> {
        self.slots
            .iter()
            .filter_map(|slot| self.debug_metadata.get(&slot.path))
    }
}

/// Build a stable root path segment for layout slot indexing.
///
/// 为布局槽位索引构建稳定的根路径片段。
pub fn layout_root_path(index: usize, node: &ViewNodeDef) -> String {
    layout_path_segment(index, node, "root")
}

/// Build a stable child path for layout slot indexing.
///
/// 为布局槽位索引构建稳定的子路径。
pub fn layout_child_path(parent_path: &str, index: usize, node: &ViewNodeDef) -> String {
    format!("{parent_path}/{}", layout_path_segment(index, node, "node"))
}

/// Build a stable path for one repeat instance in layout slot indexing.
///
/// 为布局槽位索引构建一个 repeat 实例的稳定路径。
pub fn layout_repeat_path(node_path: &str, index: usize) -> String {
    format!("{node_path}#{index}")
}

fn layout_path_segment(index: usize, node: &ViewNodeDef, fallback_name: &str) -> String {
    let name = if node.name.is_empty() {
        fallback_name
    } else {
        node.name.as_str()
    };
    format!("{index}:{name}")
}
