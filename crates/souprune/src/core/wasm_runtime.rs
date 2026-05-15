//! WASM runtime infrastructure for loading and running mod components.
//!
//! Manages the Wasmtime engine, store, and component instances.
//! Provides the bridge between WIT host-api imports and Bevy ECS.
//!
//! WASM 运行时基础设施，用于加载和运行模组组件。
//! 管理 Wasmtime 引擎、Store 和组件实例。
//! 提供 WIT host-api 导入与 Bevy ECS 之间的桥梁。

use bevy::prelude::*;
use bevy_fact_rule_event::FactValue;
use souprune_api::Action;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use wasmtime::component::{Component, HasSelf, Linker, ResourceTable};
use wasmtime::{Engine, Store};
use wasmtime_wasi::WasiCtxBuilder;

// Generate host-side bindings from the WIT interface
wasmtime::component::bindgen!({
    path: "wit",
    world: "souprune-mod",
});

use self::souprune::plugin::host_api::{
    Action as WitAction, ColliderShape as WitColliderShape, FactValue as WitFact, Rgba as WitRgba,
    SpriteEntityDesc as WitSpriteEntityDesc, Vec2 as WitVec2,
};
use super::collision::{
    CollisionBoundary, CollisionRegion, CollisionRegionStore, ConstraintHandle, PhysicsCollider,
    RegionHandle, TriggerCollider,
};

type SharedCollisionRegionStore = Arc<Mutex<CollisionRegionStore>>;

/// Host-owned side effects requested by a WASM callback.
///
/// WASM 回调请求的宿主侧副作用。
#[derive(Debug, Clone, PartialEq)]
pub enum PendingHostEffect {
    /// Spawn a visible ViewBox primitive and bind it to the opaque handle.
    ///
    /// 生成可见 ViewBox primitive，并绑定到不透明句柄。
    SpawnViewBox {
        handle: u64,
        center: Vec2,
        size: Vec2,
        border_width: f32,
    },
    /// Spawn a visible sprite entity primitive and bind it to the opaque handle.
    ///
    /// 生成可见 sprite 实体 primitive，并绑定到不透明句柄。
    SpawnSpriteEntity {
        handle: u64,
        texture: String,
        position: Vec2,
        z: f32,
        color: Color,
        physics_collider: Option<PhysicsCollider>,
        trigger_collider: Option<TriggerCollider>,
        behavior_id: Option<String>,
        behavior_context: Option<String>,
        bullet_target: bool,
        mode_scope: Option<String>,
        name: Option<String>,
    },
    /// Update a ViewBox primitive's center and size.
    ///
    /// 更新 ViewBox primitive 的中心点和尺寸。
    SetViewBoxBounds {
        handle: u64,
        center: Vec2,
        size: Vec2,
    },
    /// Tween a ViewBox primitive's center and size on the host.
    ///
    /// 在宿主侧对 ViewBox primitive 的中心点和尺寸执行 tween。
    TweenViewBoxBounds {
        handle: u64,
        center: Vec2,
        size: Vec2,
        duration_secs: f32,
    },
    /// Update a ViewBox primitive's visibility.
    ///
    /// 更新 ViewBox primitive 的可见性。
    SetViewBoxVisible { handle: u64, visible: bool },
    /// Remove a host-owned entity primitive.
    ///
    /// 移除宿主拥有的实体 primitive。
    RemoveEntity { handle: u64 },
}

/// Audio playback requested by a WASM callback.
///
/// WASM 回调请求的音频播放。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PendingSoundEffect {
    /// Resolve the value through the configured audio resource roots.
    ///
    /// 通过配置的音频资源根目录解析此值。
    SoundKey(String),
    /// Load the value as a full Bevy asset path.
    ///
    /// 将此值作为完整 Bevy 资源路径加载。
    FullPath(String),
}

/// Pending side effects collected after a WASM callback returns.
///
/// WASM 回调返回后收集的待提交副作用。
#[derive(Default, Debug, Clone, PartialEq)]
pub struct PendingSideEffects {
    pub fact_mutations: Vec<(String, FactValue)>,
    pub global_fact_mutations: Vec<(String, FactValue)>,
    pub events: Vec<String>,
    pub sounds: Vec<PendingSoundEffect>,
    pub host_effects: Vec<PendingHostEffect>,
}

