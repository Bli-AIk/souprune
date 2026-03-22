//! Renders the FRE debug panel tab that inspects registered rules and recent triggers.
//!
//! 渲染 FRE 调试面板中用于查看已注册规则与最近触发记录的标签页。
//!
//! This file focuses on one question: which rules are currently active, where
//! they came from, and whether they have fired recently. It groups global,
//! local, and view-scoped rules and expands each rule's trigger, conditions,
//! modifications, and outputs for debugging.
//!
//! 这个文件只回答一件事：当前有哪些规则处于有效状态、它们来自哪里、最近是否
//! 触发过。它会把全局、本地以及 View 作用域的规则分组展示，并展开每条规则的
//! trigger、conditions、modifications 和 outputs，方便排查规则行为。

use super::FREPanelState;
use crate::core::game_action::{GameActionDef, GameRule, GameRuleRegistry};
use crate::extra::debug::RuleTriggerHistory;
use bevy::prelude::*;
use bevy_fact_rule_event::{FactEventId, FactModification, RuleRegistry};
use bevy_inspector_egui::egui;

/// Render the Rules tab.
pub(super) fn render_rules_tab(ui: &mut egui::Ui, world: &mut World) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.label("📜 Registered Rules");
        ui.separator();

        let current_time = world
            .get_resource::<Time>()
            .map(|t| t.elapsed_secs_f64())
            .unwrap_or(0.0);
        let trigger_history = world.get_resource::<RuleTriggerHistory>();

        let Some(registry) = world.get_resource::<GameRuleRegistry>() else {
            ui.label("LayeredRuleRegistry not available.");
            ui.label("Make sure FREPlugin is installed.");
            return;
        };

        let global_count = registry.global_iter().count();
        let local_count = registry.local_iter().count();
        let view_count: usize = registry.view_iter().map(|(_, r)| r.iter().count()).sum();
        let total_count = global_count + local_count + view_count;

        ui.label(format!(
            "Total rules: {} (Global: {}, Local: {}, View: {})",
            total_count, global_count, local_count, view_count
        ));
        ui.separator();

        if total_count == 0 {
            ui.label("No rules registered.");
            ui.label("Rules are loaded from .fre.ron files.");
            return;
        }

        if global_count > 0 {
            let mut rules: Vec<_> = registry.global_iter().collect();
            render_rule_group(
                ui,
                "🌍 Global Rules",
                &mut rules,
                trigger_history,
                current_time,
            );
        }

        if local_count > 0 {
            let mut rules: Vec<_> = registry.local_iter().collect();
            render_rule_group(
                ui,
                "📍 Local Rules",
                &mut rules,
                trigger_history,
                current_time,
            );
        }

        if view_count > 0 {
            render_view_rules_group(ui, registry, trigger_history, current_time, view_count);
        }

        ui.separator();

        egui::CollapsingHeader::new("How Rules Work").show(ui, |ui| {
            ui.label("• Rules are defined in .fre.ron files");
            ui.label("• Each rule has: trigger, condition, modifications, actions, outputs");
            ui.label("• Rules are evaluated when their trigger event fires");
            ui.label("• Conditions use facts from the LayeredFactDatabase");
        });
    });
}

/// Render a group of rules under a collapsing header.
fn render_rule_group(
    ui: &mut egui::Ui,
    title: &str,
    rules: &mut [&GameRule],
    trigger_history: Option<&RuleTriggerHistory>,
    current_time: f64,
) {
    rules.sort_by(|a, b| b.priority.cmp(&a.priority));
    egui::CollapsingHeader::new(format!("{} ({})", title, rules.len()))
        .default_open(true)
        .show(ui, |ui| {
            for rule in rules.iter() {
                let is_triggered = trigger_history
                    .is_some_and(|h| h.was_recently_triggered(&rule.id, current_time, 1.0));
                show_rule_entry(ui, rule, is_triggered);
            }
        });
}

