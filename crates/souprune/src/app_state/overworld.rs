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

use crate::app_state::{AppState, cleanup_entities_system};
use crate::core::camera::Followable;
use bevy::app::{App, Plugin};
use bevy::prelude::*;

pub(crate) mod character;
mod collision;
pub(crate) mod player;
pub(crate) mod tilemap;
pub mod trigger;
pub(crate) mod ui;

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
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct OverworldUpdate;

pub(crate) struct OverworldPlugin;

impl Plugin for OverworldPlugin {
    fn build(&self, app: &mut App) {
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
            crate::core::ui::CoreUIPlugin,
            bevy_fact_rule_event::FREPlugin,
        ))
        .init_resource::<trigger::LoadedRuleSets>()
        .add_systems(
            OnEnter(AppState::Overworld),
            (
                create_overworld_entities_system,
                trigger::setup_action_handlers_system,
            ),
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
            Update,
            (
                trigger::load_fre_rules_system,
                trigger::register_loaded_rules_system,
                trigger::spawn_demo_trigger_zone_system,
                trigger::trigger_zone_detection_system,
                trigger::apply_hp_change_system,
                trigger::log_fact_changes_system,
            )
                .chain()
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
    }
}

fn create_overworld_entities_system(mut spawn_events: MessageWriter<player::SpawnPlayerRequest>) {
    spawn_events.write(player::SpawnPlayerRequest);
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
