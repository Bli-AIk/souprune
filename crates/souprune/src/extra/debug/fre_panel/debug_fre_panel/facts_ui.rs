//! Facts-related UI rendering for the FRE debug panel.
//! FRE 调试面板的事实相关 UI 渲染。

mod event_history;
mod layered_facts;
mod list_editors;
mod view_facts;

use super::{FREPanelState, FactLayerSelection};
use bevy::prelude::*;
use bevy_fact_rule_event::LayeredFactDatabase;
use bevy_inspector_egui::egui;

/// Render the Facts tab.
pub(super) fn render_facts_tab(ui: &mut egui::Ui, world: &mut World) {
    let mut search_filter = world.resource::<FREPanelState>().search_filter.clone();
    ui.horizontal(|ui| {
        ui.label("🔍");
        if ui.text_edit_singleline(&mut search_filter).changed() {
            world.resource_mut::<FREPanelState>().search_filter = search_filter.clone();
        }
    });

    ui.separator();

    let has_layered = world.get_resource::<LayeredFactDatabase>().is_some();

    egui::ScrollArea::vertical().show(ui, |ui| {
        if !has_layered {
            ui.label("⚠️ No LayeredFactDatabase found.");
            ui.label("FRE system may not be initialized.");
            return;
        }

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

        egui::CollapsingHeader::new("➕ Add New Fact").show(ui, |ui| {
            render_add_fact_form(ui, world);
        });
    });
}

pub(super) fn render_events_tab(ui: &mut egui::Ui, world: &mut World) {
    event_history::render_events_tab(ui, world);
}

pub(super) fn render_view_facts_tab(ui: &mut egui::Ui, world: &mut World) {
    view_facts::render_view_facts_tab(ui, world);
}

fn render_layered_facts(
    ui: &mut egui::Ui,
    world: &mut World,
    layer: FactLayerSelection,
    filter: &str,
) {
    layered_facts::render_layered_facts(ui, world, layer, filter);
}

fn render_add_fact_form(ui: &mut egui::Ui, world: &mut World) {
    layered_facts::render_add_fact_form(ui, world);
}
