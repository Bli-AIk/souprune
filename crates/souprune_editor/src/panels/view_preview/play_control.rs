//! # play_control.rs
//!
//! # play_control.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This file owns the "Play mode" control logic for the View preview panel. It decides when the
//! preview should enter or leave simulated runtime mode, forwards keyboard input into FRE events,
//! and initializes the temporary `ViewRoot` state that the preview needs.
//!
//! 这个文件负责 View 预览面板里的“播放模式”控制逻辑。它决定预览何时进入或退出模拟运行时，
//! 把键盘输入转发成 FRE 事件，并初始化预览临时使用的 `ViewRoot` 状态。

use super::*;
use std::collections::HashMap;

/// 预览输入映射资源：KeyCode → 动作名称。
#[derive(Resource)]
pub struct ViewPreviewKeyMap(pub HashMap<KeyCode, String>);

/// Forward keyboard input to FRE events when preview is hovered during Play mode.
pub fn preview_input_to_fre_system(
    state: Res<ViewPreviewState>,
    key_map: Res<ViewPreviewKeyMap>,
    keys: Res<ButtonInput<KeyCode>>,
    mut event_writer: MessageWriter<bevy_fact_rule_event::FactEvent>,
) {
    if !state.playing || !state.hovered {
        return;
    }

    for (keycode, action_name) in key_map.0.iter() {
        let action_lower = action_name.to_lowercase();

        if keys.just_pressed(*keycode) {
            let event_id = format!("action:{}:just_pressed", action_lower);
            info!(
                "[ViewPreview] FRE: {} just_pressed → {}",
                action_name, event_id
            );
            event_writer.write(bevy_fact_rule_event::FactEvent::new(event_id));
        }

        if keys.just_released(*keycode) {
            let event_id = format!("action:{}:just_released", action_lower);
            debug!(
                "[ViewPreview] FRE: {} just_released → {}",
                action_name, event_id
            );
            event_writer.write(bevy_fact_rule_event::FactEvent::new(event_id));
        }
    }
}

fn load_initial_facts_into_view_root(view_root: &mut ViewRoot, layout: &SchemaViewLayoutAsset) {
    let Some(facts) = &layout.facts else { return };
    for (key, value) in facts {
        match value {
            InitialFactValue::Int(i) => view_root.local_facts.set(key.clone(), *i),
            InitialFactValue::Float(f) => view_root.local_facts.set(key.clone(), *f),
            InitialFactValue::Bool(b) => view_root.local_facts.set(key.clone(), *b),
            InitialFactValue::String(s) => view_root.local_facts.set(key.clone(), s.clone()),
            InitialFactValue::StringList(list) => {
                view_root.local_facts.set(key.clone(), list.clone());
            }
            InitialFactValue::IntList(list) => {
                view_root.local_facts.set(key.clone(), list.clone());
            }
        }
    }
    info!(
        "[ViewPreview] Loaded {} inline facts from view layout",
        facts.len()
    );
}

fn register_rule_def(
    rule_def: &game_action::GameRuleDef,
    idx: usize,
    scope: bevy_fact_rule_event::RuleScope,
    rule_registry: &mut game_action::GameRuleRegistry,
    view_entity: Entity,
    registered_ids: &mut Vec<String>,
) {
    let effective_scope = if scope == bevy_fact_rule_event::RuleScope::Local {
        bevy_fact_rule_event::RuleScope::View
    } else {
        scope
    };
    let rule = rule_def.to_rule_with_index(idx, effective_scope);
    let rule_id = rule_def.generate_id(idx);
    if effective_scope == bevy_fact_rule_event::RuleScope::View {
        rule_registry.register_view_rule(view_entity, rule);
    } else {
        rule_registry.register(rule);
    }
    registered_ids.push(rule_id);
}

/// 检测 Play/Stop 状态变化，执行 FRE 初始化或清理。
pub fn preview_play_control_system(
    mut state: ResMut<ViewPreviewState>,
    editor_state: Res<ViewEditorState>,
    fre_state: Option<Res<super::super::view_fre_panel::ViewFreState>>,
    mut rule_registry: ResMut<game_action::GameRuleRegistry>,
    mut fact_db: ResMut<bevy_fact_rule_event::LayeredFactDatabase>,
    mortar_strings: Res<mortar::MortarStringTable>,
    mut commands: Commands,
    mut view_roots: Query<(Entity, &mut ViewRoot)>,
    enum_registry: Res<bevy_fact_rule_event::EnumRegistry>,
) {
    let playing = state.playing;
    let was_playing = state.was_playing;

    if playing == was_playing {
        return;
    }
    state.was_playing = playing;

    if playing && !was_playing {
        let Some(fre_state) = fre_state else {
            warn!("[ViewPreview] Play: ViewFreState not found");
            state.playing = false;
            state.was_playing = false;
            return;
        };

        fact_db.clear_local();

        let mut found_entity = None;
        for e in &state.preview_entities {
            if view_roots.get(*e).is_ok() {
                found_entity = Some(*e);
                break;
            }
        }

        let Some(view_entity) = found_entity else {
            warn!("[ViewPreview] Play: no preview ViewRoot entity found");
            state.playing = false;
            state.was_playing = false;
            return;
        };

        let mut view_root = view_roots.get_mut(view_entity).unwrap().1;
        view_root.local_facts = bevy_fact_rule_event::FactDatabase::default();

        if let Some(layout) = &editor_state.layout {
            load_initial_facts_into_view_root(&mut view_root, layout);
        }

        let mut registered_ids = Vec::new();
        for fre_asset in fre_state.loaded_fre.values() {
            load_fre_into_view_root(&mut view_root, fre_asset, &mortar_strings, &enum_registry);

            let rule_defs = fre_asset.get_rule_defs();
            let scope = fre_asset.scope();
            for (idx, rule_def) in rule_defs.iter().enumerate() {
                register_rule_def(
                    rule_def,
                    idx,
                    scope,
                    &mut rule_registry,
                    view_entity,
                    &mut registered_ids,
                );
            }
        }

        commands.entity(view_entity).insert(ActiveView);

        info!(
            "[ViewPreview] Play started: registered {} rules, local_facts initialized",
            registered_ids.len()
        );
        state.registered_rule_ids = registered_ids;
    } else if !playing && was_playing {
        for entity in &state.preview_entities {
            if view_roots.get(*entity).is_ok() {
                rule_registry.clear_view(*entity);
                commands.entity(*entity).remove::<ActiveView>();
            }
        }

        state.registered_rule_ids.clear();
        fact_db.clear_local();

        for entity in &state.preview_entities {
            if let Ok((_, mut view_root)) = view_roots.get_mut(*entity) {
                view_root.local_facts = bevy_fact_rule_event::FactDatabase::default();
            }
        }

        state.last_layout_hash = 0;
        info!("[ViewPreview] Play stopped: rules cleaned up, preview will rebuild");
    }
}
