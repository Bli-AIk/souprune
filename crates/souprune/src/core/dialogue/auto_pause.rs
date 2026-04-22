//! Automatic punctuation-based typewriter pausing.
//!
//! 基于标点符号的打字机自动停顿。
//!
//! Watches typewriter character progress and automatically pauses when
//! punctuation characters are revealed, creating natural dialogue rhythm.
//! All pause rules are loaded from `narrative/dialogue.ron` — resolved through
//! the project's asset root chain (current mod → preset dependencies).
//! Named presets allow per-character or per-scene overrides.
//!
//! 监视打字机字符进度，在显示标点符号时自动暂停，创造自然的对话节奏。
//! 所有停顿规则从 `narrative/dialogue.ron` 加载——通过项目资产根链解析
//! （当前 mod → preset 依赖）。命名预设允许按角色或按场景覆盖。

use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

use bevy::prelude::*;
use bevy_ecs_typewriter::{Typewriter, TypewriterState};
use bevy_fact_rule_event::LayeredFactDatabase;
use souprune_schema::dialogue::{
    AutoPauseConfig as SchemaAutoPauseConfig, DialogueConfig as SchemaDialogueConfig,
};

use super::components::MortarController;
use crate::core::fre_facts;

/// Configuration resource for automatic punctuation pausing.
///
/// 自动标点停顿的配置资源。
///
/// Loaded from `narrative/dialogue.ron` via [`resolve_path`](crate::config::resolve_path).
/// Contains named presets that map punctuation characters to pause durations.
/// No hardcoded defaults — the preset or mod must provide this configuration.
///
/// 通过 [`resolve_path`](crate::config::resolve_path) 从 `narrative/dialogue.ron` 加载。
/// 包含命名预设，将标点字符映射到暂停时长。
/// 没有硬编码默认值——preset 或 mod 必须提供此配置。
#[derive(Resource, Debug, Clone, Default)]
pub struct AutoPauseConfig(pub SchemaAutoPauseConfig);

