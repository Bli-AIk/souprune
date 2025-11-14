use bevy::prelude::*;

/// 2D rectangular collider component with size and offset.
///
/// 2D矩形碰撞体组件，包含尺寸和偏移量。
#[derive(Component, Debug, Clone)]
pub struct Rect2DCollider {
    /// Size of the collider rectangle.
    /// 碰撞体矩形的尺寸。
    pub size: Vec2,

    /// Offset from the entity's transform position.
    /// 相对于实体变换位置的偏移量。
    pub offset: Vec2,
}

impl Rect2DCollider {
    /// Create a new rectangular collider with the specified size and offset.
    ///
    /// 创建具有指定尺寸和偏移量的新矩形碰撞体。
    pub fn new(size: Vec2, offset: Vec2) -> Self {
        Self { size, offset }
    }

    /// Create a new rectangular collider with the specified size and no offset.
    ///
    /// 创建具有指定尺寸且无偏移量的新矩形碰撞体。
    pub fn with_size(size: Vec2) -> Self {
        Self::new(size, Vec2::ZERO)
    }
}

/// Debug resource to control collider visibility
/// 控制碰撞体可见性的调试资源
#[derive(Resource)]
pub struct ColliderDebugSettings {
    pub show_colliders: bool,
}

impl Default for ColliderDebugSettings {
    fn default() -> Self {
        Self {
            show_colliders: false,
        }
    }
}
