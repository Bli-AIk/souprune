//! Mirrors runtime app state into FRE facts and emits state-change FRE events.
//!
//! 把运行时状态同步进 FRE 事实，同时产生状态变化对应的 FRE 事件。
//!
//! Keeps the rule engine aware of which app state the game is currently in.
//! Input is bridged through `fre_bridge::input` from unified input envelopes.
//!
//! 让规则引擎持续感知游戏当前处于哪个应用状态。
//! 输入则由 `fre_bridge::input` 从统一输入事务桥接。

use bevy::prelude::*;
use bevy_fact_rule_event::{FactEvent, FactValue, LayeredFactDatabase};

use crate::core::fre_facts;
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
        facts.set_if_changed(
            fre_facts::STATE_SEQUENCE_SUB_STATE,
            FactValue::String(state_name),
        );
    }

    if let Some(state) = app_state {
        let state_name = format!("{:?}", state.get());
        facts.set_if_changed(fre_facts::STATE_APP_STATE, FactValue::String(state_name));
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
