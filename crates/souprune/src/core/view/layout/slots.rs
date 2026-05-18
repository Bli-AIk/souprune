//! Taffy layout slot storage.
//!
//! Taffy 布局槽位存储。

use std::collections::HashMap;

use bevy::prelude::Component;

use super::ViewNodeDef;

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

/// Computed layout slots indexed by stable node path.
///
/// 按稳定节点路径索引的计算布局槽位集合。
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ViewLayoutSlots {
    slots: Vec<ViewLayoutSlot>,
    by_path: HashMap<String, usize>,
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
