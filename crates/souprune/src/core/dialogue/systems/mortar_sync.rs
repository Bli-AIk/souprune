//! Synchronizes Mortar dialogue runtime data into the active typewriter presentation.
//!
//! 把 Mortar 对话运行时中的数据同步到当前正在显示的打字机表现上。
//!
//! Acts as the bridge from Mortar's dialogue model back into Souprune's UI.
//! It prepares item-dialogue variables/functions, evaluates Mortar conditions and
//! interpolated text, and updates the dialogue typewriter whenever Mortar moves
//! to a new piece of content.
//!
//! Mortar 对话模型回流到 Souprune UI 的桥。它会准备物品对话需要的
//! 变量和函数，评估 Mortar 的条件与插值文本，并在 Mortar 切换到新内容时更新
//! 对话打字机。

use bevy::prelude::*;
use bevy_ecs_typewriter::Typewriter;
use bevy_mortar_bond::{
    CachedCondition, MortarDialogueVariables, MortarEvent, MortarRuntime, MortarVariableState,
    MortarVariableValue, evaluate_condition_cached, process_interpolated_text,
};

use super::lifecycle::DialogueControllerEntity;
use crate::core::fre_facts;

pub fn prepare_item_dialogue_mortar_system(
    mut runtime: ResMut<MortarRuntime>,
    facts: Res<bevy_fact_rule_event::LayeredFactDatabase>,
    mortar_strings: Res<crate::extra::mortar::MortarStringTable>,
    mut variables: ResMut<MortarDialogueVariables>,
) {
    if !runtime.has_active_dialogues() {
        return;
    }

    let hp = facts.get_int(fre_facts::PLAYER_HP).unwrap_or(20) as f64;
    let hp_max = facts.get_int(fre_facts::PLAYER_HP_MAX).unwrap_or(20) as f64;
    let heal_amount = facts
        .get_int(fre_facts::DIALOGUE_ITEM_HEAL_AMOUNT)
        .unwrap_or(0) as f64;
    let item_value = facts.get_int(fre_facts::DIALOGUE_ITEM_VALUE).unwrap_or(0) as f64;

    use bevy_mortar_bond::{MortarNumber, MortarValue};
    runtime.functions.register("get_player_hp", move |_| {
        MortarValue::Number(MortarNumber(hp))
    });
    runtime.functions.register("get_player_hp_max", move |_| {
        MortarValue::Number(MortarNumber(hp_max))
    });
    runtime.functions.register("get_heal_amount", move |_| {
        MortarValue::Number(MortarNumber(heal_amount))
    });
    runtime.functions.register("get_item_value", move |_| {
        MortarValue::Number(MortarNumber(item_value))
    });

    let vs = variables.state.get_or_insert_with(MortarVariableState::new);
    if let Some(locale_key) = facts.get_string(fre_facts::DIALOGUE_ITEM_NAME) {
        let display_name = mortar_strings.resolve(locale_key).to_string();
        vs.set("item_name", MortarVariableValue::String(display_name));
    }
    if let Some(desc) = facts.get_string(fre_facts::DIALOGUE_ITEM_DESCRIPTION) {
        vs.set(
            "item_description",
            MortarVariableValue::String(desc.to_string()),
        );
    }
    vs.set("heal_amount", MortarVariableValue::Number(heal_amount));
}

pub fn sync_mortar_text_to_typewriter_system(
    runtime: Res<MortarRuntime>,
    variables: Option<Res<MortarDialogueVariables>>,
    mut query: Query<&mut Typewriter, With<DialogueControllerEntity>>,
    mut mortar_events: MessageWriter<MortarEvent>,
    mut cached_condition: Local<Option<CachedCondition>>,
) {
    let Some(state) = runtime.primary_dialogue_state() else {
        *cached_condition = None;
        return;
    };

    let default_vs = MortarVariableState::new();
    let variable_state = variables
        .as_ref()
        .and_then(|v| v.state.as_ref())
        .unwrap_or(&default_vs);

    let Some(text_data) = state.current_text_data() else {
        return;
    };

    let new_text = if text_data.is_line {
        let group = state.current_line_group().unwrap_or(&[]);
        let mut result_lines = Vec::new();
        for line_data in group {
            if let Some(condition) = &line_data.condition
                && !evaluate_condition_cached(
                    condition,
                    &runtime.functions,
                    variable_state,
                    &mut cached_condition,
                )
            {
                continue;
            }
            let line_text =
                process_interpolated_text(line_data, &runtime.functions, &[], variable_state);
            if !line_text.is_empty() {
                result_lines.push(line_text);
            }
        }
        if result_lines.is_empty() {
            mortar_events.write(MortarEvent::next_text());
            return;
        }
        result_lines.join("\n")
    } else {
        if let Some(condition) = &text_data.condition {
            let result = evaluate_condition_cached(
                condition,
                &runtime.functions,
                variable_state,
                &mut cached_condition,
            );
            if !result {
                mortar_events.write(MortarEvent::next_text());
                return;
            }
        }
        process_interpolated_text(text_data, &runtime.functions, &[], variable_state)
    };

    for mut typewriter in &mut query {
        if typewriter.source_text != new_text {
            info!(
                "[DEBUG] sync_mortar: setting typewriter text (is_line={}, lines={}): '{}'",
                text_data.is_line,
                new_text.matches('\n').count() + 1,
                new_text
            );
            typewriter.source_text = new_text.clone();
            typewriter.current_text.clear();
            typewriter.current_char_index = 0;
            typewriter.play();
        }
    }
}
