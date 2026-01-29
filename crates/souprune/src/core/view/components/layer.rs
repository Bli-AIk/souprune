//! UI Layer and related enums.
//!
//! UI 层以及相关枚举。

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
    pub const BACKPACK_ITEM_CHOOSES: UILayer = UILayer::new_static("BackpackItemOptions");
    pub const BACKPACK_STATUS: UILayer = UILayer::new_static("BackpackStatus");

    /// Defined options for the backpack menu, determining order and count.
    ///
    /// 背包菜单的定义选项，决定顺序和数量。
    pub const BACKPACK_MENU_OPTIONS: &'static [UILayer] =
        &[Self::BACKPACK_ITEM, Self::BACKPACK_STATUS];

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
    #[allow(dead_code)]
    pub fn name(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for UILayer {
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
