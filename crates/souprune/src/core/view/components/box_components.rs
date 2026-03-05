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