/// Render the view rules group with per-entity sub-groups.
fn render_view_rules_group(
    ui: &mut egui::Ui,
    registry: &GameRuleRegistry,
    trigger_history: Option<&RuleTriggerHistory>,
    current_time: f64,
    view_count: usize,
) {
    egui::CollapsingHeader::new(format!("👁 View Rules ({})", view_count))
        .default_open(true)
        .show(ui, |ui| {
            for (entity, view_registry) in registry.view_iter() {
                render_view_entity_rules(ui, entity, view_registry, trigger_history, current_time);
            }
        });
}

/// Render rules for a single view entity.
fn render_view_entity_rules(
    ui: &mut egui::Ui,
    entity: Entity,
    view_registry: &RuleRegistry<GameActionDef>,
    trigger_history: Option<&RuleTriggerHistory>,
    current_time: f64,
) {
    let view_rule_count = view_registry.iter().count();
    let mut view_rules: Vec<_> = view_registry.iter().collect();
    view_rules.sort_by(|a, b| b.priority.cmp(&a.priority));
    egui::CollapsingHeader::new(format!("Entity {:?} ({} rules)", entity, view_rule_count))
        .default_open(true)
        .show(ui, |ui| {
            for rule in view_rules {
                let is_triggered = trigger_history
                    .is_some_and(|h| h.was_recently_triggered(&rule.id, current_time, 1.0));
                show_rule_entry(ui, rule, is_triggered);
            }
        });
}

/// Helper function to display a single rule entry with optional trigger highlight.
fn show_rule_entry(ui: &mut egui::Ui, rule: &GameRule, is_recently_triggered: bool) {
    let status_icon = if rule.enabled { "✅" } else { "❌" };
    let trigger_indicator = if is_recently_triggered { "🔥 " } else { "" };
    let header_text = format!(
        "{}{} {} [Priority: {}]",
        trigger_indicator, status_icon, rule.id, rule.priority
    );

    let header_color = if is_recently_triggered {
        egui::Color32::from_rgb(100, 255, 100)
    } else {
        ui.visuals().text_color()
    };

    let enabled_text = if rule.enabled { "Yes" } else { "No" };
    let consume_text = if rule.consume_event { "Yes" } else { "No" };

    egui::CollapsingHeader::new(egui::RichText::new(header_text).color(header_color))
        .default_open(false)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("Trigger:");
                ui.monospace(&rule.trigger.0);
            });

            ui.horizontal(|ui| {
                ui.label("Enabled:");
                ui.label(enabled_text);
            });

            ui.horizontal(|ui| {
                ui.label("Consume Event:");
                ui.label(consume_text);
            });

            show_rule_conditions(ui, &rule.condition_expressions);
            show_rule_modifications(ui, &rule.modifications);
            show_rule_outputs(ui, &rule.outputs);
        });
}

/// Render condition expressions for a rule.
fn show_rule_conditions(ui: &mut egui::Ui, expressions: &[String]) {
    if expressions.is_empty() {
        return;
    }

    egui::CollapsingHeader::new(format!("Conditions ({})", expressions.len()))
        .default_open(false)
        .show(ui, |ui| {
            for (i, expr) in expressions.iter().enumerate() {
                ui.monospace(format!("{}: {}", i + 1, expr));
            }
        });
}

/// Render modifications for a rule.
fn show_rule_modifications(ui: &mut egui::Ui, modifications: &[FactModification]) {
    if modifications.is_empty() {
        return;
    }

    egui::CollapsingHeader::new(format!("Modifications ({})", modifications.len()))
        .default_open(false)
        .show(ui, |ui| {
            for (i, modification) in modifications.iter().enumerate() {
                ui.monospace(format!("{}: {:?}", i + 1, modification));
            }
        });
}

/// Render output events for a rule.
fn show_rule_outputs(ui: &mut egui::Ui, outputs: &[FactEventId]) {
    if outputs.is_empty() {
        return;
    }

    egui::CollapsingHeader::new(format!("Outputs ({})", outputs.len()))
        .default_open(false)
        .show(ui, |ui| {
            for output in outputs {
                ui.monospace(&output.0);
            }
        });
}
