//! # beat.rs
//!
//! # beat.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This module provides a beat detection system for synchronizing game events with BGM.
//! It sends events on each beat and subdivisions (whole note to 32nd note).
//!
//! 本模块提供节拍检测系统，用于将游戏事件与 BGM 同步。
//! 在每个节拍和分音符（从全音符到 32 分音符）时发送事件。
//!
//! ## Experimental Feature
//!
//! ## 实验功能
//!
//! This module only runs when the `experimental` feature is enabled.
//!
//! 本模块仅在启用 `experimental` feature 时运行。

use bevy::prelude::*;

// ========== BGM TIMING CONFIGURATION ==========
// BGM 时间配置

/// BPM (Beats Per Minute) of the current BGM.
/// BGM 的 BPM（每分钟节拍数）。
pub const BGM_BPM: f32 = 198.0;

/// Offset in seconds to skip the silent part at the beginning of the BGM.
/// 偏移量（秒），用于跳过 BGM 开头的静音部分。
pub const BGM_OFFSET: f32 = 1.325;

// ========== END BGM TIMING CONFIGURATION ==========

/// Duration of one beat in seconds.
/// 一拍的时长（秒）。
pub const BEAT_DURATION: f32 = 60.0 / BGM_BPM;

/// Note subdivision types.
/// 音符细分类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BeatEvent {
    /// Whole note (1 beat = 1 whole note in 4/4 time, but we use 1 beat as reference)
    /// 全音符
    WholeNote,
    /// Half note (1/2 beat)
    /// 二分音符
    HalfNote,
    /// Quarter note (1/4 beat) - This is the "beat" in most music
    /// 四分音符 - 这是大多数音乐中的"拍"
    QuarterNote,
    /// Eighth note (1/8 beat)
    /// 八分音符
    EighthNote,
    /// Sixteenth note (1/16 beat)
    /// 十六分音符
    SixteenthNote,
    /// Thirty-second note (1/32 beat)
    /// 三十二分音符
    ThirtySecondNote,
}

// Implement Message trait for BeatEvent (required for Bevy 0.17 events)
// 为 BeatEvent 实现 Message trait（Bevy 0.17 事件所需）
impl bevy::ecs::message::Message for BeatEvent {}

impl BeatEvent {
    /// Get the subdivision factor for this note type.
    /// 1 = whole note (every 4 beats), 2 = half note, 4 = quarter note, etc.
    /// 获取此音符类型的细分因子。
    /// 1 = 全音符（每 4 拍），2 = 二分音符，4 = 四分音符，等等。
    pub fn subdivision(&self) -> u32 {
        match self {
            BeatEvent::WholeNote => 1,
            BeatEvent::HalfNote => 2,
            BeatEvent::QuarterNote => 4,
            BeatEvent::EighthNote => 8,
            BeatEvent::SixteenthNote => 16,
            BeatEvent::ThirtySecondNote => 32,
        }
    }

    /// Get the duration of this note in seconds.
    /// 获取此音符的时长（秒）。
    pub fn duration(&self) -> f32 {
        // A whole note is 4 beats
        // 全音符是 4 拍
        (BEAT_DURATION * 4.0) / self.subdivision() as f32
    }
}

/// Resource to track beat timing.
/// 跟踪节拍时间的资源。
#[derive(Resource)]
pub struct BeatTracker {
    /// Time elapsed since BGM started (accounting for offset).
    /// 自 BGM 开始以来经过的时间（考虑偏移量）。
    pub elapsed: f32,

    /// Whether the beat tracker is active.
    /// 节拍跟踪器是否激活。
    pub active: bool,

    /// Count of each subdivision that has been triggered.
    /// 已触发的每种细分的计数。
    pub counts: BeatCounts,
}

/// Counts for each beat subdivision.
/// 每种节拍细分的计数。
#[derive(Default, Clone)]
pub struct BeatCounts {
    pub whole: u32,
    pub half: u32,
    pub quarter: u32,
    pub eighth: u32,
    pub sixteenth: u32,
    pub thirty_second: u32,
}

impl Default for BeatTracker {
    fn default() -> Self {
        Self {
            elapsed: -BGM_OFFSET, // Start negative to account for offset
            active: false,
            counts: BeatCounts::default(),
        }
    }
}

impl BeatTracker {
    /// Get the current beat number for a given subdivision.
    /// 获取给定细分的当前节拍编号。
    pub fn current_beat(&self, subdivision: u32) -> u32 {
        if self.elapsed < 0.0 {
            return 0;
        }
        let note_duration = (BEAT_DURATION * 4.0) / subdivision as f32;
        (self.elapsed / note_duration).floor() as u32
    }

    /// Check if we should trigger an event for this subdivision.
    /// 检查是否应该为此细分触发事件。
    fn should_trigger(&self, subdivision: u32, count: u32) -> bool {
        if self.elapsed < 0.0 {
            return false;
        }
        let current = self.current_beat(subdivision);
        current > count
    }
}

/// Plugin for the beat detection system.
/// 节拍检测系统的插件。
pub struct BeatPlugin;

impl Plugin for BeatPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BeatTracker>()
            .add_message::<BeatEvent>()
            .add_systems(
                Update,
                (
                    activate_beat_tracker_system,
                    update_beat_tracker_system,
                    // ========== TEST METRONOME SYSTEM ==========
                    // This system plays sound effects as a metronome for testing.
                    // DELETE OR DISABLE THIS SYSTEM when no longer needed.
                    // 此系统播放音效作为测试用节拍器。
                    // 不再需要时删除或禁用此系统。
                    //test_metronome_system,
                    // ========== END TEST METRONOME SYSTEM ==========
                )
                    .chain()
                    .in_set(super::super::OverworldUpdate),
            );
    }
}

