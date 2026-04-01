//! Dispatch FRE custom actions to WASM mod handlers.
//!
//! FRE 自定义动作 → WASM mod 处理器的分发系统。

use bevy::prelude::*;
use bevy_fact_rule_event::{FactEvent, FactValue, LayeredFactDatabase};
use std::collections::HashMap;

use super::LoadedMods;
use crate::core::fre_bridge::FreCustomActionEvent;
use crate::core::wasm_runtime;

pub fn dispatch_wasm_custom_actions_system(
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
            let result =
                iface.call_handle_action(&mut loaded.store, &event.action_type, &wit_params);
            match result {
                Ok(false) => {
                    debug!("WASM mod did not handle action '{}'", event.action_type);
                }
                Ok(true) => {}
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

pub(super) fn build_fact_snapshot(fact_db: &LayeredFactDatabase) -> HashMap<String, FactValue> {
    fact_db
        .iter_local()
        .chain(fact_db.iter_global())
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect()
}

pub(super) fn apply_pending_side_effects(
    mutations: Vec<(String, FactValue)>,
    events: Vec<String>,
    fact_db: &mut LayeredFactDatabase,
    fact_writer: &mut MessageWriter<FactEvent>,
) {
    for (key, value) in mutations {
        fact_db.set(key, value);
    }
    for event_name in events {
        fact_writer.write(FactEvent::new(event_name));
    }
}