impl PendingSideEffects {
    /// Return whether this bundle has no queued side effects.
    ///
    /// 返回此副作用集合是否为空。
    pub fn is_empty(&self) -> bool {
        self.fact_mutations.is_empty()
            && self.global_fact_mutations.is_empty()
            && self.events.is_empty()
            && self.sounds.is_empty()
            && self.host_effects.is_empty()
    }
}

/// Per-call context: set by the host before invoking a mod callback.
#[derive(Default)]
pub struct CallContext {
    pub input_pressed: [bool; 7],
    pub input_just_pressed: [bool; 7],
    pub velocity: Vec2,
    pub entity_position: Vec2,
    pub delta_time: f32,
    /// Snapshot of the current FRE fact database (key → typed value).
    pub fact_snapshot: Arc<HashMap<String, FactValue>>,
    /// Fact mutations queued by the mod during a callback; applied afterwards.
    pub pending_fact_mutations: Vec<(String, FactValue)>,
    /// Persistent fact mutations queued by the mod during a callback.
    pub pending_global_fact_mutations: Vec<(String, FactValue)>,
    /// FRE events queued by the mod during a callback; emitted afterwards.
    pub pending_events: Vec<String>,
    /// Named entity positions for `get-entity-position-by-tag` queries.
    pub entity_positions_by_tag: HashMap<String, Vec2>,
    /// Current mode name for `get-current-mode`.
    pub current_mode: Option<String>,
    /// Current sub-state name for `get-current-sub-state`.
    pub current_sub_state: String,
    /// Pending view open requests (view-id strings).
    pub pending_view_opens: Vec<String>,
    /// Whether a close-view was requested.
    pub pending_view_close: bool,
    /// Pending sound play requests.
    pub pending_sounds: Vec<PendingSoundEffect>,
    /// Pending emitter spawn requests: (pattern_id, position) → assigned handle.
    pub pending_emitter_spawns: Vec<(String, Vec2)>,
    /// Pending emitter despawn requests (handles).
    pub pending_emitter_despawns: Vec<u64>,
    /// Host-owned entity primitive effects queued by the mod during a callback.
    pub pending_host_effects: Vec<PendingHostEffect>,
    /// Counter for assigning emitter handles within a single callback invocation.
    pub emitter_handle_counter: u64,
    /// Counter for assigning host-owned entity primitive handles.
    pub host_entity_handle_counter: u64,
}

impl CallContext {
    /// Clear callback-scoped pending side effects before entering WASM.
    ///
    /// 进入 WASM 前清空回调范围内的待提交副作用。
    pub fn clear_pending_side_effects(&mut self) {
        self.pending_fact_mutations.clear();
        self.pending_global_fact_mutations.clear();
        self.pending_events.clear();
        self.pending_sounds.clear();
        self.pending_host_effects.clear();
    }

    /// Take all pending side effects after a WASM callback returns.
    ///
    /// WASM 回调返回后取出所有待提交副作用。
    pub fn take_pending_side_effects(&mut self) -> PendingSideEffects {
        PendingSideEffects {
            fact_mutations: std::mem::take(&mut self.pending_fact_mutations),
            global_fact_mutations: std::mem::take(&mut self.pending_global_fact_mutations),
            events: std::mem::take(&mut self.pending_events),
            sounds: std::mem::take(&mut self.pending_sounds),
            host_effects: std::mem::take(&mut self.pending_host_effects),
        }
    }

    fn next_host_entity_handle(&mut self) -> u64 {
        self.host_entity_handle_counter = self.host_entity_handle_counter.saturating_add(1).max(1);
        self.host_entity_handle_counter
    }
}

/// Host state stored inside the Wasmtime `Store`.
pub struct HostState {
    pub wasi: wasmtime_wasi::WasiCtx,
    pub table: ResourceTable,
    pub call_ctx: CallContext,
    collision_regions: SharedCollisionRegionStore,
}

impl HostState {
    fn new_for_mod(collision_regions: SharedCollisionRegionStore) -> Self {
        Self {
            wasi: WasiCtxBuilder::new().build(),
            table: ResourceTable::new(),
            call_ctx: CallContext::default(),
            collision_regions,
        }
    }

    fn collision_region_store(&self) -> Option<std::sync::MutexGuard<'_, CollisionRegionStore>> {
        match self.collision_regions.lock() {
            Ok(store) => Some(store),
            Err(err) => {
                error!("Collision region store lock is poisoned: {err}");
                None
            }
        }
    }
}

