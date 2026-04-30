//! Battle enemy speech bubble integration.
//!
//! 战斗敌人对话气泡集成。

use bevy::prelude::*;
use bevy_fact_rule_event::{FactValue, LayeredFactDatabase};
use bevy_mortar_bond::MortarEvent;
use std::collections::HashMap;

use crate::core::danmaku::DanmakuTimelineCueEvent;
use crate::core::dialogue::{DialogueChannel, DialogueControllerEntity};
use crate::core::fre_facts;
use crate::core::view::{ActiveView, ViewRoot};
use crate::preset::battle_runtime::BattleUpdate;

/// Custom action and cue name used to request an enemy speech bubble.
///
/// 请求敌人对话气泡的自定义 action 与 cue 名称。
pub const BATTLE_SPEECH_BUBBLE_ACTION: &str = "battle:speech_bubble";

/// Default dialogue channel for battle enemy speech.
///
/// 战斗敌人对话的默认 dialogue 通道。
pub const BATTLE_ENEMY_SPEECH_CHANNEL: &str = "battle_enemy_speech";

/// How the speech bubble advances after it starts.
///
/// 对话气泡启动后的推进方式。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BattleSpeechBubbleAdvanceMode {
    /// Player confirmation advances the Mortar dialogue.
    ///
    /// 由玩家确认键推进 Mortar 对话。
    Manual,
    /// A timer hides the bubble without focus.
    ///
    /// 由计时器隐藏无焦点气泡。
    Timed,
}

impl BattleSpeechBubbleAdvanceMode {
    fn parse(value: Option<&str>) -> Self {
        match value {
            Some("Timed") | Some("timed") => Self::Timed,
            _ => Self::Manual,
        }
    }
}

/// Static presentation profile for a battle speech bubble.
///
/// 战斗对话气泡的静态表现配置。
#[derive(Debug, Clone)]
pub struct BattleSpeechBubbleProfile {
    /// Bubble sprite visual path.
    ///
    /// 气泡贴图资源路径。
    pub bubble_visual: &'static str,
    /// Bubble world x coordinate.
    ///
    /// 气泡世界 x 坐标。
    pub bubble_x: f64,
    /// Bubble world y coordinate.
    ///
    /// 气泡世界 y 坐标。
    pub bubble_y: f64,
    /// Text world x coordinate.
    ///
    /// 文本世界 x 坐标。
    pub text_x: f64,
    /// Text world y coordinate.
    ///
    /// 文本世界 y 坐标。
    pub text_y: f64,
    /// Intended text width in pixels.
    ///
    /// 期望文本宽度（像素）。
    pub text_width: f64,
    /// Default typewriter voice path.
    ///
    /// 默认打字机语音路径。
    pub voice: &'static str,
    /// Default typewriter speed.
    ///
    /// 默认打字机速度。
    pub typewriter_speed: f64,
}

impl BattleSpeechBubbleProfile {
    fn mad_dummy_wide() -> Self {
        Self {
            bubble_visual: "battle/speech_bubble/mad_dummy_wide.png",
            bubble_x: 370.0,
            bubble_y: 80.0,
            text_x: 395.0,
            text_y: 90.0,
            text_width: 190.0,
            voice: "assets/audios/voice/voice_typewriter_default.wav",
            typewriter_speed: 0.03,
        }
    }
}

/// Request to start a battle enemy speech bubble.
///
/// 启动战斗敌人对话气泡的请求。
#[derive(Message, Debug, Clone)]
pub struct BattleSpeechBubbleRequest {
    /// String parameters from a custom action or danmaku cue.
    ///
    /// 来自自定义 action 或弹幕 cue 的字符串参数。
    pub params: HashMap<String, String>,
}

#[derive(Resource, Default)]
struct BattleSpeechBubbleRuntime {
    active: Option<BattleSpeechBubbleActive>,
}

struct BattleSpeechBubbleActive {
    channel: String,
    mode: BattleSpeechBubbleAdvanceMode,
    timer: Option<Timer>,
    hide_on_finish: bool,
}

/// Plugin that binds preset battle speech bubbles to Core Dialogue.
///
/// 将预设战斗对话气泡绑定到 Core Dialogue 的插件。
pub struct BattleSpeechBubblePlugin;

