//! Hosts Souprune's WASM mod runtime and the registries that expose mod-provided gameplay hooks.
//!
//! 承载 Souprune 的 WASM mod 运行时，以及对外暴露 mod 扩展能力的各类注册表。
//!
//! Acts as the integration layer between the framework and user-provided WASM
//! components. It loads built-in and project mods, records which mod provides
//! each behavior/danmaku/pattern ID, and wires the runtime update systems that
//! let the rest of the game call into those mod-defined capabilities.
//!
//! 框架与用户提供的 WASM 组件之间的集成层。它负责加载内置与项目 mod，
//! 记录每个行为、弹幕算法和生成模式 ID 由哪个 mod 提供，并接入运行时更新系统，
//! 让游戏其余部分可以调用这些由 mod 定义的能力。

use bevy::prelude::*;
use std::collections::HashMap;

use super::wasm_runtime::{self, LoadedMod, WasmRuntime};

mod danmaku_runtime;

mod custom_actions;

mod behaviors;

pub use behaviors::{BehaviorContext, BehaviorParams, BehaviorVelocity};
pub use danmaku_runtime::{ActiveDanmaku, ActiveDanmakuStack};

// === Plugin ===

pub struct ModPlugin;

impl Plugin for ModPlugin {
    fn build(&self, app: &mut App) {
        let runtime = match WasmRuntime::new() {
            Ok(rt) => rt,
            Err(e) => {
                error!("WASM runtime unavailable: {e}. Mod support disabled.");
                app.init_resource::<BehaviorRegistry>()
                    .init_resource::<DanmakuRegistry>()
                    .init_resource::<SpawnPatternRegistry>();
                return;
            }
        };

        let schedule = crate::game_schedule(app);
        app.insert_non_send_resource(runtime)
            .insert_non_send_resource(LoadedMods::default())
            .init_resource::<BehaviorRegistry>()
            .init_resource::<DanmakuRegistry>()
            .init_resource::<SpawnPatternRegistry>()
            .add_systems(Startup, load_mods_system)
            .add_systems(
                schedule,
                (
                    behaviors::init_behaviors_system,
                    behaviors::dispatch_behavior_input_system
                        .after(crate::core::input::InputTransactionSet),
                    behaviors::update_behaviors_system,
                )
                    .chain()
                    .in_set(crate::core::battle_runtime::BattleMovementSet),
            )
            .add_systems(
                schedule,
                custom_actions::dispatch_wasm_custom_actions_system
                    .after(crate::core::fre_bridge::dispatch_custom_actions_system),
            )
            .add_systems(schedule, dispatch_mode_lifecycle_system);
    }
}

/// Registry for behavior factories loaded from WASM mods.
#[derive(Resource, Default)]
pub struct BehaviorRegistry {
    /// Behavior ID → which mod provides it.
    behavior_mods: HashMap<String, usize>,
}

/// Registry for danmaku factories loaded from WASM mods.
#[derive(Resource, Default)]
pub struct DanmakuRegistry {
    /// Algorithm ID → which mod provides it.
    algorithm_mods: HashMap<String, usize>,
}

impl DanmakuRegistry {
    pub fn contains(&self, id: &str) -> bool {
        self.algorithm_mods.contains_key(id)
    }

    pub fn algorithm_ids(&self) -> impl Iterator<Item = &String> {
        self.algorithm_mods.keys()
    }
}

/// Registry for spawn pattern factories loaded from WASM mods.
#[derive(Resource, Default)]
pub struct SpawnPatternRegistry {
    /// Pattern ID → which mod provides it.
    pattern_mods: HashMap<String, usize>,
}

impl SpawnPatternRegistry {
    pub fn contains(&self, id: &str) -> bool {
        self.pattern_mods.contains_key(id)
    }

    /// Create a pattern instance and generate spawn points.
    pub fn generate(
        &self,
        id: &str,
        ctx: &souprune_api::SpawnContext,
        params: &[souprune_api::PatternParam],
        loaded_mods: &mut LoadedMods,
    ) -> Option<Vec<souprune_api::SpawnPoint>> {
        let &mod_index = self.pattern_mods.get(id)?;
        let loaded = loaded_mods.mods.get_mut(mod_index)?;

        let pattern_iface = loaded.bindings.souprune_plugin_spawn_pattern();

        let handle = match pattern_iface
            .pattern_instance()
            .call_constructor(&mut loaded.store, id)
        {
            Ok(h) => h,
            Err(e) => {
                error!("Failed to create pattern '{}': {:?}", id, e);
                return None;
            }
        };

        let wit_ctx = wasm_runtime::exports::souprune::plugin::spawn_pattern::SpawnContext {
            center_x: ctx.center_x,
            center_y: ctx.center_y,
            player_x: ctx.player_x,
            player_y: ctx.player_y,
            time: ctx.time,
        };
        let wit_params: Vec<wasm_runtime::exports::souprune::plugin::spawn_pattern::PatternParam> =
            params
                .iter()
                .map(
                    |p| wasm_runtime::exports::souprune::plugin::spawn_pattern::PatternParam {
                        name: p.name.clone(),
                        value: p.value,
                    },
                )
                .collect();

        match pattern_iface.pattern_instance().call_generate(
            &mut loaded.store,
            handle,
            wit_ctx,
            &wit_params,
        ) {
            Ok(points) => Some(
                points
                    .into_iter()
                    .map(|p| souprune_api::SpawnPoint {
                        x: p.x,
                        y: p.y,
                        angle: p.angle,
                        radius: p.radius,
                    })
                    .collect(),
            ),
            Err(e) => {
                error!("Pattern '{}' generate failed: {:?}", id, e);
                None
            }
        }
    }
}

