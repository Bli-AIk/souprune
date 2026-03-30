//! WASM runtime infrastructure for loading and running mod components.
//!
//! Manages the Wasmtime engine, store, and component instances.
//! Provides the bridge between WIT host-api imports and Bevy ECS.
//!
//! WASM 运行时基础设施，用于加载和运行模组组件。
//! 管理 Wasmtime 引擎、Store 和组件实例。
//! 提供 WIT host-api 导入与 Bevy ECS 之间的桥梁。

use bevy::prelude::*;
use souprune_api::Action;
use std::collections::HashMap;

use wasmtime::component::{Component, HasSelf, Linker, ResourceTable};
use wasmtime::{Engine, Store};
use wasmtime_wasi::WasiCtxBuilder;

// Generate host-side bindings from the WIT interface
wasmtime::component::bindgen!({
    path: "wit",
    world: "souprune-mod",
});

/// Per-call context: set by the host before invoking a mod callback.
#[derive(Default)]
pub struct CallContext {
    pub input_pressed: [bool; 7],
    pub input_just_pressed: [bool; 7],
    pub velocity: Vec2,
    pub entity_position: Vec2,
    pub delta_time: f32,
    /// Snapshot of the current FRE fact database (key → string value).
    pub fact_snapshot: HashMap<String, String>,
    /// Fact mutations queued by the mod during a callback; applied afterwards.
    pub pending_fact_mutations: Vec<(String, String)>,
    /// FRE events queued by the mod during a callback; emitted afterwards.
    pub pending_events: Vec<String>,
}

/// Host state stored inside the Wasmtime `Store`.
pub struct HostState {
    pub wasi: wasmtime_wasi::WasiCtx,
    pub table: ResourceTable,
    pub call_ctx: CallContext,
}

impl wasmtime_wasi::WasiView for HostState {
    fn ctx(&mut self) -> wasmtime_wasi::WasiCtxView<'_> {
        wasmtime_wasi::WasiCtxView {
            ctx: &mut self.wasi,
            table: &mut self.table,
        }
    }
}

impl souprune::plugin::host_api::Host for HostState {
    fn log(&mut self, level: u32, message: String) {
        match level {
            0 => info!("[MOD] {}", message),
            1 => warn!("[MOD] {}", message),
            _ => error!("[MOD] {}", message),
        }
    }

    fn is_action_pressed(&mut self, action: souprune::plugin::host_api::Action) -> bool {
        let idx = action_to_index(action);
        self.call_ctx.input_pressed[idx]
    }

    fn is_action_just_pressed(&mut self, action: souprune::plugin::host_api::Action) -> bool {
        let idx = action_to_index(action);
        self.call_ctx.input_just_pressed[idx]
    }

    fn set_velocity(&mut self, velocity: souprune::plugin::host_api::Vec2) {
        self.call_ctx.velocity = Vec2::new(velocity.x, velocity.y);
    }

    fn get_entity_position(&mut self) -> souprune::plugin::host_api::Vec2 {
        souprune::plugin::host_api::Vec2 {
            x: self.call_ctx.entity_position.x,
            y: self.call_ctx.entity_position.y,
        }
    }

    fn delta_time(&mut self) -> f32 {
        self.call_ctx.delta_time
    }

    fn get_fact(&mut self, key: String) -> Option<String> {
        self.call_ctx.fact_snapshot.get(&key).cloned()
    }

    fn set_fact(&mut self, key: String, value: String) {
        self.call_ctx.pending_fact_mutations.push((key, value));
    }

    fn emit_event(&mut self, event_name: String) {
        self.call_ctx.pending_events.push(event_name);
    }
}

fn action_to_index(action: souprune::plugin::host_api::Action) -> usize {
    match action {
        souprune::plugin::host_api::Action::Up => Action::Up as usize,
        souprune::plugin::host_api::Action::Down => Action::Down as usize,
        souprune::plugin::host_api::Action::Left => Action::Left as usize,
        souprune::plugin::host_api::Action::Right => Action::Right as usize,
        souprune::plugin::host_api::Action::Confirm => Action::Confirm as usize,
        souprune::plugin::host_api::Action::Cancel => Action::Cancel as usize,
        souprune::plugin::host_api::Action::Menu => Action::Menu as usize,
    }
}

/// A loaded WASM mod component, ready to instantiate behaviors, danmaku, and patterns.
pub struct LoadedMod {
    pub store: Store<HostState>,
    pub bindings: SoupruneMod,
    pub behavior_ids: Vec<String>,
    pub algorithm_ids: Vec<String>,
    pub pattern_ids: Vec<String>,
    /// Custom FRE action types handled by this mod.
    pub handled_action_ids: Vec<String>,
}

/// WASM runtime — holds the shared engine and linker.
/// Uses NonSend because Linker<HostState> contains !Sync types.
pub struct WasmRuntime {
    engine: Engine,
    linker: Linker<HostState>,
}

impl WasmRuntime {
    pub fn new() -> anyhow::Result<Self> {
        let mut config = wasmtime::Config::new();
        config.wasm_component_model(true);
        let engine = Engine::new(&config)?;

        let mut linker = Linker::new(&engine);
        wasmtime_wasi::p2::add_to_linker_sync(&mut linker)?;
        SoupruneMod::add_to_linker::<_, HasSelf<_>>(&mut linker, |state| state)?;

        Ok(Self { engine, linker })
    }

    /// Load a WASM component from a file path.
    pub fn load_mod(&self, path: &std::path::Path) -> anyhow::Result<LoadedMod> {
        let component = Component::from_file(&self.engine, path)?;

        let wasi = WasiCtxBuilder::new().build();
        let mut store = Store::new(
            &self.engine,
            HostState {
                wasi,
                table: ResourceTable::new(),
                call_ctx: CallContext::default(),
            },
        );

        let bindings = SoupruneMod::instantiate(&mut store, &component, &self.linker)?;

        let behavior_ids = bindings
            .souprune_plugin_behavior()
            .call_list_behaviors(&mut store)?;
        let algorithm_ids = bindings
            .souprune_plugin_danmaku()
            .call_list_algorithms(&mut store)?;
        let pattern_ids = bindings
            .souprune_plugin_spawn_pattern()
            .call_list_patterns(&mut store)?;
        let handled_action_ids = bindings
            .souprune_plugin_custom_action_handler()
            .call_list_handled_actions(&mut store)?;

        Ok(LoadedMod {
            store,
            bindings,
            behavior_ids,
            algorithm_ids,
            pattern_ids,
            handled_action_ids,
        })
    }
}
