//! WIT host-api implementation for the standalone mock host.
//!
//! Prints each host callback and forwards stateful collision/view operations to
//! the in-memory simulation state.
//!
//! 独立 mock 宿主的 WIT host-api 实现。
//! 打印每个宿主回调，并把有状态的碰撞与视图操作转发到内存模拟状态。

use crate::souprune::plugin::host_api as wit;
use crate::state::MockHostState;

impl wasmtime_wasi::WasiView for MockHostState {
    fn ctx(&mut self) -> wasmtime_wasi::WasiCtxView<'_> {
        wasmtime_wasi::WasiCtxView {
            ctx: &mut self.wasi,
            table: &mut self.table,
        }
    }
}

impl wit::Host for MockHostState {
    fn log(&mut self, _level: u32, message: String) {
        println!("[HOST] Log: {}", message);
    }

    fn is_action_pressed(&mut self, action: wit::Action) -> bool {
        // Simulate: only Right is pressed
        matches!(action, wit::Action::Right)
    }

    fn is_action_just_pressed(&mut self, _action: wit::Action) -> bool {
        false
    }

    fn set_velocity(&mut self, velocity: wit::Vec2) {
        println!("[HOST] Set velocity: ({}, {})", velocity.x, velocity.y);
    }

    fn create_collision_region(&mut self, center: wit::Vec2, half_size: wit::Vec2) -> u64 {
        let handle = self.collision.create_region(center, half_size);
        println!("[HOST] create_collision_region -> {}", handle);
        handle
    }

    fn remove_collision_region(&mut self, region: u64) {
        println!("[HOST] remove_collision_region: {}", region);
        self.collision.remove_region(region);
    }

    fn set_collision_region_bounds(
        &mut self,
        region: u64,
        center: wit::Vec2,
        half_size: wit::Vec2,
    ) {
        println!(
            "[HOST] set_collision_region_bounds: {} center=({}, {}) half_size=({}, {})",
            region, center.x, center.y, half_size.x, half_size.y
        );
        self.collision.set_region_bounds(region, center, half_size);
    }

    fn create_movement_constraint(
        &mut self,
        region: u64,
        collider: wit::ColliderShape,
    ) -> Option<u64> {
        let handle = self.collision.create_constraint(region, collider);
        println!("[HOST] create_movement_constraint -> {:?}", handle);
        handle
    }

    fn remove_movement_constraint(&mut self, constraint: u64) {
        println!("[HOST] remove_movement_constraint: {}", constraint);
        self.collision.remove_constraint(constraint);
    }

    fn constrain_movement(&mut self, constraint: u64, position: wit::Vec2) -> Option<wit::Vec2> {
        let constrained = self.collision.constrain_movement(constraint, position);
        println!("[HOST] constrain_movement -> {:?}", constrained);
        constrained
    }

    fn spawn_view_box(&mut self, center: wit::Vec2, size: wit::Vec2, border_width: f32) -> u64 {
        let handle = self
            .view_entities
            .spawn_view_box(center, size, border_width);
        if let Some(view_box) = self.view_entities.view_box(handle) {
            println!(
                "[HOST] spawn_view_box -> {} center=({}, {}) size=({}, {}) border={} visible={}",
                handle,
                view_box.center.x,
                view_box.center.y,
                view_box.size.x,
                view_box.size.y,
                view_box.border_width,
                view_box.visible
            );
        } else {
            println!("[HOST] spawn_view_box -> 0");
        }
        handle
    }

    fn set_view_box_bounds(&mut self, handle: u64, center: wit::Vec2, size: wit::Vec2) {
        self.view_entities.set_view_box_bounds(handle, center, size);
        if let Some(view_box) = self.view_entities.view_box(handle) {
            println!(
                "[HOST] set_view_box_bounds: {} center=({}, {}) size=({}, {}) border={} visible={}",
                handle,
                view_box.center.x,
                view_box.center.y,
                view_box.size.x,
                view_box.size.y,
                view_box.border_width,
                view_box.visible
            );
        } else {
            println!(
                "[HOST] set_view_box_bounds ignored unknown handle={}",
                handle
            );
        }
    }

    fn tween_view_box_bounds(
        &mut self,
        handle: u64,
        center: wit::Vec2,
        size: wit::Vec2,
        duration_secs: f32,
    ) {
        self.view_entities.set_view_box_bounds(handle, center, size);
        if let Some(view_box) = self.view_entities.view_box(handle) {
            println!(
                "[HOST] tween_view_box_bounds: {} center=({}, {}) size=({}, {}) border={} visible={} duration={}",
                handle,
                view_box.center.x,
                view_box.center.y,
                view_box.size.x,
                view_box.size.y,
                view_box.border_width,
                view_box.visible,
                duration_secs
            );
        } else {
            println!(
                "[HOST] tween_view_box_bounds ignored unknown handle={}",
                handle
            );
        }
    }

    fn set_view_box_visible(&mut self, handle: u64, visible: bool) {
        self.view_entities.set_view_box_visible(handle, visible);
        if let Some(view_box) = self.view_entities.view_box(handle) {
            println!(
                "[HOST] set_view_box_visible: {} center=({}, {}) size=({}, {}) border={} visible={}",
                handle,
                view_box.center.x,
                view_box.center.y,
                view_box.size.x,
                view_box.size.y,
                view_box.border_width,
                view_box.visible
            );
        } else {
            println!(
                "[HOST] set_view_box_visible ignored unknown handle={}",
                handle
            );
        }
    }

    fn remove_entity(&mut self, handle: u64) {
        println!("[HOST] remove_entity: {}", handle);
        self.view_entities.remove_entity(handle);
    }

    fn get_entity_position(&mut self) -> wit::Vec2 {
        wit::Vec2 { x: 0.0, y: 0.0 }
    }

    fn delta_time(&mut self) -> f32 {
        0.016
    }

    fn get_fact(&mut self, key: String) -> Option<wit::FactValue> {
        match key.as_str() {
            "player:pos_x" => Some(wit::FactValue::FloatVal(100.0)),
            "player:pos_y" => Some(wit::FactValue::FloatVal(-200.0)),
            _ => None,
        }
    }

    fn set_fact(&mut self, key: String, value: wit::FactValue) {
        println!("[HOST] set_fact: {}={:?}", key, value);
    }

    fn emit_event(&mut self, event_name: String) {
        println!("[HOST] emit_event: {}", event_name);
    }

    fn get_entity_position_by_tag(&mut self, tag: String) -> Option<wit::Vec2> {
        println!("[HOST] get_entity_position_by_tag: {}", tag);
        None
    }

    fn spawn_emitter(&mut self, pattern_id: String, position: wit::Vec2) -> u64 {
        println!(
            "[HOST] spawn_emitter: {} at ({}, {})",
            pattern_id, position.x, position.y
        );
        0
    }

    fn despawn_emitter(&mut self, handle: u64) {
        println!("[HOST] despawn_emitter: {}", handle);
    }

    fn open_view(&mut self, view_id: String) {
        println!("[HOST] open_view: {}", view_id);
    }

    fn close_view(&mut self) {
        println!("[HOST] close_view");
    }

    fn play_sound(&mut self, sound_key: String) {
        println!("[HOST] play_sound: {}", sound_key);
    }

    fn get_current_mode(&mut self) -> Option<String> {
        Some("overworld".to_string())
    }

    fn get_current_sub_state(&mut self) -> String {
        "Normal".to_string()
    }
}
