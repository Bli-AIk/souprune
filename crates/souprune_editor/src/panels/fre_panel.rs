//! # FRE (Fact-Rule-Event) Panel
//!
//! # FRE (事实-规则-事件) 面板
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! FRE (Fact-Rule-Event) panel for the editor.
//! Provides a dockable panel to inspect and modify facts, view rules,
//! track events, and monitor game states.
//!
//! 编辑器的 FRE (事实-规则-事件) 面板。
//! 提供一个可停靠面板来检查和修改事实、查看规则、跟踪事件和监控游戏状态。

use bevy::prelude::*;
use bevy_fact_rule_event::{FactEvent, FactValue, LayeredFactDatabase};

use bevy_workbench::i18n::FluentArgs;
use bevy_workbench::prelude::*;
use souprune::core::game_action::GameRuleRegistry;
use souprune::extra::debug::RuleTriggerHistory;
use std::collections::VecDeque;

use crate::i18n::{t, t_args};

const MAX_EVENT_HISTORY: usize = 100;

/// Resource to track recent FactEvents in the editor.
#[derive(Resource, Default)]
pub struct EditorFactEventHistory {
    pub events: VecDeque<FactEventRecord>,
}

pub struct FactEventRecord {
    pub event_id: String,
    pub timestamp: f64,
    pub data_keys: Vec<String>,
}

/// System to track FactEvents for the editor FRE panel.
pub fn track_fact_events_system(
    mut events: MessageReader<FactEvent>,
    mut history: ResMut<EditorFactEventHistory>,
    time: Res<Time>,
) {
    for event in events.read() {
        let record = FactEventRecord {
            event_id: event.id.0.clone(),
            timestamp: time.elapsed_secs_f64(),
            data_keys: event.data.keys().cloned().collect(),
        };
        history.events.push_front(record);
        while history.events.len() > MAX_EVENT_HISTORY {
            history.events.pop_back();
        }
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
enum FreTab {
    #[default]
    Facts,
    Rules,
    Events,
    States,
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
enum FactTypeInput {
    #[default]
    Int,
    Float,
    Bool,
    String,
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
enum FactLayer {
    #[default]
    Local,
    Global,
}

/// FRE debug panel as a dockable editor panel.
pub struct FrePanel {
    tab: FreTab,
    search: String,
    new_fact_key: String,
    new_fact_value: String,
    new_fact_type: FactTypeInput,
    new_fact_layer: FactLayer,
    cached_title: String,
}

impl FrePanel {
    pub fn new() -> Self {
        Self {
            tab: FreTab::default(),
            search: String::new(),
            new_fact_key: String::new(),
            new_fact_value: String::new(),
            new_fact_type: FactTypeInput::default(),
            new_fact_layer: FactLayer::default(),
            cached_title: "FRE Panel".to_string(),
        }
    }
}

impl WorkbenchPanel for FrePanel {
    fn id(&self) -> &str {
        "fre_panel"
    }

    fn title(&self) -> String {
        self.cached_title.clone()
    }

    fn needs_world(&self) -> bool {
        true
    }

    fn default_visible(&self) -> bool {
        false
    }

    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.label(&self.cached_title);
    }

    fn ui_world(&mut self, ui: &mut egui::Ui, world: &mut World) {
        self.cached_title = t(world, "panel-fre");

        // Tab bar
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.tab, FreTab::Facts, t(world, "fre-tabs-facts"));
            ui.selectable_value(&mut self.tab, FreTab::Rules, t(world, "fre-tabs-rules"));
            ui.selectable_value(&mut self.tab, FreTab::Events, t(world, "fre-tabs-events"));
            ui.selectable_value(&mut self.tab, FreTab::States, t(world, "fre-tabs-states"));
        });

        ui.separator();

        match self.tab {
            FreTab::Facts => self.render_facts(ui, world),
            FreTab::Rules => render_rules(ui, world),
            FreTab::Events => render_events(ui, world),
            FreTab::States => render_states(ui, world),
        }
    }
}

