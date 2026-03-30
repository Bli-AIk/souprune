//! Hosts Souprune's WASM mod runtime and the registries that expose mod-provided gameplay hooks.
//!
//! 承载 Souprune 的 WASM mod 运行时，以及对外暴露 mod 扩展能力的各类注册表。
//!
//! Acts as the integration layer between the engine and user-provided WASM
//! components. It loads built-in and project mods, records which mod provides
//! each behavior/danmaku/pattern ID, and wires the runtime update systems that
//! let the rest of the game call into those mod-defined capabilities.
//!
//! 引擎与用户提供的 WASM 组件之间的集成层。它负责加载内置与项目 mod，
//! 记录每个行为、弹幕算法和生成模式 ID 由哪个 mod 提供，并接入运行时更新系统，
//! 让游戏其余部分可以调用这些由 mod 定义的能力。

use bevy::prelude::*;
use bevy_fact_rule_event::{FactEvent, FactValue, LayeredFactDatabase};
use souprune_api::Action;
use std::collections::HashMap;

use super::wasm_runtime::{self, LoadedMod, WasmRuntime};
use crate::core::fre_bridge::FreCustomActionEvent;

mod danmaku_runtime;

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
                (init_behaviors_system, update_behaviors_system)
                    .chain()
                    .in_set(crate::core::battle_runtime::BattleMovementSet),
            )
            .add_systems(
                schedule,
                dispatch_wasm_custom_actions_system
                    .after(crate::core::fre_bridge::dispatch_custom_actions_system),
            );
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
    loaded: &wasm_runtime::LoadedMod,
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
    let candidates = [
        std::path::PathBuf::from("builtins/souprune_builtins.wasm"),
        std::path::PathBuf::from("assets/builtins/souprune_builtins.wasm"),
        std::path::PathBuf::from(
            "crates/souprune_builtins/target/wasm32-wasip2/release/souprune_builtins.wasm",
        ),
    ];

    let wasm_path = candidates.iter().find(|p| p.exists());
    let Some(wasm_path) = wasm_path else {
        warn!(
            "Builtin WASM not found. Searched: {:?}",
            candidates
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
    let mod_name = &config.project.mod_name;
    let base_path = crate::config::get_projects_base_path().join(mod_name);

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

// === Runtime Components ===

/// Which game context a WASM behavior is allowed to run in.
/// `update_behaviors_system` skips behaviors whose context does not match the current mode.
///
/// WASM 行为允许运行的游戏上下文。
/// `update_behaviors_system` 会跳过上下文与当前模式不匹配的行为。
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BehaviorContext {
    Battle,
    Overworld,
    #[default]
    Any,
}

impl BehaviorContext {
    fn matches(&self, mode: &crate::core::mode::SequenceMode) -> bool {
        match self {
            BehaviorContext::Any => true,
            BehaviorContext::Battle => mode.is("battle"),
            BehaviorContext::Overworld => mode.is("overworld"),
        }
    }
}

#[derive(Component)]
pub struct BehaviorParams {
    pub behavior_id: String,
    pub context: BehaviorContext,
}

impl BehaviorParams {
    pub fn new(behavior_id: impl Into<String>) -> Self {
        Self {
            behavior_id: behavior_id.into(),
            context: BehaviorContext::Any,
        }
    }

    pub fn with_context(mut self, context: BehaviorContext) -> Self {
        self.context = context;
        self
    }
}

#[derive(Component, Default)]
pub struct BehaviorVelocity(pub Vec2);

/// Active WASM behavior instance, holding a resource handle inside the WASM store.
#[derive(Component)]
pub struct ActiveBehavior {
    mod_index: usize,
    resource_handle: wasmtime::component::ResourceAny,
}

unsafe impl Send for ActiveBehavior {}
unsafe impl Sync for ActiveBehavior {}

fn init_behaviors_system(
    mut commands: Commands,
    query: Query<(Entity, &BehaviorParams), Added<BehaviorParams>>,
    behavior_registry: Res<BehaviorRegistry>,
    mut loaded_mods: NonSendMut<LoadedMods>,
    fact_db: Res<LayeredFactDatabase>,
) {
    if query.is_empty() {
        return;
    }

    let fact_snapshot = build_fact_snapshot(&fact_db);

    for (entity, params) in query.iter() {
        let Some(&mod_index) = behavior_registry.behavior_mods.get(&params.behavior_id) else {
            error!("Behavior ID not found: {}", params.behavior_id);
            continue;
        };

        let Some(loaded) = loaded_mods.mods.get_mut(mod_index) else {
            error!("Mod index {} not loaded", mod_index);
            continue;
        };

        let behavior_iface = loaded.bindings.souprune_plugin_behavior();
        match behavior_iface
            .behavior_instance()
            .call_constructor(&mut loaded.store, &params.behavior_id)
        {
            Ok(handle) => {
                {
                    let ctx = loaded.store.data_mut();
                    ctx.call_ctx.fact_snapshot.clone_from(&fact_snapshot);
                    ctx.call_ctx.pending_fact_mutations.clear();
                    ctx.call_ctx.pending_events.clear();
                }

                if let Err(e) = behavior_iface
                    .behavior_instance()
                    .call_on_enter(&mut loaded.store, handle)
                {
                    error!(
                        "Behavior on_enter failed for {}: {:?}",
                        params.behavior_id, e
                    );
                }

                commands.entity(entity).insert((
                    ActiveBehavior {
                        mod_index,
                        resource_handle: handle,
                    },
                    params.context,
                ));
            }
            Err(e) => {
                error!("Failed to create behavior {}: {:?}", params.behavior_id, e);
            }
        }
    }
}

fn update_behaviors_system(
    mut query: Query<(
        Entity,
        &BehaviorContext,
        &ActiveBehavior,
        &mut BehaviorVelocity,
        &mut Transform,
    )>,
    action_states: Query<
        &leafwing_input_manager::action_state::ActionState<crate::core::input::actions::Action>,
        With<crate::core::battle_runtime::BattleInputManager>,
    >,
    registry: Res<crate::core::input::actions::ActionRegistry>,
    time: Res<Time>,
    mode: Res<crate::core::mode::SequenceMode>,
    mut loaded_mods: NonSendMut<LoadedMods>,
    mut fact_db: ResMut<LayeredFactDatabase>,
    mut fact_writer: MessageWriter<FactEvent>,
) {
    let mut pressed = [false; 7];
    let mut just_pressed = [false; 7];
    if let Some(state) = action_states.iter().next() {
        use crate::core::input::actions::ActionStateExt;
        pressed[Action::Up as usize] = state.action_pressed(&registry, "Up");
        pressed[Action::Down as usize] = state.action_pressed(&registry, "Down");
        pressed[Action::Left as usize] = state.action_pressed(&registry, "Left");
        pressed[Action::Right as usize] = state.action_pressed(&registry, "Right");
        pressed[Action::Confirm as usize] = state.action_pressed(&registry, "Confirm");
        pressed[Action::Cancel as usize] = state.action_pressed(&registry, "Cancel");
        pressed[Action::Menu as usize] = state.action_pressed(&registry, "Menu");

        just_pressed[Action::Up as usize] = state.action_just_pressed(&registry, "Up");
        just_pressed[Action::Down as usize] = state.action_just_pressed(&registry, "Down");
        just_pressed[Action::Left as usize] = state.action_just_pressed(&registry, "Left");
        just_pressed[Action::Right as usize] = state.action_just_pressed(&registry, "Right");
        just_pressed[Action::Confirm as usize] = state.action_just_pressed(&registry, "Confirm");
        just_pressed[Action::Cancel as usize] = state.action_just_pressed(&registry, "Cancel");
        just_pressed[Action::Menu as usize] = state.action_just_pressed(&registry, "Menu");
    }

    let fact_snapshot = build_fact_snapshot(&fact_db);
    let dt = time.delta_secs();

    for (_entity, ctx, active, mut velocity, mut transform) in query.iter_mut() {
        if !ctx.matches(&mode) {
            continue;
        }

        let Some(loaded) = loaded_mods.mods.get_mut(active.mod_index) else {
            continue;
        };

        {
            let ctx = loaded.store.data_mut();
            ctx.call_ctx.input_pressed = pressed;
            ctx.call_ctx.input_just_pressed = just_pressed;
            ctx.call_ctx.velocity = velocity.0;
            ctx.call_ctx.entity_position = transform.translation.truncate();
            ctx.call_ctx.delta_time = dt;
            ctx.call_ctx.fact_snapshot.clone_from(&fact_snapshot);
            ctx.call_ctx.pending_fact_mutations.clear();
            ctx.call_ctx.pending_events.clear();
        }

        let behavior_iface = loaded.bindings.souprune_plugin_behavior();
        if let Err(e) = behavior_iface.behavior_instance().call_on_update(
            &mut loaded.store,
            active.resource_handle,
            dt,
        ) {
            error!("Behavior on_update failed: {:?}", e);
            continue;
        }

        let new_velocity = loaded.store.data().call_ctx.velocity;
        velocity.0 = new_velocity;
        transform.translation += velocity.0.extend(0.0) * dt;

        let mutations =
            std::mem::take(&mut loaded.store.data_mut().call_ctx.pending_fact_mutations);
        let events = std::mem::take(&mut loaded.store.data_mut().call_ctx.pending_events);
        if !mutations.is_empty() || !events.is_empty() {
            apply_pending_side_effects(mutations, events, &mut fact_db, &mut fact_writer);
        }
    }
}

fn dispatch_wasm_custom_actions_system(
    mut events: MessageReader<FreCustomActionEvent>,
    mut loaded_mods: NonSendMut<LoadedMods>,
    mut fact_db: ResMut<LayeredFactDatabase>,
    mut fact_writer: MessageWriter<FactEvent>,
) {
    let events: Vec<FreCustomActionEvent> = events.read().cloned().collect();
    if events.is_empty() {
        return;
    }

    let fact_snapshot = build_fact_snapshot(&fact_db);

    for event in &events {
        let wit_params: Vec<
            wasm_runtime::exports::souprune::plugin::custom_action_handler::ActionParam,
        > = event
            .params
            .iter()
            .map(|(k, v)| {
                wasm_runtime::exports::souprune::plugin::custom_action_handler::ActionParam {
                    name: k.clone(),
                    value: v.clone(),
                }
            })
            .collect();

        for loaded in &mut loaded_mods.mods {
            if !loaded.handled_action_ids.contains(&event.action_type) {
                continue;
            }

            {
                let ctx = loaded.store.data_mut();
                ctx.call_ctx.fact_snapshot.clone_from(&fact_snapshot);
                ctx.call_ctx.pending_fact_mutations.clear();
                ctx.call_ctx.pending_events.clear();
            }

            let iface = loaded.bindings.souprune_plugin_custom_action_handler();
            match iface.call_handle_action(&mut loaded.store, &event.action_type, &wit_params) {
                Ok(handled) => {
                    if !handled {
                        debug!("WASM mod did not handle action '{}'", event.action_type);
                    }
                }
                Err(e) => {
                    error!("WASM handle-action '{}' failed: {:?}", event.action_type, e);
                }
            }

            let mutations =
                std::mem::take(&mut loaded.store.data_mut().call_ctx.pending_fact_mutations);
            let pending_events =
                std::mem::take(&mut loaded.store.data_mut().call_ctx.pending_events);
            apply_pending_side_effects(mutations, pending_events, &mut fact_db, &mut fact_writer);
        }
    }
}

fn build_fact_snapshot(fact_db: &LayeredFactDatabase) -> HashMap<String, String> {
    fact_db
        .iter_local()
        .chain(fact_db.iter_global())
        .map(|(k, v)| (k.clone(), fact_value_to_string(v)))
        .collect()
}

fn apply_pending_side_effects(
    mutations: Vec<(String, String)>,
    events: Vec<String>,
    fact_db: &mut LayeredFactDatabase,
    fact_writer: &mut MessageWriter<FactEvent>,
) {
    for (key, value) in mutations {
        fact_db.set(key, parse_fact_string(&value));
    }
    for event_name in events {
        fact_writer.write(FactEvent::new(event_name));
    }
}

/// Parse a WASM fact string into the appropriate `FactValue`.
/// Attempts bool → int → float → string, in that order.
///
/// 将 WASM fact 字符串解析为合适的 `FactValue`。
/// 按 bool → int → float → string 的顺序尝试。
fn parse_fact_string(s: &str) -> FactValue {
    if s == "true" {
        return FactValue::Bool(true);
    }
    if s == "false" {
        return FactValue::Bool(false);
    }
    if let Ok(n) = s.parse::<i64>() {
        return FactValue::Int(n);
    }
    if let Ok(f) = s.parse::<f64>() {
        return FactValue::Float(f);
    }
    FactValue::String(s.to_string())
}

fn fact_value_to_string(v: &FactValue) -> String {
    match v {
        FactValue::Int(n) => n.to_string(),
        FactValue::Float(f) => f.to_string(),
        FactValue::Bool(b) => b.to_string(),
        FactValue::String(s) => s.clone(),
        FactValue::StringList(list) => list.join(","),
        FactValue::IntList(list) => list
            .iter()
            .map(|n| n.to_string())
            .collect::<Vec<_>>()
            .join(","),
        FactValue::FloatList(list) => list
            .iter()
            .map(|f| f.to_string())
            .collect::<Vec<_>>()
            .join(","),
        FactValue::BoolList(list) => list
            .iter()
            .map(|b| b.to_string())
            .collect::<Vec<_>>()
            .join(","),
    }
}
