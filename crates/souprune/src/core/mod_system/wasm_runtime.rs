//! WASM runtime infrastructure for loading and running mod components.
//!
//! Manages the Wasmtime engine, store, and component instances.
//! Provides the bridge between WIT host-api imports and Bevy ECS.
//!
//! WASM 运行时基础设施，用于加载和运行模组组件。
//! 管理 Wasmtime 引擎、Store 和组件实例。
//! 提供 WIT host-api 导入与 Bevy ECS 之间的桥梁。

use bevy::prelude::*;
use std::sync::{Arc, Mutex};
use wasmtime::component::{Component, HasSelf, Linker};
use wasmtime::{Engine, Store};

use crate::core::collision::CollisionRegionStore;
use host_state::SharedCollisionRegionStore;

mod context;
mod conversions;
mod effects;
mod host_api;
mod host_state;
#[cfg(test)]
mod tests;

pub use effects::{PendingHostEffect, PendingSideEffects, PendingSoundEffect};
pub use host_state::HostState;

// Generate host-side bindings from the WIT interface.
wasmtime::component::bindgen!({
    path: "wit",
    world: "souprune-mod",
});

/// A loaded WASM mod component, ready to instantiate behaviors, danmaku, and patterns.
pub struct LoadedMod {
    pub store: Store<HostState>,
    pub bindings: SoupruneMod,
    pub behavior_ids: Vec<String>,
    pub algorithm_ids: Vec<String>,
    pub pattern_ids: Vec<String>,
    /// Custom FRE action types handled by this mod.
    pub handled_action_ids: Vec<String>,
    /// Whether this mod exports mode-lifecycle callbacks.
    pub has_mode_lifecycle: bool,
    /// Human-readable name for this mod (used in diagnostics/tracing).
    pub name: String,
}

/// WASM runtime — holds the shared engine and linker.
/// Uses NonSend because Linker<HostState> contains !Sync types.
pub struct WasmRuntime {
    engine: Engine,
    linker: Linker<HostState>,
    collision_regions: SharedCollisionRegionStore,
}

impl WasmRuntime {
    pub fn new() -> anyhow::Result<Self> {
        let mut config = wasmtime::Config::new();
        config.wasm_component_model(true);
        let engine = Engine::new(&config)?;

        let mut linker = Linker::new(&engine);
        wasmtime_wasi::p2::add_to_linker_sync(&mut linker)?;
        SoupruneMod::add_to_linker::<_, HasSelf<_>>(&mut linker, |state| state)?;

        Ok(Self {
            engine,
            linker,
            collision_regions: Arc::new(Mutex::new(CollisionRegionStore::default())),
        })
    }

    /// Load a WASM component from a file path.
    pub fn load_mod(&self, path: &std::path::Path) -> anyhow::Result<LoadedMod> {
        let mod_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();
        let component = Component::from_file(&self.engine, path)?;

        let mut store = Store::new(
            &self.engine,
            HostState::new_for_mod(Arc::clone(&self.collision_regions)),
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

        // Collect rules from the rule-provider interface.
        let provided_rules = bindings
            .souprune_plugin_rule_provider()
            .call_list_rules(&mut store)?;
        let has_rules = !provided_rules.is_empty();
        if has_rules {
            info!(
                "WASM mod '{}' provides {} FRE rule(s)",
                mod_name,
                provided_rules.len()
            );
        }

        Ok(LoadedMod {
            store,
            bindings,
            behavior_ids,
            algorithm_ids,
            pattern_ids,
            handled_action_ids,
            has_mode_lifecycle: true,
            name: mod_name,
        })
    }

    /// Collect FRE rule definitions from a loaded mod.
    pub fn collect_rules(
        loaded: &mut LoadedMod,
    ) -> anyhow::Result<Vec<exports::souprune::plugin::rule_provider::RuleDef>> {
        let rules = loaded
            .bindings
            .souprune_plugin_rule_provider()
            .call_list_rules(&mut loaded.store)?;
        Ok(rules)
    }
}