impl FrePanel {
    fn render_facts(&mut self, ui: &mut egui::Ui, world: &mut World) {
        // Search filter
        ui.horizontal(|ui| {
            ui.label(t(world, "fre-filter"));
            ui.text_edit_singleline(&mut self.search);
        });

        ui.separator();

        let has_db = world.get_resource::<LayeredFactDatabase>().is_some();
        if !has_db {
            ui.label(t(world, "fre-db-unavailable"));
            return;
        }

        egui::ScrollArea::vertical().show(ui, |ui| {
            // Global facts
            egui::CollapsingHeader::new(t(world, "fre-global-layer"))
                .default_open(true)
                .show(ui, |ui| {
                    render_fact_layer(ui, world, true, &self.search);
                });

            // Local facts
            egui::CollapsingHeader::new(t(world, "fre-local-layer"))
                .default_open(true)
                .show(ui, |ui| {
                    render_fact_layer(ui, world, false, &self.search);
                });

            ui.separator();

            // Add new fact
            egui::CollapsingHeader::new(t(world, "fre-add-fact")).show(ui, |ui| {
                self.draw_add_fact_form(ui, world);
            });
        });
    }

    fn draw_add_fact_form(&mut self, ui: &mut egui::Ui, world: &mut World) {
        use crate::i18n::t;
        ui.horizontal(|ui| {
            ui.label(t(world, "fre-key"));
            ui.text_edit_singleline(&mut self.new_fact_key);
        });
        ui.horizontal(|ui| {
            ui.label(t(world, "fre-value"));
            ui.text_edit_singleline(&mut self.new_fact_value);
        });
        ui.horizontal(|ui| {
            ui.label(t(world, "fre-type"));
            ui.selectable_value(&mut self.new_fact_type, FactTypeInput::Int, "Int");
            ui.selectable_value(&mut self.new_fact_type, FactTypeInput::Float, "Float");
            ui.selectable_value(&mut self.new_fact_type, FactTypeInput::Bool, "Bool");
            ui.selectable_value(&mut self.new_fact_type, FactTypeInput::String, "String");
        });
        ui.horizontal(|ui| {
            ui.label(t(world, "fre-layer"));
            ui.selectable_value(&mut self.new_fact_layer, FactLayer::Local, "Local");
            ui.selectable_value(&mut self.new_fact_layer, FactLayer::Global, "Global");
        });

        if ui.button(t(world, "action-add")).clicked() && !self.new_fact_key.is_empty() {
            let value = parse_fact_value(&self.new_fact_value, self.new_fact_type);
            if let Some(value) = value {
                let key = self.new_fact_key.clone();
                let mut db = world.resource_mut::<LayeredFactDatabase>();
                match self.new_fact_layer {
                    FactLayer::Global => db.set_global(key, value),
                    FactLayer::Local => db.set_local(key, value),
                }
                self.new_fact_key.clear();
                self.new_fact_value.clear();
            }
        }
    }
}

fn parse_fact_value(s: &str, ty: FactTypeInput) -> Option<FactValue> {
    match ty {
        FactTypeInput::Int => s.parse::<i64>().ok().map(FactValue::Int),
        FactTypeInput::Float => s.parse::<f64>().ok().map(FactValue::Float),
        FactTypeInput::Bool => s.parse::<bool>().ok().map(FactValue::Bool),
        FactTypeInput::String => Some(FactValue::String(s.to_string())),
    }
}

fn delete_fact(world: &mut World, global: bool, key: &str) {
    let mut db = world.resource_mut::<LayeredFactDatabase>();
    if global {
        db.remove_global(key);
    } else {
        db.remove(key);
    }
}

fn render_fact_layer(ui: &mut egui::Ui, world: &mut World, global: bool, search: &str) {
    let db = world.resource::<LayeredFactDatabase>();
    let facts: Vec<(String, FactValue)> = if global {
        db.iter_global()
            .filter(|(k, _)| search.is_empty() || k.contains(search))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    } else {
        db.iter_local()
            .filter(|(k, _)| search.is_empty() || k.contains(search))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    };

    if facts.is_empty() {
        ui.label(egui::RichText::new(t(world, "label-empty")).color(egui::Color32::GRAY));
        return;
    }

    let mut sorted = facts;
    sorted.sort_by(|a, b| a.0.cmp(&b.0));

    egui::Grid::new(if global {
        "global_facts"
    } else {
        "local_facts"
    })
    .striped(true)
    .show(ui, |ui| {
        for (key, value) in &sorted {
            ui.label(key);
            ui.label(format_fact_value(value));

            // Delete button
            if ui.small_button("x").clicked() {
                delete_fact(world, global, key);
            }
            ui.end_row();
        }
    });
}