impl Plugin for BattleSpeechBubblePlugin {
    fn build(&self, app: &mut App) {
        let schedule = crate::game_schedule(app);
        app.add_message::<BattleSpeechBubbleRequest>()
            .init_resource::<BattleSpeechBubbleRuntime>()
            .add_systems(
                schedule,
                (
                    forward_danmaku_cues_to_speech_bubble_requests,
                    start_battle_speech_bubble_requests,
                    update_battle_speech_bubble_runtime,
                )
                    .chain()
                    .in_set(BattleUpdate),
            );
    }
}

fn param<'a>(params: &'a HashMap<String, String>, key: &str) -> Option<&'a str> {
    params
        .get(key)
        .map(String::as_str)
        .filter(|value| !value.is_empty())
}

fn parse_f64(params: &HashMap<String, String>, key: &str, default: f64) -> f64 {
    param(params, key)
        .and_then(|value| value.parse::<f64>().ok())
        .unwrap_or(default)
}

fn parse_bool(params: &HashMap<String, String>, key: &str, default: bool) -> bool {
    param(params, key)
        .and_then(|value| value.parse::<bool>().ok())
        .unwrap_or(default)
}

fn resolve_profile(params: &HashMap<String, String>) -> BattleSpeechBubbleProfile {
    match param(params, "bubble_profile") {
        Some("mad_dummy_wide") | None => BattleSpeechBubbleProfile::mad_dummy_wide(),
        Some(other) => {
            warn!(
                "Unknown speech bubble profile '{}', using mad_dummy_wide",
                other
            );
            BattleSpeechBubbleProfile::mad_dummy_wide()
        }
    }
}

fn forward_danmaku_cues_to_speech_bubble_requests(
    mut cues: MessageReader<DanmakuTimelineCueEvent>,
    mut requests: MessageWriter<BattleSpeechBubbleRequest>,
) {
    for cue in cues.read() {
        if cue.action_type == BATTLE_SPEECH_BUBBLE_ACTION {
            requests.write(BattleSpeechBubbleRequest {
                params: cue.params.clone(),
            });
        }
    }
}

