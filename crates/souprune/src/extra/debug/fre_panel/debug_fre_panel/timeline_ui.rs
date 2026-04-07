//! Renders the Fact Timeline tab in the FRE debug panel — shows fact change history.
//!
//! 渲染 FRE 调试面板的 Timeline 标签页——显示每个 Fact 键的值变化历史。

use crate::core::trace::FactChangeHistory;
use bevy::prelude::*;
use bevy_fact_rule_event::FactValue;
use bevy_inspector_egui::egui;

/// Render the Fact Timeline tab.
pub(super) fn render_timeline_tab(ui: &mut egui::Ui, world: &mut World) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.label("📈 Fact Timeline");
        ui.separator();

        let Some(history) = world.get_resource::<FactChangeHistory>() else {
            ui.label("FactChangeHistory not available.");
            return;
        };

        let total_keys = history.changes.len();
        let total_changes: usize = history.changes.values().map(|v| v.len()).sum();
        ui.label(format!(
            "Tracked keys: {}  |  Total changes: {}",
            total_keys, total_changes
        ));
        ui.separator();

        if history.changes.is_empty() {
            ui.label("No fact changes recorded yet.");
            ui.label("Changes are tracked when FRE rules modify facts.");
            return;
        }

        // Search/filter.
        let filter = world
            .get_resource::<super::FREPanelState>()
            .map(|s| s.search_filter.clone())
            .unwrap_or_default();

        let keys = history.keys_sorted();
        let filtered_keys: Vec<&str> = if filter.is_empty() {
            keys
        } else {
            keys.into_iter()
                .filter(|k| k.contains(filter.as_str()))
                .collect()
        };

        ui.label(format!(
            "Showing {} / {} keys{}",
            filtered_keys.len(),
            total_keys,
            if filter.is_empty() {
                String::new()
            } else {
                format!(" (filter: '{filter}')")
            }
        ));
        ui.separator();

        for key in &filtered_keys {
            let Some(changes) = history.changes.get(*key) else {
                continue;
            };
            if changes.is_empty() {
                continue;
            }

            let latest = changes.back().unwrap();
            let latest_str = format_fact_value(&latest.new_value);

            egui::CollapsingHeader::new(format!("🔑 {key} = {latest_str}"))
                .default_open(false)
                .show(ui, |ui| {
                    render_fact_changes(ui, changes, history.max_changes_per_key)
                });
        }
    });
}

fn format_fact_value(value: &FactValue) -> String {
    match value {
        FactValue::Int(v) => format!("{v}"),
        FactValue::Float(v) => format!("{v:.2}"),
        FactValue::Bool(v) => format!("{v}"),
        FactValue::String(v) => format!("\"{v}\""),
        FactValue::StringList(v) => format!("[{}]", v.join(", ")),
        FactValue::IntList(v) => format!("{v:?}"),
        FactValue::FloatList(v) => format!("{v:?}"),
        FactValue::BoolList(v) => format!("{v:?}"),
    }
}

fn render_fact_changes(
    ui: &mut egui::Ui,
    changes: &std::collections::VecDeque<crate::core::trace::FactChange>,
    max_changes_per_key: usize,
) {
    ui.label(format!(
        "{} changes recorded (max {})",
        changes.len(),
        max_changes_per_key
    ));

    for (i, change) in changes.iter().enumerate().rev() {
        let old_str = change
            .old_value
            .as_ref()
            .map(format_fact_value)
            .unwrap_or_else(|| "∅".to_string());
        let new_str = format_fact_value(&change.new_value);

        ui.horizontal(|ui| {
            ui.monospace(format!(
                "  #{} F{}: {} → {}  [{}]",
                i, change.frame_number, old_str, new_str, change.caused_by
            ));
        });
    }
}
