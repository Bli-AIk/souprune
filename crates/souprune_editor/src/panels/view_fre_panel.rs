//! View 编辑器 FRE 面板。
//!
//! 加载 View 引用的 .fre.ron 文件，显示规则表，提供 Fact 模拟。

use std::collections::HashMap;
use std::path::PathBuf;

use bevy::prelude::Resource;
use bevy_fact_rule_event::asset::{
    FactModificationDef, FactValueDef, FreAsset, RuleActionDef, RuleDef, RuleEventDef,
};
use souprune::core::view::layout::DataRequirement;

/// View FRE 编辑器状态。
#[derive(Resource, Default)]
pub struct ViewFreState {
    /// 已加载的 FRE 资产，按路径索引。
    pub loaded_fre: HashMap<String, FreAsset>,
    /// 模拟 Fact 值（用于预览 visible_when）。
    pub simulated_facts: HashMap<String, SimFactValue>,
    /// 上次加载 FRE 对应的 view 文件路径。
    pub loaded_for_path: Option<PathBuf>,
    /// 新增 fact 的 key 输入缓冲。
    pub new_fact_key: String,
}

/// 简单的模拟 Fact 值。
#[derive(Clone, Debug)]
pub enum SimFactValue {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
}

impl ViewFreState {
    /// 当 view 文件变更时，重新加载关联的 FRE 文件。
    pub fn sync_for_view(&mut self, view_path: &std::path::Path, requires: &[DataRequirement]) {
        if self.loaded_for_path.as_deref() == Some(view_path) {
            return;
        }
        self.loaded_fre.clear();
        self.loaded_for_path = Some(view_path.to_path_buf());

        for req in requires {
            let DataRequirement::File(rel_path) = req else {
                continue;
            };
            let Some(full) = souprune::config::resolve_path(rel_path) else {
                continue;
            };
            let content = match std::fs::read_to_string(&full) {
                Ok(c) => c,
                Err(e) => {
                    bevy::log::warn!("FRE 读取失败 {rel_path}: {e}");
                    continue;
                }
            };
            let fre = match ron::from_str::<FreAsset>(&content) {
                Ok(f) => f,
                Err(e) => {
                    bevy::log::warn!("FRE 解析失败 {rel_path}: {e}");
                    continue;
                }
            };
            // 从 FRE 的 facts 初始化模拟值
            for (k, v) in &fre.facts {
                self.simulated_facts
                    .entry(k.clone())
                    .or_insert_with(|| fact_value_def_to_sim(v));
            }
            self.loaded_fre.insert(rel_path.clone(), fre);
        }
    }

    /// 强制重新加载 FRE 文件。
    #[allow(dead_code)]
    pub fn force_reload(&mut self) {
        self.loaded_for_path = None;
    }
}

fn fact_value_def_to_sim(v: &FactValueDef) -> SimFactValue {
    match v {
        FactValueDef::Int(i) => SimFactValue::Int(*i),
        FactValueDef::Float(f) => SimFactValue::Float(*f),
        FactValueDef::Bool(b) => SimFactValue::Bool(*b),
        FactValueDef::String(s) => SimFactValue::String(s.clone()),
        FactValueDef::StringList(_) | FactValueDef::IntList(_) => SimFactValue::Int(0),
    }
}

// ─── UI 渲染 ────────────────────────────────────────────────

/// 在 inspector 底部渲染 FRE 区域。
pub fn render_view_fre_section(
    ui: &mut egui::Ui,
    state: &mut ViewFreState,
    live_facts: Option<&mut bevy_fact_rule_event::FactDatabase>,
) -> bool {
    let mut changed = false;

    if state.loaded_fre.is_empty() {
        return false;
    }

    ui.separator();
    egui::CollapsingHeader::new(format!("FRE Rules ({})", total_rules(state)))
        .default_open(false)
        .show(ui, |ui| {
            for (path, fre) in &state.loaded_fre {
                render_fre_file(ui, path, fre);
            }
        });

    // Show simulated fact editor only when not in Play mode
    if live_facts.is_none() {
        egui::CollapsingHeader::new("Fact Simulator")
            .default_open(false)
            .show(ui, |ui| {
                changed |= render_fact_simulator(ui, state);
            });
    }

    changed
}

/// Render live fact simulator bound to ViewRoot.local_facts during Play mode.
pub fn render_live_facts_section(
    ui: &mut egui::Ui,
    facts: &mut bevy_fact_rule_event::FactDatabase,
) {
    egui::CollapsingHeader::new("Fact Simulator (Live)")
        .default_open(true)
        .show(ui, |ui| {
            render_live_fact_simulator(ui, facts);
        });
}