fn format_fact_value(v: &FactValue) -> String {
    match v {
        FactValue::Int(i) => format!("{i} (int)"),
        FactValue::Float(f) => format!("{f:.2} (float)"),
        FactValue::Bool(b) => format!("{b} (bool)"),
        FactValue::String(s) => format!("\"{s}\" (str)"),
        FactValue::StringList(l) => format!("{l:?} (str[])"),
        FactValue::IntList(l) => format!("{l:?} (int[])"),
        FactValue::FloatList(l) => format!("{l:?} (float[])"),
        FactValue::BoolList(l) => format!("{l:?} (bool[])"),
    }
}

fn render_rules(ui: &mut egui::Ui, world: &mut World) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        let current_time = world
            .get_resource::<Time>()
            .map(|t| t.elapsed_secs_f64())
            .unwrap_or(0.0);
        let trigger_history = world.get_resource::<RuleTriggerHistory>();
        let registry = world.get_resource::<GameRuleRegistry>();

        let Some(registry) = registry else {
            ui.label(t(world, "fre-registry-unavailable"));
            return;
        };

        let global_count = registry.global_iter().count();
        let local_count = registry.local_iter().count();
        let total = global_count + local_count;

        {
            let mut args = FluentArgs::new();
            args.set("total", total as i64);
            args.set("global", global_count as i64);
            args.set("local", local_count as i64);
            ui.label(t_args(world, "fre-rules-total", &args));
        }
        ui.separator();

        if total == 0 {
            ui.label(t(world, "fre-no-rules"));
            return;
        }

        if global_count > 0 {
            let header = {
                let mut args = FluentArgs::new();
                args.set("count", global_count as i64);
                t_args(world, "fre-global-rules", &args)
            };
            egui::CollapsingHeader::new(header)
                .default_open(true)
                .show(ui, |ui| {
                    let mut rules: Vec<_> = registry.global_iter().collect();
                    rules.sort_by(|a, b| b.priority.cmp(&a.priority));
                    render_rule_list(ui, world, &rules, trigger_history, current_time);
                });
        }

        if local_count > 0 {
            let header = {
                let mut args = FluentArgs::new();
                args.set("count", local_count as i64);
                t_args(world, "fre-local-rules", &args)
            };
            egui::CollapsingHeader::new(header)
                .default_open(true)
                .show(ui, |ui| {
                    let mut rules: Vec<_> = registry.local_iter().collect();
                    rules.sort_by(|a, b| b.priority.cmp(&a.priority));
                    render_rule_list(ui, world, &rules, trigger_history, current_time);
                });
        }
    });
}

fn render_rule_list(
    ui: &mut egui::Ui,
    world: &World,
    rules: &[&souprune::core::game_action::GameRule],
    trigger_history: Option<&RuleTriggerHistory>,
    current_time: f64,
) {
    for rule in rules {
        let triggered = trigger_history
            .map(|h| h.was_recently_triggered(&rule.id, current_time, 1.0))
            .unwrap_or(false);
        show_rule(ui, world, rule, triggered);
    }
}

fn show_rule(
    ui: &mut egui::Ui,
    world: &World,
    rule: &souprune::core::game_action::GameRule,
    triggered: bool,
) {
    let status = if rule.enabled { "[on]" } else { "[off]" };
    let fire = if triggered { " !" } else { "" };
    let text = format!("{status} {} [P:{}]{fire}", rule.id, rule.priority);

    let color = if triggered {
        egui::Color32::from_rgb(100, 255, 100)
    } else {
        ui.visuals().text_color()
    };

    egui::CollapsingHeader::new(egui::RichText::new(text).color(color))
        .default_open(false)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(t(world, "fre-trigger"));
                ui.monospace(&rule.trigger.0);
            });

            if !rule.condition_expressions.is_empty() {
                egui::CollapsingHeader::new(format!(
                    "Conditions ({})",
                    rule.condition_expressions.len()
                ))
                .show(ui, |ui| {
                    show_rule_conditions(ui, rule);
                });
            }

            if !rule.modifications.is_empty() {
                egui::CollapsingHeader::new(format!(
                    "Modifications ({})",
                    rule.modifications.len()
                ))
                .show(ui, |ui| {
                    show_rule_modifications(ui, rule);
                });
            }

            if !rule.outputs.is_empty() {
                egui::CollapsingHeader::new(format!("Outputs ({})", rule.outputs.len())).show(
                    ui,
                    |ui| {
                        show_rule_outputs(ui, rule);
                    },
                );
            }
        });
}

fn show_rule_conditions(ui: &mut egui::Ui, rule: &souprune::core::game_action::GameRule) {
    for (i, expr) in rule.condition_expressions.iter().enumerate() {
        ui.monospace(format!("{}: {}", i + 1, expr));
    }
}

