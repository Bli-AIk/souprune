//! Mirrors runtime app/input state into FRE facts and emits FRE input events.
//!
//! 把运行时的状态与输入同步进 FRE 事实，同时产生对应的 FRE 输入事件。
//!
//! Keeps the rule engine aware of two moving pieces: which app state
//! the game is currently in, and which high-level actions were just pressed or
//! released. Without this synchronization layer, FRE rules could not react to
//! mode changes or to the input abstraction built by Souprune's action system.
//!
//! 让规则引擎持续感知两类变化：游戏当前处于哪个应用状态，以及哪些
//! 高层输入动作刚刚被按下或释放。没有这层同步，FRE 规则就无法对模式变化
//! 或 Souprune 输入系统抽象出来的动作作出响应。

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

/// Emit FRE events when the active SequenceMode changes.
///
/// 当 SequenceMode 变化时发出 FRE 事件。
/// WASM mods 可通过 FRE 规则和 custom-action-handler 响应模式切换。
pub fn mode_change_to_fre_event_system(
    mut mode_events: MessageReader<crate::core::mode::ModeChanged>,
    mut fre_events: MessageWriter<FactEvent>,
    mut facts: ResMut<LayeredFactDatabase>,
) {
    use bevy_fact_rule_event::FactValue;
    for event in mode_events.read() {
        if let Some(ref from) = event.from {
            let exit_event = format!("mode:{}:exit", from);
            info!("FRE Bridge: mode exit → {}", exit_event);
            fre_events.write(FactEvent::new(exit_event));
        }
        if let Some(ref to) = event.to {
            let enter_event = format!("mode:{}:enter", to);
            info!("FRE Bridge: mode enter → {}", enter_event);
            fre_events.write(FactEvent::new(enter_event));
            facts.set("state_mode", FactValue::String(to.clone()));
        } else {
            facts.set("state_mode", FactValue::String(String::new()));
        }
    }
}
