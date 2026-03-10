use bevy::prelude::*;
use souprune_api::Action;
use std::collections::HashMap;

use super::wasm_runtime::{self, LoadedMod, WasmRuntime, update_input_snapshot};

// === Plugin ===

pub struct ModPlugin;

impl Plugin for ModPlugin {
    fn build(&self, app: &mut App) {
        // Initialize the WASM runtime
        let runtime = WasmRuntime::new().expect("Failed to initialize WASM runtime");

        let schedule = crate::game_schedule(app);
        app.insert_non_send_resource(runtime)
            .insert_non_send_resource(LoadedMods::default())
            .init_resource::<BehaviorRegistry>()
            .init_resource::<DanmakuRegistry>()
            .add_systems(Startup, load_mods_system)
            .add_systems(
                schedule,
                (init_behaviors_system, update_behaviors_system)
                    .chain()
                    .in_set(crate::app_state::battle::BattleMovementSet),
            );
    }
}

/// Registry for behavior factories loaded from WASM mods.
///
/// 从 WASM 模组加载的行为工厂注册表。
#[derive(Resource, Default)]
pub struct BehaviorRegistry {
    /// Behavior ID → which mod provides it
    behavior_mods: HashMap<String, usize>,
}

/// Registry for danmaku factories loaded from WASM mods.
///
/// 从 WASM 模组加载的弹幕工厂注册表。
#[derive(Resource, Default)]
pub struct DanmakuRegistry {
    /// Algorithm ID → which mod provides it
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

/// Resource holding all loaded WASM mod instances.
/// Uses NonSend because wasmtime Store contains !Sync types.
///
/// 持有所有已加载 WASM 模组实例的资源。
/// 使用 NonSend 因为 wasmtime Store 包含 !Sync 类型。
#[derive(Default)]
pub struct LoadedMods {
    pub mods: Vec<LoadedMod>,
}

fn load_mods_system(
    mut behavior_registry: ResMut<BehaviorRegistry>,
    mut danmaku_registry: ResMut<DanmakuRegistry>,
    runtime: NonSend<WasmRuntime>,
    mut loaded_mods: NonSendMut<LoadedMods>,
) {
    let config = crate::config::load_config();
    let mod_name = &config.project.mod_name;
    let base_path = crate::config::get_projects_base_path().join(mod_name);

    // Use wasm filename from mod.toml [mod_library], fallback to {mod_name}.wasm
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
        eprintln!(
            "[Souprune] Warning: Mod WASM file not found: {}",
            wasm_path.display()
        );
        return;
    }

    info!("Loading WASM mod: {}", wasm_path.display());
    eprintln!("[Souprune] Loading WASM mod: {}", wasm_path.display());

    match runtime.load_mod(&wasm_path) {
        Ok(loaded) => {
            let mod_index = loaded_mods.mods.len();

            for id in &loaded.behavior_ids {
                info!("Registered Behavior: {}", id);
                behavior_registry
                    .behavior_mods
                    .insert(id.clone(), mod_index);
            }
            eprintln!("[Souprune] Loaded {} behaviors", loaded.behavior_ids.len());

            for id in &loaded.algorithm_ids {
                info!("Registered Danmaku Algorithm: {}", id);
                danmaku_registry
                    .algorithm_mods
                    .insert(id.clone(), mod_index);
            }
            eprintln!(
                "[Souprune] Loaded {} danmaku algorithms",
                loaded.algorithm_ids.len()
            );

            loaded_mods.mods.push(loaded);
        }
        Err(e) => {
            error!("Failed to load WASM mod: {:?}", e);
            eprintln!("[Souprune] Error loading WASM mod: {:?}", e);
        }
    }
}

// === Runtime Components ===

#[derive(Component)]
pub struct BehaviorParams {
    pub mode_id: String,
}

#[derive(Component, Default)]
pub struct BehaviorVelocity(pub Vec2);

/// Active WASM behavior instance, holding a resource handle inside the WASM store.
#[derive(Component)]
pub struct ActiveBehavior {
    mod_index: usize,
    resource_handle: wasmtime::component::ResourceAny,
}

// WASM instances are accessed only from the main thread
unsafe impl Send for ActiveBehavior {}
unsafe impl Sync for ActiveBehavior {}