/// Activate the beat tracker when BGM starts playing.
/// 当 BGM 开始播放时激活节拍跟踪器。
fn activate_beat_tracker_system(
    mut beat_tracker: ResMut<BeatTracker>,
    current_bgm: Res<super::CurrentMapBgm>,
) {
    // Activate when BGM is set and tracker is not yet active
    // 当 BGM 设置且跟踪器尚未激活时激活
    if current_bgm.0.is_some() && !beat_tracker.active {
        beat_tracker.active = true;
        beat_tracker.elapsed = -BGM_OFFSET;
        beat_tracker.counts = BeatCounts::default();
        info!(
            "Beat tracker activated: BPM={}, offset={}s, beat_duration={}s",
            BGM_BPM, BGM_OFFSET, BEAT_DURATION
        );
    }
}

/// Update beat tracker and send beat events.
/// 更新节拍跟踪器并发送节拍事件。
fn update_beat_tracker_system(
    time: Res<Time>,
    mut beat_tracker: ResMut<BeatTracker>,
    mut beat_events: MessageWriter<BeatEvent>,
) {
    if !beat_tracker.active {
        return;
    }

    beat_tracker.elapsed += time.delta_secs();

    // Check and send events for each subdivision
    // 检查并发送每种细分的事件

    // Whole note (every 4 beats)
    if beat_tracker.should_trigger(1, beat_tracker.counts.whole) {
        beat_tracker.counts.whole = beat_tracker.current_beat(1);
        beat_events.write(BeatEvent::WholeNote);
    }

    // Half note
    if beat_tracker.should_trigger(2, beat_tracker.counts.half) {
        beat_tracker.counts.half = beat_tracker.current_beat(2);
        beat_events.write(BeatEvent::HalfNote);
    }

    // Quarter note (the "beat")
    if beat_tracker.should_trigger(4, beat_tracker.counts.quarter) {
        beat_tracker.counts.quarter = beat_tracker.current_beat(4);
        beat_events.write(BeatEvent::QuarterNote);
    }

    // Eighth note
    if beat_tracker.should_trigger(8, beat_tracker.counts.eighth) {
        beat_tracker.counts.eighth = beat_tracker.current_beat(8);
        beat_events.write(BeatEvent::EighthNote);
    }

    // Sixteenth note
    if beat_tracker.should_trigger(16, beat_tracker.counts.sixteenth) {
        beat_tracker.counts.sixteenth = beat_tracker.current_beat(16);
        beat_events.write(BeatEvent::SixteenthNote);
    }

    // Thirty-second note
    if beat_tracker.should_trigger(32, beat_tracker.counts.thirty_second) {
        beat_tracker.counts.thirty_second = beat_tracker.current_beat(32);
        beat_events.write(BeatEvent::ThirtySecondNote);
    }
}

// ========== TEST METRONOME SYSTEM ==========
// This entire system is for testing purposes only.
// DELETE OR COMMENT OUT THIS SYSTEM when no longer needed.
// 此系统仅用于测试目的。
// 不再需要时删除或注释掉此系统。

/// Test metronome system that plays sound effects on beats.
/// Uses "confirm.wav" on the first beat of each bar (4 beats), and "choice.wav" on beats 2-4.
/// 测试用节拍器系统，在节拍时播放音效。
/// 每小节（4拍）的第1拍播放 "confirm.wav"，第2-4拍播放 "choice.wav"。
#[cfg(all(feature = "bevy_kira_audio", not(feature = "firewheel")))]
fn test_metronome_system(
    mut beat_events: MessageReader<BeatEvent>,
    beat_tracker: Res<BeatTracker>,
    audio: Res<bevy_kira_audio::Audio>,
    asset_server: Res<AssetServer>,
) {
    for event in beat_events.read() {
        if matches!(event, BeatEvent::QuarterNote) {
            // Get the current quarter note count (which beat we're on)
            // 获取当前四分音符计数（我们在第几拍）
            let beat_in_bar = (beat_tracker.counts.quarter - 1) % 4;

            if beat_in_bar == 0 {
                // First beat of the bar: play "confirm.wav"
                // 小节的第一拍：播放 "confirm.wav"
                crate::core::audio::play_sound(&audio, &asset_server, "confirm.wav");
            } else {
                // Beats 2-4: play "choice.wav"
                // 第2-4拍：播放 "choice.wav"
                crate::core::audio::play_sound(&audio, &asset_server, "choice.wav");
            }
        }
    }
}

#[cfg(feature = "firewheel")]
fn test_metronome_system(
    mut commands: Commands,
    mut beat_events: MessageReader<BeatEvent>,
    beat_tracker: Res<BeatTracker>,
    asset_server: Res<AssetServer>,
) {
    for event in beat_events.read() {
        if matches!(event, BeatEvent::QuarterNote) {
            // Get the current quarter note count (which beat we're on)
            // 获取当前四分音符计数（我们在第几拍）
            let beat_in_bar = (beat_tracker.counts.quarter - 1) % 4;

            if beat_in_bar == 0 {
                // First beat of the bar: play "confirm.wav"
                // 小节的第一拍：播放 "confirm.wav"
                crate::core::audio::play_sound(&mut commands, &asset_server, "confirm.wav");
            } else {
                // Beats 2-4: play "choice.wav"
                // 第2-4拍：播放 "choice.wav"
                crate::core::audio::play_sound(&mut commands, &asset_server, "choice.wav");
            }
        }
    }
}

// ========== END TEST METRONOME SYSTEM ==========
