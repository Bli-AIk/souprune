//! # top_down.rs
//!
//! # top_down.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! The `top_down` module manages the game's top_down state, including player movement, tilemaps, characters, and UI interactions.
//!
//! `top_down` 模块管理游戏的大地图（Overworld）状态，包括玩家移动、瓦片地图、角色和用户界面交互。
//!
//! It orchestrates sub-plugins and handles camera binding to the player.
//!
//! 它负责协调子插件，并处理相机对玩家的跟随逻辑。

use crate::config::ModePrimitiveConfig;
use crate::core::mode::{
    AppState, ModeRegistry, ModeScoped, SequenceMode, SequenceSubState, current_mode_has_primitive,
    on_entering_mode_with_primitive, on_exiting_mode_with_primitive,
};
use bevy::app::{App, Plugin};
use bevy::prelude::*;

use crate::core::camera::Followable;
use crate::core::danmaku::{DanmakuSpawnContext, DanmakuUpdate};
use bevy_fact_rule_event::LayeredFactDatabase;

use crate::core::game_action::{GameActionDef, GameRuleRegistry};

pub(crate) mod character;
pub mod chase;
pub mod chase_config;
pub mod chase_damage;
mod collision;
pub(crate) mod player;
mod screen_facts;
pub(crate) mod tilemap;
pub mod trigger;

/// Create a scope marker for the active top-down mode.
///
/// 为当前 top-down mode 创建作用域标记。
pub(crate) fn top_down_scoped(sequence_mode: &SequenceMode) -> Option<ModeScoped> {
    sequence_mode
        .0
        .as_ref()
        .map(|mode| ModeScoped(mode.clone()))
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TopDownUpdate;

/// System set for FRE trigger processing (experimental).
/// Runs before DanmakuUpdate to ensure PlayPerformanceEvent is written first.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct FRETriggerSet;

pub struct TopDownPlugin;

impl Plugin for TopDownPlugin {
    fn build(&self, app: &mut App) {
        let schedule = crate::game_schedule(app);
        app.configure_sets(
            schedule,
            TopDownUpdate.run_if(current_mode_has_primitive(ModePrimitiveConfig::TopDownMap)),
        )
        .add_plugins((
            tilemap::TilemapPlugin,
            player::PlayerPlugin,
            character::CharacterPlugin,
            crate::core::view::CoreViewPlugin,
        ))
        // Mode enter/exit systems react to ModeChanged events
        .add_systems(
            schedule,
            on_enter_overworld_system.run_if(on_entering_mode_with_primitive(
                ModePrimitiveConfig::TopDownMap,
            )),
        )
        .add_systems(
            schedule,
            on_exit_overworld_system.run_if(on_exiting_mode_with_primitive(
                ModePrimitiveConfig::TopDownMap,
            )),
        )
        .add_systems(
            schedule,
            trigger::setup_action_handlers_system.run_if(on_entering_mode_with_primitive(
                ModePrimitiveConfig::InteractionZones,
            )),
        )
        .add_systems(schedule, bind_camera_target_system.in_set(TopDownUpdate))
        .add_systems(
            schedule,
            screen_facts::sync_overworld_screen_facts_system
                .after(crate::core::camera::CameraUpdateSet)
                .before(crate::core::view::lifecycle::StateViewTransitionSet)
                .before(bevy_fact_rule_event::FRESystemSet::EmitEvents)
                .in_set(TopDownUpdate),
        )
        .add_systems(
            schedule,
            mark_tilemap_as_overworld_scoped.in_set(TopDownUpdate),
        )
        .add_systems(
            schedule,
            process_overworld_player_spawn_system.in_set(crate::core::sequencer::SequencerUpdate),
        )
        .add_systems(
            schedule,
            collision::player_tilemap_collision_system
                .after(character::MovementSet)
                .before(crate::core::camera::CameraUpdateSet)
                .in_set(TopDownUpdate),
        )
        .add_systems(
            schedule,
            force_player_idle_on_non_movable_state_system.in_set(TopDownUpdate),
        );

        // FRE + Danmaku integration + Chase
        app.add_plugins({
            let mut plugin = bevy_fact_rule_event::FREPlugin::<GameActionDef>::default();
            plugin.schedule = Some(schedule);
            plugin
        })
        .add_plugins(chase::ChasePlugin)
        .configure_sets(schedule, FRETriggerSet.in_set(TopDownUpdate))
        .configure_sets(
            schedule,
            DanmakuUpdate
                .run_if(in_state(AppState::Running))
                .after(FRETriggerSet),
        )
        .init_resource::<trigger::LoadedRuleSets>()
        .init_resource::<trigger::PendingDanmakuActions>()
        .init_resource::<trigger::PendingViewActions>()
        .init_resource::<trigger::FocusedInteractable>()
        .add_systems(
            schedule,
            (
                trigger::load_fre_rules_system,
                trigger::register_loaded_rules_system,
                trigger::trigger_zone_detection_system,
                trigger::interactable_detection_system,
                trigger::handle_interaction_input_system,
                trigger::handle_overworld_custom_actions_system,
                trigger::apply_pending_view_actions_system,
                trigger::play_danmaku_from_actions_system,
                trigger::log_fact_changes_system,
            )
                .chain()
                .in_set(FRETriggerSet),
        );
    }
}

/// Reacts to entering top_down mode via ModeChanged event.
fn on_enter_overworld_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    sequence_mode: Res<SequenceMode>,
    mode_registry: Res<ModeRegistry>,
    mut spawn_context: ResMut<DanmakuSpawnContext>,
) {
    let Some(mode_name) = sequence_mode.0.as_deref() else {
        warn!("TopDown: no active mode while entering top-down primitive.");
        return;
    };

    let Some(mode_config) = mode_registry.mode(mode_name) else {
        warn!("TopDown: active mode '{mode_name}' is not registered.");
        return;
    };

    if let Some(sequence_path) = mode_config.entry_sequence.as_deref() {
        let handle =
            asset_server.load::<crate::core::sequencer::SequenceAsset>(sequence_path.to_string());
        commands.insert_resource(crate::core::sequencer::CurrentSequenceFlow(handle));
        info!("TopDown: Loading entry sequence from '{sequence_path}' for mode '{mode_name}'");
    } else {
        warn!("TopDown: mode '{mode_name}' has no entry_sequence.");
    }

    // Set danmaku context
    *spawn_context = DanmakuSpawnContext::with_mode(mode_name);
    info!("Danmaku: Set spawn context to {mode_name}");
}