fn show_rule_modifications(ui: &mut egui::Ui, rule: &souprune::core::game_action::GameRule) {
    for (i, m) in rule.modifications.iter().enumerate() {
        ui.monospace(format!("{}: {:?}", i + 1, m));
    }
}

fn show_rule_outputs(ui: &mut egui::Ui, rule: &souprune::core::game_action::GameRule) {
    for o in &rule.outputs {
        ui.monospace(&o.0);
    }
}

fn render_events(ui: &mut egui::Ui, world: &mut World) {
    let Some(history) = world.get_resource::<EditorFactEventHistory>() else {
        ui.label(t(world, "fre-event-tracking-not-init"));
        return;
    };

    {
        let mut args = FluentArgs::new();
        args.set("count", history.events.len() as i64);
        ui.label(t_args(world, "fre-recent-events", &args));
    }
    ui.separator();

    egui::ScrollArea::vertical().show(ui, |ui| {
        if history.events.is_empty() {
            ui.label(egui::RichText::new(t(world, "fre-no-events")).color(egui::Color32::GRAY));
            return;
        }

        for record in &history.events {
            ui.horizontal(|ui| {
                render_event_record(ui, record);
            });
        }
    });
}

fn render_event_record(ui: &mut egui::Ui, record: &FactEventRecord) {
    ui.label(format!("{:.1}s", record.timestamp));
    ui.strong(&record.event_id);
    if !record.data_keys.is_empty() {
        ui.label(format!("[{}]", record.data_keys.join(", ")));
    }
}

fn render_states(ui: &mut egui::Ui, world: &mut World) {
    use souprune::app_state::{AppState, SequenceMode};

    egui::ScrollArea::vertical().show(ui, |ui| {
        // AppState
        egui::CollapsingHeader::new("AppState")
            .default_open(true)
            .show(ui, |ui| {
                let current = world.get_resource::<State<AppState>>().map(|s| *s.get());
                for (state, desc) in [
                    (AppState::Loading, "Resource loading"),
                    (AppState::Running, "Running"),
                ] {
                    let is_cur = current == Some(state);
                    render_app_state_row(ui, state, is_cur, desc);
                }
            });

        // SequenceMode
        egui::CollapsingHeader::new("SequenceMode")
            .default_open(true)
            .show(ui, |ui| {
                let mode = world.get_resource::<SequenceMode>().map(|m| m.0.clone());
                ui.horizontal(|ui| {
                    ui.label(t(world, "fre-current"));
                    match mode.flatten() {
                        Some(m) => ui.colored_label(egui::Color32::GREEN, &m),
                        None => ui.colored_label(egui::Color32::GRAY, "None"),
                    };
                });
            });

        // SequenceSubState
        let has_mode = world
            .get_resource::<SequenceMode>()
            .map(|m| m.0.is_some())
            .unwrap_or(false);
        if has_mode {
            egui::CollapsingHeader::new("SequenceSubState")
                .default_open(true)
                .show(ui, |ui| {
                    render_sequence_sub_state(ui, world);
                });
        }
    });
}

fn render_app_state_row(
    ui: &mut egui::Ui,
    state: souprune::app_state::AppState,
    is_cur: bool,
    desc: &str,
) {
    ui.horizontal(|ui| {
        if is_cur {
            ui.colored_label(egui::Color32::GREEN, format!("> {:?}", state));
            ui.small(desc);
        } else {
            ui.colored_label(egui::Color32::GRAY, format!("  {:?}", state));
        }
    });
}

fn render_sequence_sub_state(ui: &mut egui::Ui, world: &mut World) {
    use souprune::app_state::SequenceSubState;
    let current = world
        .get_resource::<State<SequenceSubState>>()
        .map(|s| s.name().to_string());
    let config = world.get_resource::<souprune::core::state_config::LoadedStateConfig>();
    if let Some(config) = config {
        let mut names: Vec<&String> = config.0.states.keys().collect();
        names.sort();
        for name in names {
            let is_cur = current.as_deref() == Some(name.as_str());
            ui.colored_label(
                if is_cur {
                    egui::Color32::GREEN
                } else {
                    egui::Color32::GRAY
                },
                format!("{}{name}", if is_cur { "> " } else { "  " }),
            );
        }
    } else {
        ui.label(t(world, "fre-state-config-not-loaded"));
    }
}