/// System to initialize new behaviors
fn init_behaviors_system(
    mut commands: Commands,
    query: Query<(Entity, &BehaviorParams), Added<BehaviorParams>>,
    behavior_registry: Res<BehaviorRegistry>,
    mut loaded_mods: NonSendMut<LoadedMods>,
) {
    for (entity, params) in query.iter() {
        let Some(&mod_index) = behavior_registry.behavior_mods.get(&params.mode_id) else {
            error!("Behavior ID not found: {}", params.mode_id);
            continue;
        };

        let Some(loaded) = loaded_mods.mods.get_mut(mod_index) else {
            error!("Mod index {} not loaded", mod_index);
            continue;
        };

        let behavior_iface = loaded.bindings.souprune_plugin_behavior();
        match behavior_iface
            .behavior_instance()
            .call_constructor(&mut loaded.store, &params.mode_id)
        {
            Ok(handle) => {
                // Call on_enter
                if let Err(e) = behavior_iface
                    .behavior_instance()
                    .call_on_enter(&mut loaded.store, handle)
                {
                    error!("Behavior on_enter failed for {}: {:?}", params.mode_id, e);
                }

                commands.entity(entity).insert(ActiveBehavior {
                    mod_index,
                    resource_handle: handle,
                });
            }
            Err(e) => {
                error!("Failed to create behavior {}: {:?}", params.mode_id, e);
            }
        }
    }
}

/// System to update active behaviors
fn update_behaviors_system(
    mut query: Query<(
        Entity,
        &ActiveBehavior,
        &mut BehaviorVelocity,
        &mut Transform,
    )>,
    action_states: Query<
        &leafwing_input_manager::action_state::ActionState<crate::core::input::actions::Action>,
        With<crate::app_state::battle::BattleInputManager>,
    >,
    registry: Res<crate::core::input::actions::ActionRegistry>,
    time: Res<Time>,
    mut loaded_mods: NonSendMut<LoadedMods>,
) {
    // 1. Update input snapshot from ActionState
    let mut pressed = [false; 7];
    if let Some(state) = action_states.iter().next() {
        use crate::core::input::actions::ActionStateExt;
        pressed[Action::Up as usize] = state.action_pressed(&registry, "Up");
        pressed[Action::Down as usize] = state.action_pressed(&registry, "Down");
        pressed[Action::Left as usize] = state.action_pressed(&registry, "Left");
        pressed[Action::Right as usize] = state.action_pressed(&registry, "Right");
        pressed[Action::Confirm as usize] = state.action_pressed(&registry, "Confirm");
        pressed[Action::Cancel as usize] = state.action_pressed(&registry, "Cancel");
        pressed[Action::Menu as usize] = state.action_pressed(&registry, "Menu");
    }
    update_input_snapshot(pressed);

    // 2. Update each active behavior
    for (_entity, active, mut velocity, mut transform) in query.iter_mut() {
        let Some(loaded) = loaded_mods.mods.get_mut(active.mod_index) else {
            continue;
        };

        // Set input context for this call
        loaded.store.data_mut().call_ctx.input_pressed = pressed;
        loaded.store.data_mut().call_ctx.velocity = velocity.0;

        let behavior_iface = loaded.bindings.souprune_plugin_behavior();
        if let Err(e) = behavior_iface.behavior_instance().call_on_update(
            &mut loaded.store,
            active.resource_handle,
            time.delta_secs(),
        ) {
            error!("Behavior on_update failed: {:?}", e);
            continue;
        }

        // Read back velocity changes
        let new_velocity = loaded.store.data().call_ctx.velocity;
        velocity.0 = new_velocity;

        // Apply velocity to transform
        transform.translation += velocity.0.extend(0.0) * time.delta_secs();
    }
}

// === Danmaku WASM Support ===

/// Active WASM danmaku instance.
///
/// 活跃的 WASM 弹幕实例。
#[derive(Component)]
pub struct ActiveDanmaku {
    mod_index: usize,
    resource_handle: wasmtime::component::ResourceAny,
    initialized: bool,
    pub props: HashMap<String, f32>,
}

unsafe impl Send for ActiveDanmaku {}
unsafe impl Sync for ActiveDanmaku {}