fn start_battle_speech_bubble_requests(
    mut requests: MessageReader<BattleSpeechBubbleRequest>,
    mut facts: ResMut<LayeredFactDatabase>,
    mut view_roots: Query<&mut ViewRoot, With<ActiveView>>,
    mut runtime: ResMut<BattleSpeechBubbleRuntime>,
) {
    let Ok(mut view_root) = view_roots.single_mut() else {
        return;
    };

    for request in requests.read() {
        let Some(mortar_path) = param(&request.params, "mortar_path") else {
            warn!("Battle speech bubble request missing mortar_path");
            continue;
        };
        let Some(mortar_node) = param(&request.params, "mortar_node") else {
            warn!("Battle speech bubble request missing mortar_node");
            continue;
        };

        let profile = resolve_profile(&request.params);
        let channel = param(&request.params, "channel")
            .map(fre_facts::normalize_dialogue_channel)
            .unwrap_or(BATTLE_ENEMY_SPEECH_CHANNEL)
            .to_string();
        let mode = BattleSpeechBubbleAdvanceMode::parse(param(&request.params, "advance_mode"));
        let duration = parse_f64(&request.params, "duration", 2.0);
        let hide_on_finish = parse_bool(&request.params, "hide_on_finish", true);

        let bubble_x = parse_f64(&request.params, "bubble_x", profile.bubble_x);
        let bubble_y = parse_f64(&request.params, "bubble_y", profile.bubble_y);
        let text_x = parse_f64(&request.params, "text_x", profile.text_x);
        let text_y = parse_f64(&request.params, "text_y", profile.text_y);
        let text_width = parse_f64(&request.params, "text_width", profile.text_width);
        let voice = param(&request.params, "voice").unwrap_or(profile.voice);
        let typewriter_speed = parse_f64(
            &request.params,
            "typewriter_speed",
            profile.typewriter_speed,
        );

        view_root
            .local_facts
            .set("enemy_speech_visible", FactValue::Bool(true));
        view_root
            .local_facts
            .set("enemy_speech_bubble_x", FactValue::Float(bubble_x));
        view_root
            .local_facts
            .set("enemy_speech_bubble_y", FactValue::Float(bubble_y));
        view_root
            .local_facts
            .set("enemy_speech_text_x", FactValue::Float(text_x));
        view_root
            .local_facts
            .set("enemy_speech_text_y", FactValue::Float(text_y));
        view_root
            .local_facts
            .set("enemy_speech_text_width", FactValue::Float(text_width));
        view_root.local_facts.set(
            "enemy_speech_bubble_visual",
            FactValue::String(profile.bubble_visual.to_string()),
        );

        facts.set(
            fre_facts::DIALOGUE_PENDING_CHANNEL,
            FactValue::String(channel.clone()),
        );
        facts.set(
            fre_facts::DIALOGUE_PENDING_MORTAR_PATH,
            FactValue::String(mortar_path.to_string()),
        );
        facts.set(
            fre_facts::DIALOGUE_PENDING_MORTAR_NODE,
            FactValue::String(mortar_node.to_string()),
        );
        facts.set(fre_facts::DIALOGUE_PENDING_START, FactValue::Bool(true));
        facts.set(
            fre_facts::dialogue_channel_key(&channel, "has_typewriter"),
            FactValue::Bool(true),
        );
        facts.set(
            fre_facts::dialogue_channel_key(&channel, "has_focus"),
            FactValue::Bool(mode == BattleSpeechBubbleAdvanceMode::Manual),
        );
        facts.set(
            fre_facts::dialogue_channel_key(&channel, "voice"),
            FactValue::String(voice.to_string()),
        );
        facts.set(
            fre_facts::dialogue_channel_key(&channel, "typewriter_speed"),
            FactValue::Float(typewriter_speed),
        );

        runtime.active = Some(BattleSpeechBubbleActive {
            channel,
            mode,
            timer: (mode == BattleSpeechBubbleAdvanceMode::Timed)
                .then(|| Timer::from_seconds(duration as f32, TimerMode::Once)),
            hide_on_finish,
        });
    }
}

fn update_battle_speech_bubble_runtime(
    time: Res<Time>,
    mut runtime: ResMut<BattleSpeechBubbleRuntime>,
    mut view_roots: Query<&mut ViewRoot, With<ActiveView>>,
    facts: Res<LayeredFactDatabase>,
    mut mortar_events: MessageWriter<MortarEvent>,
    controller_query: Query<(Entity, &DialogueChannel), With<DialogueControllerEntity>>,
) {
    let Some(active) = runtime.active.as_mut() else {
        return;
    };

    let mut should_hide = false;
    if let Some(timer) = active.timer.as_mut() {
        timer.tick(time.delta());
        should_hide = timer.just_finished();
    } else if active.hide_on_finish {
        should_hide = !facts
            .get_bool(&fre_facts::dialogue_channel_key(&active.channel, "active"))
            .unwrap_or(false)
            && facts
                .get_bool(&fre_facts::dialogue_channel_key(
                    &active.channel,
                    "finished",
                ))
                .unwrap_or(false);
    }

    if !should_hide {
        return;
    }

    if let Ok(mut view_root) = view_roots.single_mut() {
        view_root
            .local_facts
            .set("enemy_speech_visible", FactValue::Bool(false));
    }

    if active.mode == BattleSpeechBubbleAdvanceMode::Timed {
        for (entity, channel) in &controller_query {
            if channel.name == active.channel {
                mortar_events.write(MortarEvent::stop_dialogue_for(entity));
            }
        }
    }

    runtime.active = None;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn advance_mode_defaults_to_manual() {
        assert_eq!(
            BattleSpeechBubbleAdvanceMode::parse(None),
            BattleSpeechBubbleAdvanceMode::Manual
        );
    }

    #[test]
    fn advance_mode_accepts_timed_values() {
        assert_eq!(
            BattleSpeechBubbleAdvanceMode::parse(Some("Timed")),
            BattleSpeechBubbleAdvanceMode::Timed
        );
        assert_eq!(
            BattleSpeechBubbleAdvanceMode::parse(Some("timed")),
            BattleSpeechBubbleAdvanceMode::Timed
        );
    }
}
