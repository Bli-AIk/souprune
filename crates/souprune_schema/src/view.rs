//! # view.rs
//!
//! View Layout schema types for `.view.ron` and `.sdf.ron` files.
//! Mirrors souprune's view_schema.rs types without Bevy dependency.
//!
//! `.view.ron` 和 `.sdf.ron` 文件的 View Layout Schema 类型。
//! 对应 souprune 的 view_schema.rs 类型，无 Bevy 依赖。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::val::Val;
pub use crate::val::{expression, static_float};

mod material;
mod sdf;

pub use material::*;
pub use sdf::*;

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

/// Coordinate system preset for view layouts.
///
/// 视图布局的坐标系预设。
#[derive(Debug, Deserialize, Serialize, Clone, Copy, Default, PartialEq, Eq)]
pub enum CoordinateSystem {
    #[default]
    Standard,
    YDown,
}

/// Full coordinate-space description for imported View layouts.
///
/// 导入型 View 布局的完整坐标空间描述。
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CoordinateSpaceDef {
    /// Normalized source-canvas position of coordinate `(0, 0)`.
    ///
    /// 源画布中坐标 `(0, 0)` 的归一化位置。
    pub axis_origin: SerializableVec2,
    /// Source coordinate Y-axis direction.
    ///
    /// 源坐标 Y 轴方向。
    pub y_axis: YAxisDirectionDef,
    /// Source positive rotation direction.
    ///
    /// 源坐标正旋转方向。
    pub rotation: RotationDirectionDef,
    /// Source canvas size in source units.
    ///
    /// 源画布尺寸，以源坐标单位表示。
    pub extent: CoordinateExtentDef,
}

/// Source coordinate Y-axis direction.
///
/// 源坐标 Y 轴方向。
#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq, Default)]
pub enum YAxisDirectionDef {
    #[default]
    Up,
    Down,
}

/// Source positive rotation direction.
///
/// 源坐标正旋转方向。
#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq, Default)]
pub enum RotationDirectionDef {
    #[default]
    CounterClockwise,
    Clockwise,
}

/// Source canvas extent.
///
/// 源画布尺寸。
#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq)]
pub enum CoordinateExtentDef {
    Explicit((f32, f32)),
}

/// View root placement space.
///
/// View 根放置空间。
#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum ViewSpaceDef {
    /// Parent the View root to the active 2D camera.
    ///
    /// 将 View 根挂到当前 2D 相机下。
    Camera2dRelative,
    /// Keep the View root in 2D world space.
    ///
    /// 将 View 根保持在 2D 世界空间。
    World2d,
    /// Place the 2D layout result on a plane in 3D world space.
    ///
    /// 将二维布局结果放置到 3D 世界空间平面上。
    World3dPlane(Box<ViewWorld3dPlaneDef>),
}

/// Anchor strategy for a spatial View.
///
/// 空间 View 的锚点策略。
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, Default)]
pub enum ViewSpatialAnchorDef {
    /// Anchor the View plane in world space.
    ///
    /// 将 View 平面锚定在世界空间。
    #[default]
    World,
    /// Anchor the View plane to a named spatial anchor.
    ///
    /// 将 View 平面锚定到具名空间锚点。
    Named(String),
}

/// Orientation strategy for a spatial View.
///
/// 空间 View 的朝向策略。
#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq, Default)]
pub enum ViewSpatialOrientationDef {
    /// Keep the authored plane orientation fixed.
    ///
    /// 保持作者配置的平面朝向固定。
    #[default]
    Fixed,
    /// Rotate the plane to face the active camera.
    ///
    /// 旋转平面以朝向当前相机。
    FaceCamera,
    /// Rotate the plane around yaw only to face the active camera.
    ///
    /// 仅绕偏航轴旋转平面以朝向当前相机。
    FaceCameraYaw,
}

/// Depth ordering strategy for a spatial View.
///
/// 空间 View 的深度排序策略。
#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq, Default)]
pub enum ViewSpatialDepthDef {
    /// Use View tree order for draw depth.
    ///
    /// 使用 View 树顺序作为绘制深度。
    #[default]
    TreeOrder,
    /// Use the layout Z value for draw depth.
    ///
    /// 使用布局 Z 值作为绘制深度。
    LayoutZ,
    /// Use camera distance for draw depth.
    ///
    /// 使用相机距离作为绘制深度。
    DistanceToCamera,
}