impl DanmakuRegistry {
    /// Create a new danmaku instance by algorithm ID.
    pub fn create(&self, id: &str, loaded_mods: &mut LoadedMods) -> Option<ActiveDanmaku> {
        let &mod_index = self.algorithm_mods.get(id)?;
        let loaded = loaded_mods.mods.get_mut(mod_index)?;

        let danmaku_iface = loaded.bindings.souprune_plugin_danmaku();
        let handle = danmaku_iface
            .danmaku_instance()
            .call_constructor(&mut loaded.store, id)
            .ok()?;

        Some(ActiveDanmaku {
            mod_index,
            resource_handle: handle,
            initialized: false,
            props: HashMap::new(),
        })
    }
}

impl ActiveDanmaku {
    pub fn new_with_props(
        mod_index: usize,
        resource_handle: wasmtime::component::ResourceAny,
        props: HashMap<String, f32>,
    ) -> Self {
        Self {
            mod_index,
            resource_handle,
            initialized: false,
            props,
        }
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Call on_enter via WASM.
    pub fn call_on_enter(
        &mut self,
        ctx: &souprune_api::BulletContext,
        loaded_mods: &mut LoadedMods,
    ) {
        if self.initialized {
            return;
        }
        let Some(loaded) = loaded_mods.mods.get_mut(self.mod_index) else {
            return;
        };

        let wit_ctx = to_wit_bullet_context(ctx);
        let danmaku_iface = loaded.bindings.souprune_plugin_danmaku();
        if let Err(e) = danmaku_iface.danmaku_instance().call_on_enter(
            &mut loaded.store,
            self.resource_handle,
            &wit_ctx,
        ) {
            error!("Danmaku on_enter failed: {:?}", e);
        }
        self.initialized = true;
    }

    /// Call on_update via WASM and return the output.
    pub fn call_on_update(
        &mut self,
        ctx: &souprune_api::BulletContext,
        loaded_mods: &mut LoadedMods,
    ) -> souprune_api::BulletOutput {
        let Some(loaded) = loaded_mods.mods.get_mut(self.mod_index) else {
            return souprune_api::BulletOutput::ZERO;
        };

        let wit_ctx = to_wit_bullet_context(ctx);
        let danmaku_iface = loaded.bindings.souprune_plugin_danmaku();
        match danmaku_iface.danmaku_instance().call_on_update(
            &mut loaded.store,
            self.resource_handle,
            &wit_ctx,
        ) {
            Ok(output) => souprune_api::BulletOutput {
                offset: souprune_api::Vec2::new(output.offset.x, output.offset.y),
                rotation: output.rotation,
            },
            Err(e) => {
                error!("Danmaku on_update failed: {:?}", e);
                souprune_api::BulletOutput::ZERO
            }
        }
    }

    /// Call on_exit via WASM.
    pub fn call_on_exit(&mut self, loaded_mods: &mut LoadedMods) {
        if !self.initialized {
            return;
        }
        let Some(loaded) = loaded_mods.mods.get_mut(self.mod_index) else {
            return;
        };

        let danmaku_iface = loaded.bindings.souprune_plugin_danmaku();
        if let Err(e) = danmaku_iface
            .danmaku_instance()
            .call_on_exit(&mut loaded.store, self.resource_handle)
        {
            error!("Danmaku on_exit failed: {:?}", e);
        }
    }
}

/// Convert souprune_api::BulletContext to WIT-generated BulletContext type.
fn to_wit_bullet_context(
    ctx: &souprune_api::BulletContext,
) -> wasm_runtime::exports::souprune::plugin::danmaku::BulletContext {
    wasm_runtime::exports::souprune::plugin::danmaku::BulletContext {
        elapsed: ctx.elapsed,
        delta_time: ctx.delta_time,
        spawn_pos: wasm_runtime::souprune::plugin::host_api::Vec2 {
            x: ctx.spawn_pos.x,
            y: ctx.spawn_pos.y,
        },
        offset: wasm_runtime::souprune::plugin::host_api::Vec2 {
            x: ctx.offset.x,
            y: ctx.offset.y,
        },
        initial_angle: ctx.initial_angle,
        initial_radius: ctx.initial_radius,
        player_pos: wasm_runtime::souprune::plugin::host_api::Vec2 {
            x: ctx.player_pos.x,
            y: ctx.player_pos.y,
        },
        props: ctx
            .props
            .iter()
            .map(|p| wasm_runtime::exports::souprune::plugin::danmaku::Prop {
                name: p.name.clone(),
                value: p.value,
            })
            .collect(),
    }
}
