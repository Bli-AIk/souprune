//! Battle enemy speech bubble integration.
//!
//! 战斗敌人对话气泡集成。

use bevy::prelude::*;
use bevy_fact_rule_event::{FactValue, LayeredFactDatabase};
use bevy_mortar_bond::MortarEvent;
use souprune_schema::battle::{
    BattleSpeechBubbleAdvance, BattleSpeechBubbleDef, BattleSpeechBubbleFrame,
};

use crate::core::danmaku::{DanmakuTimelineCueEvent, TimelineCueDef};
use crate::core::dialogue::{DialogueChannel, DialogueControllerEntity};
use crate::core::fre_facts;
use crate::core::sequencer::chapter_schema::Chapter;
use crate::core::sequencer::context::{ActiveChapter, ChapterFinished};
use crate::core::view::{ActiveView, ViewRoot};
use crate::preset::battle_runtime::BattleUpdate;

/// Default dialogue channel for battle enemy speech.
///
/// 战斗敌人对话的默认 dialogue 通道。
pub const BATTLE_ENEMY_SPEECH_CHANNEL: &str = "battle_enemy_speech";

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
    /// Typed bubble request data.
    ///
    /// 类型化的气泡请求数据。
    pub bubble: BattleSpeechBubbleDef,
}

#[derive(Resource, Default)]
struct BattleSpeechBubbleRuntime {
    active: Option<BattleSpeechBubbleActive>,
}

struct BattleSpeechBubbleActive {
    channel: String,
    advance: BattleSpeechBubbleAdvance,
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
                    process_battle_speech_bubble_chapter_system
                        .after(crate::core::sequencer::flow::advance_battle_flow_system),
                    forward_danmaku_cues_to_speech_bubble_requests,
                    start_battle_speech_bubble_requests,
                    update_battle_speech_bubble_runtime,
                )
                    .chain()
                    .in_set(BattleUpdate),
            );
    }
}

/// Processes typed battle speech bubble sequencer chapters.
///
/// 处理类型化战斗对话气泡序列章节。
pub fn process_battle_speech_bubble_chapter_system(
    mut commands: Commands,
    query: Query<(Entity, &ActiveChapter), Without<ChapterFinished>>,
    mut requests: MessageWriter<BattleSpeechBubbleRequest>,
) {
    for (entity, active_chapter) in &query {
        let Chapter::BattleSpeechBubble(bubble) = &active_chapter.chapter else {
            continue;
        };

        requests.write(BattleSpeechBubbleRequest {
            bubble: bubble.clone(),
        });
        commands.entity(entity).insert(ChapterFinished);
    }
}

fn resolve_frame(frame: BattleSpeechBubbleFrame) -> BattleSpeechBubbleProfile {
    match frame {
        BattleSpeechBubbleFrame::MadDummyWide => BattleSpeechBubbleProfile::mad_dummy_wide(),
    }
}

fn forward_danmaku_cues_to_speech_bubble_requests(
    mut cues: MessageReader<DanmakuTimelineCueEvent>,
    mut requests: MessageWriter<BattleSpeechBubbleRequest>,
) {
    for cue in cues.read() {
        if let TimelineCueDef::BattleSpeechBubble(bubble) = &cue.cue {
            requests.write(BattleSpeechBubbleRequest {
                bubble: bubble.clone(),
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
        let bubble = &request.bubble;
        let profile = resolve_frame(bubble.frame);
        let channel = fre_facts::normalize_dialogue_channel(&bubble.channel).to_string();
        let is_manual = matches!(bubble.advance, BattleSpeechBubbleAdvance::Manual);
        let timer = match bubble.advance {
            BattleSpeechBubbleAdvance::Manual => None,
            BattleSpeechBubbleAdvance::Timed { duration } => {
                Some(Timer::from_seconds(duration, TimerMode::Once))
            }
        };

        let voice = bubble.voice.as_deref().unwrap_or(profile.voice);
        let typewriter_speed = bubble
            .typewriter_speed
            .map(f64::from)
            .unwrap_or(profile.typewriter_speed);

        view_root
            .local_facts
            .set("enemy_speech_visible", FactValue::Bool(true));
        view_root
            .local_facts
            .set("enemy_speech_bubble_x", FactValue::Float(profile.bubble_x));
        view_root
            .local_facts
            .set("enemy_speech_bubble_y", FactValue::Float(profile.bubble_y));
        view_root
            .local_facts
            .set("enemy_speech_text_x", FactValue::Float(profile.text_x));
        view_root
            .local_facts
            .set("enemy_speech_text_y", FactValue::Float(profile.text_y));
        view_root.local_facts.set(
            "enemy_speech_text_width",
            FactValue::Float(profile.text_width),
        );
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
            FactValue::String(bubble.mortar_path.clone()),
        );
        facts.set(
            fre_facts::DIALOGUE_PENDING_MORTAR_NODE,
            FactValue::String(bubble.mortar_node.clone()),
        );
        facts.set(fre_facts::DIALOGUE_PENDING_START, FactValue::Bool(true));
        facts.set(
            fre_facts::dialogue_channel_key(&channel, "has_typewriter"),
            FactValue::Bool(true),
        );
        facts.set(
            fre_facts::dialogue_channel_key(&channel, "has_focus"),
            FactValue::Bool(is_manual),
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
            advance: bubble.advance,
            timer,
            hide_on_finish: bubble.hide_on_finish,
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

    if matches!(active.advance, BattleSpeechBubbleAdvance::Timed { .. }) {
        for (entity, channel) in &controller_query {
            if channel.name == active.channel {
                mortar_events.write(MortarEvent::stop_dialogue_for(entity));
            }
        }
    }

    runtime.active = None;
}
