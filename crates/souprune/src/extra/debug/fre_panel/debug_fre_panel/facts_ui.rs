//! Facts-related UI rendering for the FRE debug panel.
//! FRE 调试面板的事实相关 UI 渲染。

use super::{
    FREPanelState, FactEventHistory, FactEventRecord, FactLayerSelection, FactTypeSelection,
    MAX_EVENT_HISTORY,
};
use bevy::prelude::*;
use bevy_fact_rule_event::{FactValue, LayeredFactDatabase};
use bevy_inspector_egui::egui;

/// Render the Facts tab.
pub(super) fn render_facts_tab(ui: &mut egui::Ui, world: &mut World) {
    // Search filter
    let mut search_filter = world.resource::<FREPanelState>().search_filter.clone();
    ui.horizontal(|ui| {
        ui.label("🔍");
        if ui.text_edit_singleline(&mut search_filter).changed() {
            world.resource_mut::<FREPanelState>().search_filter = search_filter.clone();
        }
    });

    ui.separator();

    // Check if LayeredFactDatabase exists
    let has_layered = world.get_resource::<LayeredFactDatabase>().is_some();

    egui::ScrollArea::vertical().show(ui, |ui| {
        if !has_layered {
            ui.label("⚠️ No LayeredFactDatabase found.");
            ui.label("FRE system may not be initialized.");
            return;
        }

        // Show LayeredFactDatabase with Global/Local layers
        egui::CollapsingHeader::new("🌍 Global Layer")
            .default_open(true)
            .show(ui, |ui| {
                render_layered_facts(ui, world, FactLayerSelection::Global, &search_filter);
            });

        egui::CollapsingHeader::new("🎮 Local Layer")
            .default_open(true)
            .show(ui, |ui| {
                render_layered_facts(ui, world, FactLayerSelection::Local, &search_filter);
            });

        ui.separator();

        // Add new fact form
        egui::CollapsingHeader::new("➕ Add New Fact").show(ui, |ui| {
            render_add_fact_form(ui, world, true);
        });
    });
}

/// Render the View Local Facts tab.
/// Shows local facts from all active ViewRoot components.
///
/// 渲染 View 局部事实选项卡。
/// 显示所有活跃 ViewRoot 组件的局部事实。
pub(super) fn render_view_facts_tab(ui: &mut egui::Ui, world: &mut World) {
    use crate::core::view::components::ViewRoot;

    // Search filter
    let search_filter = world.resource::<FREPanelState>().search_filter.clone();
    let mut new_filter = search_filter.clone();
    ui.horizontal(|ui| {
        ui.label("🔍");
        if ui.text_edit_singleline(&mut new_filter).changed() {
            world.resource_mut::<FREPanelState>().search_filter = new_filter.clone();
        }
    });

    ui.separator();

    // Query all ViewRoot components
    let mut view_roots: Vec<(Entity, String, String, Vec<(String, FactValue)>)> = Vec::new();

    // Use a scope to avoid borrowing world mutably while iterating
    {
        let mut query = world.query::<(Entity, &ViewRoot, Option<&Name>)>();
        for (entity, view_root, name) in query.iter(world) {
            let display_name = name
                .map(|n| n.to_string())
                .unwrap_or_else(|| format!("Entity {:?}", entity));

            let facts: Vec<_> = view_root
                .local_facts
                .iter()
                .filter(|(k, _)| search_filter.is_empty() || k.0.contains(&search_filter))
                .map(|(k, v)| (k.0.clone(), v.clone()))
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

        // Collect modifications to apply after iteration
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

        // Apply modifications to ViewRoot local_facts
        for (entity, key, value) in modifications {
            if let Some(mut view_root) = world.get_mut::<ViewRoot>(entity) {
                view_root.local_facts.set(key.as_str(), value);
            }
        }
    });
}

/// Render facts for a single ViewRoot entity.
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

/// Render a single fact row in the view facts tab.
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