/// Input projection strategy for a spatial View.
///
/// 空间 View 的输入投射策略。
#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq, Default)]
pub enum ViewSpatialInputDef {
    /// Disable spatial input for this View plane.
    ///
    /// 禁用此 View 平面的空间输入。
    #[default]
    Disabled,
    /// Project pointer input onto the View plane by raycast.
    ///
    /// 通过射线投射将指针输入投射到 View 平面。
    PlaneRay,
}

/// 3D plane placement data for a View root.
///
/// View 根的 3D 平面放置数据。
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ViewWorld3dPlaneDef {
    /// Plane root transform in world space.
    ///
    /// 平面根在世界空间中的变换。
    #[serde(default)]
    pub transform: SerializableTransform,
    /// Additional plane rotation in XYZ degrees.
    ///
    /// 额外的平面 XYZ 角度旋转。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rotation_degrees: Option<SerializableVec3>,
    /// Plane size in world units.
    ///
    /// 平面尺寸，单位为世界单位。
    pub plane_size: (f32, f32),
    /// Layout pixels represented by one world unit.
    ///
    /// 每个世界单位对应的布局像素数。
    pub pixels_per_unit: f32,
    /// Camera target used to make this spatial View active.
    ///
    /// 用于激活此空间 View 的相机目标。
    pub camera: ViewCameraTargetDef,
    /// Anchor strategy for this spatial View.
    ///
    /// 此空间 View 的锚点策略。
    #[serde(default)]
    pub anchor: ViewSpatialAnchorDef,
    /// Orientation strategy for this spatial View.
    ///
    /// 此空间 View 的朝向策略。
    #[serde(default)]
    pub orientation: ViewSpatialOrientationDef,
    /// Depth ordering strategy for this spatial View.
    ///
    /// 此空间 View 的深度排序策略。
    #[serde(default)]
    pub depth: ViewSpatialDepthDef,
    /// Input projection strategy for this spatial View.
    ///
    /// 此空间 View 的输入投射策略。
    #[serde(default)]
    pub input: ViewSpatialInputDef,
}

/// Camera selection target for a View root.
///
/// View 根的相机选择目标。
#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum ViewCameraTargetDef {
    /// Use the main game camera.
    ///
    /// 使用主游戏相机。
    Main,
    /// Use a named camera target.
    ///
    /// 使用具名相机目标。
    Named(String),
}

// ============================================================================
// ViewLayout (top-level asset, mirrors ViewLayoutAsset)
// ============================================================================

/// View Layout — the top-level schema for `.view.ron` files.
///
/// 视图布局——`.view.ron` 文件的顶层 Schema。
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct ViewLayout {
    /// Root nodes of the View tree.
    ///
    /// View 树的根节点。
    pub roots: Vec<ViewNodeDef>,

    /// External data dependencies (e.g., locale files, interfaces).
    ///
    /// 外部数据依赖（如语言文件、接口）。
    #[serde(default)]
    pub requires: Vec<DataRequirement>,

    /// Initial state of FRE facts.
    ///
    /// FRE facts 的初始状态。
    #[serde(
        default,
        serialize_with = "crate::ordered_map::serialize_optional_ordered_map"
    )]
    pub facts: Option<HashMap<String, InitialFactValue>>,

    /// Explicit View root placement space.
    ///
    /// 显式 View 根放置空间。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub space: Option<ViewSpaceDef>,

    /// Coordinate system used for absolute positioning.
    ///
    /// 用于绝对定位的坐标系。
    #[serde(default)]
    pub coordinate_system: CoordinateSystem,

    /// Coordinate space conversion for imported layouts.
    ///
    /// 导入布局的坐标空间转换。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub coordinate_space: Option<CoordinateSpaceDef>,
}

