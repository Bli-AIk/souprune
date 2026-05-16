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
use crate::core::fixed_scene::FixedSceneUpdate;
use crate::core::fre_facts;
use crate::core::sequencer::chapter_schema::Chapter;
use crate::core::sequencer::context::{ActiveChapter, ChapterFinished};
use crate::core::view::{ActiveView, ViewRoot};

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

/// Runtime presentation projected into the active battle View.
///
/// 投影到当前战斗 View 的运行时表现状态。
#[derive(Debug, Clone)]
struct BattleSpeechBubblePresentation {
    visible: bool,
    bubble_visual: String,
    bubble_x: f64,
    bubble_y: f64,
    text_x: f64,
    text_y: f64,
    text_width: f64,
}

impl BattleSpeechBubblePresentation {
    fn visible(profile: &BattleSpeechBubbleProfile) -> Self {
        Self {
            visible: true,
            bubble_visual: profile.bubble_visual.to_string(),
            bubble_x: profile.bubble_x,
            bubble_y: profile.bubble_y,
            text_x: profile.text_x,
            text_y: profile.text_y,
            text_width: profile.text_width,
        }
    }

    fn hidden_from_current(view_root: &ViewRoot) -> Self {
        Self {
            visible: false,
            bubble_visual: view_root
                .local_state()
                .get_string("enemy_speech_bubble_visual")
                .unwrap_or("")
                .to_string(),
            bubble_x: view_root
                .local_state()
                .get_float("enemy_speech_bubble_x")
                .unwrap_or(0.0),
            bubble_y: view_root
                .local_state()
                .get_float("enemy_speech_bubble_y")
                .unwrap_or(0.0),
            text_x: view_root
                .local_state()
                .get_float("enemy_speech_text_x")
                .unwrap_or(0.0),
            text_y: view_root
                .local_state()
                .get_float("enemy_speech_text_y")
                .unwrap_or(0.0),
            text_width: view_root
                .local_state()
                .get_float("enemy_speech_text_width")
                .unwrap_or(0.0),
        }
    }

