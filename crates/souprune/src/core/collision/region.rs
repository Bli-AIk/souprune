//! # region.rs
//!
//! Host-side collision regions and movement constraints.
//!
//! 宿主侧碰撞区域与移动约束。

use super::{CollisionBoundary, PhysicsCollider};
use bevy::prelude::*;
use std::collections::HashMap;

/// Opaque handle for a host-owned collision region.
///
/// 宿主拥有的碰撞区域的不透明句柄。
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RegionHandle(u64);

impl RegionHandle {
    /// Create a handle from its raw numeric value.
    ///
    /// 从原始数字值创建句柄。
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    /// Return the raw numeric value of this handle.
    ///
    /// 返回此句柄的原始数字值。
    pub const fn raw(self) -> u64 {
        self.0
    }
}

/// Opaque handle for a host-owned movement constraint.
///
/// 宿主拥有的移动约束的不透明句柄。
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConstraintHandle(u64);

impl ConstraintHandle {
    /// Create a handle from its raw numeric value.
    ///
    /// 从原始数字值创建句柄。
    pub const fn from_raw(raw: u64) -> Self {
        Self(raw)
    }

    /// Return the raw numeric value of this handle.
    ///
    /// 返回此句柄的原始数字值。
    pub const fn raw(self) -> u64 {
        self.0
    }
}

/// Host-owned collision region.
///
/// 宿主拥有的碰撞区域。
#[derive(Debug, Clone)]
pub struct CollisionRegion {
    pub boundary: CollisionBoundary,
}

impl CollisionRegion {
    /// Create a collision region from a boundary.
    ///
    /// 从边界创建碰撞区域。
    pub fn new(boundary: CollisionBoundary) -> Self {
        Self { boundary }
    }
}

/// A movement constraint that keeps a collider inside a collision region.
///
/// 将碰撞体保持在碰撞区域内的移动约束。
#[derive(Debug, Clone)]
pub struct MovementConstraint {
    pub region: RegionHandle,
    pub collider: PhysicsCollider,
}

impl MovementConstraint {
    /// Create a movement constraint for a region and collider.
    ///
    /// 为区域和碰撞体创建移动约束。
    pub fn new(region: RegionHandle, collider: PhysicsCollider) -> Self {
        Self { region, collider }
    }
}

/// Host-side registry for collision regions.
///
/// 宿主侧碰撞区域注册表。
#[derive(Debug, Resource)]
pub struct CollisionRegionStore {
    next_handle: u64,
    next_constraint_handle: u64,
    regions: HashMap<RegionHandle, CollisionRegion>,
    constraints: HashMap<ConstraintHandle, MovementConstraint>,
}

impl Default for CollisionRegionStore {
    fn default() -> Self {
        Self {
            next_handle: 1,
            next_constraint_handle: 1,
            regions: HashMap::new(),
            constraints: HashMap::new(),
        }
    }
}

impl CollisionRegionStore {
    /// Store a region and return its opaque handle.
    ///
    /// 保存区域并返回其不透明句柄。
    pub fn create_region(&mut self, region: CollisionRegion) -> RegionHandle {
        let handle = RegionHandle(self.next_handle);
        self.next_handle = self.next_handle.saturating_add(1).max(1);
        self.regions.insert(handle, region);
        handle
    }

    /// Remove a region by handle. Missing handles are ignored.
    ///
    /// 按句柄移除区域。缺失句柄会被忽略。
    pub fn remove_region(&mut self, handle: RegionHandle) {
        self.regions.remove(&handle);
        self.constraints
            .retain(|_, constraint| constraint.region != handle);
    }

    /// Get a region by handle.
    ///
    /// 按句柄获取区域。
    pub fn region(&self, handle: RegionHandle) -> Option<&CollisionRegion> {
        self.regions.get(&handle)
    }

    /// Replace an existing region boundary. Missing handles are ignored.
    ///
    /// 替换已有区域边界。缺失句柄会被忽略。
    pub fn update_region(&mut self, handle: RegionHandle, region: CollisionRegion) {
        if let Some(stored) = self.regions.get_mut(&handle) {
            *stored = region;
        }
    }

    /// Constrain a position using a host-owned movement constraint.
    ///
    /// 使用宿主拥有的移动约束限制位置。
    pub fn constrain_position(
        &self,
        constraint: &MovementConstraint,
        position: Vec2,
    ) -> Option<Vec2> {
        let region = self.region(constraint.region)?;
        Some(
            region
                .boundary
                .constrain_with_collider(position, &constraint.collider),
        )
    }

    /// Store a movement constraint and return its opaque handle.
    ///
    /// 保存移动约束并返回其不透明句柄。
    pub fn create_movement_constraint(
        &mut self,
        region: RegionHandle,
        collider: PhysicsCollider,
    ) -> Option<ConstraintHandle> {
        self.region(region)?;
        let handle = ConstraintHandle(self.next_constraint_handle);
        self.next_constraint_handle = self.next_constraint_handle.saturating_add(1).max(1);
        self.constraints
            .insert(handle, MovementConstraint::new(region, collider));
        Some(handle)
    }