pub type ViewLayoutAsset = ViewLayout;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum DataRequirement {
    File(String),
    Interface {
        interface: String,
        #[serde(default)]
        expects: Vec<String>,
    },
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum InitialFactValue {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    StringList(Vec<String>),
    IntList(Vec<i64>),
}

// ============================================================================
// ViewNodeDef
// ============================================================================

/// Node definition in the View tree.
///
/// View 树中的节点定义。
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct ViewNodeDef {
    /// Unique name of the node (used for identification and animation).
    ///
    /// 节点的唯一名称（用于标识和动画）。
    pub name: String,
    /// Metadata tags for categorization.
    ///
    /// 用于分类的元数据标签。
    #[serde(default)]
    pub tags: Vec<String>,
    /// Flexbox-based layout and styling.
    ///
    /// 基于 Flexbox 的布局与样式。
    #[serde(default)]
    pub style: StyleDef,
    /// Node transform for containers or explicit object hierarchy nodes.
    ///
    /// 容器节点或显式对象层级节点的变换。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transform: Option<SerializableTransform>,
    /// Focus participation policy for this View node.
    ///
    /// 此 View 节点的焦点参与策略。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub focus_policy: Option<ViewFocusPolicyDef>,
    /// Conditional visibility based on a FRE expression.
    ///
    /// 基于 FRE 表达式的条件可见性。
    #[serde(default)]
    pub visible_when: Option<String>,
    /// Solid background color.
    ///
    /// 纯色背景。
    #[serde(default)]
    pub background_color: Option<SerializableColor>,
    /// Solid border color.
    ///
    /// 纯色边框颜色。
    #[serde(default)]
    pub border_color: Option<SerializableColor>,
    /// Static image content.
    ///
    /// 静态图片内容。
    #[serde(default)]
    pub image: Option<ImageDef>,
    /// Single sprite visual.
    ///
    /// 单个 Sprite 视觉资源。
    #[serde(default)]
    pub sprite: Option<SpriteDef>,
    /// Multi-state sprite configuration.
    ///
    /// 多状态 Sprite 配置。
    #[serde(default)]
    pub state_sprite: Option<StateSpriteConfig>,
    /// Text elements associated with this node.
    ///
    /// 与此节点关联的文本元素。
    #[serde(default)]
    pub texts: Vec<TextDef>,
    /// Game-specific "view box" logic (Undertale/Deltarune style boxes).
    ///
    /// 游戏特定的 "view box" 逻辑（Undertale/Deltarune 风格的边框）。
    #[serde(default)]
    pub view_box: Option<ViewBoxLogicDef>,
    /// Child nodes.
    ///
    /// 子节点。
    #[serde(default)]
    pub children: Vec<ViewNodeDef>,
    /// Dynamic repetition logic (e.g., list rendering).
    ///
    /// 动态重复逻辑（如列表渲染）。
    #[serde(default)]
    pub repeat: Option<RepeatDef>,
}

// ============================================================================
// Child types
// ============================================================================

/// Flexbox-based layout style.
///
/// 基于 Flexbox 的布局样式。
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct StyleDef {
    /// Node width.
    ///
    /// 节点宽度。
    #[serde(default)]
    pub width: Option<SerializableVal>,
    /// Node height.
    ///
    /// 节点高度。
    #[serde(default)]
    pub height: Option<SerializableVal>,
    /// Left offset (for absolute or relative positioning).
    ///
    /// 左侧偏移（用于绝对或相对定位）。
    #[serde(default)]
    pub left: Option<SerializableVal>,
    /// Right offset.
    ///
    /// 右侧偏移。
    #[serde(default)]
    pub right: Option<SerializableVal>,
    /// Top offset.
    ///
    /// 顶部偏移。
    #[serde(default)]
    pub top: Option<SerializableVal>,
    /// Bottom offset.
    ///
    /// 底部偏移。
    #[serde(default)]
    pub bottom: Option<SerializableVal>,
    /// Positioning strategy (Relative or Absolute).
    ///
    /// 定位策略（相对或绝对）。
    #[serde(default)]
    pub position_type: Option<SerializablePositionType>,
    /// Layout direction for children.
    ///
    /// 子节点的布局方向。
    #[serde(default)]
    pub flex_direction: Option<UiFlexDirection>,
    /// Alignment along the main axis.
    ///
    /// 主轴方向的对齐方式。
    #[serde(default)]
    pub justify_content: Option<SerializableJustifyContent>,
    /// Alignment along the cross axis.
    ///
    /// 交叉轴方向的对齐方式。
    #[serde(default)]
    pub align_items: Option<SerializableAlignItems>,
    /// Per-node alignment override inside the parent layout.
    ///
    /// 父布局内的单节点对齐覆盖。
    #[serde(default)]
    pub align_self: Option<SerializableAlignSelf>,
    /// Margin around this node.
    ///
    /// 此节点外侧的边距。
    #[serde(default)]
    pub margin: Option<SerializableRect>,
    /// Padding inside this node.
    ///
    /// 此节点内侧的填充。
    #[serde(default)]
    pub padding: Option<SerializableRect>,
    /// Border size around this node.
    ///
    /// 此节点周围的边框尺寸。
    #[serde(default)]
    pub border: Option<SerializableRect>,
    /// Gap between child nodes.
    ///
    /// 子节点之间的间距。
    #[serde(default)]
    pub gap: Option<StyleGap>,
    /// Layout display mode.
    ///
    /// 布局显示模式。
    #[serde(default)]
    pub display: Option<SerializableDisplay>,
    /// Overflow clipping and scrolling behavior.
    ///
    /// 溢出裁剪和滚动行为。
    #[serde(default)]
    pub overflow: Option<ViewOverflowDef>,
    /// Author-facing sizing shorthand.
    ///
    /// 面向作者的尺寸简写。
    #[serde(default)]
    pub sizing: Option<ViewSizingDef>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct ImageDef {
    pub path: String,
    #[serde(default)]
    pub color: Option<SerializableColor>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct SpriteDef {
    pub visual: Visual,
    #[serde(default)]
    pub initial_state: Option<String>,
    #[serde(default)]
    pub color: Option<SerializableColor>,
    #[serde(default)]
    pub flip_x: bool,
    #[serde(default)]
    pub flip_y: bool,
    #[serde(default)]
    pub transform: Option<SerializableTransform>,
    #[serde(default)]
    pub pivot: Option<SerializableVec2>,
    #[serde(default)]
    pub frame_duration: Option<f32>,
    #[serde(default)]
    pub visible_when: Option<String>,
    #[serde(default)]
    pub material: Option<MaterialDef>,
}

/// Text element definition.
///
/// 文本元素定义。
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TextDef {
    /// Unique identifier for the text element.
    ///
    /// 文本元素的唯一标识符。
    pub id: String,
    /// Initial string content.
    ///
    /// 初始字符串内容。
    #[serde(default)]
    pub content: Option<String>,
    /// Font identifier.
    ///
    /// 字体标识符。
    pub font: ViewFontDef,
    /// Text alignment within its bounding box.
    ///
    /// 文本在其包围框内的对齐方式。
    #[serde(default)]
    pub align: Option<TextAlignDef>,
    /// Anchor point for the text's coordinate system.
    ///
    /// 文本坐标系的锚点。
    #[serde(default)]
    pub anchor: Option<TextAnchorDef>,
    /// Scale factor in world units.
    ///
    /// 世界单位下的缩放比例。
    pub world_scale: SerializableVec2,
    /// Base color.
    ///
    /// 基础颜色。
    pub color: SerializableColor,
    /// Transform components.
    ///
    /// 变换组件。
    pub transform: SerializableTransform,
    /// Line height override.
    ///
    /// 行高覆盖。
    #[serde(default)]
    pub line_height: Option<f32>,
    /// Character spacing offset.
    ///
    /// 字符间距偏移。
    #[serde(default)]
    pub char_spacing: Option<f32>,
    /// Word spacing offset.
    ///
    /// 单词间距偏移。
    #[serde(default)]
    pub word_spacing: Option<f32>,
    /// Text animation style preset applied directly to this text block.
    ///
    /// 直接应用到此文本块的文本动画风格预设。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_style: Option<String>,
    /// Alternative style applied when a condition is met.
    ///
    /// 当满足条件时应用的备选样式。
    #[serde(default)]
    pub conditional_style: Option<ConditionalStyleDef>,
    /// Conditional visibility.
    ///
    /// 条件可见性。
    #[serde(default)]
    pub visible_when: Option<String>,
}

impl Default for TextDef {
    fn default() -> Self {
        Self {
            id: String::new(),
            content: None,
            font: String::new(),
            align: None,
            anchor: None,
            world_scale: vector2(1.0, 1.0),
            color: white(),
            transform: SerializableTransform::default(),
            line_height: None,
            char_spacing: None,
            word_spacing: None,
            text_style: None,
            conditional_style: None,
            visible_when: None,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ConditionalStyleDef {
    pub condition: String,
    pub color: SerializableColor,
}

impl Default for ConditionalStyleDef {
    fn default() -> Self {
        Self {
            condition: String::new(),
            color: white(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ViewBoxLogicDef {
    pub width: f32,
    pub height: f32,
    #[serde(default)]
    pub border_width: f32,
    pub offset: SerializableVec3,
    #[serde(default)]
    pub fill_shader: Option<String>,
    #[serde(default)]
    pub structure_file: Option<String>,
    #[serde(default)]
    pub fill_color: Option<SerializableColor>,
}

impl Default for ViewBoxLogicDef {
    fn default() -> Self {
        Self {
            width: 0.0,
            height: 0.0,
            border_width: 0.0,
            offset: vector3(0.0, 0.0, 0.0),
            fill_shader: None,
            structure_file: None,
            fill_color: None,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct RepeatDef {
    pub source: String,
    #[serde(default)]
    pub limit: Option<usize>,
    #[serde(default)]
    pub index_var: Option<String>,
    #[serde(default)]
    pub item_var: Option<String>,
}

// ============================================================================
// State Sprite Configuration
// ============================================================================

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct StateSpriteConfig {
    pub default: String,
    #[serde(default, serialize_with = "crate::ordered_map::serialize_ordered_map")]
    pub variants: HashMap<String, String>,
    #[serde(default)]
    pub rules: Vec<StateRuleDef>,
    #[serde(default)]
    pub transform: Option<SerializableTransform>,
    #[serde(default)]
    pub visible_when: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StateRuleDef {
    pub trigger: StateTriggerDef,
    pub state: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum StateTriggerDef {
    InteractiveLayerSelected { layer_id: String, index: usize },
}

// ============================================================================
// View Visibility Rule
// ============================================================================

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ViewVisibilityRuleDef {
    pub rule_type: String,
    #[serde(default)]
    pub layers: Option<Vec<String>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_view_layout_with_dynamic_values_and_text_spacing() {
        let ron = r#"(
            roots: [
                (
                    name: "HudRoot",
                    focus_policy: Some(Focusable),
                    style: (
                        overflow: Some(Axes(horizontal: Hidden, vertical: Scroll)),
                    ),
                    background_color: Some((0.1, 0.2, 0.3, 1.0)),
                    texts: [(
                        id: "label",
                        content: Some("HP"),
                        font: "hud",
                        align: Some(Center),
                        anchor: Some(TopLeft),
                        world_scale: (1.0, "$ui_scale"),
                        color: (1.0, 1.0, 1.0, 1.0),
                        transform: (
                            translation: Some((8.0, "4.0 + @i", 2.0)),
                            scale: Some((1.0, 1.0, 1.0)),
                        ),
                        line_height: Some(12.0),
                        char_spacing: Some(1.5),
                        word_spacing: Some(3.0),
                        text_style: Some("battle_narration"),
                    )],
                ),
            ],
            facts: Some({
                "enemy_names": ["Mush", "Soup"],
            }),
        )"#;

        let layout: ViewLayoutAsset = ron::from_str(ron).expect("view layout should parse");
        let root = &layout.roots[0];
        let text = &layout.roots[0].texts[0];

        assert!(matches!(
            root.focus_policy,
            Some(ViewFocusPolicyDef::Focusable)
        ));
        assert!(matches!(
            root.style.overflow,
            Some(ViewOverflowDef::Axes {
                horizontal: ViewOverflowAxisDef::Hidden,
                vertical: ViewOverflowAxisDef::Scroll,
            })
        ));
        assert_eq!(text.char_spacing, Some(1.5));
        assert_eq!(text.word_spacing, Some(3.0));
        assert_eq!(text.text_style.as_deref(), Some("battle_narration"));
        assert!(matches!(text.align, Some(TextAlignDef::Center)));
        assert!(matches!(text.anchor, Some(TextAnchorDef::TopLeft)));
        assert!(matches!(text.world_scale.1, Val::Expr(ref expr) if expr == "$ui_scale"));
        assert!(matches!(
            text.transform.translation.as_ref().expect("translation").1,
            Val::Expr(ref expr) if expr == "4.0 + @i"
        ));
    }

    #[test]
    fn parses_sdf_structure_with_custom_color_source() {
        let ron = r#"(
            layer_count: 2,
            root: (
                name: "frame",
                sdf_type: Outer,
                color_source: Custom((1.0, 0.5, 0.25, "$alpha")),
                children: [(
                    name: "fill",
                    sdf_type: Inner,
                    is_filler: true,
                )],
            ),
        )"#;

        let sdf: SdfStructureAsset = ron::from_str(ron).expect("sdf structure should parse");

        match &sdf.root.color_source {
            SdfColorSource::Custom(color) => {
                assert!(matches!(color.0, Val::Static(v) if (v - 1.0).abs() < f32::EPSILON));
                assert!(matches!(color.3, Val::Expr(ref expr) if expr == "$alpha"));
            }
            other => panic!("unexpected color source parsed: {other:?}"),
        }
    }
}
