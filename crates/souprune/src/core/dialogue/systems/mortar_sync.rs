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
    CachedCondition, MortarDialogueVariables, MortarEvent, MortarNumber, MortarRuntime,
    MortarValue, MortarVariableState, MortarVariableValue, evaluate_condition_cached,
    process_interpolated_text,
};

use super::lifecycle::DialogueControllerEntity;
use crate::core::fre_facts;

/// Configurable bindings between FRE facts and Mortar runtime functions/variables.
///
/// 可配置的 FRE 事实到 Mortar 运行时函数/变量的绑定。
///
/// The game/preset initializes this resource at startup to register mortar functions
/// and variables that read from specific FRE facts. This eliminates game-specific
/// knowledge from the dialogue system.
///
/// 游戏/preset 在启动时初始化此资源，注册从特定 FRE 事实读取数据的 mortar
/// 函数和变量。这消除了对话系统中的游戏特定知识。
#[derive(Resource, Default)]
pub struct MortarFactBindings {
    /// Number function bindings: (mortar_function_name, fact_key, default_value).
    /// Each entry registers a mortar function that returns the fact value as a number.
    pub number_functions: Vec<(String, String, f64)>,

    /// String variable bindings: (mortar_variable_name, fact_key, resolve_via_locale).
    /// If resolve_via_locale is true, the fact value is treated as a locale key and
    /// resolved via MortarStringTable; otherwise used directly as a string.
    pub string_variables: Vec<(String, String, bool)>,

    /// Number variable bindings: (mortar_variable_name, fact_key).
    /// Sets a mortar variable to the numeric value of the fact.
    pub number_variables: Vec<(String, String)>,
}

pub fn prepare_item_dialogue_mortar_system(
    mut runtime: ResMut<MortarRuntime>,
    facts: Res<bevy_fact_rule_event::LayeredFactDatabase>,
    mortar_strings: Res<crate::extra::mortar::MortarStringTable>,
    mut variables: ResMut<MortarDialogueVariables>,
    bindings: Option<Res<MortarFactBindings>>,
) {
    if !runtime.has_active_dialogues() {
        return;
    }

    // Core dialogue item facts — always available
    let heal_amount = facts
        .get_int(fre_facts::DIALOGUE_ITEM_HEAL_AMOUNT)
        .unwrap_or(0) as f64;
    let item_value = facts.get_int(fre_facts::DIALOGUE_ITEM_VALUE).unwrap_or(0) as f64;

    runtime.functions.register("get_heal_amount", move |_| {
        MortarValue::Number(MortarNumber(heal_amount))
    });
    runtime.functions.register("get_item_value", move |_| {
        MortarValue::Number(MortarNumber(item_value))
    });

    // Configurable bindings — registered by game/preset
    if let Some(bindings) = bindings.as_ref() {
        for (func_name, fact_key, default) in &bindings.number_functions {
            let value = facts
                .get_int(fact_key)
                .map(|v| v as f64)
                .unwrap_or(*default);
            let func_name = func_name.clone();
            runtime.functions.register(&func_name, move |_| {
                MortarValue::Number(MortarNumber(value))
            });
        }
    }

    let vs = variables.state.get_or_insert_with(MortarVariableState::new);

    // Core dialogue item variables
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

    // Configurable variable bindings
    if let Some(bindings) = bindings.as_ref() {
        for (var_name, fact_key, resolve_locale) in &bindings.string_variables {
            if let Some(value) = facts.get_string(fact_key) {
                let resolved = if *resolve_locale {
                    mortar_strings.resolve(value).to_string()
                } else {
                    value.to_string()
                };
                vs.set(var_name, MortarVariableValue::String(resolved));
            }
        }
        for (var_name, fact_key) in &bindings.number_variables {
            if let Some(value) = facts.get_int(fact_key) {
                vs.set(var_name, MortarVariableValue::Number(value as f64));
            }
        }
    }
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