impl wasmtime_wasi::WasiView for HostState {
    fn ctx(&mut self) -> wasmtime_wasi::WasiCtxView<'_> {
        wasmtime_wasi::WasiCtxView {
            ctx: &mut self.wasi,
            table: &mut self.table,
        }
    }
}

impl self::souprune::plugin::host_api::Host for HostState {
    fn log(&mut self, level: u32, message: String) {
        match level {
            0 => info!("[MOD] {}", message),
            1 => warn!("[MOD] {}", message),
            _ => error!("[MOD] {}", message),
        }
    }

    fn is_action_pressed(&mut self, action: WitAction) -> bool {
        let idx = action_to_index(action);
        self.call_ctx.input_pressed[idx]
    }

    fn is_action_just_pressed(&mut self, action: WitAction) -> bool {
        let idx = action_to_index(action);
        self.call_ctx.input_just_pressed[idx]
    }

    fn set_velocity(&mut self, velocity: WitVec2) {
        self.call_ctx.velocity = Vec2::new(velocity.x, velocity.y);
    }

    fn create_collision_region(&mut self, center: WitVec2, half_size: WitVec2) -> u64 {
        let center = Vec2::new(center.x, center.y);
        let half_size = Vec2::new(half_size.x, half_size.y);
        if !is_valid_positive_vec2(half_size) || !center.is_finite() {
            warn!(
                "Ignoring invalid collision region center={:?} half_size={:?}",
                center, half_size
            );
            return 0;
        }

        let Some(mut collision_regions) = self.collision_region_store() else {
            return 0;
        };
        collision_regions
            .create_region(CollisionRegion::new(CollisionBoundary {
                center,
                half_size,
            }))
            .raw()
    }

    fn remove_collision_region(&mut self, region: u64) {
        if let Some(mut collision_regions) = self.collision_region_store() {
            collision_regions.remove_region(RegionHandle::from_raw(region));
        }
    }

    fn set_collision_region_bounds(&mut self, region: u64, center: WitVec2, half_size: WitVec2) {
        let center = Vec2::new(center.x, center.y);
        let half_size = Vec2::new(half_size.x, half_size.y);
        if region == 0 || !center.is_finite() || !is_valid_positive_vec2(half_size) {
            warn!(
                "Ignoring invalid collision region update handle={} center={:?} half_size={:?}",
                region, center, half_size
            );
            return;
        }

        if let Some(mut collision_regions) = self.collision_region_store() {
            collision_regions.update_region(
                RegionHandle::from_raw(region),
                CollisionRegion::new(CollisionBoundary { center, half_size }),
            );
        }
    }

    fn create_movement_constraint(
        &mut self,
        region: u64,
        collider: WitColliderShape,
    ) -> Option<u64> {
        let collider = wit_to_physics_collider(collider)?;
        self.collision_region_store()?
            .create_movement_constraint(RegionHandle::from_raw(region), collider)
            .map(ConstraintHandle::raw)
    }

    fn remove_movement_constraint(&mut self, constraint: u64) {
        if let Some(mut collision_regions) = self.collision_region_store() {
            collision_regions.remove_movement_constraint(ConstraintHandle::from_raw(constraint));
        }
    }

    fn constrain_movement(&mut self, constraint: u64, position: WitVec2) -> Option<WitVec2> {
        self.collision_region_store()?
            .constrain_movement(
                ConstraintHandle::from_raw(constraint),
                Vec2::new(position.x, position.y),
            )
            .map(|pos| WitVec2 { x: pos.x, y: pos.y })
    }

    fn spawn_view_box(&mut self, center: WitVec2, size: WitVec2, border_width: f32) -> u64 {
        let center = Vec2::new(center.x, center.y);
        let size = Vec2::new(size.x, size.y);
        if !center.is_finite()
            || !is_valid_positive_vec2(size)
            || !border_width.is_finite()
            || border_width < 0.0
        {
            warn!(
                "Ignoring invalid ViewBox primitive center={:?} size={:?} border_width={}",
                center, size, border_width
            );
            return 0;
        }

        let handle = self.call_ctx.next_host_entity_handle();
        self.call_ctx
            .pending_host_effects
            .push(PendingHostEffect::SpawnViewBox {
                handle,
                center,
                size,
                border_width,
            });
        handle
    }