fn total_rules(state: &ViewFreState) -> usize {
    state.loaded_fre.values().map(|f| f.rules.len()).sum()
}

fn render_fre_facts(ui: &mut egui::Ui, facts: &HashMap<String, FactValueDef>) {
    for (k, v) in facts {
        ui.horizontal(|ui| {
            ui.label(k);
            ui.monospace(format_fact_value_def(v));
        });
    }
}

fn render_fre_file(ui: &mut egui::Ui, path: &str, fre: &FreAsset) {
    let short = path.rsplit('/').next().unwrap_or(path);
    egui::CollapsingHeader::new(format!("{short} ({} rules)", fre.rules.len()))
        .default_open(true)
        .show(ui, |ui| {
            if !fre.facts.is_empty() {
                egui::CollapsingHeader::new(format!("Facts ({})", fre.facts.len()))
                    .default_open(false)
                    .show(ui, |ui| {
                        render_fre_facts(ui, &fre.facts);
                    });
            }
            for (i, rule) in fre.rules.iter().enumerate() {
                render_rule_def(ui, rule, i);
            }
        });
}

fn render_rule_def(ui: &mut egui::Ui, rule: &RuleDef, index: usize) {
    let id_str = if rule.id.is_empty() {
        format!("rule_{index:03}")
    } else {
        rule.id.clone()
    };
    let event_str = format_event(&rule.event);
    let status = if rule.enabled { "" } else { " [off]" };
    let header = format!("{id_str}{status} ← {event_str}");

    egui::CollapsingHeader::new(&header)
        .id_salt(format!("rule_{index}_{}", rule.id))
        .default_open(false)
        .show(ui, |ui| {
            if rule.priority != 0 {
                ui.label(format!("Priority: {}", rule.priority));
            }
            if !rule.conditions.is_empty() {
                ui.label("Conditions:");
                for c in &rule.conditions {
                    ui.monospace(format!("  {c}"));
                }
            }
            if !rule.actions.is_empty() {
                ui.label("Actions:");
                for a in &rule.actions {
                    ui.monospace(format!("  {}", format_action(a)));
                }
            }
            if !rule.modifications.is_empty() {
                ui.label("Modifications:");
                for m in &rule.modifications {
                    ui.monospace(format!("  {}", format_modification(m)));
                }
            }
            if !rule.outputs.is_empty() {
                ui.label(format!("Outputs: {}", rule.outputs.join(", ")));
            }
        });
}

fn format_event(ev: &RuleEventDef) -> String {
    match ev {
        RuleEventDef::Event(s) => s.clone(),
        RuleEventDef::ActionEvent { action, kind } => {
            format!("action:{action}:{kind:?}")
        }
    }
}

fn format_action(a: &RuleActionDef) -> String {
    match a {
        RuleActionDef::Log { message } => format!("Log(\"{message}\")"),
        RuleActionDef::PlaySound(s) => format!("PlaySound(\"{s}\")"),
        RuleActionDef::PlaySoundFullPath(s) => format!("PlaySoundFullPath(\"{s}\")"),
        RuleActionDef::SetLocalFact(k, v) => format!("SetLocalFact(\"{k}\", {v:?})"),
        RuleActionDef::CloseView => "CloseView".into(),
        RuleActionDef::SwitchState(s) => format!("SwitchState(\"{s}\")"),
        RuleActionDef::EmitEvent(s) => format!("EmitEvent(\"{s}\")"),
        RuleActionDef::Custom {
            action_type,
            params,
        } => format!("Custom({action_type}, {params:?})"),
    }
}

fn format_modification(m: &FactModificationDef) -> String {
    match m {
        FactModificationDef::Set { key, value } => {
            format!("Set(\"{key}\", {})", format_fact_value_def(value))
        }
        FactModificationDef::Increment { key, amount } => format!("Inc(\"{key}\", {amount})"),
        FactModificationDef::Add { key, value } => format!("Add(\"{key}\", {value})"),
        FactModificationDef::Sub { key, value } => format!("Sub(\"{key}\", {value})"),
        FactModificationDef::Mul { key, value } => format!("Mul(\"{key}\", {value})"),
        FactModificationDef::Div { key, value } => format!("Div(\"{key}\", {value})"),
        FactModificationDef::Mod { key, value } => format!("Mod(\"{key}\", {value})"),
        FactModificationDef::Clamp { key, min, max } => {
            format!("Clamp(\"{key}\", {min}..{max})")
        }
        FactModificationDef::Wrap { key, min, max } => {
            format!("Wrap(\"{key}\", {min}..{max})")
        }
        FactModificationDef::Eval { key, expr } => format!("Eval(\"{key}\", \"{expr}\")"),
        FactModificationDef::Remove(k) => format!("Remove(\"{k}\")"),
        FactModificationDef::Toggle(k) => format!("Toggle(\"{k}\")"),
    }
}

