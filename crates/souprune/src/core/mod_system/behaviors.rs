//! WASM behavior components and runtime systems.
//!
//! WASM 行为组件与运行时系统。

use bevy::prelude::*;
use bevy_fact_rule_event::{FactEvent, FactValue, LayeredFactDatabase};
use souprune_api::Action;
use std::collections::HashMap;
use std::sync::Arc;

use super::custom_actions::{apply_pending_side_effects, build_fact_snapshot};
use super::{BehaviorRegistry, LoadedMods, wasm_runtime};
use crate::core::input::{
    Direction, InputCommand, InputContextId, InputEnvelope, InputEnvelopeEvent, InputTarget,
};

/// Which game context a WASM behavior is allowed to run in.
/// An empty string means "any mode". Otherwise, the tag is matched against
/// the current `SequenceMode` name (e.g. `"battle"`, `"overworld"`).
///
/// WASM 行为允许运行的游戏上下文。
/// 空字符串表示"任何模式"。否则标签与当前 `SequenceMode` 名称匹配。
#[derive(Component, Clone, Debug, Default, PartialEq, Eq)]
pub struct BehaviorContext(pub String);

impl BehaviorContext {
    pub fn any() -> Self {
        Self(String::new())
    }

    pub fn new(tag: impl Into<String>) -> Self {
        Self(tag.into())
    }

    pub(super) fn matches(&self, mode: &crate::core::mode::SequenceMode) -> bool {
        self.0.is_empty() || mode.is(&self.0)
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
            context: BehaviorContext::any(),
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
    behavior_id: String,
    mod_index: usize,
    resource_handle: wasmtime::component::ResourceAny,
}

unsafe impl Send for ActiveBehavior {}
unsafe impl Sync for ActiveBehavior {}

pub(super) fn init_behaviors_system(
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
                    ctx.call_ctx.fact_snapshot = Arc::clone(&fact_snapshot);
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
                        behavior_id: params.behavior_id.clone(),
                        mod_index,
                        resource_handle: handle,
                    },
                    params.context.clone(),
                ));
            }
            Err(e) => {
                error!("Failed to create behavior {}: {:?}", params.behavior_id, e);
            }
        }
    }
}

pub(super) fn dispatch_behavior_input_system(
    mut events: MessageReader<InputEnvelopeEvent>,
    query: Query<(
        &BehaviorContext,
        &ActiveBehavior,
        Option<&BehaviorVelocity>,
        &Transform,
    )>,
    mode: Res<crate::core::mode::SequenceMode>,
    mut loaded_mods: NonSendMut<LoadedMods>,
    mut fact_db: ResMut<LayeredFactDatabase>,
    mut fact_writer: MessageWriter<FactEvent>,
    mut fact_history: ResMut<crate::core::trace::FactChangeHistory>,
    frame_count: Res<bevy::diagnostic::FrameCount>,
    mut wasm_tracer: ResMut<crate::core::trace::WasmCallTracer>,
) {
    let events: Vec<InputEnvelopeEvent> = events.read().cloned().collect();
    if events.is_empty() {
        return;
    }

    let fact_snapshot = build_fact_snapshot(&fact_db);

    for event in &events {
        for (behavior_context, active, velocity, transform) in query.iter() {
            if !behavior_context.matches(&mode)
                || !input_envelope_targets_behavior(&event.envelope, &active.behavior_id)
            {
                continue;
            }

            let Some(loaded) = loaded_mods.mods.get_mut(active.mod_index) else {
                continue;
            };

            {
                let ctx = loaded.store.data_mut();
                ctx.call_ctx.velocity = velocity.map_or(Vec2::ZERO, |v| v.0);
                ctx.call_ctx.entity_position = transform.translation.truncate();
                ctx.call_ctx.fact_snapshot = Arc::clone(&fact_snapshot);
                ctx.call_ctx.pending_fact_mutations.clear();
                ctx.call_ctx.pending_events.clear();
            }

            let behavior_iface = loaded.bindings.souprune_plugin_behavior();
            let wit_context = input_context_to_wit(&event.envelope.context);
            let wit_command = input_command_to_wit(&event.envelope.command);
            let start = std::time::Instant::now();
            let result = behavior_iface.behavior_instance().call_on_input(
                &mut loaded.store,
                active.resource_handle,
                &wit_context,
                wit_command,
            );
            let elapsed = start.elapsed();
            wasm_tracer.record(&loaded.name, "behavior", "on_input", elapsed);

            if let Err(e) = result {
                error!(
                    "Behavior on_input failed for {}: {:?}",
                    active.behavior_id, e
                );
                continue;
            }

            let mod_name = loaded.name.clone();
            let mutations =
                std::mem::take(&mut loaded.store.data_mut().call_ctx.pending_fact_mutations);
            let pending_events =
                std::mem::take(&mut loaded.store.data_mut().call_ctx.pending_events);
            if !mutations.is_empty() || !pending_events.is_empty() {
                apply_pending_side_effects(
                    mutations,
                    pending_events,
                    &mut fact_db,
                    &mut fact_writer,
                    &mut fact_history,
                    frame_count.0 as u64,
                    &format!("behavior-input:{mod_name}"),
                );
            }
        }
    }
}

