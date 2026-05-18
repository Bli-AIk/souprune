//! Taffy layout slot storage.
//!
//! Taffy 布局槽位存储。

use std::collections::HashMap;

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

/// Computed layout slots indexed by stable node path.
///
/// 按稳定节点路径索引的计算布局槽位集合。
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ViewLayoutSlots {
    slots: Vec<ViewLayoutSlot>,
    by_path: HashMap<String, usize>,
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

    /// Return a slot by its stable path.
    ///
    /// 根据稳定路径返回槽位。
    pub fn get(&self, path: &str) -> Option<&ViewLayoutSlot> {
        self.by_path.get(path).and_then(|idx| self.slots.get(*idx))
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

fn layout_path_segment(index: usize, node: &ViewNodeDef, fallback_name: &str) -> String {
    let name = if node.name.is_empty() {
        fallback_name
    } else {
        node.name.as_str()
    };
    format!("{index}:{name}")
}