/// Resource holding all loaded WASM mod instances.
#[derive(Default)]
pub struct LoadedMods {
    pub mods: Vec<LoadedMod>,
}

fn register_mod(
    loaded: &LoadedMod,
    mod_index: usize,
    behavior_registry: &mut BehaviorRegistry,
    danmaku_registry: &mut DanmakuRegistry,
    pattern_registry: &mut SpawnPatternRegistry,
) {
    for id in &loaded.behavior_ids {
        info!("Registered Behavior: {}", id);
        behavior_registry
            .behavior_mods
            .insert(id.clone(), mod_index);
    }
    for id in &loaded.algorithm_ids {
        info!("Registered Danmaku Algorithm: {}", id);
        danmaku_registry
            .algorithm_mods
            .insert(id.clone(), mod_index);
    }
    for id in &loaded.pattern_ids {
        info!("Registered Spawn Pattern: {}", id);
        pattern_registry.pattern_mods.insert(id.clone(), mod_index);
    }
}

fn load_builtin_wasm(
    runtime: &WasmRuntime,
    behavior_registry: &mut BehaviorRegistry,
    danmaku_registry: &mut DanmakuRegistry,
    pattern_registry: &mut SpawnPatternRegistry,
    loaded_mods: &mut LoadedMods,
) {
    let builtin_candidates = builtin_wasm_candidates();

    let wasm_path = builtin_candidates.iter().find(|p| p.exists());
    let Some(wasm_path) = wasm_path else {
        warn!(
            "Builtin WASM not found. Searched: {:?}",
            builtin_candidates
                .iter()
                .map(|p| p.display().to_string())
                .collect::<Vec<_>>()
        );
        return;
    };

    info!("Loading builtin WASM: {}", wasm_path.display());

    match runtime.load_mod(wasm_path) {
        Ok(loaded) => {
            let mod_index = loaded_mods.mods.len();
            let b = loaded.behavior_ids.len();
            let d = loaded.algorithm_ids.len();
            let p = loaded.pattern_ids.len();
            register_mod(
                &loaded,
                mod_index,
                behavior_registry,
                danmaku_registry,
                pattern_registry,
            );
            loaded_mods.mods.push(loaded);
            info!(
                "Builtins loaded: {} behaviors, {} danmaku, {} patterns",
                b, d, p
            );
        }
        Err(e) => {
            error!("Failed to load builtin WASM: {:?}", e);
        }
    }
}

fn builtin_wasm_candidates() -> Vec<std::path::PathBuf> {
    // CWD-relative candidates. Keep both packaged and development layouts so
    // running from the repo root does not depend on one specific target dir.
    //
    // CWD 相对候选路径。兼容发行包布局和开发布局，避免仓库根目录运行时
    // 依赖某一个固定的 target 目录结构。
    let mut candidates = vec![
        std::path::PathBuf::from("builtins/souprune_builtins.wasm"),
        std::path::PathBuf::from("assets/builtins/souprune_builtins.wasm"),
        std::path::PathBuf::from("target/builtins/wasm32-wasip2/debug/souprune_builtins.wasm"),
        std::path::PathBuf::from("target/builtins/wasm32-wasip2/release/souprune_builtins.wasm"),
        std::path::PathBuf::from(
            "crates/souprune_builtins/target/wasm32-wasip2/debug/souprune_builtins.wasm",
        ),
        std::path::PathBuf::from(
            "crates/souprune_builtins/target/wasm32-wasip2/release/souprune_builtins.wasm",
        ),
    ];

    #[cfg(target_os = "android")]
    candidates
        .push(crate::config::android_private_base_path().join("builtins/souprune_builtins.wasm"));

    // Exe-relative candidates (packaged distribution)
    if let Ok(exe) = std::env::current_exe()
        && let Some(exe_dir) = exe.parent()
    {
        candidates.push(exe_dir.join("builtins/souprune_builtins.wasm"));
    }

    candidates
}