    /// Remove a movement constraint by handle. Missing handles are ignored.
    ///
    /// 按句柄移除移动约束。缺失句柄会被忽略。
    pub fn remove_movement_constraint(&mut self, handle: ConstraintHandle) {
        self.constraints.remove(&handle);
    }

    /// Get a movement constraint by handle.
    ///
    /// 按句柄获取移动约束。
    pub fn movement_constraint(&self, handle: ConstraintHandle) -> Option<&MovementConstraint> {
        self.constraints.get(&handle)
    }

    /// Constrain a position through a stored movement constraint.
    ///
    /// 通过已保存的移动约束限制位置。
    pub fn constrain_movement(&self, handle: ConstraintHandle, position: Vec2) -> Option<Vec2> {
        let constraint = self.movement_constraint(handle)?;
        self.constrain_position(constraint, position)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::collision::{CollisionBoundary, PhysicsCollider};
    use bevy::prelude::Vec2;

    #[test]
    fn region_store_returns_stable_handles_for_created_regions() {
        let mut store = CollisionRegionStore::default();

        let first = store.create_region(CollisionRegion::new(CollisionBoundary {
            center: Vec2::ZERO,
            half_size: Vec2::new(10.0, 10.0),
        }));
        let second = store.create_region(CollisionRegion::new(CollisionBoundary {
            center: Vec2::new(100.0, 0.0),
            half_size: Vec2::new(20.0, 20.0),
        }));

        assert_ne!(first, second);
        assert_eq!(store.region(first).unwrap().boundary.center, Vec2::ZERO);
        assert_eq!(store.region(second).unwrap().boundary.center.x, 100.0);
    }

    #[test]
    fn movement_constraint_clamps_circle_inside_region() {
        let mut store = CollisionRegionStore::default();
        let handle = store.create_region(CollisionRegion::new(CollisionBoundary {
            center: Vec2::ZERO,
            half_size: Vec2::new(50.0, 30.0),
        }));
        let constraint = MovementConstraint::new(handle, PhysicsCollider::Circle { radius: 8.0 });

        let constrained = store
            .constrain_position(&constraint, Vec2::new(70.0, -50.0))
            .unwrap();

        assert_eq!(constrained, Vec2::new(42.0, -22.0));
    }

    #[test]
    fn movement_constraint_clamps_box_inside_region() {
        let mut store = CollisionRegionStore::default();
        let handle = store.create_region(CollisionRegion::new(CollisionBoundary {
            center: Vec2::new(10.0, -10.0),
            half_size: Vec2::new(50.0, 30.0),
        }));
        let constraint = MovementConstraint::new(
            handle,
            PhysicsCollider::Box {
                half_size: Vec2::new(5.0, 12.0),
            },
        );

        let constrained = store
            .constrain_position(&constraint, Vec2::new(-100.0, 100.0))
            .unwrap();

        assert_eq!(constrained, Vec2::new(-35.0, 8.0));
    }

    #[test]
    fn missing_region_returns_none_without_panicking() {
        let store = CollisionRegionStore::default();
        let constraint = MovementConstraint::new(
            RegionHandle::from_raw(404),
            PhysicsCollider::Circle { radius: 4.0 },
        );

        assert_eq!(
            store.constrain_position(&constraint, Vec2::new(1.0, 2.0)),
            None
        );
    }

    #[test]
    fn store_owned_constraint_constrains_position_by_handle() {
        let mut store = CollisionRegionStore::default();
        let region = store.create_region(CollisionRegion::new(CollisionBoundary {
            center: Vec2::ZERO,
            half_size: Vec2::new(20.0, 20.0),
        }));
        let constraint = store
            .create_movement_constraint(region, PhysicsCollider::Circle { radius: 5.0 })
            .unwrap();

        let constrained = store
            .constrain_movement(constraint, Vec2::new(30.0, 0.0))
            .unwrap();

        assert_eq!(constrained, Vec2::new(15.0, 0.0));
    }

    #[test]
    fn update_region_keeps_handle_and_updates_existing_constraints() {
        let mut store = CollisionRegionStore::default();
        let region = store.create_region(CollisionRegion::new(CollisionBoundary {
            center: Vec2::ZERO,
            half_size: Vec2::new(20.0, 20.0),
        }));
        let constraint = store
            .create_movement_constraint(region, PhysicsCollider::Circle { radius: 5.0 })
            .unwrap();

        store.update_region(
            region,
            CollisionRegion::new(CollisionBoundary {
                center: Vec2::new(10.0, 0.0),
                half_size: Vec2::new(40.0, 20.0),
            }),
        );

        let constrained = store
            .constrain_movement(constraint, Vec2::new(100.0, 0.0))
            .unwrap();

        assert_eq!(store.region(region).unwrap().boundary.center.x, 10.0);
        assert_eq!(constrained, Vec2::new(45.0, 0.0));
    }

    #[test]
    fn creating_constraint_for_missing_region_returns_none() {
        let mut store = CollisionRegionStore::default();

        assert_eq!(
            store.create_movement_constraint(
                RegionHandle::from_raw(404),
                PhysicsCollider::Circle { radius: 5.0 },
            ),
            None
        );
    }
}
