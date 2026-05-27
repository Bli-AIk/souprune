//! Serializable helper types for View schema authoring.
//!
//! View schema 编写所用的可序列化辅助类型。

use serde::{Deserialize, Serialize};

use crate::val::Val;

// ============================================================================
// Serializable Helper Types (mirrors serde_types.rs)
// ============================================================================

/// Floating-point value or a dynamic expression string.
///
/// 浮点值或动态表达式字符串。
pub type FloatOrExpr = Val<f32>;

/// Three-dimensional vector with dynamic channel values.
///
/// 使用动态通道值的三维向量。
pub type SerializableVec3 = (Val<f32>, Val<f32>, Val<f32>);

/// Two-dimensional vector with dynamic channel values.
///
/// 使用动态通道值的二维向量。
pub type SerializableVec2 = (Val<f32>, Val<f32>);

/// RGBA color with dynamic channel values (0.0–1.0).
///
/// 使用动态通道值的 RGBA 颜色 (0.0–1.0)。
pub type SerializableColor = (Val<f32>, Val<f32>, Val<f32>, Val<f32>);

/// Alias for `SerializableColor` used for dynamic color properties.
///
/// 用于动态颜色属性的 `SerializableColor` 别名。
pub type DynamicColor = SerializableColor;

/// Two-dimensional vector with static or dynamic floating-point channels.
///
/// 使用静态或动态浮点通道的二维向量。
pub fn vector2(x: impl Into<Val<f32>>, y: impl Into<Val<f32>>) -> SerializableVec2 {
    (x.into(), y.into())
}

/// Two-dimensional vector with explicit value channels.
///
/// 使用显式值通道的二维向量。
pub fn vector2_value(x: impl Into<Val<f32>>, y: impl Into<Val<f32>>) -> SerializableVec2 {
    vector2(x, y)
}

/// Three-dimensional vector with static or dynamic floating-point channels.
///
/// 使用静态或动态浮点通道的三维向量。
pub fn vector3(
    x: impl Into<Val<f32>>,
    y: impl Into<Val<f32>>,
    z: impl Into<Val<f32>>,
) -> SerializableVec3 {
    (x.into(), y.into(), z.into())
}

/// Three-dimensional vector with explicit value channels.
///
/// 使用显式值通道的三维向量。
pub fn vector3_value(
    x: impl Into<Val<f32>>,
    y: impl Into<Val<f32>>,
    z: impl Into<Val<f32>>,
) -> SerializableVec3 {
    vector3(x, y, z)
}

/// RGBA color with static or dynamic floating-point channels.
///
/// 使用静态或动态浮点通道的 RGBA 颜色。
pub fn color(
    red: impl Into<Val<f32>>,
    green: impl Into<Val<f32>>,
    blue: impl Into<Val<f32>>,
    alpha: impl Into<Val<f32>>,
) -> SerializableColor {
    (red.into(), green.into(), blue.into(), alpha.into())
}

/// RGBA color with explicit value channels.
///
/// 使用显式值通道的 RGBA 颜色。
pub fn color_value(
    red: impl Into<Val<f32>>,
    green: impl Into<Val<f32>>,
    blue: impl Into<Val<f32>>,
    alpha: impl Into<Val<f32>>,
) -> SerializableColor {
    color(red, green, blue, alpha)
}

/// Opaque white color.
///
/// 不透明白色。
pub fn white() -> SerializableColor {
    color(1.0, 1.0, 1.0, 1.0)
}

/// Opaque red color.
///
/// 不透明红色。
pub fn red() -> SerializableColor {
    color(1.0, 0.0, 0.0, 1.0)
}

/// Serializable transform components for view entities.
///
/// 视图实体的可序列化变换组件。
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct SerializableTransform {
    /// Translation offset (X, Y, Z).
    ///
    /// 位移偏移 (X, Y, Z)。
    #[serde(default)]
    pub translation: Option<SerializableVec3>,
    /// Rotation angle in degrees or expression.
    ///
    /// 旋转角度（度）或表达式。
    #[serde(default)]
    pub rotation: Option<FloatOrExpr>,
    /// Scale factor (X, Y, Z).
    ///
    /// 缩放因子 (X, Y, Z)。
    #[serde(default)]
    pub scale: Option<SerializableVec3>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq)]
pub enum SerializableVal {
    Auto,
    Px(f32),
    Percent(f32),
    Vw(f32),
    Vh(f32),
}

/// Four-sided layout value rectangle.
///
/// 四边布局值矩形。
#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq)]
pub struct SerializableRect {
    /// Left side value.
    ///
    /// 左侧值。
    pub left: SerializableVal,
    /// Right side value.
    ///
    /// 右侧值。
    pub right: SerializableVal,
    /// Top side value.
    ///
    /// 上侧值。
    pub top: SerializableVal,
    /// Bottom side value.
    ///
    /// 下侧值。
    pub bottom: SerializableVal,
}

/// Two-axis layout gap values.
///
/// 双轴布局间距值。
#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq)]
pub struct StyleGap {
    /// Row-axis gap value.
    ///
    /// 行轴间距值。
    pub row: SerializableVal,
    /// Column-axis gap value.
    ///
    /// 列轴间距值。
    pub column: SerializableVal,
}

