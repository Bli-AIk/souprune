//! Runtime tracing infrastructure for FRE events, WASM calls, and fact changes.
//!
//! 运行时追踪基础设施：FRE 事件链、WASM 调用性能、Fact 变更历史。
//!
//! These resources collect diagnostic data that the debug UI (F2 panel) can
//! visualise. Data collection itself does not depend on the `debug` feature —
//! only the UI rendering does. This keeps the trace data available for
//! automated tests or future telemetry without pulling in egui.
//!
//! 这些资源收集诊断数据，供调试 UI（F2 面板）可视化。数据收集本身不依赖
//! `debug` feature——只有 UI 渲染部分依赖。这样即使没有 egui，自动测试
//! 或未来的遥测系统也能使用追踪数据。

use bevy::prelude::*;
use std::collections::{HashMap, VecDeque};

// ── EventTraceLog ───────────────────────────────────────────────────────────

/// Phase classification for traced events.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventPhase {
    /// Physics-level events (collisions, movement).
    Physics,
    /// Logic-level events (FRE rule evaluation, fact mutations).
    Logic,
    /// Presentation-level events (sound, animation, VFX).
    Presentation,
}

/// A single traced event within a frame.
#[derive(Debug, Clone)]
pub struct TracedEvent {
    /// Wall-clock timestamp (seconds since app start).
    pub timestamp: f64,
    /// Which pipeline phase produced this event.
    pub phase: EventPhase,
    /// Short type tag, e.g. `"FactEvent"`, `"RuleTriggered"`, `"CollisionEvent"`.
    pub event_type: String,
    /// Human-readable detail string.
    pub detail: String,
    /// Index into the same frame's `events` vec that caused this one (causal link).
    pub caused_by: Option<usize>,
}

/// All traced events that occurred during a single frame.
#[derive(Debug, Clone)]
pub struct FrameEventTrace {
    pub frame_number: u64,
    pub events: Vec<TracedEvent>,
}

/// Ring-buffered per-frame event trace log.
///
/// 环形缓冲的逐帧事件追踪日志。
#[derive(Resource)]
pub struct EventTraceLog {
    pub frames: VecDeque<FrameEventTrace>,
    pub max_frames: usize,
    /// Accumulator for events within the current frame (flushed at frame end).
    pub current_frame_events: Vec<TracedEvent>,
    pub current_frame_number: u64,
}

impl Default for EventTraceLog {
    fn default() -> Self {
        Self {
            frames: VecDeque::with_capacity(300),
            max_frames: 300,
            current_frame_events: Vec::new(),
            current_frame_number: 0,
        }
    }
}

impl EventTraceLog {
    /// Record a traced event in the current frame.
    pub fn record(
        &mut self,
        phase: EventPhase,
        event_type: impl Into<String>,
        detail: impl Into<String>,
        timestamp: f64,
        caused_by: Option<usize>,
    ) {
        self.current_frame_events.push(TracedEvent {
            timestamp,
            phase,
            event_type: event_type.into(),
            detail: detail.into(),
            caused_by,
        });
    }

    /// Flush current frame events into the ring buffer. Called once per frame.
    pub fn flush_frame(&mut self, frame_number: u64) {
        if self.current_frame_events.is_empty() {
            self.current_frame_number = frame_number;
            return;
        }

        let trace = FrameEventTrace {
            frame_number,
            events: std::mem::take(&mut self.current_frame_events),
        };

        if self.frames.len() >= self.max_frames {
            self.frames.pop_front();
        }
        self.frames.push_back(trace);
        self.current_frame_number = frame_number;
    }

    /// Total number of events across all retained frames.
    pub fn total_events(&self) -> usize {
        self.frames.iter().map(|f| f.events.len()).sum()
    }
}

// ── WasmCallTracer ──────────────────────────────────────────────────────────

/// A single WASM call record within a frame.
#[derive(Debug, Clone)]
pub struct WasmCallRecord {
    pub module_name: String,
    pub interface_name: String,
    pub method_name: String,
    pub duration_us: f64,
}

/// Summary of all WASM calls from a single module within a frame.
#[derive(Debug, Clone, Default)]
pub struct ModuleCallSummary {
    pub calls: usize,
    pub total_us: f64,
    /// method_name → (call_count, total_microseconds).
    pub by_method: HashMap<String, (usize, f64)>,
}

/// Per-frame summary of WASM call statistics.
#[derive(Debug, Clone)]
pub struct WasmFrameSummary {
    pub frame_number: u64,
    pub total_calls: usize,
    pub total_duration_us: f64,
    pub by_module: HashMap<String, ModuleCallSummary>,
}

/// Tracks WASM call counts and durations per frame.
///
/// 追踪每帧的 WASM 调用次数和耗时。
#[derive(Resource)]
pub struct WasmCallTracer {
    /// Raw records for the current frame (flushed at frame end).
    pub current_frame: Vec<WasmCallRecord>,
    /// Ring buffer of per-frame summaries.
    pub history: VecDeque<WasmFrameSummary>,
    pub max_history: usize,
}

impl Default for WasmCallTracer {
    fn default() -> Self {
        Self {
            current_frame: Vec::new(),
            history: VecDeque::with_capacity(300),
            max_history: 300,
        }
    }
}

impl WasmCallTracer {
    /// Record a single WASM call.
    pub fn record(
        &mut self,
        module_name: impl Into<String>,
        interface_name: impl Into<String>,
        method_name: impl Into<String>,
        duration: std::time::Duration,
    ) {
        self.current_frame.push(WasmCallRecord {
            module_name: module_name.into(),
            interface_name: interface_name.into(),
            method_name: method_name.into(),
            duration_us: duration.as_secs_f64() * 1_000_000.0,
        });
    }