    fn spawn_sprite_entity(&mut self, desc: WitSpriteEntityDesc) -> u64 {
        let position = Vec2::new(desc.position.x, desc.position.y);
        let Some(color) = wit_rgba_to_color(desc.color) else {
            warn!("Ignoring sprite entity with invalid color");
            return 0;
        };
        if desc.texture.trim().is_empty() || !position.is_finite() || !desc.z.is_finite() {
            warn!(
                "Ignoring invalid sprite entity texture='{}' position={:?} z={}",
                desc.texture, position, desc.z
            );
            return 0;
        }

        let physics_collider = desc.physics_collider.and_then(wit_to_physics_collider);
        let trigger_collider = desc.trigger_collider.and_then(wit_to_trigger_collider);
        let handle = self.call_ctx.next_host_entity_handle();
        self.call_ctx
            .pending_host_effects
            .push(PendingHostEffect::SpawnSpriteEntity {
                handle,
                texture: desc.texture,
                position,
                z: desc.z,
                color,
                physics_collider,
                trigger_collider,
                behavior_id: desc.behavior_id,
                behavior_context: desc.behavior_context,
                bullet_target: desc.bullet_target,
                mode_scope: desc.mode_scope,
                name: desc.name,
            });
        handle
    }

    fn set_view_box_bounds(&mut self, handle: u64, center: WitVec2, size: WitVec2) {
        let center = Vec2::new(center.x, center.y);
        let size = Vec2::new(size.x, size.y);
        if handle == 0 || !center.is_finite() || !is_valid_positive_vec2(size) {
            warn!(
                "Ignoring invalid ViewBox bounds handle={} center={:?} size={:?}",
                handle, center, size
            );
            return;
        }

        self.call_ctx
            .pending_host_effects
            .push(PendingHostEffect::SetViewBoxBounds {
                handle,
                center,
                size,
            });
    }

    fn tween_view_box_bounds(
        &mut self,
        handle: u64,
        center: WitVec2,
        size: WitVec2,
        duration_secs: f32,
    ) {
        let center = Vec2::new(center.x, center.y);
        let size = Vec2::new(size.x, size.y);
        if handle == 0
            || !center.is_finite()
            || !is_valid_positive_vec2(size)
            || !duration_secs.is_finite()
            || duration_secs <= 0.0
        {
            warn!(
                "Ignoring invalid ViewBox tween handle={} center={:?} size={:?} duration={}",
                handle, center, size, duration_secs
            );
            return;
        }

        self.call_ctx
            .pending_host_effects
            .push(PendingHostEffect::TweenViewBoxBounds {
                handle,
                center,
                size,
                duration_secs,
            });
    }

    fn set_view_box_visible(&mut self, handle: u64, visible: bool) {
        if handle == 0 {
            warn!("Ignoring invalid ViewBox visibility handle=0");
            return;
        }

        self.call_ctx
            .pending_host_effects
            .push(PendingHostEffect::SetViewBoxVisible { handle, visible });
    }

    fn remove_entity(&mut self, handle: u64) {
        if handle == 0 {
            warn!("Ignoring invalid host entity handle=0");
            return;
        }

        self.call_ctx
            .pending_host_effects
            .push(PendingHostEffect::RemoveEntity { handle });
    }

    fn get_entity_position(&mut self) -> WitVec2 {
        WitVec2 {
            x: self.call_ctx.entity_position.x,
            y: self.call_ctx.entity_position.y,
        }
    }

    fn delta_time(&mut self) -> f32 {
        self.call_ctx.delta_time
    }

    fn get_fact(&mut self, key: String) -> Option<WitFact> {
        self.call_ctx.fact_snapshot.get(&key).map(fre_to_wit_fact)
    }

    fn set_fact(&mut self, key: String, value: WitFact) {
        self.call_ctx
            .pending_fact_mutations
            .push((key, wit_to_fre_fact(value)));
    }

    fn set_global_fact(&mut self, key: String, value: WitFact) {
        self.call_ctx
            .pending_global_fact_mutations
            .push((key, wit_to_fre_fact(value)));
    }

    fn emit_event(&mut self, event_name: String) {
        self.call_ctx.pending_events.push(event_name);
    }

    fn get_entity_position_by_tag(&mut self, tag: String) -> Option<WitVec2> {
        self.call_ctx
            .entity_positions_by_tag
            .get(&tag)
            .map(|pos| WitVec2 { x: pos.x, y: pos.y })
    }

