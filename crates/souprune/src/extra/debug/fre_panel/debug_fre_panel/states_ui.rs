use bevy::prelude::*;
use bevy_inspector_egui::egui;

/// Render the States tab.
pub(super) fn render_states_tab(ui: &mut egui::Ui, world: &mut World) {
    ui.heading("Game States");
    ui.separator();

    egui::ScrollArea::vertical().show(ui, |ui| {
        render_app_state_section(ui, world);
        ui.add_space(5.0);
        render_sequence_mode_section(ui, world);
        ui.add_space(10.0);
        render_sequence_sub_state_section(ui, world);
        render_chase_state_section(ui, world);
    });
}

/// Render a state indicator row with current/inactive styling.
fn render_state_row(
    ui: &mut egui::Ui,
    is_current: bool,
    state_name: &str,
    description: Option<&str>,
) {
    let desc_to_show = if is_current { description } else { None };
    ui.horizontal(|ui| {
        if is_current {
            ui.colored_label(egui::Color32::GREEN, "> ");
            ui.colored_label(egui::Color32::GREEN, state_name);
        } else {
            ui.label("  ");
            ui.colored_label(egui::Color32::GRAY, state_name);
        }
        if let Some(desc) = desc_to_show {
            ui.small(desc);
        }
    });
}

/// Render the AppState section.
fn render_app_state_section(ui: &mut egui::Ui, world: &mut World) {
    use crate::app_state::AppState;

    egui::CollapsingHeader::new("AppState")
        .default_open(true)
        .show(ui, |ui| {
            let current_app_state = world.get_resource::<State<AppState>>().map(|s| *s.get());

            let all_states = [
                (AppState::Loading, "Resource loading"),
                (AppState::Running, "Running"),
            ];

            for (state, description) in all_states {
                let is_current = current_app_state == Some(state);
                render_state_row(ui, is_current, &format!("{:?}", state), Some(description));
            }
        });
}

/// Render the SequenceMode section.
fn render_sequence_mode_section(ui: &mut egui::Ui, world: &mut World) {
    use crate::app_state::SequenceMode;

    egui::CollapsingHeader::new("SequenceMode")
        .default_open(true)
        .show(ui, |ui| {
            let current_mode = world
                .get_resource::<SequenceMode>()
                .and_then(|m| m.0.clone());

            let (text, color) = match &current_mode {
                Some(mode) => (mode.as_str(), egui::Color32::GREEN),
                None => ("None", egui::Color32::GRAY),
            };

            ui.horizontal(|ui| {
                ui.label("Current:");
                ui.colored_label(color, text);
            });
        });
}

/// Render the SequenceSubState section.
fn render_sequence_sub_state_section(ui: &mut egui::Ui, world: &mut World) {
    use crate::app_state::{SequenceMode, SequenceSubState};

    let has_mode = world
        .get_resource::<SequenceMode>()
        .map(|m| m.0.is_some())
        .unwrap_or(false);

    if !has_mode {
        return;
    }

    egui::CollapsingHeader::new("SequenceSubState")
        .default_open(true)
        .show(ui, |ui| {
            let current_sub_state = world
                .get_resource::<State<SequenceSubState>>()
                .map(|s| s.name().to_string());

            let state_config = world.get_resource::<crate::core::state_config::LoadedStateConfig>();
            let Some(config) = state_config else {
                ui.label("StateConfig not loaded");
                return;
            };

            let mut state_names = config.state_names();
            state_names.sort();

            for state_name in state_names {
                let is_current = current_sub_state.as_deref() == Some(state_name.as_str());
                render_sub_state_row(ui, state_name, is_current, config);
            }
        });
}

/// Render a single sub-state row with details when current.
fn render_sub_state_row(
    ui: &mut egui::Ui,
    state_name: &str,
    is_current: bool,
    config: &crate::core::state_config::LoadedStateConfig,
) {
    render_state_row(ui, is_current, state_name, None);

    if !is_current {
        return;
    }

    let is_view_interactive = config.is_view_interactive(state_name);
    let is_player_movable = config.is_player_movable(state_name);
    let view_layout = config.get_view_layout(state_name);
    let chase_config = config.get_chase_config_path(state_name);

    let interactive_color = if is_view_interactive {
        egui::Color32::GREEN
    } else {
        egui::Color32::GRAY
    };
    let interactive_text = if is_view_interactive { "Yes" } else { "No" };
    let movable_color = if is_player_movable {
        egui::Color32::GREEN
    } else {
        egui::Color32::GRAY
    };
    let movable_text = if is_player_movable { "Yes" } else { "No" };

    ui.indent(state_name, |ui| {
        ui.horizontal(|ui| {
            ui.label("UI Interactive:");
            ui.colored_label(interactive_color, interactive_text);
        });

        ui.horizontal(|ui| {
            ui.label("Player Movable:");
            ui.colored_label(movable_color, movable_text);
        });

        render_view_layout_row(ui, view_layout);
        render_chase_config_row(ui, chase_config);
    });
}

/// Render the view layout row.
fn render_view_layout_row(ui: &mut egui::Ui, view_layout: Option<&str>) {
    ui.horizontal(|ui| {
        ui.label("View Layout:");
        if let Some(layout) = view_layout {
            ui.small(layout);
        } else {
            ui.colored_label(egui::Color32::GRAY, "None");
        }
    });
}

/// Render the chase config path row.
fn render_chase_config_row(ui: &mut egui::Ui, chase_config: Option<&str>) {
    let Some(chase_path) = chase_config else {
        return;
    };
    ui.horizontal(|ui| {
        ui.label("Chase Config:");
        ui.small(chase_path);
    });
}

/// Render the Chase State Info section.
fn render_chase_state_section(ui: &mut egui::Ui, world: &mut World) {
    let has_mode = world
        .get_resource::<crate::app_state::SequenceMode>()
        .map(|m| m.0.is_some())
        .unwrap_or(false);

    if !has_mode {
        return;
    }

    ui.add_space(10.0);

    let chase_enabled_display: Option<(&str, egui::Color32)> = world
        .get_resource::<crate::app_state::overworld::chase::ChaseEnabled>()
        .map(|c| {
            if c.0 {
                ("Yes", egui::Color32::GREEN)
            } else {
                ("No", egui::Color32::GRAY)
            }
        });

    let chase_name_val = world
        .get_resource::<crate::app_state::overworld::chase::ChaseStateName>()
        .and_then(|c| c.0.clone());

    egui::CollapsingHeader::new("Chase State Info")
        .default_open(true)
        .show(ui, |ui| {
            render_chase_enabled_row(ui, chase_enabled_display);
            render_chase_name_row(ui, &chase_name_val);
        });
}

/// Render the chase enabled status row.
fn render_chase_enabled_row(ui: &mut egui::Ui, display: Option<(&str, egui::Color32)>) {
    let Some((text, color)) = display else {
        return;
    };
    ui.horizontal(|ui| {
        ui.label("Chase Enabled:");
        ui.colored_label(color, text);
    });
}

/// Render the chase state name row.
fn render_chase_name_row(ui: &mut egui::Ui, name: &Option<String>) {
    let (display_text, use_strong) = match name {
        Some(n) => (n.as_str(), true),
        None => ("Not configured", false),
    };
    ui.horizontal(|ui| {
        ui.label("Chase State Name:");
        if use_strong {
            ui.strong(display_text);
        } else {
            ui.colored_label(egui::Color32::GRAY, display_text);
        }
    });
}
