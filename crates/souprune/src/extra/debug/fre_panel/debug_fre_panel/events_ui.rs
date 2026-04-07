//! Renders the Events tab in the FRE debug panel — shows traced events per frame.
//!
//! 渲染 FRE 调试面板的 Events 标签页——按帧显示追踪到的事件及其因果链。

use crate::core::trace::{EventPhase, EventTraceLog};
use bevy::prelude::*;
use bevy_inspector_egui::egui;

/// Render the Events tab.
pub(super) fn render_trace_events_tab(ui: &mut egui::Ui, world: &mut World) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.label("📡 Event Trace Log");
        ui.separator();

        let Some(trace_log) = world.get_resource::<EventTraceLog>() else {
            ui.label("EventTraceLog not available.");
            return;
        };

        let total_frames = trace_log.frames.len();
        let total_events = trace_log.total_events();
        ui.label(format!(
            "Frames retained: {} / {}  |  Total events: {}",
            total_frames, trace_log.max_frames, total_events
        ));
        ui.separator();

        if trace_log.frames.is_empty() {
            ui.label("No events recorded yet.");
            ui.label("Events are captured when FRE rules fire.");
            return;
        }

        // Show frames in reverse order (newest first).
        for frame in trace_log.frames.iter().rev().take(50) {
            let header = format!(
                "▸ Frame {} ({} events)",
                frame.frame_number,
                frame.events.len()
            );
            egui::CollapsingHeader::new(header)
                .default_open(total_frames <= 5)
                .show(ui, |ui| render_frame_events(ui, &frame.events));
        }
    });
}

fn render_frame_events(ui: &mut egui::Ui, events: &[crate::core::trace::TracedEvent]) {
    for (i, event) in events.iter().enumerate() {
        let phase_icon = match event.phase {
            EventPhase::Physics => "⚙",
            EventPhase::Logic => "🧠",
            EventPhase::Presentation => "🎨",
        };
        let indent = if event.caused_by.is_some() {
            "  └─ "
        } else {
            ""
        };
        let causal = event
            .caused_by
            .map(|idx| format!(" (← #{idx})"))
            .unwrap_or_default();

        ui.horizontal(|ui| {
            ui.label(format!(
                "{}#{} {} [{}] {}{}",
                indent, i, phase_icon, event.event_type, event.detail, causal
            ));
        });
    }
}