/// Render the value editor for a single view fact.
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
        FactValue::StringList(list) => {
            render_view_list_string(ui, entity, key, list, modifications)
        }
        FactValue::IntList(list) => render_view_list_int(ui, entity, key, list, modifications),
        FactValue::FloatList(list) => render_view_list_float(ui, entity, key, list, modifications),
        FactValue::BoolList(list) => render_view_list_bool(ui, entity, key, list, modifications),
    }
}

/// Render an editable string list for view facts.
fn render_view_list_string(
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

/// Render an editable int list for view facts.
fn render_view_list_int(
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

/// Render an editable float list for view facts.
fn render_view_list_float(
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

/// Render an editable bool list for view facts.
fn render_view_list_bool(
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
            .filter(|(k, _)| filter.is_empty() || k.0.contains(filter))
            .map(|(k, v)| (k.0.clone(), v.clone()))
            .collect(),
        FactLayerSelection::Local => db
            .local()
            .iter()
            .filter(|(k, _)| filter.is_empty() || k.0.contains(filter))
            .map(|(k, v)| (k.0.clone(), v.clone()))
            .collect(),
    };

    if facts.is_empty() {
        ui.label("(empty)");
        return;
    }

    // Group facts by namespace (part before the colon)
    // 按命名空间分组（冒号前的部分）
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

    // Render each namespace group
    // 渲染每个命名空间分组
    for (namespace, facts_in_group) in &grouped {
        let header_text = format!("📁 {} ({})", namespace, facts_in_group.len());
        egui::CollapsingHeader::new(header_text)
            .default_open(namespace != "(ungrouped)" || grouped.len() == 1)
            .show(ui, |ui| {
                render_namespace_facts(ui, facts_in_group, layer, &mut modifications);
            });
    }

    // Apply modifications to LayeredFactDatabase
    for (key, value, layer) in modifications {
        let mut db = world.resource_mut::<LayeredFactDatabase>();
        match layer {
            FactLayerSelection::Global => db.set_global(key.as_str(), value),
            FactLayerSelection::Local => db.set_local(key.as_str(), value),
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
/// 渲染单个事实值的编辑器组件。
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
        FactValue::StringList(list) => {
            render_layered_list_string(ui, key, list, layer, modifications)
        }
        FactValue::IntList(list) => render_layered_list_int(ui, key, list, layer, modifications),
        FactValue::FloatList(list) => {
            render_layered_list_float(ui, key, list, layer, modifications)
        }
        FactValue::BoolList(list) => render_layered_list_bool(ui, key, list, layer, modifications),
    }
}

/// Render an editable string list for layered facts.
fn render_layered_list_string(
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

/// Render an editable int list for layered facts.
fn render_layered_list_int(
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

/// Render an editable float list for layered facts.
fn render_layered_list_float(
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

/// Render an editable bool list for layered facts.
fn render_layered_list_bool(
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

/// Shared editable string list UI. Returns `Some(new_list)` if changed.
fn edit_string_list(ui: &mut egui::Ui, list: &[String]) -> Option<Vec<String>> {
    let mut new_list = list.to_vec();
    let mut changed = false;
    let mut to_remove: Option<usize> = None;

    for (idx, item) in new_list.iter_mut().enumerate() {
        ui.horizontal(|ui| {
            ui.label(format!("  {}:", idx));
            let response = ui.add(egui::TextEdit::singleline(item).desired_width(120.0));
            if response.changed() {
                changed = true;
            }
            if ui.small_button("🗑").clicked() {
                to_remove = Some(idx);
            }
        });
    }

    if let Some(idx) = to_remove {
        new_list.remove(idx);
        changed = true;
    }

    if ui.small_button("➕ Add").clicked() {
        new_list.push(String::new());
        changed = true;
    }

    changed.then_some(new_list)
}

/// Shared editable int list UI. Returns `Some(new_list)` if changed.
fn edit_int_list(ui: &mut egui::Ui, list: &[i64]) -> Option<Vec<i64>> {
    let mut new_list = list.to_vec();
    let mut changed = false;
    let mut to_remove: Option<usize> = None;

    for (idx, item) in new_list.iter_mut().enumerate() {
        ui.horizontal(|ui| {
            ui.label(format!("  {}:", idx));
            if ui.add(egui::DragValue::new(item)).changed() {
                changed = true;
            }
            if ui.small_button("🗑").clicked() {
                to_remove = Some(idx);
            }
        });
    }

    if let Some(idx) = to_remove {
        new_list.remove(idx);
        changed = true;
    }

    if ui.small_button("➕ Add").clicked() {
        new_list.push(0);
        changed = true;
    }

    changed.then_some(new_list)
}

/// Shared editable float list UI. Returns `Some(new_list)` if changed.
fn edit_float_list(ui: &mut egui::Ui, list: &[f64]) -> Option<Vec<f64>> {
    let mut new_list = list.to_vec();
    let mut changed = false;
    let mut to_remove: Option<usize> = None;

    for (idx, item) in new_list.iter_mut().enumerate() {
        ui.horizontal(|ui| {
            ui.label(format!("  {}:", idx));
            if ui.add(egui::DragValue::new(item).speed(0.1)).changed() {
                changed = true;
            }
            if ui.small_button("🗑").clicked() {
                to_remove = Some(idx);
            }
        });
    }

    if let Some(idx) = to_remove {
        new_list.remove(idx);
        changed = true;
    }

    if ui.small_button("➕ Add").clicked() {
        new_list.push(0.0);
        changed = true;
    }

    changed.then_some(new_list)
}

/// Shared editable bool list UI. Returns `Some(new_list)` if changed.
fn edit_bool_list(ui: &mut egui::Ui, list: &[bool]) -> Option<Vec<bool>> {
    let mut new_list = list.to_vec();
    let mut changed = false;
    let mut to_remove: Option<usize> = None;

    for (idx, item) in new_list.iter_mut().enumerate() {
        ui.horizontal(|ui| {
            ui.label(format!("  {}:", idx));
            if ui.checkbox(item, "").changed() {
                changed = true;
            }
            if ui.small_button("🗑").clicked() {
                to_remove = Some(idx);
            }
        });
    }

    if let Some(idx) = to_remove {
        new_list.remove(idx);
        changed = true;
    }

    if ui.small_button("➕ Add").clicked() {
        new_list.push(false);
        changed = true;
    }

    changed.then_some(new_list)
}

/// Render the add fact form.
pub(super) fn render_add_fact_form(ui: &mut egui::Ui, world: &mut World, _has_layered: bool) {
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

    // Layer selection for LayeredFactDatabase
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

    // Update state
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

            // Clear input
            let mut state = world.resource_mut::<FREPanelState>();
            state.new_fact_key.clear();
            state.new_fact_value_str.clear();

            info!("Added fact: {} = {:?}", key, value_str);
        }
    }
}

/// Render the Event History tab.
pub(super) fn render_events_tab(ui: &mut egui::Ui, world: &mut World) {
    let event_count = world.resource::<FactEventHistory>().events.len();

    ui.horizontal(|ui| {
        ui.label(format!(
            "📨 Recent Events ({}/{})",
            event_count, MAX_EVENT_HISTORY
        ));
        if ui.button("Clear").clicked() {
            world.resource_mut::<FactEventHistory>().events.clear();
        }
    });

    ui.separator();

    egui::ScrollArea::vertical().show(ui, |ui| {
        let history = world.resource::<FactEventHistory>();

        if history.events.is_empty() {
            ui.label("No events recorded yet.");
            ui.label("Events will appear here when FactEvents are emitted.");
            return;
        }

        for record in history.events.iter() {
            render_event_record(ui, record);
        }
    });
}

/// Render a single event record row.
fn render_event_record(ui: &mut egui::Ui, record: &FactEventRecord) {
    ui.horizontal(|ui| {
        ui.label(format!("[{:.2}s]", record.timestamp));
        ui.strong(&record.event_id);
        if !record.data_keys.is_empty() {
            ui.label(format!("(data: {})", record.data_keys.join(", ")));
        }
    });
}