    fn spawn_emitter(&mut self, pattern_id: String, position: WitVec2) -> u64 {
        let handle = self.call_ctx.emitter_handle_counter;
        self.call_ctx.emitter_handle_counter += 1;
        self.call_ctx
            .pending_emitter_spawns
            .push((pattern_id, Vec2::new(position.x, position.y)));
        handle
    }

    fn despawn_emitter(&mut self, handle: u64) {
        self.call_ctx.pending_emitter_despawns.push(handle);
    }

    fn open_view(&mut self, view_id: String) {
        self.call_ctx.pending_view_opens.push(view_id);
    }

    fn close_view(&mut self) {
        self.call_ctx.pending_view_close = true;
    }

    fn play_sound(&mut self, sound_key: String) {
        self.call_ctx
            .pending_sounds
            .push(PendingSoundEffect::SoundKey(sound_key));
    }

    fn play_sound_full_path(&mut self, sound_path: String) {
        self.call_ctx
            .pending_sounds
            .push(PendingSoundEffect::FullPath(sound_path));
    }

    fn get_current_mode(&mut self) -> Option<String> {
        self.call_ctx.current_mode.clone()
    }

    fn get_current_sub_state(&mut self) -> String {
        self.call_ctx.current_sub_state.clone()
    }
}

fn action_to_index(action: WitAction) -> usize {
    match action {
        WitAction::Up => Action::Up as usize,
        WitAction::Down => Action::Down as usize,
        WitAction::Left => Action::Left as usize,
        WitAction::Right => Action::Right as usize,
        WitAction::Confirm => Action::Confirm as usize,
        WitAction::Cancel => Action::Cancel as usize,
        WitAction::Menu => Action::Menu as usize,
    }
}

fn wit_to_physics_collider(collider: WitColliderShape) -> Option<PhysicsCollider> {
    match collider {
        WitColliderShape::Circle(radius) if radius.is_finite() && radius > 0.0 => {
            Some(PhysicsCollider::Circle { radius })
        }
        WitColliderShape::Rectangle(half_size) => {
            let half_size = Vec2::new(half_size.x, half_size.y);
            is_valid_positive_vec2(half_size).then_some(PhysicsCollider::Box { half_size })
        }
        _ => None,
    }
}

fn wit_to_trigger_collider(collider: WitColliderShape) -> Option<TriggerCollider> {
    match collider {
        WitColliderShape::Circle(radius) if radius.is_finite() && radius > 0.0 => {
            Some(TriggerCollider::Circle { radius })
        }
        WitColliderShape::Rectangle(half_size) => {
            let half_size = Vec2::new(half_size.x, half_size.y);
            is_valid_positive_vec2(half_size).then_some(TriggerCollider::Box { half_size })
        }
        _ => None,
    }
}

fn wit_rgba_to_color(color: WitRgba) -> Option<Color> {
    [color.red, color.green, color.blue, color.alpha]
        .iter()
        .all(|component| component.is_finite())
        .then(|| Color::srgba(color.red, color.green, color.blue, color.alpha))
}

fn is_valid_positive_vec2(value: Vec2) -> bool {
    value.is_finite() && value.x > 0.0 && value.y > 0.0
}

/// Convert FRE `FactValue` to the WIT-generated `FactValue` variant.
fn fre_to_wit_fact(v: &FactValue) -> WitFact {
    match v {
        FactValue::Int(n) => WitFact::IntVal(*n),
        FactValue::Float(f) => WitFact::FloatVal(*f),
        FactValue::Bool(b) => WitFact::BoolVal(*b),
        FactValue::String(s) => WitFact::TextVal(s.clone()),
        FactValue::StringList(list) => WitFact::TextList(list.clone()),
        FactValue::IntList(list) => WitFact::IntList(list.clone()),
        FactValue::FloatList(list) => WitFact::FloatList(list.clone()),
        FactValue::BoolList(list) => WitFact::BoolList(list.clone()),
    }
}