fn format_fact_value_def(v: &FactValueDef) -> String {
    match v {
        FactValueDef::Int(i) => format!("{i}"),
        FactValueDef::Float(f) => format!("{f}"),
        FactValueDef::Bool(b) => format!("{b}"),
        FactValueDef::String(s) => format!("\"{s}\""),
        FactValueDef::StringList(l) => format!("{l:?}"),
        FactValueDef::IntList(l) => format!("{l:?}"),
    }
}

fn render_sim_fact_value(ui: &mut egui::Ui, val: &mut SimFactValue) -> bool {
    let mut changed = false;
    match val {
        SimFactValue::Int(v) => {
            let mut f = *v as f64;
            if ui.add(egui::DragValue::new(&mut f).speed(1.0)).changed() {
                *v = f as i64;
                changed = true;
            }
        }
        SimFactValue::Float(v) => {
            if ui.add(egui::DragValue::new(v).speed(0.1)).changed() {
                changed = true;
            }
        }
        SimFactValue::Bool(v) => {
            if ui.checkbox(v, "").changed() {
                changed = true;
            }
        }
        SimFactValue::String(s) => {
            if ui.text_edit_singleline(s).changed() {
                changed = true;
            }
        }
    }
    changed
}

// ─── Fact 模拟器 ────────────────────────────────────────────

fn render_fact_simulator(ui: &mut egui::Ui, state: &mut ViewFreState) -> bool {
    let mut changed = false;

    if state.simulated_facts.is_empty() {
        ui.label("No simulated facts");
    }

    let keys: Vec<String> = state.simulated_facts.keys().cloned().collect();
    for key in &keys {
        if let Some(val) = state.simulated_facts.get_mut(key) {
            ui.horizontal(|ui| {
                ui.label(key);
                changed |= render_sim_fact_value(ui, val);
            });
        }
    }

    // 添加新 fact
    ui.horizontal(|ui| {
        ui.text_edit_singleline(&mut state.new_fact_key);
        if ui.button("+").clicked() && !state.new_fact_key.is_empty() {
            let key = std::mem::take(&mut state.new_fact_key);
            state.simulated_facts.insert(key, SimFactValue::Int(0));
            changed = true;
        }
    });

    changed
}

fn render_live_fact_value(
    ui: &mut egui::Ui,
    key: &str,
    value: &bevy_fact_rule_event::FactValue,
    facts: &mut bevy_fact_rule_event::FactDatabase,
) -> bool {
    use bevy_fact_rule_event::FactValue;
    let mut changed = false;
    match value {
        FactValue::Int(v) => {
            let mut f = *v as f64;
            if ui.add(egui::DragValue::new(&mut f).speed(1.0)).changed() {
                facts.set(key.to_string(), f as i64);
                changed = true;
            }
        }
        FactValue::Float(v) => {
            let mut f = *v;
            if ui.add(egui::DragValue::new(&mut f).speed(0.1)).changed() {
                facts.set(key.to_string(), f);
                changed = true;
            }
        }
        FactValue::Bool(v) => {
            let mut b = *v;
            if ui.checkbox(&mut b, "").changed() {
                facts.set(key.to_string(), b);
                changed = true;
            }
        }
        FactValue::String(s) => {
            let mut s = s.clone();
            if ui.text_edit_singleline(&mut s).changed() {
                facts.set(key.to_string(), s);
                changed = true;
            }
        }
        other => {
            ui.label(format!("{other:?}"));
        }
    }
    changed
}

/// Render Fact Simulator bound to live ViewRoot.local_facts during Play mode.
fn render_live_fact_simulator(
    ui: &mut egui::Ui,
    facts: &mut bevy_fact_rule_event::FactDatabase,
) -> bool {
    use bevy_fact_rule_event::FactValue;

    let mut changed = false;

    if facts.is_empty() {
        ui.label("(no facts)");
        return false;
    }

    let entries: Vec<(String, FactValue)> = facts
        .iter()
        .map(|(k, v)| (k.0.clone(), v.clone()))
        .collect();

    for (key, value) in &entries {
        ui.horizontal(|ui| {
            ui.monospace(key);
            changed |= render_live_fact_value(ui, key, value, facts);
        });
    }

    changed
}