/// Per-axis View sizing mode.
///
/// 单轴 View 尺寸模式。
#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq)]
pub enum ViewSizeAxisDef {
    /// Use measured content size.
    ///
    /// 使用内容测量尺寸。
    Fit,
    /// Fill available parent space on the flex axis.
    ///
    /// 在 Flex 轴上填充父级可用空间。
    Fill,
    /// Use a fixed serialized length value.
    ///
    /// 使用固定的序列化长度值。
    Fixed(SerializableVal),
}

/// Author-facing View sizing shorthand.
///
/// 面向作者的 View 尺寸简写。
#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq)]
pub enum ViewSizingDef {
    /// Both axes use measured content size.
    ///
    /// 两个轴都使用内容测量尺寸。
    Fit,
    /// Both axes try to fill available space.
    ///
    /// 两个轴都尝试填充可用空间。
    Fill,
    /// Both axes use fixed values.
    ///
    /// 两个轴都使用固定值。
    Fixed {
        /// Fixed width.
        ///
        /// 固定宽度。
        width: SerializableVal,
        /// Fixed height.
        ///
        /// 固定高度。
        height: SerializableVal,
    },
    /// Configure each axis separately.
    ///
    /// 分别配置每个轴。
    Axes {
        /// Width sizing mode.
        ///
        /// 宽度尺寸模式。
        width: ViewSizeAxisDef,
        /// Height sizing mode.
        ///
        /// 高度尺寸模式。
        height: ViewSizeAxisDef,
    },
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq, Default)]
pub enum UiFlexDirection {
    #[default]
    Row,
    Column,
    RowReverse,
    ColumnReverse,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
pub enum SerializablePositionType {
    Relative,
    Absolute,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
pub enum SerializableJustifyContent {
    Start,
    End,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
pub enum SerializableAlignItems {
    Start,
    End,
    Center,
    Baseline,
    Stretch,
}

/// Per-node alignment override inside the parent layout.
///
/// 父布局内的单节点对齐覆盖。
#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
pub enum SerializableAlignSelf {
    /// Use the parent alignment.
    ///
    /// 使用父级对齐方式。
    Auto,
    /// Align to the start edge.
    ///
    /// 对齐到起始边。
    Start,
    /// Align to the end edge.
    ///
    /// 对齐到结束边。
    End,
    /// Center on the cross axis.
    ///
    /// 在交叉轴居中。
    Center,
    /// Align text baselines.
    ///
    /// 对齐文本基线。
    Baseline,
    /// Stretch to fill the available cross-axis space.
    ///
    /// 拉伸以填充可用交叉轴空间。
    Stretch,
}

/// Per-axis View overflow behavior.
///
/// 单轴 View 溢出行为。
#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
pub enum ViewOverflowAxisDef {
    /// Let content render outside this View.
    ///
    /// 允许内容绘制到此 View 外。
    Visible,
    /// Clip content to this View without scroll state.
    ///
    /// 将内容裁剪到此 View 内且不生成滚动状态。
    Hidden,
    /// Clip content to this View and create scroll state.
    ///
    /// 将内容裁剪到此 View 内并生成滚动状态。
    Scroll,
}

/// Author-facing View overflow shorthand.
///
/// 面向作者的 View 溢出简写。
#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
pub enum ViewOverflowDef {
    /// Both axes are visible.
    ///
    /// 两个轴都可见。
    Visible,
    /// Both axes are clipped without scroll state.
    ///
    /// 两个轴都裁剪且不生成滚动状态。
    Hidden,
    /// Both axes are clipped and scrollable.
    ///
    /// 两个轴都裁剪并可滚动。
    Scroll,
    /// Configure each axis separately.
    ///
    /// 分别配置每个轴。
    Axes {
        /// Horizontal overflow behavior.
        ///
        /// 水平方向溢出行为。
        horizontal: ViewOverflowAxisDef,
        /// Vertical overflow behavior.
        ///
        /// 垂直方向溢出行为。
        vertical: ViewOverflowAxisDef,
    },
}

/// View focus participation policy.
///
/// View 焦点参与策略。
#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
pub enum ViewFocusPolicyDef {
    /// This View explicitly does not participate in focus navigation.
    ///
    /// 此 View 显式不参与焦点导航。
    Disabled,
    /// This View can receive focus.
    ///
    /// 此 View 可以接收焦点。
    Focusable,
    /// This View groups focusable children.
    ///
    /// 此 View 分组可聚焦子节点。
    Scope,
}

/// Layout display mode.
///
/// 布局显示模式。
#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
pub enum SerializableDisplay {
    /// Participate in flex layout.
    ///
    /// 参与 Flex 布局。
    Flex,
    /// Do not display this node.
    ///
    /// 不显示此节点。
    None,
}

/// Font identifier — file name stem of the font file (e.g., "DTM-Mono", "hud").
///
/// 字体标识符 — 字体文件名（不含扩展名，如 "DTM-Mono"、"hud"）。
pub type ViewFontDef = String;

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextAlignDef {
    #[default]
    Left,
    Center,
    Right,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextAnchorDef {
    TopLeft,
    TopCenter,
    TopRight,
    CenterLeft,
    Center,
    CenterRight,
    BottomLeft,
    BottomCenter,
    #[default]
    BottomRight,
}

/// Visual resource path (transparent string wrapper).
///
/// 视觉资源路径（透明字符串包装）。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Visual(pub String);