impl Deref for AutoPauseConfig {
    type Target = SchemaAutoPauseConfig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for AutoPauseConfig {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl AutoPauseConfig {
    /// Returns the rules for the given preset name, falling back to `default_preset`.
    ///
    /// 返回指定预设名称的规则，回退到 `default_preset`。
    pub fn active_rules(&self, preset_name: Option<&str>) -> Option<&HashMap<String, f64>> {
        let name = preset_name.unwrap_or(&self.default_preset);
        self.presets.get(name)
    }
}

/// Tracks the last observed character index for auto-pause change detection.
///
/// 追踪上次观察到的字符索引，用于自动停顿的变化检测。
///
/// Follows the same `last_char_index` pattern as [`TypewriterVoice`](super::TypewriterVoice).
///
/// 与 [`TypewriterVoice`](super::TypewriterVoice) 使用相同的 `last_char_index` 模式。
#[derive(Component, Debug, Default)]
pub struct AutoPauseState {
    /// Last observed `Typewriter::current_char_index`.
    ///
    /// 上次观察到的 `Typewriter::current_char_index`。
    pub last_char_index: usize,
}

/// Timer component for timed typewriter pauses.
///
/// 定时打字机暂停的计时器组件。
///
/// Attached to a typewriter entity when it is paused with a duration.
/// Removed automatically when the timer expires or the pause is cancelled.
///
/// 当打字机被定时暂停时附加到实体上。
/// 计时器到期或暂停被取消时自动移除。
#[derive(Component, Debug)]
pub struct AutoPauseTimer {
    /// Countdown timer for the pause duration.
    ///
    /// 暂停时长的倒计时器。
    pub timer: Timer,
}

impl AutoPauseTimer {
    /// Creates a new timer with the given duration in seconds.
    ///
    /// 用给定的秒数创建新的计时器。
    pub fn new(duration_secs: f64) -> Self {
        Self {
            timer: Timer::from_seconds(duration_secs as f32, TimerMode::Once),
        }
    }
}

/// Ticks auto-pause timers and resumes typewriters when expired.
///
/// 计时自动停顿计时器，到期时恢复打字机。
///
/// Runs **before** `auto_pause_scan_system` so that an expired pause is cleared
/// before the same frame scans for new punctuation triggers.
///
/// 在 `auto_pause_scan_system` **之前** 运行，确保到期暂停在同帧扫描新标点前清除。
pub fn auto_pause_resume_system(
    time: Res<Time>,
    mut query: Query<(Entity, &mut Typewriter, &mut AutoPauseTimer), With<MortarController>>,
    mut commands: Commands,
) {
    for (entity, mut typewriter, mut timer) in &mut query {
        timer.timer.tick(time.delta());

        if timer.timer.just_finished() {
            if typewriter.state == TypewriterState::Paused {
                typewriter.resume();
            }
            commands.entity(entity).remove::<AutoPauseTimer>();
        }
    }
}

/// Scans typewriter progress and pauses on configured punctuation.
///
/// 扫描打字机进度并在配置的标点处暂停。
///
/// Reads the active preset from FRE Fact [`DIALOGUE_AUTO_PAUSE_PRESET`].
/// If no preset override is set, uses [`AutoPauseConfig::default_preset`].
///
/// 从 FRE Fact [`DIALOGUE_AUTO_PAUSE_PRESET`] 读取当前预设。
/// 若未设置覆盖，使用 [`AutoPauseConfig::default_preset`]。
///
/// [`DIALOGUE_AUTO_PAUSE_PRESET`]: fre_facts::DIALOGUE_AUTO_PAUSE_PRESET
pub fn auto_pause_scan_system(
    config: Res<AutoPauseConfig>,
    facts: Res<LayeredFactDatabase>,
    mut query: Query<
        (
            Entity,
            &mut Typewriter,
            &mut AutoPauseState,
            Option<&AutoPauseTimer>,
        ),
        With<MortarController>,
    >,
    mut commands: Commands,
) {
    let enabled = facts
        .get_bool(fre_facts::DIALOGUE_AUTO_PAUSE_ENABLED)
        .unwrap_or(true);
    if !enabled {
        return;
    }

    let preset_name = facts.get_string(fre_facts::DIALOGUE_AUTO_PAUSE_PRESET);
    let Some(rules) = config.active_rules(preset_name) else {
        return;
    };

    for (entity, mut typewriter, mut state, existing_timer) in &mut query {
        if typewriter.state != TypewriterState::Playing {
            state.last_char_index = typewriter.current_char_index;
            continue;
        }

        if existing_timer.is_some() {
            state.last_char_index = typewriter.current_char_index;
            continue;
        }

        if typewriter.current_char_index <= state.last_char_index {
            continue;
        }

        let chars: Vec<char> = typewriter.source_text.chars().collect();
        let mut max_pause: Option<f64> = None;

        for idx in state.last_char_index..typewriter.current_char_index {
            if idx >= chars.len() {
                break;
            }
            let ch = chars[idx].to_string();
            if let Some(&duration) = rules.get(&ch) {
                max_pause = Some(max_pause.map_or(duration, |prev: f64| prev.max(duration)));
            }
        }

        state.last_char_index = typewriter.current_char_index;

        if let Some(duration) = max_pause {
            typewriter.pause();
            commands
                .entity(entity)
                .insert(AutoPauseTimer::new(duration));
        }
    }
}

/// Removes stale [`AutoPauseTimer`] when typewriter is no longer paused.
///
/// 当打字机不再处于暂停状态时移除残留的 [`AutoPauseTimer`]。
///
/// Handles cases where the player skips text or dialogue is stopped externally.
/// Uses [`Changed<Typewriter>`] filter to only run when typewriter state changes.
///
/// 处理玩家跳过文本或对话被外部停止的情况。
/// 使用 [`Changed<Typewriter>`] 过滤器，仅在打字机状态变化时运行。
pub fn auto_pause_cleanup_system(
    query: Query<
        (Entity, &Typewriter, &AutoPauseTimer),
        (With<MortarController>, Changed<Typewriter>),
    >,
    mut commands: Commands,
) {
    for (entity, typewriter, _) in &query {
        if typewriter.state != TypewriterState::Paused {
            commands.entity(entity).remove::<AutoPauseTimer>();
        }
    }
}

/// Startup system: loads dialogue config from `narrative/dialogue.ron`.
///
/// 启动系统：从 `narrative/dialogue.ron` 加载对话配置。
///
/// Uses [`resolve_path`](crate::config::resolve_path) to search the current mod first,
/// then its dependencies (e.g. `undertale_preset`). If no file is found or parsing fails,
/// the default (empty) configs are used, silently disabling auto-pause and voice rules.
///
/// 使用 [`resolve_path`](crate::config::resolve_path) 先搜索当前 mod，
/// 再搜索其依赖（如 `undertale_preset`）。若未找到文件或解析失败，
/// 使用默认（空）配置，自动停顿和语音规则将静默不激活。
pub fn load_dialogue_config_system(
    mut auto_pause: ResMut<AutoPauseConfig>,
    mut voice: ResMut<super::voice_config::VoiceConfig>,
) {
    let config = crate::config::load_config();
    let Some(config_path) = crate::config::resolve_path(&config.game.dialogue_config_path) else {
        info!(
            "[Dialogue] {} not found in any asset root. Config disabled.",
            config.game.dialogue_config_path
        );
        return;
    };

    match std::fs::read_to_string(&config_path) {
        Ok(content) => match ron::from_str::<SchemaDialogueConfig>(&content) {
            Ok(loaded) => {
                info!(
                    "[Dialogue] Loaded config from {}: auto_pause={} presets, voice={} presets",
                    config_path.display(),
                    loaded.auto_pause.presets.len(),
                    loaded.voice.presets.len()
                );
                *auto_pause = AutoPauseConfig(loaded.auto_pause);
                *voice = super::voice_config::VoiceConfig(loaded.voice);
            }
            Err(e) => {
                warn!(
                    "[Dialogue] Failed to parse {}: {}. Config disabled.",
                    config_path.display(),
                    e
                );
            }
        },
        Err(e) => {
            warn!(
                "[Dialogue] Cannot read {}: {}. Config disabled.",
                config_path.display(),
                e
            );
        }
    }
}
