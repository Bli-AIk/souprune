//! Renders the WASM Profiler tab in the FRE debug panel — shows WASM call statistics.
//!
//! 渲染 FRE 调试面板的 WASM 标签页——显示 WASM 调用统计（次数、耗时）。

use crate::core::trace::WasmCallTracer;
use bevy::prelude::*;
use bevy_inspector_egui::egui;

/// Render the WASM Profiler tab.
pub(super) fn render_wasm_tab(ui: &mut egui::Ui, world: &mut World) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.label("⚡ WASM Call Profiler");
        ui.separator();

        let Some(tracer) = world.get_resource::<WasmCallTracer>() else {
            ui.label("WasmCallTracer not available.");
            return;
        };

        let history_len = tracer.history.len();
        let avg_us = tracer.avg_frame_us();
        ui.label(format!(
            "History: {} frames  |  Avg WASM time/frame: {:.1}μs ({:.2}ms)",
            history_len,
            avg_us,
            avg_us / 1000.0
        ));

        // Budget bar (assuming 16.67ms = 60fps frame budget).
        let budget_pct = (avg_us / 16670.0 * 100.0).min(100.0);
        ui.horizontal(|ui| {
            ui.label("Budget:");
            let bar = egui::ProgressBar::new(budget_pct as f32 / 100.0)
                .text(format!("{:.1}%", budget_pct));
            ui.add(bar);
        });
        ui.separator();

        if tracer.history.is_empty() {
            ui.label("No WASM calls recorded yet.");
            ui.label("Calls are tracked when danmaku bullets update or custom actions execute.");
            return;
        }

        // Aggregate last N frames for a combined view.
        let window = tracer.history.len().min(60);
        let recent: Vec<_> = tracer.history.iter().rev().take(window).collect();

        let mut module_totals: std::collections::HashMap<
            String,
            (usize, f64, std::collections::HashMap<String, (usize, f64)>),
        > = std::collections::HashMap::new();

        for summary in &recent {
            for (mod_name, mod_summary) in &summary.by_module {
                let entry = module_totals
                    .entry(mod_name.clone())
                    .or_insert_with(|| (0, 0.0, std::collections::HashMap::new()));
                entry.0 += mod_summary.calls;
                entry.1 += mod_summary.total_us;
                for (method, &(count, us)) in &mod_summary.by_method {
                    let me = entry.2.entry(method.clone()).or_insert((0, 0.0));
                    me.0 += count;
                    me.1 += us;
                }
            }
        }

        ui.label(format!("📊 Aggregated over last {} frames:", window));
        ui.separator();

        let mut sorted_modules: Vec<_> = module_totals.iter().collect();
        sorted_modules.sort_by(|a, b| b.1.1.partial_cmp(&a.1.1).unwrap());

        for (mod_name, (total_calls, total_us, methods)) in &sorted_modules {
            let avg_per_call = if *total_calls > 0 {
                total_us / *total_calls as f64
            } else {
                0.0
            };
            let calls_per_frame = *total_calls as f64 / window as f64;

            egui::CollapsingHeader::new(format!("📦 {mod_name}"))
                .default_open(true)
                .show(ui, |ui| {
                    ui.label(format!(
                        "  × {total_calls} calls ({:.0}/frame)  total {:.1}μs  avg {:.1}μs/call",
                        calls_per_frame, total_us, avg_per_call
                    ));

                    let mut sorted_methods: Vec<_> = methods.iter().collect();
                    sorted_methods.sort_by(|a, b| b.1.1.partial_cmp(&a.1.1).unwrap());

                    for (method, (count, us)) in &sorted_methods {
                        let avg = if *count > 0 { us / *count as f64 } else { 0.0 };
                        ui.monospace(format!("    {method}  × {count}  avg {avg:.1}μs"));
                    }
                });
        }

        // Warnings.
        let last = tracer.history.back();
        if let Some(last) = last
            && last.total_calls > 1000
        {
            ui.separator();
            ui.colored_label(
                egui::Color32::YELLOW,
                format!(
                    "⚠ Last frame: {} WASM calls — consider BuiltinMotion for common patterns",
                    last.total_calls
                ),
            );
        }
    });
}
