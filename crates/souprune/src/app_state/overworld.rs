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
pub(crate) mod player;
pub(crate) mod tilemap;
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
        .init_state::<OverworldState>()
        .add_plugins((
            tilemap::TilemapPlugin,
            player::PlayerPlugin,
            character::CharacterPlugin,
            ui::OverworldUIPlugin,
        ))
        .add_systems(
            OnEnter(AppState::Overworld),
            create_overworld_entities_system,
        )
        .add_systems(
            OnExit(AppState::Overworld),
            cleanup_entities_system::<OverworldEntity>,
        )
        .add_systems(Update, bind_camera_target_system.in_set(OverworldUpdate))
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
