use bevy::prelude::*;

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
    /// Create a new Followable with bounds disabled
    /// 创建一个新的不限制边界的Followable
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

    /// Create a new Followable with bounds enabled
    /// 创建一个新的限制边界的Followable
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

    /// Enable bounds restriction
    /// 启用边界限制
    pub fn enable_bounds(&mut self, min_x: f32, max_x: f32, min_y: f32, max_y: f32) {
        self.bounds_enabled = true;
        self.min_x = min_x;
        self.max_x = max_x;
        self.min_y = min_y;
        self.max_y = max_y;
    }

    /// Disable bounds restriction
    /// 禁用边界限制
    pub fn disable_bounds(&mut self) {
        self.bounds_enabled = false;
    }
}