fn input_envelope_targets_behavior(envelope: &InputEnvelope, behavior_id: &str) -> bool {
    match &envelope.target {
        InputTarget::Behavior(target_id) => target_id == behavior_id,
        InputTarget::ActiveView | InputTarget::FreScope => true,
    }
}

fn input_context_to_wit(
    context: &InputContextId,
) -> wasm_runtime::exports::souprune::plugin::behavior::InputContextId {
    use wasm_runtime::exports::souprune::plugin::behavior::{
        InputContextId as WitInputContextId, InputContextKind,
    };

    let (kind, custom_name) = match context {
        InputContextId::Overworld => (InputContextKind::Overworld, None),
        InputContextId::Battle => (InputContextKind::Battle, None),
        InputContextId::Dialogue => (InputContextKind::Dialogue, None),
        InputContextId::View => (InputContextKind::View, None),
        InputContextId::Custom(name) => (InputContextKind::Custom, Some(name.clone())),
    };

    WitInputContextId { kind, custom_name }
}

fn input_command_to_wit(
    command: &InputCommand,
) -> wasm_runtime::exports::souprune::plugin::behavior::InputCommand {
    use wasm_runtime::exports::souprune::plugin::behavior::{
        Direction as WitDirection, InputCommand as WitInputCommand,
    };

    match command {
        InputCommand::Navigate(direction) => WitInputCommand::Navigate(match direction {
            Direction::Up => WitDirection::Up,
            Direction::Down => WitDirection::Down,
            Direction::Left => WitDirection::Left,
            Direction::Right => WitDirection::Right,
        }),
        InputCommand::Confirm => WitInputCommand::Confirm,
        InputCommand::Cancel => WitInputCommand::Cancel,
        InputCommand::Menu => WitInputCommand::Menu,
    }
}

pub(super) fn update_behaviors_system(
    mut query: Query<(
        Entity,
        &BehaviorContext,
        &ActiveBehavior,
        Option<&mut BehaviorVelocity>,
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
    mut fact_history: ResMut<crate::core::trace::FactChangeHistory>,
    frame_count: Res<bevy::diagnostic::FrameCount>,
    mut cached_snapshot: Local<Arc<HashMap<String, FactValue>>>,
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

    // Rebuild snapshot only when facts changed; reuse cached copy otherwise.
    if fact_db.is_changed() || cached_snapshot.is_empty() {
        *cached_snapshot = build_fact_snapshot(&fact_db);
    }
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
            ctx.call_ctx.velocity = velocity.as_ref().map_or(Vec2::ZERO, |v| v.0);
            ctx.call_ctx.entity_position = transform.translation.truncate();
            ctx.call_ctx.delta_time = dt;
            ctx.call_ctx.fact_snapshot = Arc::clone(&cached_snapshot);
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

        // Only apply velocity-based movement when BehaviorVelocity is present.
        if let Some(ref mut vel) = velocity {
            let new_velocity = loaded.store.data().call_ctx.velocity;
            vel.0 = new_velocity;
            transform.translation += vel.0.extend(0.0) * dt;
        }

        let mod_name = loaded.name.clone();

        let mutations =
            std::mem::take(&mut loaded.store.data_mut().call_ctx.pending_fact_mutations);
        let events = std::mem::take(&mut loaded.store.data_mut().call_ctx.pending_events);
        if !mutations.is_empty() || !events.is_empty() {
            apply_pending_side_effects(
                mutations,
                events,
                &mut fact_db,
                &mut fact_writer,
                &mut fact_history,
                frame_count.0 as u64,
                &format!("behavior:{mod_name}"),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_runtime::exports::souprune::plugin::behavior::{
        Direction as WitDirection, InputCommand as WitInputCommand, InputContextKind,
    };

    #[test]
    fn behavior_target_only_matches_requested_behavior_id() {
        let envelope = InputEnvelope::new(
            InputContextId::Battle,
            InputTarget::Behavior("menu".to_string()),
            InputCommand::Confirm,
            "Confirm",
        );

        assert!(input_envelope_targets_behavior(&envelope, "menu"));
        assert!(!input_envelope_targets_behavior(&envelope, "soul"));
    }

    #[test]
    fn side_channel_targets_reach_active_behaviors_for_phase_five() {
        let fre_envelope = InputEnvelope::new(
            InputContextId::Battle,
            InputTarget::FreScope,
            InputCommand::Confirm,
            "Confirm",
        );
        let view_envelope = InputEnvelope::new(
            InputContextId::Battle,
            InputTarget::ActiveView,
            InputCommand::Confirm,
            "Confirm",
        );

        assert!(input_envelope_targets_behavior(&fre_envelope, "menu"));
        assert!(input_envelope_targets_behavior(&view_envelope, "menu"));
    }

    #[test]
    fn input_context_converts_to_wit_custom_context() {
        let context = input_context_to_wit(&InputContextId::Custom("pause".to_string()));

        assert!(matches!(context.kind, InputContextKind::Custom));
        assert_eq!(context.custom_name.as_deref(), Some("pause"));
    }

    #[test]
    fn input_command_converts_to_wit_navigation_command() {
        let command = input_command_to_wit(&InputCommand::Navigate(Direction::Left));

        assert!(matches!(
            command,
            WitInputCommand::Navigate(WitDirection::Left)
        ));
    }
}