/// Convert WIT-generated `FactValue` variant to FRE `FactValue`.
fn wit_to_fre_fact(v: WitFact) -> FactValue {
    match v {
        WitFact::IntVal(n) => FactValue::Int(n),
        WitFact::FloatVal(f) => FactValue::Float(f),
        WitFact::BoolVal(b) => FactValue::Bool(b),
        WitFact::TextVal(s) => FactValue::String(s),
        WitFact::IntList(list) => FactValue::IntList(list),
        WitFact::FloatList(list) => FactValue::FloatList(list),
        WitFact::BoolList(list) => FactValue::BoolList(list),
        WitFact::TextList(list) => FactValue::StringList(list),
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

#[cfg(test)]
mod tests {
    use self::souprune::plugin::host_api::Host;
    use super::*;
    use std::sync::Mutex;

    #[test]
    fn host_states_share_collision_region_registry() {
        let shared = Arc::new(Mutex::new(CollisionRegionStore::default()));
        let mut first = HostState::new_for_mod(Arc::clone(&shared));
        let mut second = HostState::new_for_mod(Arc::clone(&shared));

        let region =
            first.create_collision_region(WitVec2 { x: 0.0, y: 0.0 }, WitVec2 { x: 20.0, y: 20.0 });
        let constraint = second
            .create_movement_constraint(region, WitColliderShape::Circle(5.0))
            .unwrap();

        let constrained = second
            .constrain_movement(constraint, WitVec2 { x: 30.0, y: 0.0 })
            .unwrap();

        assert_eq!(constrained.x, 15.0);
        assert_eq!(constrained.y, 0.0);

        first.set_collision_region_bounds(
            region,
            WitVec2 { x: 0.0, y: 0.0 },
            WitVec2 { x: 10.0, y: 10.0 },
        );
        let constrained_after_update = second
            .constrain_movement(constraint, WitVec2 { x: 30.0, y: 0.0 })
            .unwrap();

        assert_eq!(constrained_after_update.x, 5.0);
        assert_eq!(constrained_after_update.y, 0.0);
    }

    #[test]
    fn call_context_takes_fact_event_and_host_entity_effects_together() {
        let mut ctx = CallContext::default();
        ctx.pending_fact_mutations
            .push(("cursor:index".to_string(), FactValue::Int(1)));
        ctx.pending_global_fact_mutations
            .push(("player:hp".to_string(), FactValue::Int(12)));
        ctx.pending_events.push("view.cursor.moved".to_string());
        ctx.pending_sounds.push(PendingSoundEffect::FullPath(
            "assets/audios/snd_heal_c.wav".to_string(),
        ));
        ctx.pending_host_effects
            .push(PendingHostEffect::SpawnViewBox {
                handle: 1,
                center: Vec2::new(10.0, 20.0),
                size: Vec2::new(120.0, 48.0),
                border_width: 4.0,
            });

        let pending = ctx.take_pending_side_effects();

        assert_eq!(pending.fact_mutations.len(), 1);
        assert_eq!(pending.global_fact_mutations.len(), 1);
        assert_eq!(pending.events, vec!["view.cursor.moved".to_string()]);
        assert_eq!(
            pending.sounds,
            vec![PendingSoundEffect::FullPath(
                "assets/audios/snd_heal_c.wav".to_string()
            )]
        );
        assert_eq!(pending.host_effects.len(), 1);
        assert!(ctx.pending_fact_mutations.is_empty());
        assert!(ctx.pending_global_fact_mutations.is_empty());
        assert!(ctx.pending_events.is_empty());
        assert!(ctx.pending_sounds.is_empty());
        assert!(ctx.pending_host_effects.is_empty());
    }

    #[test]
    fn wit_fact_conversion_preserves_list_fact_types() {
        match fre_to_wit_fact(&FactValue::StringList(vec!["a".into(), "b".into()])) {
            WitFact::TextList(list) => assert_eq!(list, vec!["a".to_string(), "b".to_string()]),
            _ => panic!("expected TextList"),
        }
        match fre_to_wit_fact(&FactValue::IntList(vec![1, 2])) {
            WitFact::IntList(list) => assert_eq!(list, vec![1, 2]),
            _ => panic!("expected IntList"),
        }
        match fre_to_wit_fact(&FactValue::FloatList(vec![1.5, 2.5])) {
            WitFact::FloatList(list) => assert_eq!(list, vec![1.5, 2.5]),
            _ => panic!("expected FloatList"),
        }
        match fre_to_wit_fact(&FactValue::BoolList(vec![true, false])) {
            WitFact::BoolList(list) => assert_eq!(list, vec![true, false]),
            _ => panic!("expected BoolList"),
        }

        assert_eq!(
            wit_to_fre_fact(WitFact::TextList(vec!["a".into(), "b".into()])),
            FactValue::StringList(vec!["a".into(), "b".into()])
        );
        assert_eq!(
            wit_to_fre_fact(WitFact::IntList(vec![1, 2])),
            FactValue::IntList(vec![1, 2])
        );
        assert_eq!(
            wit_to_fre_fact(WitFact::FloatList(vec![1.5, 2.5])),
            FactValue::FloatList(vec![1.5, 2.5])
        );
        assert_eq!(
            wit_to_fre_fact(WitFact::BoolList(vec![true, false])),
            FactValue::BoolList(vec![true, false])
        );
    }

    #[test]
    fn view_box_host_api_queues_primitives_with_opaque_handles() {
        let shared = Arc::new(Mutex::new(CollisionRegionStore::default()));
        let mut host = HostState::new_for_mod(shared);

        let handle = host.spawn_view_box(
            WitVec2 { x: 10.0, y: 20.0 },
            WitVec2 { x: 120.0, y: 48.0 },
            4.0,
        );
        host.set_view_box_bounds(
            handle,
            WitVec2 { x: 16.0, y: 24.0 },
            WitVec2 { x: 144.0, y: 60.0 },
        );
        host.set_view_box_visible(handle, false);
        host.tween_view_box_bounds(
            handle,
            WitVec2 { x: 32.0, y: 40.0 },
            WitVec2 { x: 180.0, y: 72.0 },
            0.25,
        );
        host.remove_entity(handle);
        let invalid = host.spawn_view_box(
            WitVec2 { x: 0.0, y: 0.0 },
            WitVec2 { x: -1.0, y: 48.0 },
            4.0,
        );

        assert_eq!(handle, 1);
        assert_eq!(invalid, 0);
        assert_eq!(
            host.call_ctx.pending_host_effects,
            vec![
                PendingHostEffect::SpawnViewBox {
                    handle,
                    center: Vec2::new(10.0, 20.0),
                    size: Vec2::new(120.0, 48.0),
                    border_width: 4.0,
                },
                PendingHostEffect::SetViewBoxBounds {
                    handle,
                    center: Vec2::new(16.0, 24.0),
                    size: Vec2::new(144.0, 60.0),
                },
                PendingHostEffect::SetViewBoxVisible {
                    handle,
                    visible: false,
                },
                PendingHostEffect::TweenViewBoxBounds {
                    handle,
                    center: Vec2::new(32.0, 40.0),
                    size: Vec2::new(180.0, 72.0),
                    duration_secs: 0.25,
                },
                PendingHostEffect::RemoveEntity { handle },
            ]
        );
    }

    #[test]
    fn sprite_entity_host_api_queues_generic_entity_primitive() {
        let shared = Arc::new(Mutex::new(CollisionRegionStore::default()));
        let mut host = HostState::new_for_mod(shared);

        let handle = host.spawn_sprite_entity(WitSpriteEntityDesc {
            texture: "assets/textures/common/view/heart.png".into(),
            position: WitVec2 { x: 0.0, y: -80.0 },
            z: 10.0,
            color: WitRgba {
                red: 1.0,
                green: 0.0,
                blue: 0.0,
                alpha: 1.0,
            },
            physics_collider: Some(WitColliderShape::Circle(8.0)),
            trigger_collider: Some(WitColliderShape::Rectangle(WitVec2 { x: 2.0, y: 2.0 })),
            behavior_id: Some("soul_red".into()),
            behavior_context: Some("battle".into()),
            bullet_target: true,
            mode_scope: Some("battle".into()),
            name: Some("Player".into()),
        });

        assert_eq!(handle, 1);
        assert_eq!(
            host.call_ctx.pending_host_effects,
            vec![PendingHostEffect::SpawnSpriteEntity {
                handle,
                texture: "assets/textures/common/view/heart.png".into(),
                position: Vec2::new(0.0, -80.0),
                z: 10.0,
                color: Color::srgba(1.0, 0.0, 0.0, 1.0),
                physics_collider: Some(PhysicsCollider::Circle { radius: 8.0 }),
                trigger_collider: Some(TriggerCollider::Box {
                    half_size: Vec2::new(2.0, 2.0),
                }),
                behavior_id: Some("soul_red".into()),
                behavior_context: Some("battle".into()),
                bullet_target: true,
                mode_scope: Some("battle".into()),
                name: Some("Player".into()),
            }]
        );
    }
}
