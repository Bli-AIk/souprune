//! Camera-anchored UI components.
//!
//! 摄像机锚定的 UI 组件。

use bevy::prelude::*;

#[cfg(feature = "debug")]
use bevy::reflect::Reflect;

/// Marks UI entities that should stick to the camera with a constant offset.
///
/// 标记需要根据摄像机位置保持固定偏移的 UI 实体
#[derive(Component, Debug)]
#[cfg_attr(feature = "debug", derive(Reflect))]
pub struct CameraAnchored {
    pub(crate) offset: Vec3,
}

impl CameraAnchored {
    pub(crate) fn new(offset: Vec3) -> Self {
        Self { offset }
    }
}

/// Marks UI entities that should stick to the camera with a dynamic offset evaluated from expressions.
///
/// 标记需要根据从表达式评估的动态偏移量粘附在相机上的 UI 实体。
#[derive(Component, Debug, Clone)]
#[cfg_attr(feature = "debug", derive(Reflect))]
pub(crate) struct CameraAnchoredDynamic {
    pub(crate) x_expression: Option<String>,
    pub(crate) y_expression: Option<String>,
    pub(crate) z_expression: Option<String>,
}

/// Convenience bundle to apply [`CameraAnchored`] with the correct transform in one go.
///
/// 方便的 Bundle，便于一次性添加 [`CameraAnchored`] 与正确的 Transform。
#[derive(Bundle, Debug)]
#[cfg_attr(feature = "debug", derive(Reflect))]
pub(crate) struct CameraAnchoredBundle {
    anchor: CameraAnchored,
    transform: Transform,
}

impl CameraAnchoredBundle {
    pub(crate) fn from_camera_transform(camera_transform: &Transform, offset: Vec3) -> Self {
        Self {
            anchor: CameraAnchored::new(offset),
            transform: Transform::from_translation(camera_transform.translation + offset),
        }
    }
}
