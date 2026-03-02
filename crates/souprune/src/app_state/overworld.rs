//! # overworld.rs
//!
//! # overworld.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! The `overworld` module manages the game's overworld state, including player movement, tilemaps, characters, and UI interactions.
//!
//! `overworld` 模块管理游戏的大地图（Overworld）状态，包括玩家移动、瓦片地图、角色和用户界面交互。
//!
//! It orchestrates sub-plugins and handles camera binding to the player.
//!
//! 它负责协调子插件，并处理相机对玩家的跟随逻辑。

use crate::app_state::{AppState, ModeChanged, ModeScoped, SequenceSubState, is_mode};
use bevy::app::{App, Plugin};
use bevy::ecs::message::MessageReader;
use bevy::prelude::*;

use crate::core::camera::Followable;
use crate::core::danmaku::{DanmakuSpawnContext, DanmakuUpdate};
use bevy_fact_rule_event::{LayeredFactDatabase, LayeredRuleRegistry};

pub(crate) mod character;
pub mod chase;
pub mod chase_config;
pub mod chase_damage;
mod collision;
pub(crate) mod player;
pub(crate) mod tilemap;
pub mod trigger;
pub(crate) mod view;

/// 创建 `ModeScoped("overworld")` 标记的便捷方法。
pub(crate) fn overworld_scoped() -> ModeScoped {
    ModeScoped("overworld".to_string())
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct OverworldUpdate;

/// System set for FRE trigger processing (experimental).
/// Runs before DanmakuUpdate to ensure PlayPerformanceEvent is written first.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct FRETriggerSet;

pub(crate) struct OverworldPlugin;

impl Plugin for OverworldPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(Update, OverworldUpdate.run_if(is_mode("overworld")))
            .add_plugins((
                tilemap::TilemapPlugin,
                player::PlayerPlugin,
                character::CharacterPlugin,
                crate::core::view::CoreViewPlugin,
            ))
            // Mode enter/exit systems react to ModeChanged events
            .add_systems(
                Update,
                (on_enter_overworld_system, on_exit_overworld_system),
            )
            .add_systems(
                Update,
                trigger::setup_action_handlers_system.run_if(on_entering_mode("overworld")),
            )
            .add_systems(Update, bind_camera_target_system.in_set(OverworldUpdate))
            .add_systems(
                Update,
                mark_tilemap_as_overworld_scoped.in_set(OverworldUpdate),
            )
            .add_systems(
                Update,
                process_overworld_player_spawn_system
                    .in_set(crate::core::sequencer::SequencerUpdate),
            )
            .add_systems(
                Update,
                collision::player_tilemap_collision_system
                    .after(character::MovementSet)
                    .before(crate::core::camera::CameraUpdateSet)
                    .in_set(OverworldUpdate),
            )
            .add_systems(
                Update,
                force_player_idle_on_non_movable_state_system.in_set(OverworldUpdate),
            );

        // FRE + Danmaku integration + Chase
        app.add_plugins(bevy_fact_rule_event::FREPlugin)
            .add_plugins(chase::ChasePlugin)
            .configure_sets(Update, FRETriggerSet.in_set(OverworldUpdate))
            .configure_sets(
                Update,
                DanmakuUpdate
                    .run_if(in_state(AppState::Running))
                    .after(FRETriggerSet),
            )
            .init_resource::<trigger::LoadedRuleSets>()
            .init_resource::<trigger::RuleActionDefs>()
            .init_resource::<trigger::PendingDanmakuActions>()
            .init_resource::<trigger::PendingViewActions>()
            .init_resource::<trigger::FocusedInteractable>()
            .add_systems(
                Update,
                (
                    view::input_to_fre_event_bridge_system,
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

/// Helper: returns true when entering a specific mode (for run_if conditions).
fn on_entering_mode(mode: &'static str) -> impl FnMut(MessageReader<ModeChanged>) -> bool {
    move |mut events: MessageReader<ModeChanged>| {
        events.read().any(|e| e.to.as_deref() == Some(mode))
    }
}

/// Reacts to entering overworld mode via ModeChanged event.
fn on_enter_overworld_system(
    mut mode_events: MessageReader<ModeChanged>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    souprune_config: Res<crate::config::SoupruneConfig>,
    mut spawn_context: ResMut<DanmakuSpawnContext>,
) {
    if !mode_events
        .read()
        .any(|e| e.to.as_deref() == Some("overworld"))
    {
        return;
    }

    // Load overworld sequence
    match souprune_config.game.initial_sequence_path {
        Some(ref sequence_path) => {
            let handle = asset_server.load::<crate::core::sequencer::SequenceAsset>(sequence_path);
            commands.insert_resource(crate::core::sequencer::CurrentSequenceFlow(handle));
            info!("Overworld: Loading entry sequence from '{}'", sequence_path);
        }
        None => {
            error!("Overworld: No initial_sequence_path configured in mod.toml.");
        }
    }

    // Set danmaku context
    *spawn_context = DanmakuSpawnContext::with_mode("overworld");
    info!("Danmaku: Set spawn context to overworld");
}

/// Reacts to exiting overworld mode via ModeChanged event.
fn on_exit_overworld_system(
    mut mode_events: MessageReader<ModeChanged>,
    mut bgm_handle: ResMut<tilemap::CurrentBgmHandle>,
    mut current_map_bgm: ResMut<tilemap::CurrentMapBgm>,
    mut audio_instances: ResMut<Assets<bevy_kira_audio::AudioInstance>>,
    mut layered_db: ResMut<LayeredFactDatabase>,
    mut registry: ResMut<LayeredRuleRegistry>,
    mut loaded_rule_sets: ResMut<trigger::LoadedRuleSets>,
) {
    if !mode_events
        .read()
        .any(|e| e.from.as_deref() == Some("overworld"))
    {
        return;
    }

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

    info!("Overworld: Cleaned up on exit (BGM + FRE)");
}

/// Process `SetPlayer(Spawn { .. })` chapters in overworld for non-battle configs.
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
    mut spawn_events: MessageWriter<player::SpawnPlayerRequest>,
) {
    use crate::core::sequencer::chapter_schema::{Chapter, PlayerAction};

    for (entity, active_chapter) in active_chapters.iter() {
        if let Chapter::SetPlayer(PlayerAction::Spawn { config_path, .. }) = &active_chapter.chapter
            && !config_path.ends_with(".battle_player.ron")
        {
            spawn_events.write(player::SpawnPlayerRequest);
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

/// Mark TiledMap entities with ModeScoped("overworld") for cleanup.
fn mark_tilemap_as_overworld_scoped(
    mut commands: Commands,
    query: Query<
        Entity,
        (
            Added<bevy_ecs_tiled::prelude::TiledMap>,
            Without<ModeScoped>,
        ),
    >,
) {
    for entity in query.iter() {
        commands.entity(entity).insert(overworld_scoped());
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
        .and_then(|config| config.0.states.get(&current_state.0))
        .map(|def| def.player_movable)
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
