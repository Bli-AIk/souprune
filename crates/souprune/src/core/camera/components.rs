//! # components.rs
//!
//! # components.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Defines the `Followable` component, which allows a camera to follow a target entity within optional spatial bounds.
//!
//! 定义 `Followable` 组件，允许相机在可选的空间边界内跟随目标实体。

use bevy::prelude::*;

/// Marker for the game's primary camera.
///
/// 游戏主摄像机标记。编辑器可通过此标记查找并劫持游戏摄像机。
#[derive(Component)]
pub struct MainGameCamera;

/// Marker component to temporarily override normal camera control.
/// When present on a camera entity, the `Followable` system will skip it.
///
/// 临时覆盖正常摄像机控制的标记组件。
/// 当此组件存在于摄像机实体上时，`Followable` 系统将跳过该摄像机。
#[derive(Component)]
pub struct CameraControlOverride;

#[derive(Component, Default)]
pub(crate) struct Followable {
    pub(crate) target: Option<Entity>,
    pub(crate) bounds_enabled: bool,
    pub(crate) min_x: f32,
    pub(crate) max_x: f32,
    pub(crate) min_y: f32,
    pub(crate) max_y: f32,
}

impl Followable {
    /// Create a new `Followable` with bounds disabled.
    ///
    /// 创建一个新的不限制边界的 Followable。
    #[allow(dead_code)]
    pub fn new(target: Option<Entity>) -> Self {
        Self {
            target,
            bounds_enabled: false,
            min_x: 0.0,
            max_x: 0.0,
            min_y: 0.0,
            max_y: 0.0,
        }
    }

    /// Create a new `Followable` with bounds enabled.
    ///
    /// 创建一个新的限制边界的 Followable。
    #[allow(dead_code)]
    pub fn new_with_bounds(
        target: Option<Entity>,
        min_x: f32,
        max_x: f32,
        min_y: f32,
        max_y: f32,
    ) -> Self {
        Self {
            target,
            bounds_enabled: true,
            min_x,
            max_x,
            min_y,
            max_y,
        }
    }

    /// Enable bounds restriction.
    ///
    /// 启用边界限制。
    pub fn enable_bounds(&mut self, min_x: f32, max_x: f32, min_y: f32, max_y: f32) {
        self.bounds_enabled = true;
        self.min_x = min_x;
        self.max_x = max_x;
        self.min_y = min_y;
        self.max_y = max_y;
    }

    /// Disable bounds restriction.
    ///
    /// 禁用边界限制。
    #[allow(dead_code)]
    pub fn disable_bounds(&mut self) {
        self.bounds_enabled = false;
    }
}
