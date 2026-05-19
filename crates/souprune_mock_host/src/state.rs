//! In-memory simulation state for the standalone mock WASM host.
//!
//! Provides collision and view-entity storage used by host-api callbacks during
//! manual WASM component smoke tests.
//!
//! 独立 mock WASM 宿主的内存模拟状态。
//! 为手动 WASM 组件冒烟测试期间的 host-api 回调提供碰撞与视图实体存储。

use std::collections::HashMap;
use wasmtime::component::ResourceTable;
use wasmtime_wasi::WasiCtxBuilder;

use crate::souprune::plugin::host_api as wit;

pub(crate) struct MockHostState {
    pub(crate) wasi: wasmtime_wasi::WasiCtx,
    pub(crate) table: ResourceTable,
    pub(crate) collision: MockCollisionState,
    pub(crate) view_entities: MockViewEntityState,
}

#[derive(Default)]
pub(crate) struct MockCollisionState {
    next_region: u64,
    next_constraint: u64,
    regions: HashMap<u64, MockRegion>,
    constraints: HashMap<u64, MockConstraint>,
}

#[derive(Clone, Copy)]
struct MockRegion {
    center: wit::Vec2,
    half_size: wit::Vec2,
}

#[derive(Clone, Copy)]
struct MockConstraint {
    region: u64,
    collider: wit::ColliderShape,
}

#[derive(Default)]
pub(crate) struct MockViewEntityState {
    next_entity: u64,
    view_boxes: HashMap<u64, MockViewBox>,
}

#[derive(Clone, Copy)]
pub(crate) struct MockViewBox {
    pub(crate) center: wit::Vec2,
    pub(crate) size: wit::Vec2,
    pub(crate) border_width: f32,
    pub(crate) visible: bool,
}

impl MockHostState {
    pub(crate) fn new() -> Self {
        Self {
            wasi: WasiCtxBuilder::new().build(),
            table: ResourceTable::new(),
            collision: MockCollisionState::default(),
            view_entities: MockViewEntityState::default(),
        }
    }
}

impl MockCollisionState {
    pub(crate) fn create_region(&mut self, center: wit::Vec2, half_size: wit::Vec2) -> u64 {
        if !is_valid_positive_vec2(half_size) {
            return 0;
        }

        self.next_region = self.next_region.saturating_add(1).max(1);
        let handle = self.next_region;
        self.regions
            .insert(handle, MockRegion { center, half_size });
        handle
    }

    pub(crate) fn remove_region(&mut self, handle: u64) {
        self.regions.remove(&handle);
        self.constraints
            .retain(|_, constraint| constraint.region != handle);
    }

    pub(crate) fn set_region_bounds(
        &mut self,
        handle: u64,
        center: wit::Vec2,
        half_size: wit::Vec2,
    ) {
        if let Some(region) = self.regions.get_mut(&handle)
            && center.x.is_finite()
            && center.y.is_finite()
            && is_valid_positive_vec2(half_size)
        {
            region.center = center;
            region.half_size = half_size;
        }
    }

    pub(crate) fn create_constraint(
        &mut self,
        region: u64,
        collider: wit::ColliderShape,
    ) -> Option<u64> {
        if !self.regions.contains_key(&region) || !is_valid_collider(collider) {
            return None;
        }

        self.next_constraint = self.next_constraint.saturating_add(1).max(1);
        let handle = self.next_constraint;
        self.constraints
            .insert(handle, MockConstraint { region, collider });
        Some(handle)
    }

    pub(crate) fn remove_constraint(&mut self, handle: u64) {
        self.constraints.remove(&handle);
    }

    pub(crate) fn constrain_movement(&self, handle: u64, position: wit::Vec2) -> Option<wit::Vec2> {
        let constraint = self.constraints.get(&handle)?;
        let region = self.regions.get(&constraint.region)?;
        Some(constrain_position(*region, constraint.collider, position))
    }
}

impl MockViewEntityState {
    pub(crate) fn spawn_view_box(
        &mut self,
        center: wit::Vec2,
        size: wit::Vec2,
        border_width: f32,
    ) -> u64 {
        if !center.x.is_finite()
            || !center.y.is_finite()
            || !is_valid_positive_vec2(size)
            || !border_width.is_finite()
            || border_width < 0.0
        {
            return 0;
        }

        self.next_entity = self.next_entity.saturating_add(1).max(1);
        let handle = self.next_entity;
        self.view_boxes.insert(
            handle,
            MockViewBox {
                center,
                size,
                border_width,
                visible: true,
            },
        );
        handle
    }

    pub(crate) fn set_view_box_bounds(&mut self, handle: u64, center: wit::Vec2, size: wit::Vec2) {
        if !center.x.is_finite() || !center.y.is_finite() || !is_valid_positive_vec2(size) {
            return;
        }

        if let Some(view_box) = self.view_boxes.get_mut(&handle) {
            view_box.center = center;
            view_box.size = size;
        }
    }

    pub(crate) fn set_view_box_visible(&mut self, handle: u64, visible: bool) {
        if let Some(view_box) = self.view_boxes.get_mut(&handle) {
            view_box.visible = visible;
        }
    }

    pub(crate) fn remove_entity(&mut self, handle: u64) {
        self.view_boxes.remove(&handle);
    }

    pub(crate) fn view_box(&self, handle: u64) -> Option<MockViewBox> {
        self.view_boxes.get(&handle).copied()
    }
}

fn constrain_position(
    region: MockRegion,
    collider: wit::ColliderShape,
    position: wit::Vec2,
) -> wit::Vec2 {
    let inset = match collider {
        wit::ColliderShape::Circle(radius) => wit::Vec2 {
            x: radius,
            y: radius,
        },
        wit::ColliderShape::Rectangle(half_size) => half_size,
    };

    wit::Vec2 {
        x: position.x.clamp(
            region.center.x - region.half_size.x + inset.x,
            region.center.x + region.half_size.x - inset.x,
        ),
        y: position.y.clamp(
            region.center.y - region.half_size.y + inset.y,
            region.center.y + region.half_size.y - inset.y,
        ),
    }
}

fn is_valid_positive_vec2(value: wit::Vec2) -> bool {
    value.x.is_finite() && value.y.is_finite() && value.x > 0.0 && value.y > 0.0
}

fn is_valid_collider(collider: wit::ColliderShape) -> bool {
    match collider {
        wit::ColliderShape::Circle(radius) => radius.is_finite() && radius > 0.0,
        wit::ColliderShape::Rectangle(half_size) => is_valid_positive_vec2(half_size),
    }
}
