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

use crate::app_state::cleanup_entities_system;
use crate::core::camera::Followable;
use bevy::app::{App, Plugin};
use bevy::prelude::*;

use crate::core::danmaku::{DanmakuSpawnContext, DanmakuUpdate};

pub(crate) mod character;
pub mod chase;
mod collision;
pub(crate) mod player;
pub(crate) mod tilemap;
pub mod trigger;
pub(crate) mod view;

/// Marker component for overworld entities
///
/// 标记 Overworld 实体的组件
#[derive(Component)]
pub(crate) struct OverworldEntity();

/// Overworld substates
///
/// Overworld 子状态
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States)]
pub(crate) enum OverworldState {
    #[default]
    Normal,
    Backpack,
    Cutscene,
    /// Chase state - screen darkened with highlighted entities and tilemap edges
    ///
    /// 追逐战状态 - 屏幕变暗，高亮实体和瓦片地图边缘
    Chase,
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
        use crate::app_state::AppState;

        app.configure_sets(
            Update,
            OverworldUpdate.run_if(in_state(AppState::Overworld)),
        )
        // Note: UIUpdate run_if condition is configured in lib.rs to support both Overworld and Battle
        //
        // 注意：UIUpdate 的运行条件在 lib.rs 中配置，以支持 Overworld 和 Battle 两个状态
        .init_state::<OverworldState>()
        .add_plugins((
            tilemap::TilemapPlugin,
            player::PlayerPlugin,
            character::CharacterPlugin,
            crate::core::view::CoreViewPlugin,
        ))
        .add_systems(
            OnEnter(AppState::Overworld),
            create_overworld_entities_system,
        )
        .add_systems(
            OnExit(AppState::Overworld),
            (
                cleanup_entities_system::<OverworldEntity>,
                stop_bgm_on_exit_system,
            ),
        )
        .add_systems(Update, bind_camera_target_system.in_set(OverworldUpdate))
        .add_systems(
            Update,
            collision::player_tilemap_collision_system
                .after(character::MovementSet)
                .before(crate::core::camera::CameraUpdateSet)
                .in_set(OverworldUpdate),
        )
        .add_systems(
            OnEnter(OverworldState::Backpack),
            player::force_player_idle_on_state_change_system,
        )
        .add_systems(
            OnEnter(OverworldState::Cutscene),
            player::force_player_idle_on_state_change_system,
        );
        // Note: Chase state allows player movement, so we don't force idle

        // FRE + Danmaku integration + Chase
        app.add_plugins(bevy_fact_rule_event::FREPlugin)
            .add_plugins(chase::ChasePlugin)
            // Configure FRETriggerSet to run in OverworldUpdate
            .configure_sets(Update, FRETriggerSet.in_set(OverworldUpdate))
            // Configure DanmakuUpdate to run when in either Battle OR Overworld state
            // This allows the danmaku systems to work in both modes without conflicting in_set
            .configure_sets(
                Update,
                DanmakuUpdate
                    .run_if(in_state(AppState::Battle).or(in_state(AppState::Overworld)))
                    .after(FRETriggerSet),
            )
            .init_resource::<trigger::LoadedRuleSets>()
            .add_systems(
                OnEnter(AppState::Overworld),
                (
                    trigger::setup_action_handlers_system,
                    set_overworld_danmaku_context,
                ),
            )
            .add_systems(
                Update,
                (
                    trigger::load_fre_rules_system,
                    trigger::register_loaded_rules_system,
                    trigger::spawn_demo_trigger_zone_system,
                    trigger::trigger_zone_detection_system,
                    trigger::play_danmaku_on_trigger_system,
                    trigger::handle_chase_state_actions_system,
                    trigger::log_fact_changes_system,
                )
                    .chain()
                    .in_set(FRETriggerSet),
            );
    }
}

fn create_overworld_entities_system(mut spawn_events: MessageWriter<player::SpawnPlayerRequest>) {
    spawn_events.write(player::SpawnPlayerRequest);
}

fn set_overworld_danmaku_context(mut spawn_context: ResMut<DanmakuSpawnContext>) {
    *spawn_context = DanmakuSpawnContext::overworld();
    info!("Danmaku: Set spawn context to Overworld");
}

#[cfg(all(feature = "bevy_kira_audio", not(feature = "firewheel")))]
fn stop_bgm_on_exit_system(
    mut bgm_handle: ResMut<tilemap::CurrentBgmHandle>,
    mut current_map_bgm: ResMut<tilemap::CurrentMapBgm>,
    mut audio_instances: ResMut<Assets<bevy_kira_audio::AudioInstance>>,
) {
    if let Some(handle) = &bgm_handle.0
        && let Some(instance) = audio_instances.get_mut(handle)
    {
        instance.stop(bevy_kira_audio::AudioTween::default());
    }
    bgm_handle.0 = None;
    current_map_bgm.0 = None;
}

#[cfg(feature = "firewheel")]
fn stop_bgm_on_exit_system(
    mut commands: Commands,
    mut bgm_handle: ResMut<tilemap::CurrentBgmHandle>,
    mut current_map_bgm: ResMut<tilemap::CurrentMapBgm>,
) {
    // Despawn BGM entity to stop playback
    if let Some(entity) = bgm_handle.0 {
        commands.entity(entity).despawn();
    }
    bgm_handle.0 = None;
    current_map_bgm.0 = None;
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
