//! Dispatch FRE custom actions to WASM mod handlers.
//!
//! FRE 自定义动作 → WASM mod 处理器的分发系统。

use bevy::prelude::*;
use bevy_fact_rule_event::{FactEvent, FactValue, LayeredFactDatabase};
use std::collections::HashMap;

use std::sync::Arc;

use super::LoadedMods;
use super::audio_effects::PendingAudioEffects;
use super::host_entities::PendingHostEntityEffects;
use crate::core::fre_bridge::FreCustomActionEvent;
use crate::core::wasm_runtime;

pub fn dispatch_wasm_custom_actions_system(
    mut events: MessageReader<FreCustomActionEvent>,
    mut loaded_mods: NonSendMut<LoadedMods>,
    mut fact_db: ResMut<LayeredFactDatabase>,
    mut fact_writer: MessageWriter<FactEvent>,
    mut wasm_tracer: ResMut<crate::core::trace::WasmCallTracer>,
    mut fact_history: ResMut<crate::core::trace::FactChangeHistory>,
    mut host_entity_effects: ResMut<PendingHostEntityEffects>,
    mut audio_effects: ResMut<PendingAudioEffects>,
    frame_count: Res<bevy::diagnostic::FrameCount>,
) {
    let events: Vec<FreCustomActionEvent> = events.read().cloned().collect();
    if events.is_empty() {
        return;
    }

    let base_fact_snapshot = build_fact_snapshot(&fact_db);

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
                ctx.call_ctx.fact_snapshot =
                    build_event_fact_snapshot(&base_fact_snapshot, &event.local_facts);
                ctx.call_ctx.clear_pending_side_effects();
            }

            let iface = loaded.bindings.souprune_plugin_custom_action_handler();
            let start = std::time::Instant::now();
            let result =
                iface.call_handle_action(&mut loaded.store, &event.action_type, &wit_params);
            let elapsed = start.elapsed();
            wasm_tracer.record(&loaded.name, "custom-action", "handle_action", elapsed);
            match result {
                Ok(false) => {
                    debug!("WASM mod did not handle action '{}'", event.action_type);
                }
                Ok(true) => {}
                Err(e) => {
                    error!("WASM handle-action '{}' failed: {:?}", event.action_type, e);
                }
            }

            let pending = loaded.store.data_mut().call_ctx.take_pending_side_effects();
            apply_pending_side_effects(
                pending,
                &mut fact_db,
                &mut fact_writer,
                &mut fact_history,
                &mut host_entity_effects,
                &mut audio_effects,
                frame_count.0 as u64,
                &format!("wasm:{}", loaded.name),
            );
        }
    }
}

pub(super) fn build_fact_snapshot(
    fact_db: &LayeredFactDatabase,
) -> Arc<HashMap<String, FactValue>> {
    Arc::new(
        fact_db
            .iter_global()
            .chain(fact_db.iter_local())
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect(),
    )
}

fn build_event_fact_snapshot(
    base: &Arc<HashMap<String, FactValue>>,
    local_facts: &HashMap<String, FactValue>,
) -> Arc<HashMap<String, FactValue>> {
    if local_facts.is_empty() {
        return Arc::clone(base);
    }

    let mut merged = (**base).clone();
    merged.extend(
        local_facts
            .iter()
            .map(|(key, value)| (key.clone(), value.clone())),
    );
    Arc::new(merged)
}

pub(super) fn apply_pending_side_effects(
    pending: wasm_runtime::PendingSideEffects,
    fact_db: &mut LayeredFactDatabase,
    fact_writer: &mut MessageWriter<FactEvent>,
    fact_history: &mut crate::core::trace::FactChangeHistory,
    host_entity_effects: &mut PendingHostEntityEffects,
    audio_effects: &mut PendingAudioEffects,
    frame_number: u64,
    caused_by: &str,
) {
    for (key, value) in pending.fact_mutations {
        let old = fact_db.get_by_str(&key).cloned();
        fact_history.record(&key, old, value.clone(), caused_by, frame_number);
        fact_db.set(key, value);
    }
    for (key, value) in pending.global_fact_mutations {
        let old = fact_db.get_by_str(&key).cloned();
        fact_history.record(&key, old, value.clone(), caused_by, frame_number);
        fact_db.set_global(key, value);
    }
    for event_name in pending.events {
        fact_writer.write(FactEvent::new(event_name));
    }
    audio_effects.extend(pending.sounds);
    host_entity_effects.extend(pending.host_effects);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_fact_snapshot_overlays_view_local_facts() {
        let mut fact_db = LayeredFactDatabase::new();
        fact_db.set_global("selection", FactValue::Int(0));
        fact_db.set_global("player:hp", FactValue::Int(10));
        let base = build_fact_snapshot(&fact_db);
        let local = HashMap::from([("selection".to_string(), FactValue::Int(2))]);

        let snapshot = build_event_fact_snapshot(&base, &local);

        assert_eq!(snapshot.get("selection"), Some(&FactValue::Int(2)));
        assert_eq!(snapshot.get("player:hp"), Some(&FactValue::Int(10)));
    }
}