fn load_mods_system(
    mut behavior_registry: ResMut<BehaviorRegistry>,
    mut danmaku_registry: ResMut<DanmakuRegistry>,
    mut pattern_registry: ResMut<SpawnPatternRegistry>,
    runtime: NonSend<WasmRuntime>,
    mut loaded_mods: NonSendMut<LoadedMods>,
) {
    load_builtin_wasm(
        &runtime,
        &mut behavior_registry,
        &mut danmaku_registry,
        &mut pattern_registry,
        &mut loaded_mods,
    );

    let config = crate::config::load_config();
    let projects_base = crate::config::get_projects_base_path();

    // Load dependency WASMs first (lower priority — main mod can override)
    for dep in &config.resolved_dependencies {
        let dep_path = projects_base.join(&dep.name).join(&dep.wasm);
        if !dep_path.exists() {
            warn!("Dependency '{}' WASM not found: {:?}", dep.name, dep_path);
            continue;
        }

        info!(
            "Loading dependency WASM: {} ({})",
            dep.name,
            dep_path.display()
        );

        match runtime.load_mod(&dep_path) {
            Ok(loaded) => {
                let mod_index = loaded_mods.mods.len();
                let b = loaded.behavior_ids.len();
                let d = loaded.algorithm_ids.len();
                let p = loaded.pattern_ids.len();
                register_mod(
                    &loaded,
                    mod_index,
                    &mut behavior_registry,
                    &mut danmaku_registry,
                    &mut pattern_registry,
                );
                loaded_mods.mods.push(loaded);
                info!(
                    "Dependency '{}' loaded: {} behaviors, {} danmaku, {} patterns",
                    dep.name, b, d, p
                );
            }
            Err(e) => {
                error!("Failed to load dependency '{}' WASM: {:?}", dep.name, e);
            }
        }
    }

    // Load the main mod WASM (highest priority — can override dependencies)
    let mod_name = &config.project.mod_name;
    let base_path = projects_base.join(mod_name);

    let wasm_filename = if config.mod_library.wasm.is_empty() {
        format!("{}.wasm", mod_name)
    } else {
        config.mod_library.wasm.clone()
    };
    let wasm_path = base_path.join(&wasm_filename);

    if !wasm_path.exists() {
        warn!(
            "Mod WASM file not found: {:?}. Checked: {:?}",
            wasm_filename, wasm_path
        );
        return;
    }

    info!("Loading WASM mod: {}", wasm_path.display());

    match runtime.load_mod(&wasm_path) {
        Ok(loaded) => {
            let mod_index = loaded_mods.mods.len();
            let b = loaded.behavior_ids.len();
            let d = loaded.algorithm_ids.len();
            let p = loaded.pattern_ids.len();
            register_mod(
                &loaded,
                mod_index,
                &mut behavior_registry,
                &mut danmaku_registry,
                &mut pattern_registry,
            );
            loaded_mods.mods.push(loaded);
            info!("Mod loaded: {} behaviors, {} danmaku, {} patterns", b, d, p);
        }
        Err(e) => {
            error!("Failed to load WASM mod: {:?}", e);
        }
    }
}

/// Dispatches mode lifecycle events (enter/exit) to all loaded WASM mods.
fn dispatch_mode_lifecycle_system(
    mut mode_events: MessageReader<crate::core::mode::ModeChanged>,
    mut loaded_mods: NonSendMut<LoadedMods>,
    mut wasm_tracer: ResMut<crate::core::trace::WasmCallTracer>,
) {
    let events: Vec<crate::core::mode::ModeChanged> = mode_events.read().cloned().collect();
    if events.is_empty() {
        return;
    }

    for event in &events {
        if let Some(ref from) = event.from {
            dispatch_mode_call(&mut loaded_mods, &mut wasm_tracer, from, false);
        }
        if let Some(ref to) = event.to {
            dispatch_mode_call(&mut loaded_mods, &mut wasm_tracer, to, true);
        }
    }
}

fn dispatch_mode_call(
    loaded_mods: &mut LoadedMods,
    wasm_tracer: &mut crate::core::trace::WasmCallTracer,
    mode: &str,
    is_enter: bool,
) {
    for loaded in &mut loaded_mods.mods {
        if !loaded.has_mode_lifecycle {
            continue;
        }
        let iface = loaded.bindings.souprune_plugin_mode_lifecycle();
        let start = std::time::Instant::now();
        let result = if is_enter {
            iface.call_on_mode_enter(&mut loaded.store, mode)
        } else {
            iface.call_on_mode_exit(&mut loaded.store, mode)
        };
        if let Err(e) = result {
            let call_name = if is_enter {
                "on-mode-enter"
            } else {
                "on-mode-exit"
            };
            error!(
                "WASM mod '{}' {}('{}') failed: {:?}",
                loaded.name, call_name, mode, e
            );
        }
        let elapsed = start.elapsed();
        let method = if is_enter {
            "on_mode_enter"
        } else {
            "on_mode_exit"
        };
        wasm_tracer.record(&loaded.name, "mode-lifecycle", method, elapsed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_candidates_include_packaged_layouts() {
        let candidates = builtin_wasm_candidates();
        assert!(
            candidates
                .iter()
                .any(|p| p == &std::path::PathBuf::from("builtins/souprune_builtins.wasm"))
        );
        assert!(
            candidates
                .iter()
                .any(|p| p == &std::path::PathBuf::from("assets/builtins/souprune_builtins.wasm"))
        );
    }

    #[cfg(target_os = "android")]
    #[test]
    fn android_builtin_candidate_uses_private_storage() {
        assert!(builtin_wasm_candidates().iter().any(|p| {
            p == &crate::config::android_private_base_path().join("builtins/souprune_builtins.wasm")
        }));
    }
}
