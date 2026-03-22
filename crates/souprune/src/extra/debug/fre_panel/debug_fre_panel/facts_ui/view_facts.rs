use super::super::FREPanelState;
use super::list_editors::{edit_bool_list, edit_float_list, edit_int_list, edit_string_list};
use bevy::prelude::*;
use bevy_fact_rule_event::FactValue;
use bevy_inspector_egui::egui;

/// Render the View Local Facts tab.
pub(super) fn render_view_facts_tab(ui: &mut egui::Ui, world: &mut World) {
    use crate::core::view::components::ViewRoot;

    let search_filter = world.resource::<FREPanelState>().search_filter.clone();
    let mut new_filter = search_filter.clone();
    ui.horizontal(|ui| {
        ui.label("🔍");
        if ui.text_edit_singleline(&mut new_filter).changed() {
            world.resource_mut::<FREPanelState>().search_filter = new_filter.clone();
        }
    });

    ui.separator();

    let mut view_roots: Vec<(Entity, String, String, Vec<(String, FactValue)>)> = Vec::new();
    {
        let mut query = world.query::<(Entity, &ViewRoot, Option<&Name>)>();
        for (entity, view_root, name) in query.iter(world) {
            let display_name = name
                .map(|n| n.to_string())
                .unwrap_or_else(|| format!("Entity {:?}", entity));

            let facts: Vec<_> = view_root
                .local_facts
                .iter()
                .filter(|(k, _)| search_filter.is_empty() || k.contains(&search_filter))
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();

            view_roots.push((entity, display_name, view_root.namespace.clone(), facts));
        }
    }

    egui::ScrollArea::vertical().show(ui, |ui| {
        if view_roots.is_empty() {
            ui.label("⚠ No active View instances found.");
            ui.label("Views with local_facts will appear here when loaded.");
            return;
        }

        ui.label(format!("📊 {} active View instance(s)", view_roots.len()));
        ui.separator();

        let mut modifications: Vec<(Entity, String, FactValue)> = Vec::new();

        for (entity, display_name, namespace, facts) in &view_roots {
            render_view_root_facts(
                ui,
                *entity,
                display_name,
                namespace,
                facts,
                &mut modifications,
            );
        }

        for (entity, key, value) in modifications {
            if let Some(mut view_root) = world.get_mut::<ViewRoot>(entity) {
                view_root.local_facts.set(key.as_str(), value);
            }
        }
    });
}

fn render_view_root_facts(
    ui: &mut egui::Ui,
    entity: Entity,
    display_name: &str,
    namespace: &str,
    facts: &[(String, FactValue)],
    modifications: &mut Vec<(Entity, String, FactValue)>,
) {
    let header_text = format!("🖼 {} ({})", display_name, namespace);
    egui::CollapsingHeader::new(header_text)
        .default_open(true)
        .show(ui, |ui| {
            if facts.is_empty() {
                ui.label("(no local facts)");
                return;
            }

            for (key, value) in facts {
                render_view_fact_row(ui, entity, key, value, modifications);
            }
        });
}

fn render_view_fact_row(
    ui: &mut egui::Ui,
    entity: Entity,
    key: &str,
    value: &FactValue,
    modifications: &mut Vec<(Entity, String, FactValue)>,
) {
    let key_label = if key.starts_with("view.") {
        format!("  {}", key)
    } else {
        format!("  ${}", key)
    };
    ui.horizontal(|ui| {
        ui.label(&key_label);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            render_view_fact_value(ui, entity, key, value, modifications)
        });
    });
}

fn render_view_fact_value(
    ui: &mut egui::Ui,
    entity: Entity,
    key: &str,
    value: &FactValue,
    modifications: &mut Vec<(Entity, String, FactValue)>,
) {
    match value {
        FactValue::Int(v) => {
            let mut val = *v;
            if ui.add(egui::DragValue::new(&mut val)).changed() {
                modifications.push((entity, key.to_string(), FactValue::Int(val)));
            }
        }
        FactValue::Float(v) => {
            let mut val = *v;
            if ui.add(egui::DragValue::new(&mut val).speed(0.1)).changed() {
                modifications.push((entity, key.to_string(), FactValue::Float(val)));
            }
        }
        FactValue::Bool(v) => {
            let mut checked = *v;
            if ui.checkbox(&mut checked, "").changed() {
                modifications.push((entity, key.to_string(), FactValue::Bool(checked)));
            }
        }
        FactValue::String(s) => {
            let mut text = s.clone();
            let response = ui.add(egui::TextEdit::singleline(&mut text).desired_width(150.0));
            if response.changed() {
                modifications.push((entity, key.to_string(), FactValue::String(text)));
            }
        }
        FactValue::StringList(list) => render_list_string(ui, entity, key, list, modifications),
        FactValue::IntList(list) => render_list_int(ui, entity, key, list, modifications),
        FactValue::FloatList(list) => render_list_float(ui, entity, key, list, modifications),
        FactValue::BoolList(list) => render_list_bool(ui, entity, key, list, modifications),
    }
}

fn render_list_string(
    ui: &mut egui::Ui,
    entity: Entity,
    key: &str,
    list: &[String],
    modifications: &mut Vec<(Entity, String, FactValue)>,
) {
    ui.push_id(format!("strlist_{}", key), |ui| {
        ui.collapsing(format!("[{}]", list.len()), |ui| {
            if let Some(new_list) = edit_string_list(ui, list) {
                modifications.push((entity, key.to_string(), FactValue::StringList(new_list)));
            }
        });
    });
}

fn render_list_int(
    ui: &mut egui::Ui,
    entity: Entity,
    key: &str,
    list: &[i64],
    modifications: &mut Vec<(Entity, String, FactValue)>,
) {
    ui.push_id(format!("intlist_{}", key), |ui| {
        ui.collapsing(format!("[{}]", list.len()), |ui| {
            if let Some(new_list) = edit_int_list(ui, list) {
                modifications.push((entity, key.to_string(), FactValue::IntList(new_list)));
            }
        });
    });
}

fn render_list_float(
    ui: &mut egui::Ui,
    entity: Entity,
    key: &str,
    list: &[f64],
    modifications: &mut Vec<(Entity, String, FactValue)>,
) {
    ui.push_id(format!("floatlist_{}", key), |ui| {
        ui.collapsing(format!("[{}]", list.len()), |ui| {
            if let Some(new_list) = edit_float_list(ui, list) {
                modifications.push((entity, key.to_string(), FactValue::FloatList(new_list)));
            }
        });
    });
}

fn render_list_bool(
    ui: &mut egui::Ui,
    entity: Entity,
    key: &str,
    list: &[bool],
    modifications: &mut Vec<(Entity, String, FactValue)>,
) {
    ui.push_id(format!("boollist_{}", key), |ui| {
        ui.collapsing(format!("[{}]", list.len()), |ui| {
            if let Some(new_list) = edit_bool_list(ui, list) {
                modifications.push((entity, key.to_string(), FactValue::BoolList(new_list)));
            }
        });
    });
}
