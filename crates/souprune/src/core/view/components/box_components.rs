//! UI Box and Container components.
//!
//! UI 框和容器组件。

use bevy::prelude::*;

#[cfg(feature = "debug")]
use bevy::reflect::Reflect;

use super::text::ViewTextConfig;

#[derive(Component, Debug)]
#[cfg_attr(feature = "debug", derive(Reflect))]
pub struct ViewBox {
    pub(crate) width: f32,
    pub(crate) height: f32,
    pub(crate) border_width: f32,
    pub(crate) texts: Vec<ViewTextConfig>,
    #[cfg_attr(feature = "debug", reflect(ignore))]
    pub(crate) fill_shader: Option<String>,
    #[cfg_attr(feature = "debug", reflect(ignore))]
    pub(crate) structure_file: Option<String>,
    pub(crate) fill_color: Color,
    /// Overall alpha multiplier applied to all SDF layers.
    ///
    /// SDF 层的整体透明度乘数。
    pub(crate) alpha: f32,
}

impl ViewBox {
    /// Create a new `ViewBox` component with the given dimensions and border width.
    ///
    /// 创建一个新的 `ViewBox` 组件，指定尺寸和边框宽度。
    #[allow(dead_code)]
    pub(crate) fn new(width: f32, height: f32, border_width: f32) -> Self {
        Self {
            width,
            height,
            border_width,
            texts: Vec::new(),
            fill_shader: None,
            structure_file: None,
            fill_color: Color::BLACK,
            alpha: 1.0,
        }
    }

    /// Create a new `ViewBox` component with text configurations.
    ///
    /// 创建一个带有文本配置的新 `ViewBox` 组件。
    #[allow(dead_code)]
    pub(crate) fn new_with_texts(
        width: f32,
        height: f32,
        border_width: f32,
        texts: Vec<ViewTextConfig>,
    ) -> Self {
        Self {
            width,
            height,
            border_width,
            texts,
            fill_shader: None,
            structure_file: None,
            fill_color: Color::BLACK,
            alpha: 1.0,
        }
    }

    /// Create a new `ViewBox` component with full configuration.
    ///
    /// 创建一个带有完整配置的新 `ViewBox` 组件。
    pub fn new_full(
        width: f32,
        height: f32,
        border_width: f32,
        texts: Vec<ViewTextConfig>,
        fill_shader: Option<String>,
        structure_file: Option<String>,
        fill_color: Color,
    ) -> Self {
        Self {
            width,
            height,
            border_width,
            texts,
            fill_shader,
            structure_file,
            fill_color,
            alpha: 1.0,
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

    /// Get the custom fill shader path.
    ///
    /// 获取自定义填充着色器路径。
    #[allow(dead_code)]
    pub(crate) fn fill_shader(&self) -> Option<&str> {
        self.fill_shader.as_deref()
    }

    /// Get the structure file path.
    ///
    /// 获取结构文件路径。
    #[allow(dead_code)]
    pub(crate) fn structure_file(&self) -> Option<&str> {
        self.structure_file.as_deref()
    }

    /// Get the fill color.
    ///
    /// 获取填充颜色。
    #[allow(dead_code)]
    pub(crate) fn fill_color(&self) -> Color {
        self.fill_color
    }

    /// Set the box dimensions.
    ///
    /// 设置框的尺寸。
    #[allow(dead_code)]
    pub(crate) fn set_dimensions(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
    }

    /// Set the border width.
    ///
    /// 设置边框宽度。
    #[allow(dead_code)]
    pub(crate) fn set_border_width(&mut self, border_width: f32) {
        self.border_width = border_width;
    }

    /// Get the alpha multiplier.
    ///
    /// 获取透明度乘数。
    pub(crate) fn alpha(&self) -> f32 {
        self.alpha
    }
}

/// Stores anchor configuration for ViewBox entities that need size-aware positioning.
/// Added automatically when a ViewBox has an `anchor` defined in its RON layout.
///
/// The PostUpdate correction system uses this data to adjust the Transform
/// whenever the ViewBox size changes (e.g., during a tween animation).
///
/// 存储 ViewBox 实体的锚点配置，用于尺寸感知定位。
/// 当 ViewBox 在 RON 布局中定义了 `anchor` 时自动添加。
#[derive(Component, Debug, Clone)]
#[cfg_attr(feature = "debug", derive(Reflect))]
pub struct ViewBoxAnchor {
    /// Normalized anchor point (center = (0,0)).
    /// `(0, -1)` = bottom, `(0, 1)` = top.
    pub anchor: (f32, f32),
    /// The base offset from the RON layout (evaluated at spawn time).
    pub base_offset: Vec3,
    /// The original width from the RON layout definition.
    pub base_width: f32,
    /// The original height from the RON layout definition.
    pub base_height: f32,
}

/// Corrects Transform for ViewBoxes with anchors to keep the anchor edge fixed
/// when the box size changes (e.g., during tween animation).
///
/// Runs in PostUpdate after bevy_tween interpolators.
///
/// 修正带锚点的 ViewBox 的 Transform，在尺寸变化时保持锚点边缘不动。
pub(crate) fn correct_anchored_viewbox_positions(
    mut query: Query<(&ViewBox, &ViewBoxAnchor, &mut Transform)>,
) {
    for (view_box, anchor_data, mut transform) in &mut query {
        let delta_w = view_box.width - anchor_data.base_width;
        let delta_h = view_box.height - anchor_data.base_height;
        let desired_x = anchor_data.base_offset.x - delta_w * anchor_data.anchor.0 / 2.0;
        let desired_y = anchor_data.base_offset.y - delta_h * anchor_data.anchor.1 / 2.0;

        if (transform.translation.x - desired_x).abs() > f32::EPSILON
            || (transform.translation.y - desired_y).abs() > f32::EPSILON
        {
            transform.translation.x = desired_x;
            transform.translation.y = desired_y;
        }
    }
}

/// Marker placed on the filler entity that contains UI text and indicator sprites.
///
/// 标记承载 UI 文本与指示器精灵的填充实体。
#[derive(Component)]
pub(crate) struct ViewBoxFiller;

/// Marker component for a UI container node that can hold texts and children
/// without requiring a visual ViewBox (background box).
///
/// 用于标记 UI 容器节点的组件，该节点可以承载文本和子节点，
/// 而无需视觉上的 ViewBox（背景框）。
#[derive(Component, Debug)]
#[cfg_attr(feature = "debug", derive(Reflect))]
pub(crate) struct ViewContainer;
