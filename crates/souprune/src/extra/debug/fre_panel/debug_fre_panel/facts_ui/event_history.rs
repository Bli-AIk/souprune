//! Renders the recent `FactEvent` history captured by the FRE debug panel.
//!
//! 渲染 FRE 调试面板捕获到的最近 `FactEvent` 历史记录。
//!
//! Its scope is intentionally narrow: it shows which events were emitted, when
//! they fired, and what data keys they carried. That timeline is useful when a
//! rule appears not to trigger, because it lets the debugger confirm whether the
//! expected event was ever produced in the first place.
//!
//! 职责刻意很窄：只展示发出了哪些事件、它们何时发生、以及携带了哪些数据键。
//! 当某条规则看起来没有触发时，这条时间线非常有用，因为它能先确认预期事件
//! 到底有没有被发出来。

use super::super::{FactEventHistory, FactEventRecord, MAX_EVENT_HISTORY};
use bevy::prelude::*;
use bevy_inspector_egui::egui;

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
