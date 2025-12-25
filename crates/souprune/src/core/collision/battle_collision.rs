//! # battle_collision.rs
//!
//! Battle scene collision system using SDF for box boundaries.
//!
//! Battle 场景碰撞系统，使用 SDF 实现边界碰撞。

use super::systems::sdf_box;
use bevy::prelude::*;

/// Physical collider component for collision detection
///
/// 物理碰撞体组件，用于碰撞检测
#[derive(Component, Debug, Clone)]
pub enum PhysicsCollider {
    Circle { radius: f32 },
    Box { half_size: Vec2 },
}

impl PhysicsCollider {
    /// Calculate SDF distance from a point to this collider
    ///
    /// 计算点到碰撞体的 SDF 距离
    pub fn sdf_distance(&self, point: Vec2, center: Vec2) -> f32 {
        let relative_pos = point - center;
        match self {
            PhysicsCollider::Circle { radius } => relative_pos.length() - radius,
            PhysicsCollider::Box { half_size } => sdf_box(relative_pos, *half_size),
        }
    }
}

/// Trigger collider component for damage detection
///
/// 触发器碰撞体组件，用于伤害检测
#[derive(Component, Debug, Clone)]
pub enum TriggerCollider {
    Circle { radius: f32 },
    Box { half_size: Vec2 },
}

impl TriggerCollider {
    /// Check if a point is inside this trigger
    ///
    /// 检查点是否在触发器内
    pub fn contains_point(&self, point: Vec2, center: Vec2) -> bool {
        let relative_pos = point - center;
        match self {
            TriggerCollider::Circle { radius } => relative_pos.length() <= *radius,
            TriggerCollider::Box { half_size } => {
                relative_pos.x.abs() <= half_size.x && relative_pos.y.abs() <= half_size.y
            }
        }
    }
}

/// Boundary defined by a UI box using SDF
///
/// 使用 SDF 定义的 UI 边界
#[derive(Component, Debug, Clone)]
pub struct BattleBoxBoundary {
    pub half_size: Vec2,
    pub center: Vec2,
}

impl BattleBoxBoundary {
    /// Create boundary from UI box dimensions
    ///
    /// 从 UI 框尺寸创建边界
    pub fn from_ui_box(width: f32, height: f32, center: Vec2) -> Self {
        Self {
            half_size: Vec2::new(width / 2.0, height / 2.0),
            center,
        }
    }

    /// Calculate SDF distance to the boundary (negative inside, positive outside)
    ///
    /// 计算到边界的 SDF 距离（内部为负，外部为正）
    pub fn sdf_distance(&self, point: Vec2) -> f32 {
        let relative_pos = point - self.center;
        sdf_box(relative_pos, self.half_size)
    }

    /// Clamp position to stay inside the boundary
    ///
    /// 限制位置使其保持在边界内
    pub fn clamp_position(&self, position: Vec2) -> Vec2 {
        Vec2::new(
            position.x.clamp(
                self.center.x - self.half_size.x,
                self.center.x + self.half_size.x,
            ),
            position.y.clamp(
                self.center.y - self.half_size.y,
                self.center.y + self.half_size.y,
            ),
        )
    }

    /// Apply collision response to keep entity inside (SDF-based with gradient)
    ///
    /// 应用碰撞响应保持实体在内部（基于 SDF 梯度）
    pub fn constrain_with_collider(&self, position: Vec2, collider: &PhysicsCollider) -> Vec2 {
        let distance = match collider {
            PhysicsCollider::Circle { radius } => self.sdf_distance(position) + radius,
            PhysicsCollider::Box { half_size } => {
                // For box collider, we need to account for its size
                let adjusted_boundary = BattleBoxBoundary {
                    half_size: self.half_size - *half_size,
                    center: self.center,
                };
                adjusted_boundary.sdf_distance(position)
            }
        };

        // If outside or on edge, push inside using gradient
        if distance >= 0.0 {
            let epsilon = 0.01;
            let gradient = self.compute_gradient(position, epsilon);

            if gradient.length_squared() > 0.0001 {
                // Move along negative gradient to get inside
                position - gradient.normalize() * (distance + 0.1)
            } else {
                // Fallback to simple clamping
                self.clamp_position(position)
            }
        } else {
            position
        }
    }

    /// Compute SDF gradient at a position
    ///
    /// 计算位置处的 SDF 梯度
    fn compute_gradient(&self, position: Vec2, epsilon: f32) -> Vec2 {
        let dx = self.sdf_distance(position + Vec2::new(epsilon, 0.0))
            - self.sdf_distance(position - Vec2::new(epsilon, 0.0));
        let dy = self.sdf_distance(position + Vec2::new(0.0, epsilon))
            - self.sdf_distance(position - Vec2::new(0.0, epsilon));

        Vec2::new(dx, dy) / (2.0 * epsilon)
    }
}
