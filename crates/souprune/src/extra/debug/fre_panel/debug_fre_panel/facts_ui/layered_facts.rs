use super::super::{FREPanelState, FactLayerSelection, FactTypeSelection};
use super::list_editors::{edit_bool_list, edit_float_list, edit_int_list, edit_string_list};
use bevy::prelude::*;
use bevy_fact_rule_event::{FactValue, LayeredFactDatabase};
use bevy_inspector_egui::egui;

/// Render facts from LayeredFactDatabase.
pub(super) fn render_layered_facts(
    ui: &mut egui::Ui,
    world: &mut World,
    layer: FactLayerSelection,
    filter: &str,
) {
    let db = world.resource::<LayeredFactDatabase>();
    let facts: Vec<_> = match layer {
        FactLayerSelection::Global => db
            .global()
            .iter()
            .filter(|(k, _)| filter.is_empty() || k.contains(filter))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect(),
        FactLayerSelection::Local => db
            .local()
            .iter()
            .filter(|(k, _)| filter.is_empty() || k.contains(filter))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect(),
    };

    if facts.is_empty() {
        ui.label("(empty)");
        return;
    }

    let mut grouped: std::collections::BTreeMap<String, Vec<(String, String, FactValue)>> =
        std::collections::BTreeMap::new();

    for (key, value) in facts {
        let (namespace, short_key) = if let Some(pos) = key.find(':') {
            (key[..pos].to_string(), key[pos + 1..].to_string())
        } else {
            ("(ungrouped)".to_string(), key.clone())
        };
        grouped
            .entry(namespace)
            .or_default()
            .push((key, short_key, value));
    }

    let mut modifications: Vec<(String, FactValue, FactLayerSelection)> = Vec::new();

    for (namespace, facts_in_group) in &grouped {
        let header_text = format!("📁 {} ({})", namespace, facts_in_group.len());
        egui::CollapsingHeader::new(header_text)
            .default_open(namespace != "(ungrouped)" || grouped.len() == 1)
            .show(ui, |ui| {
                render_namespace_facts(ui, facts_in_group, layer, &mut modifications);
            });
    }

    for (key, value, layer) in modifications {
        let mut db = world.resource_mut::<LayeredFactDatabase>();
        match layer {
            FactLayerSelection::Global => db.set_global(key.as_str(), value),
            FactLayerSelection::Local => db.set_local(key.as_str(), value),
        }
    }
}

/// Render the add fact form.
pub(super) fn render_add_fact_form(ui: &mut egui::Ui, world: &mut World) {
    let state = world.resource::<FREPanelState>();
    let mut key = state.new_fact_key.clone();
    let mut value_str = state.new_fact_value_str.clone();
    let mut fact_type = state.new_fact_type;
    let mut fact_layer = state.new_fact_layer;

    ui.horizontal(|ui| {
        ui.label("Key:");
        ui.text_edit_singleline(&mut key);
    });

    ui.horizontal(|ui| {
        ui.label("Type:");
        egui::ComboBox::from_id_salt("fact_type")
            .selected_text(match fact_type {
                FactTypeSelection::Int => "Int",
                FactTypeSelection::Float => "Float",
                FactTypeSelection::Bool => "Bool",
                FactTypeSelection::String => "String",
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut fact_type, FactTypeSelection::Int, "Int");
                ui.selectable_value(&mut fact_type, FactTypeSelection::Float, "Float");
                ui.selectable_value(&mut fact_type, FactTypeSelection::Bool, "Bool");
                ui.selectable_value(&mut fact_type, FactTypeSelection::String, "String");
            });
    });

    ui.horizontal(|ui| {
        ui.label("Value:");
        ui.text_edit_singleline(&mut value_str);
    });

    ui.horizontal(|ui| {
        ui.label("Layer:");
        egui::ComboBox::from_id_salt("fact_layer")
            .selected_text(match fact_layer {
                FactLayerSelection::Local => "Local",
                FactLayerSelection::Global => "Global",
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut fact_layer, FactLayerSelection::Local, "Local");
                ui.selectable_value(&mut fact_layer, FactLayerSelection::Global, "Global");
            });
    });

    {
        let mut state = world.resource_mut::<FREPanelState>();
        state.new_fact_key = key.clone();
        state.new_fact_value_str = value_str.clone();
        state.new_fact_type = fact_type;
        state.new_fact_layer = fact_layer;
    }

    if ui.button("Add Fact").clicked() && !key.is_empty() {
        let value = match fact_type {
            FactTypeSelection::Int => value_str.parse::<i64>().ok().map(FactValue::Int),
            FactTypeSelection::Float => value_str.parse::<f64>().ok().map(FactValue::Float),
            FactTypeSelection::Bool => value_str.parse::<bool>().ok().map(FactValue::Bool),
            FactTypeSelection::String => Some(FactValue::String(value_str.clone())),
        };

        if let Some(value) = value {
            let mut db = world.resource_mut::<LayeredFactDatabase>();
            match fact_layer {
                FactLayerSelection::Local => db.set_local(key.as_str(), value),
                FactLayerSelection::Global => db.set_global(key.as_str(), value),
            }

            let mut state = world.resource_mut::<FREPanelState>();
            state.new_fact_key.clear();
            state.new_fact_value_str.clear();

            info!("Added fact: {} = {:?}", key, value_str);
        }
    }
}

