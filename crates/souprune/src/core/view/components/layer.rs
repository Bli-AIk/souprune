//! UI Layer and related enums.
//!
//! UI 层以及相关枚举。

use std::borrow::Cow;
use std::fmt;

#[cfg(feature = "debug")]
use bevy::reflect::Reflect;

/// Represents a UI layer identified by its name.
/// Layers are data-driven and defined in `.view_layout.ron` files.
///
/// 表示由名称标识的 UI 层。
/// 层是数据驱动的，在 `.view_layout.ron` 文件中定义。
#[derive(Clone, Eq, PartialEq, Debug, Hash)]
#[cfg_attr(feature = "debug", derive(Reflect))]
pub struct ViewLayer(Cow<'static, str>);

impl ViewLayer {
    /// Dynamically construct a layer from a name.
    ///
    /// 从名称动态构造层。
    pub fn new(name: impl Into<Cow<'static, str>>) -> ViewLayer {
        ViewLayer(name.into())
    }

    /// Get the layer name.
    ///
    /// 获取层名称。
    pub fn name(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ViewLayer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Options available when selecting an item in the backpack.
///
/// 背包中选中物品时可用的选项。
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum BackpackItemOption {
    Use = 0,
    Info = 1,
    Drop = 2,
}

impl BackpackItemOption {
    /// All available item options in order.
    ///
    /// 按顺序排列的所有可用物品选项。
    pub const ALL: &'static [Self] = &[Self::Use, Self::Info, Self::Drop];

    /// Get the total count of item options.
    ///
    /// 获取物品选项的总数。
    pub const fn count() -> usize {
        Self::ALL.len()
    }
}
