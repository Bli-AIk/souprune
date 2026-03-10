//! A standalone test host application that simulates the game engine.
//! Uses Wasmtime to load and run a WASM mod component.
//!
//! 一个独立的测试宿主应用程序，模拟游戏引擎。
//! 使用 Wasmtime 加载和运行 WASM 模组组件。

use wasmtime::component::{Component, HasSelf, Linker, ResourceTable};
use wasmtime::{Engine, Store};
use wasmtime_wasi::WasiCtxBuilder;

wasmtime::component::bindgen!({
    path: "../souprune_api/wit",
    world: "souprune-mod",
});

struct MockHostState {
    wasi: wasmtime_wasi::WasiCtx,
    table: ResourceTable,
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

    fn set_velocity(&mut self, velocity: souprune::plugin::host_api::Vec2) {
        println!("[HOST] Set velocity: ({}, {})", velocity.x, velocity.y);
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

    // Test each behavior
    for id in &behaviors {
        println!("\n--- Testing Behavior '{}' ---", id);

        let instance = behavior_iface
            .behavior_instance()
            .call_constructor(&mut store, id)?;

        behavior_iface
            .behavior_instance()
            .call_on_enter(&mut store, instance)?;

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
            spawn_pos: test_spawn_pos.clone(),
            offset: zero.clone(),
            initial_angle: 0.0,
            initial_radius: 0.0,
            player_pos: test_player_pos.clone(),
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
                spawn_pos: test_spawn_pos.clone(),
                offset: zero.clone(),
                initial_angle: 0.0,
                initial_radius: 0.0,
                player_pos: test_player_pos.clone(),
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

    println!("\n--- End Simulation ---");
    Ok(())
}
