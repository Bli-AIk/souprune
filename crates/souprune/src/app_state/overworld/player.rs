//! # player.rs
//!
//! # player.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This module manages overworld player movement, facing, and animation states.
//!
//! 该模块负责 Overworld 玩家移动、朝向与动画状态的管理。
//!
//! ## Source File Overview
//!
//! ## 源文件概述
//!
//! It defines `PlayerPlugin`, which wires systems for direction control and idle/walk/run animations.
//!
//! 文件定义了 `PlayerPlugin`，负责连接方向控制以及空闲/行走/奔跑动画系统。

use crate::app_state::overworld::player::config::PlayerBehavior;
use crate::core::animation::components::SpriteAnimationClip;
use crate::core::input::PlayerInputSettings;
use crate::core::sprite::params::SpriteParams;
use bevy::app::{App, Plugin, Update};
use bevy::log::error;
use bevy::prelude::*;

pub mod components;
pub(crate) mod config;
mod systems;
pub(crate) mod utils;

#[derive(Clone)]
pub struct SpawnPlayerRequest;

impl Message for SpawnPlayerRequest {}

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        use systems::*;
        let behavior =
            config::PlayerBehavior::load().expect("Failed to load player behavior configuration");

        app.insert_resource(behavior)
            .add_message::<SpawnPlayerRequest>()
            .add_systems(
                Update,
                (player_direction_control_system, spawn_player_on_event),
            );
    }
}

use crate::app_state::overworld::character;

fn spawn_player_on_event(
    mut events: MessageReader<SpawnPlayerRequest>,
    mut commands: Commands,
    mut sprite_params: SpriteParams,
    player_input: Res<PlayerInputSettings>,
    asset_server: Res<AssetServer>,
    player_behavior: Res<PlayerBehavior>,
) {
    if events.read().next().is_none() {
        return;
    }

    spawn_overworld_player(
        &mut commands,
        &mut sprite_params,
        &player_input,
        &asset_server,
        &player_behavior,
    );
}

pub(super) fn force_player_idle_on_state_change_system(
    mut commands: Commands,
    players: Query<Entity, With<character::components::PlayerControlled>>,
) {
    for entity in players.iter() {
        commands
            .entity(entity)
            .remove::<character::components::StateWalking>()
            .remove::<character::components::StateRunning>()
            .insert(character::components::StateIdle);
    }
}

pub fn spawn_overworld_player(
    commands: &mut Commands,
    sprite_params: &mut SpriteParams,
    player_input: &Res<PlayerInputSettings>,
    asset_server: &Res<AssetServer>,
    player_behavior: &Res<PlayerBehavior>,
) {
    use crate::app_state::overworld::character::components::*;
    use crate::app_state::overworld::player::utils::{is_player_running, is_player_walking};
    use crate::core::character_asset::{AnimationConfigAsset, CharacterAnimator};
    use crate::core::collision::Rect2DCollider;
    use crate::core::input::Action;
    use leafwing_input_manager::action_state::ActionState;
    use seldom_state::machine::StateMachine;
    use seldom_state::prelude::IntoTrigger;

    let anim_config: Handle<AnimationConfigAsset> =
        asset_server.load(&player_behavior.animation_config_path);

    let initial_clip = match SpriteAnimationClip::new(
        &mut sprite_params.create_sprite_context(),
        &player_behavior.sprite_source,
        &player_behavior.initial_clip,
    ) {
        Ok(clip) => clip,
        Err(e) => {
            error!(
                "Failed to load initial player animation '{}': {}. Using fallback.",
                player_behavior.initial_clip, e
            );
            SpriteAnimationClip::fallback(
                &mut sprite_params.create_sprite_context(),
                &player_behavior.sprite_source,
                &player_behavior.initial_clip,
            )
        }
    };

    let mut state_machine = StateMachine::default()
        .trans::<StateIdle, _>(is_player_walking, StateWalking)
        .trans::<StateWalking, _>(is_player_walking.not(), StateIdle)
        .trans::<StateRunning, _>(is_player_walking.not(), StateIdle);

    if player_behavior.run_action.is_some() {
        state_machine = state_machine
            .trans::<StateWalking, _>(is_player_running, StateRunning)
            .trans::<StateRunning, _>(is_player_running.not(), StateWalking);
    }

    commands.spawn((
        Name::new("OverworldPlayer"),
        StateIdle,
        PlayerControlled,
        state_machine,
        player_input.get_merged_map(),
        ActionState::<Action>::default(),
        Rect2DCollider::new(
            player_behavior.collider_size,
            player_behavior.collider_offset,
        ),
        CharacterBundle::new(
            player_behavior.spawn_position,
            player_behavior.initial_facing,
            player_behavior.base_speed,
            initial_clip,
            CharacterAnimator {
                config: anim_config,
            },
        ),
    ));
}
