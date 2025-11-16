use bevy::prelude::*;

/// 2D rectangular collider component with size and offset.
///
/// 2D矩形碰撞体组件，包含尺寸和偏移量。
#[derive(Component, Debug, Clone)]
pub struct Rect2DCollider {
    /// Size of the collider rectangle.
    ///
    /// 碰撞体矩形的尺寸。
    pub size: Vec2,

    /// Offset from the entity's transform position.
    ///
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
}
