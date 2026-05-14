//! A standalone test host application that simulates the game framework.
//! Uses Wasmtime to load and run a WASM mod component.
//!
//! 一个独立的测试宿主应用程序，模拟游戏框架。
//! 使用 Wasmtime 加载和运行 WASM 模组组件。

use std::collections::HashMap;
use wasmtime::component::{Component, HasSelf, Linker, ResourceTable};
use wasmtime::{Engine, Store};
use wasmtime_wasi::WasiCtxBuilder;

wasmtime::component::bindgen!({
    path: "wit",
    world: "souprune-mod",
});

struct MockHostState {
    wasi: wasmtime_wasi::WasiCtx,
    table: ResourceTable,
    collision: MockCollisionState,
    view_entities: MockViewEntityState,
}

#[derive(Default)]
struct MockCollisionState {
    next_region: u64,
    next_constraint: u64,
    regions: HashMap<u64, MockRegion>,
    constraints: HashMap<u64, MockConstraint>,
}

#[derive(Clone, Copy)]
struct MockRegion {
    center: souprune::plugin::host_api::Vec2,
    half_size: souprune::plugin::host_api::Vec2,
}

#[derive(Clone, Copy)]
struct MockConstraint {
    region: u64,
    collider: souprune::plugin::host_api::ColliderShape,
}

#[derive(Default)]
struct MockViewEntityState {
    next_entity: u64,
    view_boxes: HashMap<u64, MockViewBox>,
}

#[derive(Clone, Copy)]
struct MockViewBox {
    center: souprune::plugin::host_api::Vec2,
    size: souprune::plugin::host_api::Vec2,
    border_width: f32,
    visible: bool,
}

impl MockCollisionState {
    fn create_region(
        &mut self,
        center: souprune::plugin::host_api::Vec2,
        half_size: souprune::plugin::host_api::Vec2,
    ) -> u64 {
        if !is_valid_positive_vec2(half_size) {
            return 0;
        }

        self.next_region = self.next_region.saturating_add(1).max(1);
        let handle = self.next_region;
        self.regions
            .insert(handle, MockRegion { center, half_size });
        handle
    }

    fn remove_region(&mut self, handle: u64) {
        self.regions.remove(&handle);
        self.constraints
            .retain(|_, constraint| constraint.region != handle);
    }