/// Reacts to exiting top_down mode via ModeChanged event.
fn on_exit_overworld_system(
    mut bgm_handle: ResMut<tilemap::CurrentBgmHandle>,
    mut current_map_bgm: ResMut<tilemap::CurrentMapBgm>,
    mut audio_instances: ResMut<Assets<bevy_kira_audio::AudioInstance>>,
    mut layered_db: ResMut<LayeredFactDatabase>,
    mut registry: ResMut<GameRuleRegistry>,
    mut loaded_rule_sets: ResMut<trigger::LoadedRuleSets>,
) {
    // Stop BGM
    if let Some(handle) = &bgm_handle.0
        && let Some(instance) = audio_instances.get_mut(handle)
    {
        instance.stop(bevy_kira_audio::AudioTween::default());
    }
    bgm_handle.0 = None;
    current_map_bgm.0 = None;

    // Clean up FRE state
    layered_db.clear_local();
    registry.clear_local();
    loaded_rule_sets.handles.clear();
    loaded_rule_sets.initialized = false;
    loaded_rule_sets.registered = false;

    info!("Overworld: Cleaned up on exit (BGM + FRE)");
}

/// Process `SetPlayer(Spawn { .. })` chapters in top_down for non-battle configs.
/// Sends `SpawnPlayerRequest` using the already-loaded `PlayerBehavior` config.
///
/// 处理 Overworld 中非战斗配置的 `SetPlayer(Spawn { .. })` 章节。
/// 使用已加载的 `PlayerBehavior` 配置发送 `SpawnPlayerRequest`。
fn process_overworld_player_spawn_system(
    mut commands: Commands,
    active_chapters: Query<
        (Entity, &crate::core::sequencer::ActiveChapter),
        (
            Without<crate::core::sequencer::WaitTimer>,
            Without<crate::core::sequencer::ChapterFinished>,
        ),
    >,
    mut spawn_events: Option<MessageWriter<player::SpawnPlayerRequest>>,
) {
    use crate::core::sequencer::chapter_schema::{Chapter, PlayerAction};

    for (entity, active_chapter) in active_chapters.iter() {
        if let Chapter::SetPlayer(PlayerAction::Spawn { config_path, .. }) = &active_chapter.chapter
        {
            if let Some(ref mut writer) = spawn_events {
                writer.write(player::SpawnPlayerRequest);
            }
            commands
                .entity(entity)
                .insert(crate::core::sequencer::ChapterFinished);
            info!("Overworld: Spawning player from config '{}'", config_path);
        }
    }
}

fn bind_camera_target_system(
    mut camera: Query<&mut Followable, With<Camera2d>>,
    player: Query<Entity, Added<character::components::PlayerControlled>>,
) {
    for player_entity in player.iter() {
        for mut followable in camera.iter_mut() {
            followable.target = Some(player_entity);
        }
    }
}

/// Mark TiledMap entities with the active mode scope for cleanup.
fn mark_tilemap_as_overworld_scoped(
    mut commands: Commands,
    sequence_mode: Res<SequenceMode>,
    query: Query<
        Entity,
        (
            Added<bevy_ecs_tiled::prelude::TiledMap>,
            Without<ModeScoped>,
        ),
    >,
) {
    let Some(scope) = top_down_scoped(&sequence_mode) else {
        return;
    };
    for entity in query.iter() {
        commands.entity(entity).insert(scope.clone());
    }
}

/// System to force player idle when entering a non-movable state.
/// Reads player_movable from StateConfig.
///
/// 当进入不允许移动的状态时强制玩家空闲的系统。
/// 从 StateConfig 读取 player_movable。
fn force_player_idle_on_non_movable_state_system(
    mut commands: Commands,
    current_state: Res<State<SequenceSubState>>,
    state_config: Option<Res<crate::core::state_config::LoadedStateConfig>>,
    player_query: Query<
        Entity,
        (
            With<character::components::PlayerControlled>,
            Or<(
                With<character::components::StateWalking>,
                With<character::components::StateRunning>,
            )>,
        ),
    >,
    mut last_state: Local<String>,
) {
    // Only trigger when state changes
    if current_state.0 == *last_state {
        return;
    }
    *last_state = current_state.0.clone();

    // Check if current state allows movement
    let player_movable = state_config
        .as_ref()
        .map(|config| config.is_player_movable(&current_state.0))
        .unwrap_or(true); // Default to movable if no config

    // Force idle if movement is not allowed - remove walking/running components
    if !player_movable {
        for entity in player_query.iter() {
            commands
                .entity(entity)
                .remove::<character::components::StateWalking>()
                .remove::<character::components::StateRunning>()
                .insert(character::components::StateIdle);
        }
        info!(
            "Overworld: Forced player idle on entering non-movable state '{}'",
            current_state.0
        );
    }
}