/// Render facts within a namespace group.
fn render_namespace_facts(
    ui: &mut egui::Ui,
    facts_in_group: &[(String, String, FactValue)],
    layer: FactLayerSelection,
    modifications: &mut Vec<(String, FactValue, FactLayerSelection)>,
) {
    for (full_key, short_key, value) in facts_in_group {
        ui.horizontal(|ui| {
            ui.label(short_key);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                render_fact_value_editor(ui, full_key, value, layer, modifications);
            });
        });
    }
}

/// Render an editor widget for a single fact value.
fn render_fact_value_editor(
    ui: &mut egui::Ui,
    key: &str,
    value: &FactValue,
    layer: FactLayerSelection,
    modifications: &mut Vec<(String, FactValue, FactLayerSelection)>,
) {
    match value {
        FactValue::Int(v) => {
            let mut val = *v;
            if ui.add(egui::DragValue::new(&mut val)).changed() {
                modifications.push((key.to_string(), FactValue::Int(val), layer));
            }
        }
        FactValue::Float(v) => {
            let mut val = *v;
            if ui.add(egui::DragValue::new(&mut val).speed(0.1)).changed() {
                modifications.push((key.to_string(), FactValue::Float(val), layer));
            }
        }
        FactValue::Bool(v) => {
            let mut checked = *v;
            if ui.checkbox(&mut checked, "").changed() {
                modifications.push((key.to_string(), FactValue::Bool(checked), layer));
            }
        }
        FactValue::String(s) => {
            let mut text = s.clone();
            let response = ui.add(egui::TextEdit::singleline(&mut text).desired_width(150.0));
            if response.changed() {
                modifications.push((key.to_string(), FactValue::String(text), layer));
            }
        }
        FactValue::StringList(list) => render_list_string(ui, key, list, layer, modifications),
        FactValue::IntList(list) => render_list_int(ui, key, list, layer, modifications),
        FactValue::FloatList(list) => render_list_float(ui, key, list, layer, modifications),
        FactValue::BoolList(list) => render_list_bool(ui, key, list, layer, modifications),
    }
}

fn render_list_string(
    ui: &mut egui::Ui,
    key: &str,
    list: &[String],
    layer: FactLayerSelection,
    modifications: &mut Vec<(String, FactValue, FactLayerSelection)>,
) {
    ui.push_id(format!("layered_strlist_{}", key), |ui| {
        ui.collapsing(format!("[{}]", list.len()), |ui| {
            if let Some(new_list) = edit_string_list(ui, list) {
                modifications.push((key.to_string(), FactValue::StringList(new_list), layer));
            }
        });
    });
}

fn render_list_int(
    ui: &mut egui::Ui,
    key: &str,
    list: &[i64],
    layer: FactLayerSelection,
    modifications: &mut Vec<(String, FactValue, FactLayerSelection)>,
) {
    ui.push_id(format!("layered_intlist_{}", key), |ui| {
        ui.collapsing(format!("[{}]", list.len()), |ui| {
            if let Some(new_list) = edit_int_list(ui, list) {
                modifications.push((key.to_string(), FactValue::IntList(new_list), layer));
            }
        });
    });
}

fn render_list_float(
    ui: &mut egui::Ui,
    key: &str,
    list: &[f64],
    layer: FactLayerSelection,
    modifications: &mut Vec<(String, FactValue, FactLayerSelection)>,
) {
    ui.push_id(format!("layered_floatlist_{}", key), |ui| {
        ui.collapsing(format!("[{}]", list.len()), |ui| {
            if let Some(new_list) = edit_float_list(ui, list) {
                modifications.push((key.to_string(), FactValue::FloatList(new_list), layer));
            }
        });
    });
}

fn render_list_bool(
    ui: &mut egui::Ui,
    key: &str,
    list: &[bool],
    layer: FactLayerSelection,
    modifications: &mut Vec<(String, FactValue, FactLayerSelection)>,
) {
    ui.push_id(format!("layered_boollist_{}", key), |ui| {
        ui.collapsing(format!("[{}]", list.len()), |ui| {
            if let Some(new_list) = edit_bool_list(ui, list) {
                modifications.push((key.to_string(), FactValue::BoolList(new_list), layer));
            }
        });
    });
}
