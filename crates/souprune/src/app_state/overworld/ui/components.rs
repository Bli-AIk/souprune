//! Overworld UI components used by the overworld app state.
//!
//! Components and helpers to track which UI layer is active and the currently selected
//! index within that layer. The fields are kept private and access is provided through
//! read-only getters and controlled setters.
//!
//! 用于 overworld 应用状态的 UI 组件。
//!
//! 这些组件用于跟踪当前激活的 UI 层以及该层内被选择的索引。字段为私有，通过只读访问器
//! 和受控的设置器来访问和修改。

use bevy::prelude::Component;
use std::borrow::Cow;
use std::fmt;

#[cfg(feature = "debug")]
use bevy::reflect::Reflect;

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
#[cfg_attr(feature = "debug", derive(Reflect))]
pub struct UILayer(Cow<'static, str>);

impl UILayer {
    pub const BACKPACK_MENU: UILayer = UILayer::new_static("BackpackMenu");
    pub const BACKPACK_ITEM: UILayer = UILayer::new_static("BackpackItem");
    pub const BACKPACK_STATUS: UILayer = UILayer::new_static("BackpackStatus");

    /// Const constructor for static constants
    ///
    /// Const 构造函数，用于静态常量初始化
    const fn new_static(name: &'static str) -> UILayer {
        UILayer(Cow::Borrowed(name))
    }

    /// Dynamically construct a layer (flexible for mods or expansions)
    ///
    /// 动态构造层（灵活扩展）
    pub fn new(name: impl Into<Cow<'static, str>>) -> UILayer {
        UILayer(name.into())
    }

    /// Get the layer name
    ///
    /// 获取层名称
    pub fn name(&self) -> &str {
        &self.0
    }

    /// Get the total count of predefined UI layers
    ///
    /// 获取预定义 UI 层的总数
    pub const fn total_count() -> usize {
        //TODO: 真正计算总数
        3 // BACKPACK_MENU, BACKPACK_ITEM, BACKPACK_STATUS
    }
}

impl fmt::Display for UILayer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
/// Component that records the UI layer and the current selection index within that layer.
///
/// Access pattern:
/// - Fields are private to enforce read-only access from outside code in the crate.
/// - Use the provided getters to read `layer`, `index` and `max_index`.
/// - Use `set_layer` and `set_index` to change state in a controlled way (clamps and resets index as needed).
///
/// 记录 UI 层以及该层内当前选中项索引的组件。
///
/// 访问约定：
/// - 字段为私有以在 crate 范围内强制读取访问。
/// - 使用提供的 getter 来读取 `layer`、`index` 和 `max_index`。
/// - 使用 `set_layer` 和 `set_index` 以受控方式修改状态（会进行夹住或重置索引）。
#[derive(Component, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "debug", derive(Reflect))]
pub(crate) struct OverworldUI {
    layer: UILayer,
    index: usize,
    max_index: usize,
}

impl OverworldUI {
    /// Create a new `OverworldUI` component for `layer` with the given `max_index`.
    ///
    /// 为指定的 `layer` 创建一个新的 `OverworldUI` 组件，并设置 `max_index`。
    pub(crate) fn new(layer: UILayer, max_index: usize) -> Self {
        Self {
            layer,
            index: 0,
            max_index,
        }
    }

    /// Get the current UI layer.
    ///
    /// 获取当前的 UI 层级。
    pub(crate) fn layer(&self) -> &UILayer {
        &self.layer
    }

    /// Get the current selected index inside the active layer.
    ///
    /// 获取当前在激活层内所选的索引。
    pub(crate) fn index(&self) -> usize {
        self.index
    }

    /// Get the maximum valid index for the active layer. Indexes are clamped to this value.
    ///
    /// 获取当前激活层的最大有效索引。索引会被限制在该值之内。
    pub(crate) fn max_index(&self) -> usize {
        self.max_index
    }

    /// Change the active layer and update `max_index` accordingly.
    /// If the layer changes, the selection `index` is reset to 0. If the current `index`
    /// is greater than the new `max_index`, it will be clamped down.
    ///
    /// 更改激活层并相应地更新 `max_index`。
    /// 若层发生变化，会将选中索引 `index` 重置为 0；若当前 `index` 大于新的 `max_index`，会被夹住。
    pub(crate) fn set_layer(&mut self, layer: UILayer, max_index: usize) {
        if self.layer != layer {
            self.layer = layer;
            self.index = 0;
        }
        self.max_index = max_index;
        if self.index > self.max_index {
            self.index = self.max_index;
        }
    }

    /// Set the selection index within the current layer. The provided index will be
    /// clamped to the range [0, max_index].
    ///
    /// 设置当前层内的选择索引。提供的索引会被夹在 [0, max_index] 范围内。
    pub(crate) fn set_index(&mut self, idx: usize) {
        self.index = idx.min(self.max_index);
    }
}

#[derive(Component, Debug)]
#[cfg_attr(feature = "debug", derive(Reflect))]
pub(crate) struct OverworldUIBox {
    pub(crate) width: f32,
    pub(crate) height: f32,
    pub(crate) border_width: f32,
}

impl OverworldUIBox {
    /// Create a new `OverworldUIBox` component with the given dimensions and border width.
    ///
    /// 创建一个新的 `OverworldUIBox` 组件，指定尺寸和边框宽度。
    pub(crate) fn new(width: f32, height: f32, border_width: f32) -> Self {
        Self {
            width,
            height,
            border_width,
        }
    }

    /// Get the box width.
    ///
    /// 获取框的宽度。
    pub(crate) fn width(&self) -> f32 {
        self.width
    }

    /// Get the box height.
    ///
    /// 获取框的高度。
    pub(crate) fn height(&self) -> f32 {
        self.height
    }

    /// Get the border width.
    ///
    /// 获取边框宽度。
    pub(crate) fn border_width(&self) -> f32 {
        self.border_width
    }

    /// Set the box dimensions.
    ///
    /// 设置框的尺寸。
    pub(crate) fn set_dimensions(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
    }

    /// Set the border width.
    ///
    /// 设置边框宽度。
    pub(crate) fn set_border_width(&mut self, border_width: f32) {
        self.border_width = border_width;
    }
}
