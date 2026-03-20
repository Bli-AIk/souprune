use bevy::prelude::*;
use bevy_fact_rule_event::{FactEvent, FactValue, LayeredFactDatabase};
use leafwing_input_manager::action_state::ActionState;

use crate::core::fre_facts;
use crate::core::input::{Action, ActionRegistry, ActionStateExt};
use crate::core::mode::{AppState, SequenceSubState};

pub(crate) fn state_facts_need_sync(
    sub_state: Option<Res<State<SequenceSubState>>>,
    app_state: Option<Res<State<AppState>>>,
    facts: Res<LayeredFactDatabase>,
) -> bool {
    if let Some(ref state) = sub_state
        && state.is_changed()
    {
        return true;
    }
    if let Some(ref state) = app_state
        && state.is_changed()
    {
        return true;
    }
    if sub_state.is_some()
        && facts
            .get_by_str(fre_facts::STATE_SEQUENCE_SUB_STATE)
            .is_none()
    {
        return true;
    }
    if app_state.is_some() && facts.get_by_str(fre_facts::STATE_APP_STATE).is_none() {
        return true;
    }
    false
}

pub fn sync_state_to_facts_system(
    sub_state: Option<Res<State<SequenceSubState>>>,
    app_state: Option<Res<State<AppState>>>,
    mut facts: ResMut<LayeredFactDatabase>,
) {
    if let Some(state) = sub_state {
        let state_name = state.get().name().to_string();
        facts.set(
            fre_facts::STATE_SEQUENCE_SUB_STATE,
            FactValue::String(state_name),
        );
    }

    if let Some(state) = app_state {
        let state_name = format!("{:?}", state.get());
        facts.set(fre_facts::STATE_APP_STATE, FactValue::String(state_name));
    }
}

pub fn action_to_fre_event_system(
    registry: Res<ActionRegistry>,
    query: Query<&ActionState<Action>>,
    mut event_writer: MessageWriter<FactEvent>,
) {
    let Some(action_state) = query.iter().next() else {
        return;
    };

    for action_name in registry.all_actions() {
        let action_name_lower = action_name.to_lowercase();

        if action_state.action_just_pressed(&registry, action_name) {
            let event_id = format!("action:{}:just_pressed", action_name_lower);
            info!(
                "FRE Bridge: {} just_pressed, emitting {}",
                action_name, event_id
            );
            event_writer.write(FactEvent::new(event_id));
        }

        if action_state.action_just_released(&registry, action_name) {
            let event_id = format!("action:{}:just_released", action_name_lower);
            debug!(
                "FRE Bridge: {} just_released, emitting {}",
                action_name, event_id
            );
            event_writer.write(FactEvent::new(event_id));
        }
    }
}
