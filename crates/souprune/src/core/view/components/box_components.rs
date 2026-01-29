//! UI Box and Container components.
//!
//! UI 框和容器组件。

use bevy::prelude::*;

#[cfg(feature = "debug")]
use bevy::reflect::Reflect;

use super::layer::UILayer;
use super::text::UITextConfig;
use super::visibility::UILayerVisibilityRule;

#[derive(Component, Debug)]
#[cfg_attr(feature = "debug", derive(Reflect))]
pub(crate) struct UIBox {
    pub(crate) width: f32,
    pub(crate) height: f32,
    pub(crate) border_width: f32,
    pub(crate) texts: Vec<UITextConfig>,
    /// Optional custom fill shader path for data-driven shader loading.
    ///
    /// 可选的自定义填充着色器路径，用于数据驱动的着色器加载。
    #[cfg_attr(feature = "debug", reflect(ignore))]
    pub(crate) fill_shader: Option<String>,
    /// Optional path to load a complex SDF shape structure from file.
    /// If None, generates a single SDF shape (default behavior).
    ///
    /// 可选的路径，用于从文件加载复杂的 SDF shape 结构。
    /// 如果为 None，则生成单个 SDF shape（默认行为）。
    #[cfg_attr(feature = "debug", reflect(ignore))]
    pub(crate) structure_file: Option<String>,
    /// Fill color for the shape.
    ///
    /// 形状的填充颜色。
    pub(crate) fill_color: Color,
}

impl UIBox {
    /// Create a new `UIBox` component with the given dimensions and border width.
    ///
    /// 创建一个新的 `UIBox` 组件，指定尺寸和边框宽度。
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

    /// Create a new `UIBox` component with text configurations.
    ///
    /// 创建一个带有文本配置的新 `UIBox` 组件。
    #[allow(dead_code)]
    pub(crate) fn new_with_texts(
        width: f32,
        height: f32,
        border_width: f32,
        texts: Vec<UITextConfig>,
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

    /// Create a new `UIBox` component with full configuration.
    ///
    /// 创建一个带有完整配置的新 `UIBox` 组件。
    pub(crate) fn new_full(
        width: f32,
        height: f32,
        border_width: f32,
        texts: Vec<UITextConfig>,
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

/// Controls which [`UILayer`]s should render a given [`UIBox`].
///
/// 控制指定 [`UIBox`] 在哪些 [`UILayer`] 中可见。
#[derive(Component, Debug)]
#[cfg_attr(feature = "debug", derive(Reflect))]
pub(crate) struct UIBoxVisibility {
    rule: UILayerVisibilityRule,
}

impl UIBoxVisibility {
    pub(crate) fn new(rule: UILayerVisibilityRule) -> Self {
        Self { rule }
    }

    pub(crate) fn rule(&self) -> &UILayerVisibilityRule {
        &self.rule
    }

    pub(crate) fn is_visible_for(&self, layer: &UILayer) -> bool {
        self.rule.is_visible_for(layer)
    }
}

/// Marker placed on the filler entity that contains UI text and cursor sprites.
///
/// 标记承载 UI 文本与光标精灵的填充实体。
#[derive(Component)]
pub(crate) struct UIBoxFiller;

/// Marker component for a UI container node that can hold texts and children
/// without requiring a visual UIBox (background box).
///
/// 用于标记 UI 容器节点的组件，该节点可以承载文本和子节点，
/// 而无需视觉上的 UIBox（背景框）。
#[derive(Component, Debug)]
#[cfg_attr(feature = "debug", derive(Reflect))]
pub(crate) struct UIContainer;

/// Controls which [`UILayer`]s should render a given UI container (without UIBox).
///
/// 控制指定 UI 容器（无 UIBox）在哪些 [`UILayer`] 中可见。
#[derive(Component, Debug)]
#[cfg_attr(feature = "debug", derive(Reflect))]
pub(crate) struct UIContainerVisibility {
    rule: UILayerVisibilityRule,
}

impl UIContainerVisibility {
    pub(crate) fn new(rule: UILayerVisibilityRule) -> Self {
        Self { rule }
    }

    pub(crate) fn rule(&self) -> &UILayerVisibilityRule {
        &self.rule
    }

    pub(crate) fn is_visible_for(&self, layer: &UILayer) -> bool {
        self.rule.is_visible_for(layer)
    }
}