    /// Flush the current frame's records into a summary and push to history.
    pub fn flush_frame(&mut self, frame_number: u64) {
        if self.current_frame.is_empty() {
            return;
        }

        let mut by_module: HashMap<String, ModuleCallSummary> = HashMap::new();
        let mut total_calls = 0usize;
        let mut total_duration_us = 0.0f64;

        for record in self.current_frame.drain(..) {
            total_calls += 1;
            total_duration_us += record.duration_us;

            let module_summary = by_module.entry(record.module_name).or_default();
            module_summary.calls += 1;
            module_summary.total_us += record.duration_us;

            let method_key = format!("{}.{}", record.interface_name, record.method_name);
            let method_entry = module_summary.by_method.entry(method_key).or_default();
            method_entry.0 += 1;
            method_entry.1 += record.duration_us;
        }

        let summary = WasmFrameSummary {
            frame_number,
            total_calls,
            total_duration_us,
            by_module,
        };

        if self.history.len() >= self.max_history {
            self.history.pop_front();
        }
        self.history.push_back(summary);
    }

    /// Average WASM time per frame over the retained history window.
    pub fn avg_frame_us(&self) -> f64 {
        if self.history.is_empty() {
            return 0.0;
        }
        let total: f64 = self.history.iter().map(|s| s.total_duration_us).sum();
        total / self.history.len() as f64
    }
}

// ── FactChangeHistory ───────────────────────────────────────────────────────

/// Record of a single fact value change.
#[derive(Debug, Clone)]
pub struct FactChange {
    pub frame_number: u64,
    pub old_value: Option<bevy_fact_rule_event::FactValue>,
    pub new_value: bevy_fact_rule_event::FactValue,
    /// ID of the rule or system that triggered the change.
    pub caused_by: String,
}

/// Tracks per-key fact value change history.
///
/// 追踪每个 Fact 键的值变化历史。
#[derive(Resource)]
pub struct FactChangeHistory {
    /// key → ring buffer of changes.
    pub changes: HashMap<String, VecDeque<FactChange>>,
    pub max_changes_per_key: usize,
}

impl Default for FactChangeHistory {
    fn default() -> Self {
        Self {
            changes: HashMap::new(),
            max_changes_per_key: 100,
        }
    }
}

impl FactChangeHistory {
    /// Record a fact change.
    pub fn record(
        &mut self,
        key: impl Into<String>,
        old_value: Option<bevy_fact_rule_event::FactValue>,
        new_value: bevy_fact_rule_event::FactValue,
        caused_by: impl Into<String>,
        frame_number: u64,
    ) {
        let key = key.into();
        let entry = self
            .changes
            .entry(key)
            .or_insert_with(|| VecDeque::with_capacity(self.max_changes_per_key));

        if entry.len() >= self.max_changes_per_key {
            entry.pop_front();
        }

        entry.push_back(FactChange {
            frame_number,
            old_value,
            new_value,
            caused_by: caused_by.into(),
        });
    }

    /// Get all tracked keys sorted alphabetically.
    pub fn keys_sorted(&self) -> Vec<&str> {
        let mut keys: Vec<&str> = self.changes.keys().map(String::as_str).collect();
        keys.sort_unstable();
        keys
    }
}

// ── Frame-end flush system ──────────────────────────────────────────────────

/// System that flushes per-frame trace buffers at the end of each frame.
pub fn flush_trace_buffers_system(
    mut event_log: ResMut<EventTraceLog>,
    mut wasm_tracer: ResMut<WasmCallTracer>,
    frame_count: Res<bevy::diagnostic::FrameCount>,
) {
    let frame = frame_count.0 as u64;
    event_log.flush_frame(frame);
    wasm_tracer.flush_frame(frame);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_fact_rule_event::FactValue;

    #[test]
    fn event_trace_log_ring_buffer() {
        let mut log = EventTraceLog {
            max_frames: 3,
            ..Default::default()
        };

        for i in 0..5 {
            log.record(EventPhase::Logic, "Test", format!("event {i}"), 0.0, None);
            log.flush_frame(i);
        }

        assert_eq!(log.frames.len(), 3);
        assert_eq!(log.frames.front().unwrap().frame_number, 2);
        assert_eq!(log.frames.back().unwrap().frame_number, 4);
    }

    #[test]
    fn wasm_call_tracer_summarizes() {
        let mut tracer = WasmCallTracer::default();
        tracer.record("mod_a", "danmaku", "on_update", std::time::Duration::from_micros(10));
        tracer.record("mod_a", "danmaku", "on_update", std::time::Duration::from_micros(20));
        tracer.record("mod_b", "custom", "handle", std::time::Duration::from_micros(50));
        tracer.flush_frame(1);

        assert_eq!(tracer.history.len(), 1);
        let summary = &tracer.history[0];
        assert_eq!(summary.total_calls, 3);
        assert_eq!(summary.by_module.len(), 2);
        assert_eq!(summary.by_module["mod_a"].calls, 2);
        assert_eq!(summary.by_module["mod_b"].calls, 1);
    }

    #[test]
    fn fact_change_history_ring_buffer() {
        let mut history = FactChangeHistory {
            max_changes_per_key: 3,
            ..Default::default()
        };

        for i in 0..5 {
            history.record(
                "player.hp",
                Some(FactValue::Int(20 - i)),
                FactValue::Int(20 - i - 1),
                "damage_rule",
                i as u64,
            );
        }

        let changes = &history.changes["player.hp"];
        assert_eq!(changes.len(), 3);
        assert_eq!(changes.front().unwrap().frame_number, 2);
    }
}