    fn set_region_bounds(
        &mut self,
        handle: u64,
        center: souprune::plugin::host_api::Vec2,
        half_size: souprune::plugin::host_api::Vec2,
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

    fn create_constraint(
        &mut self,
        region: u64,
        collider: souprune::plugin::host_api::ColliderShape,
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

    fn remove_constraint(&mut self, handle: u64) {
        self.constraints.remove(&handle);
    }

    fn constrain_movement(
        &self,
        handle: u64,
        position: souprune::plugin::host_api::Vec2,
    ) -> Option<souprune::plugin::host_api::Vec2> {
        let constraint = self.constraints.get(&handle)?;
        let region = self.regions.get(&constraint.region)?;
        Some(constrain_position(*region, constraint.collider, position))
    }
}

impl MockViewEntityState {
    fn spawn_view_box(
        &mut self,
        center: souprune::plugin::host_api::Vec2,
        size: souprune::plugin::host_api::Vec2,
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

    fn set_view_box_bounds(
        &mut self,
        handle: u64,
        center: souprune::plugin::host_api::Vec2,
        size: souprune::plugin::host_api::Vec2,
    ) {
        if !center.x.is_finite() || !center.y.is_finite() || !is_valid_positive_vec2(size) {
            return;
        }

        if let Some(view_box) = self.view_boxes.get_mut(&handle) {
            view_box.center = center;
            view_box.size = size;
        }
    }

    fn set_view_box_visible(&mut self, handle: u64, visible: bool) {
        if let Some(view_box) = self.view_boxes.get_mut(&handle) {
            view_box.visible = visible;
        }
    }

    fn remove_entity(&mut self, handle: u64) {
        self.view_boxes.remove(&handle);
    }

    fn view_box(&self, handle: u64) -> Option<MockViewBox> {
        self.view_boxes.get(&handle).copied()
    }
}

fn constrain_position(
    region: MockRegion,
    collider: souprune::plugin::host_api::ColliderShape,
    position: souprune::plugin::host_api::Vec2,
) -> souprune::plugin::host_api::Vec2 {
    let inset = match collider {
        souprune::plugin::host_api::ColliderShape::Circle(radius) => {
            souprune::plugin::host_api::Vec2 {
                x: radius,
                y: radius,
            }
        }
        souprune::plugin::host_api::ColliderShape::Rectangle(half_size) => half_size,
    };

    souprune::plugin::host_api::Vec2 {
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

fn is_valid_positive_vec2(value: souprune::plugin::host_api::Vec2) -> bool {
    value.x.is_finite() && value.y.is_finite() && value.x > 0.0 && value.y > 0.0
}

fn is_valid_collider(collider: souprune::plugin::host_api::ColliderShape) -> bool {
    match collider {
        souprune::plugin::host_api::ColliderShape::Circle(radius) => {
            radius.is_finite() && radius > 0.0
        }
        souprune::plugin::host_api::ColliderShape::Rectangle(half_size) => {
            is_valid_positive_vec2(half_size)
        }
    }
}

impl wasmtime_wasi::WasiView for MockHostState {
    fn ctx(&mut self) -> wasmtime_wasi::WasiCtxView<'_> {
        wasmtime_wasi::WasiCtxView {
            ctx: &mut self.wasi,
            table: &mut self.table,
        }
    }
}

impl souprune::plugin::host_api::Host for MockHostState {
    fn log(&mut self, _level: u32, message: String) {
        println!("[HOST] Log: {}", message);
    }

    fn is_action_pressed(&mut self, action: souprune::plugin::host_api::Action) -> bool {
        // Simulate: only Right is pressed
        matches!(action, souprune::plugin::host_api::Action::Right)
    }

    fn is_action_just_pressed(&mut self, _action: souprune::plugin::host_api::Action) -> bool {
        false
    }

    fn set_velocity(&mut self, velocity: souprune::plugin::host_api::Vec2) {
        println!("[HOST] Set velocity: ({}, {})", velocity.x, velocity.y);
    }

    fn create_collision_region(
        &mut self,
        center: souprune::plugin::host_api::Vec2,
        half_size: souprune::plugin::host_api::Vec2,
    ) -> u64 {
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
        center: souprune::plugin::host_api::Vec2,
        half_size: souprune::plugin::host_api::Vec2,
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
        collider: souprune::plugin::host_api::ColliderShape,
    ) -> Option<u64> {
        let handle = self.collision.create_constraint(region, collider);
        println!("[HOST] create_movement_constraint -> {:?}", handle);
        handle
    }

    fn remove_movement_constraint(&mut self, constraint: u64) {
        println!("[HOST] remove_movement_constraint: {}", constraint);
        self.collision.remove_constraint(constraint);
    }

    fn constrain_movement(
        &mut self,
        constraint: u64,
        position: souprune::plugin::host_api::Vec2,
    ) -> Option<souprune::plugin::host_api::Vec2> {
        let constrained = self.collision.constrain_movement(constraint, position);
        println!("[HOST] constrain_movement -> {:?}", constrained);
        constrained
    }

    fn spawn_view_box(
        &mut self,
        center: souprune::plugin::host_api::Vec2,
        size: souprune::plugin::host_api::Vec2,
        border_width: f32,
    ) -> u64 {
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

    fn set_view_box_bounds(
        &mut self,
        handle: u64,
        center: souprune::plugin::host_api::Vec2,
        size: souprune::plugin::host_api::Vec2,
    ) {
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
        center: souprune::plugin::host_api::Vec2,
        size: souprune::plugin::host_api::Vec2,
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

    fn get_entity_position(&mut self) -> souprune::plugin::host_api::Vec2 {
        souprune::plugin::host_api::Vec2 { x: 0.0, y: 0.0 }
    }

    fn delta_time(&mut self) -> f32 {
        0.016
    }

    fn get_fact(&mut self, key: String) -> Option<souprune::plugin::host_api::FactValue> {
        use souprune::plugin::host_api::FactValue;
        match key.as_str() {
            "player:pos_x" => Some(FactValue::FloatVal(100.0)),
            "player:pos_y" => Some(FactValue::FloatVal(-200.0)),
            _ => None,
        }
    }

    fn set_fact(&mut self, key: String, value: souprune::plugin::host_api::FactValue) {
        println!("[HOST] set_fact: {}={:?}", key, value);
    }

    fn emit_event(&mut self, event_name: String) {
        println!("[HOST] emit_event: {}", event_name);
    }

    fn get_entity_position_by_tag(
        &mut self,
        tag: String,
    ) -> Option<souprune::plugin::host_api::Vec2> {
        println!("[HOST] get_entity_position_by_tag: {}", tag);
        None
    }

    fn spawn_emitter(
        &mut self,
        pattern_id: String,
        position: souprune::plugin::host_api::Vec2,
    ) -> u64 {
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

fn main() -> anyhow::Result<()> {
    let mut config = wasmtime::Config::new();
    config.wasm_component_model(true);
    let engine = Engine::new(&config)?;

    let mut linker = Linker::new(&engine);
    wasmtime_wasi::p2::add_to_linker_sync(&mut linker)?;
    SoupruneMod::add_to_linker::<_, HasSelf<_>>(&mut linker, |state| state)?;

    // Load the WASM mod component
    let wasm_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "target/wasm32-wasip2/debug/souprune_mod_test.wasm".to_string());

    println!("Loading WASM mod from: {}", wasm_path);
    let component = Component::from_file(&engine, &wasm_path)?;

    let wasi = WasiCtxBuilder::new().build();
    let mut store = Store::new(
        &engine,
        MockHostState {
            wasi,
            table: ResourceTable::new(),
            collision: MockCollisionState::default(),
            view_entities: MockViewEntityState::default(),
        },
    );

    let bindings = SoupruneMod::instantiate(&mut store, &component, &linker)?;

    // List behaviors
    let behavior_iface = bindings.souprune_plugin_behavior();
    let behaviors = behavior_iface.call_list_behaviors(&mut store)?;
    println!(
        "[HOST] Found {} behaviors: {:?}",
        behaviors.len(),
        behaviors
    );

    // List algorithms
    let danmaku_iface = bindings.souprune_plugin_danmaku();
    let algorithms = danmaku_iface.call_list_algorithms(&mut store)?;
    println!(
        "[HOST] Found {} algorithms: {:?}",
        algorithms.len(),
        algorithms
    );

    // List patterns
    let pattern_iface = bindings.souprune_plugin_spawn_pattern();
    let patterns = pattern_iface.call_list_patterns(&mut store)?;
    println!("[HOST] Found {} patterns: {:?}", patterns.len(), patterns);

    // Test each behavior
    for id in &behaviors {
        println!("\n--- Testing Behavior '{}' ---", id);

        let instance = behavior_iface
            .behavior_instance()
            .call_constructor(&mut store, id)?;

        behavior_iface
            .behavior_instance()
            .call_on_enter(&mut store, instance)?;

        let input_context = exports::souprune::plugin::behavior::InputContextId {
            kind: exports::souprune::plugin::behavior::InputContextKind::Battle,
            custom_name: None,
        };
        let input_command = exports::souprune::plugin::behavior::InputCommand::Confirm;
        behavior_iface.behavior_instance().call_on_input(
            &mut store,
            instance,
            &input_context,
            input_command,
        )?;

        behavior_iface
            .behavior_instance()
            .call_on_update(&mut store, instance, 0.016)?;

        behavior_iface
            .behavior_instance()
            .call_on_exit(&mut store, instance)?;

        // Resource is automatically cleaned up when dropped
        instance.resource_drop(&mut store)?;
    }

    // Test each danmaku algorithm
    for id in &algorithms {
        println!("\n--- Testing Danmaku '{}' ---", id);

        let instance = danmaku_iface
            .danmaku_instance()
            .call_constructor(&mut store, id)?;

        // Use known player position (100, -200) and spawn position (0, 0)
        let test_player_pos = souprune::plugin::host_api::Vec2 {
            x: 100.0,
            y: -200.0,
        };
        let test_spawn_pos = souprune::plugin::host_api::Vec2 { x: 0.0, y: 0.0 };
        let zero = souprune::plugin::host_api::Vec2 { x: 0.0, y: 0.0 };

        let enter_ctx = exports::souprune::plugin::danmaku::BulletContext {
            elapsed: 0.0,
            delta_time: 0.0,
            spawn_pos: test_spawn_pos,
            offset: zero,
            initial_angle: 0.0,
            initial_radius: 0.0,
            player_pos: test_player_pos,
            props: vec![],
        };

        danmaku_iface
            .danmaku_instance()
            .call_on_enter(&mut store, instance, &enter_ctx)?;

        // Simulate 3 update frames
        for frame in 0..3 {
            let elapsed = (frame + 1) as f32 * 0.016;
            let update_ctx = exports::souprune::plugin::danmaku::BulletContext {
                elapsed,
                delta_time: 0.016,
                spawn_pos: test_spawn_pos,
                offset: zero,
                initial_angle: 0.0,
                initial_radius: 0.0,
                player_pos: test_player_pos,
                props: vec![],
            };

            let output = danmaku_iface.danmaku_instance().call_on_update(
                &mut store,
                instance,
                &update_ctx,
            )?;

            println!(
                "  Frame {}: offset=({:.4}, {:.4}), rotation={:.4}",
                frame, output.offset.x, output.offset.y, output.rotation
            );

            // Verify the output makes sense: with player at (100, -200) and spawn at (0,0),
            // offset should move toward the player direction
            if frame == 0 && (output.offset.x.abs() < 0.0001 && output.offset.y.abs() < 0.0001) {
                eprintln!(
                    "  WARNING: Output offset is zero - player_pos may not be reaching the mod!"
                );
            }
        }

        danmaku_iface
            .danmaku_instance()
            .call_on_exit(&mut store, instance)?;
        instance.resource_drop(&mut store)?;
    }

    // Test each spawn pattern
    for id in &patterns {
        println!("\n--- Testing Pattern '{}' ---", id);

        let instance = pattern_iface
            .pattern_instance()
            .call_constructor(&mut store, id)?;

        let test_ctx = exports::souprune::plugin::spawn_pattern::SpawnContext {
            center_x: 0.0,
            center_y: 0.0,
            player_x: 50.0,
            player_y: -100.0,
            time: 0.0,
        };

        let test_params = vec![
            exports::souprune::plugin::spawn_pattern::PatternParam {
                name: "count".to_string(),
                value: 6.0,
            },
            exports::souprune::plugin::spawn_pattern::PatternParam {
                name: "radius".to_string(),
                value: 40.0,
            },
        ];

        let points = pattern_iface.pattern_instance().call_generate(
            &mut store,
            instance,
            test_ctx,
            &test_params,
        )?;

        println!("  Generated {} spawn points:", points.len());
        for (i, p) in points.iter().enumerate() {
            println!(
                "    [{}] pos=({:.2}, {:.2}), angle={:.3}, radius={:.1}",
                i, p.x, p.y, p.angle, p.radius
            );
        }

        instance.resource_drop(&mut store)?;
    }

    println!("\n--- End Simulation ---");

    // Test custom action handler
    let ca_iface = bindings.souprune_plugin_custom_action_handler();
    let handled = ca_iface.call_list_handled_actions(&mut store)?;
    println!("\n[HOST] Custom action types handled: {:?}", handled);

    for action_type in &handled {
        let params = vec![
            exports::souprune::plugin::custom_action_handler::ActionParam {
                name: "message".to_string(),
                value: format!("test call to '{}'", action_type),
            },
        ];
        let result = ca_iface.call_handle_action(&mut store, action_type, &params)?;
        println!("[HOST] handle_action '{}' -> {}", action_type, result);
    }

    // Test mode lifecycle
    let ml_iface = bindings.souprune_plugin_mode_lifecycle();
    println!("\n[HOST] Testing mode-lifecycle...");
    ml_iface.call_on_mode_enter(&mut store, "battle")?;
    ml_iface.call_on_sub_state_change(&mut store, "battle", "Normal", "Attack")?;
    ml_iface.call_on_mode_exit(&mut store, "battle")?;
    println!("[HOST] mode-lifecycle calls completed");

    // Test rule provider
    let rp_iface = bindings.souprune_plugin_rule_provider();
    let rules = rp_iface.call_list_rules(&mut store)?;
    println!("\n[HOST] Rule provider returned {} rule(s):", rules.len());
    for rule in &rules {
        println!(
            "  Rule '{}': trigger='{}', priority={}, conditions={}, actions={}, outputs={}",
            rule.id,
            rule.trigger_event,
            rule.priority,
            rule.conditions.len(),
            rule.actions.len(),
            rule.outputs.len(),
        );
    }

    println!("\n--- Done ---");
    Ok(())
}