    fn apply_to_view(&self, view_root: &mut ViewRoot) {
        view_root.set_local_value("enemy_speech_visible", FactValue::Bool(self.visible));
        view_root.set_local_value(
            "enemy_speech_bubble_visual",
            FactValue::String(self.bubble_visual.clone()),
        );
        view_root.set_local_value("enemy_speech_bubble_x", FactValue::Float(self.bubble_x));
        view_root.set_local_value("enemy_speech_bubble_y", FactValue::Float(self.bubble_y));
        view_root.set_local_value("enemy_speech_text_x", FactValue::Float(self.text_x));
        view_root.set_local_value("enemy_speech_text_y", FactValue::Float(self.text_y));
        view_root.set_local_value("enemy_speech_text_width", FactValue::Float(self.text_width));
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

/// Plugin that binds battle speech bubble requests to Core Dialogue.
///
/// 将战斗对话气泡请求绑定到 Core Dialogue 的插件。
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
                    .in_set(FixedSceneUpdate),
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

        BattleSpeechBubblePresentation::visible(&profile).apply_to_view(&mut view_root);

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
        if let Some(text_style) = bubble
            .text_style
            .as_deref()
            .filter(|style| !style.is_empty())
        {
            facts.set(
                fre_facts::dialogue_channel_key(&channel, fre_facts::DIALOGUE_TEXT_STYLE_FIELD),
                FactValue::String(text_style.to_string()),
            );
        }

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
        BattleSpeechBubblePresentation::hidden_from_current(&view_root)
            .apply_to_view(&mut view_root);
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

#[cfg(test)]
mod tests {
    use super::*;

    fn speech_bubble(text_style: Option<&str>) -> BattleSpeechBubbleDef {
        speech_bubble_with_advance(text_style, BattleSpeechBubbleAdvance::Manual)
    }

    fn speech_bubble_with_advance(
        text_style: Option<&str>,
        advance: BattleSpeechBubbleAdvance,
    ) -> BattleSpeechBubbleDef {
        BattleSpeechBubbleDef {
            channel: BATTLE_ENEMY_SPEECH_CHANNEL.into(),
            mortar_path: "battle/enemies/mad_dummy.mortar".into(),
            mortar_node: "enemy_speech_manual_intro".into(),
            frame: BattleSpeechBubbleFrame::MadDummyWide,
            advance,
            hide_on_finish: true,
            voice: None,
            typewriter_speed: None,
            text_style: text_style.map(str::to_string),
        }
    }

    fn app_with_speech_bubble(text_style: Option<&str>) -> App {
        let mut app = App::new();
        app.add_message::<BattleSpeechBubbleRequest>();
        app.init_resource::<BattleSpeechBubbleRuntime>();
        app.init_resource::<Time>();
        app.insert_resource(LayeredFactDatabase::new());
        app.world_mut().spawn((
            ViewRoot::new("battle/view/undertale.view.ron".into()),
            ActiveView,
        ));
        app.world_mut().write_message(BattleSpeechBubbleRequest {
            bubble: speech_bubble(text_style),
        });
        app.add_systems(Update, start_battle_speech_bubble_requests);
        app
    }

    #[test]
    fn speech_bubble_presentation_projects_visible_profile_to_view_state() {
        let profile = BattleSpeechBubbleProfile::mad_dummy_wide();
        let presentation = BattleSpeechBubblePresentation::visible(&profile);
        let mut view_root = ViewRoot::new("battle/view/undertale.view.ron".into());

        presentation.apply_to_view(&mut view_root);

        assert_eq!(
            view_root.local_state().get_bool("enemy_speech_visible"),
            Some(true)
        );
        assert_eq!(
            view_root.local_state().get_float("enemy_speech_bubble_x"),
            Some(370.0)
        );
        assert_eq!(
            view_root.local_state().get_float("enemy_speech_bubble_y"),
            Some(80.0)
        );
        assert_eq!(
            view_root.local_state().get_float("enemy_speech_text_x"),
            Some(395.0)
        );
        assert_eq!(
            view_root.local_state().get_float("enemy_speech_text_y"),
            Some(90.0)
        );
        assert_eq!(
            view_root.local_state().get_float("enemy_speech_text_width"),
            Some(190.0)
        );
        assert_eq!(
            view_root
                .local_state()
                .get_string("enemy_speech_bubble_visual"),
            Some("battle/speech_bubble/mad_dummy_wide.png")
        );
    }

    #[test]
    fn default_speech_bubble_does_not_apply_mad_dummy_text_style() {
        let mut app = app_with_speech_bubble(None);

        app.update();

        let facts = app.world().resource::<LayeredFactDatabase>();
        assert_eq!(
            facts.get_string(&fre_facts::dialogue_channel_key(
                BATTLE_ENEMY_SPEECH_CHANNEL,
                fre_facts::DIALOGUE_TEXT_STYLE_FIELD,
            )),
            None,
        );
    }

    #[test]
    fn speech_bubble_applies_explicit_text_style() {
        let mut app = app_with_speech_bubble(Some("mad_dummy"));

        app.update();

        let facts = app.world().resource::<LayeredFactDatabase>();
        assert_eq!(
            facts.get_string(&fre_facts::dialogue_channel_key(
                BATTLE_ENEMY_SPEECH_CHANNEL,
                fre_facts::DIALOGUE_TEXT_STYLE_FIELD,
            )),
            Some("mad_dummy"),
        );
    }

    #[test]
    fn manual_speech_bubble_hides_after_dialogue_finishes() {
        let mut app = app_with_speech_bubble(None);
        app.add_message::<MortarEvent>();
        app.add_systems(Update, update_battle_speech_bubble_runtime);

        app.update();
        {
            let mut facts = app.world_mut().resource_mut::<LayeredFactDatabase>();
            facts.set(
                fre_facts::dialogue_channel_key(BATTLE_ENEMY_SPEECH_CHANNEL, "active"),
                FactValue::Bool(false),
            );
            facts.set(
                fre_facts::dialogue_channel_key(BATTLE_ENEMY_SPEECH_CHANNEL, "finished"),
                FactValue::Bool(true),
            );
        }

        app.update();

        let mut query = app
            .world_mut()
            .query_filtered::<&ViewRoot, With<ActiveView>>();
        let view_root = query
            .single(app.world())
            .expect("test app should have one active view");
        assert_eq!(
            view_root.local_state().get_bool("enemy_speech_visible"),
            Some(false)
        );
    }
}
