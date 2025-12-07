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

use crate::core::animation::components::{
    SpriteAnimationClip, SpriteAnimationCurrentFrame, SpriteAnimationTimer,
};
use crate::core::basic_components::{Direction, Facing};
use crate::core::input::PlayerInputSettings;
use crate::core::sprite::params::SpriteParams;
use bevy::app::{App, Plugin, Update};
use bevy::log::error;
use bevy::prelude::*;
use bevy::prelude::{Commands, Res, Sprite};

pub mod components;
mod systems;
pub(crate) mod utils;

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        use systems::*;
        app.add_systems(
            Update,
            (
                player_direction_control_system,
                player_idle_anim_control_system,
                player_walk_anim_control_system,
                player_run_anim_control_system,
            ),
        );
    }
}

fn update_player_animation(
    sprite_params: &mut SpriteParams,
    facing: &Facing,
    sprite: &mut Sprite,
    clip: &mut SpriteAnimationClip,
    frame: &mut SpriteAnimationCurrentFrame,
    timer: &mut SpriteAnimationTimer,
    animation_prefix: &str,
) {
    let clip_name = match facing.value {
        Direction::Up => format!("{}_up", animation_prefix),
        Direction::Down => format!("{}_down", animation_prefix),
        Direction::Left => format!("{}_left", animation_prefix),
        Direction::Right => format!("{}_right", animation_prefix),
        Direction::UpLeft => format!("{}_up", animation_prefix),
        Direction::UpRight => format!("{}_up", animation_prefix),
        Direction::DownLeft => format!("{}_down", animation_prefix),
        Direction::DownRight => format!("{}_down", animation_prefix),
    };

    if clip.clip_name() != clip_name {
        let new_clip =
            change_sprite_animation(sprite_params, frame, timer, "overworld", &clip_name);
        *clip = new_clip;
        *sprite = clip.get_current_sprite().clone();
    }
}

fn change_sprite_animation(
    sprite_params: &mut SpriteParams,
    current_frame: &mut SpriteAnimationCurrentFrame,
    timer: &mut SpriteAnimationTimer,
    module_name: &str,
    clip_name: &str,
) -> SpriteAnimationClip {
    current_frame.value = 0;

    timer.reset();

    match SpriteAnimationClip::new(
        &mut sprite_params.create_sprite_context(),
        module_name,
        clip_name,
    ) {
        Ok(clip) => clip,
        Err(e) => {
            error!(
                "Failed to change animation to {}: {}. Using fallback.",
                clip_name, e
            );
            SpriteAnimationClip::fallback(module_name, clip_name)
        }
    }
}

use crate::app_state::overworld::character;

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
) {
    use crate::app_state::overworld::character::components::*;
    use crate::app_state::overworld::player::components::PlayerBundle;
    use crate::app_state::overworld::player::utils::{is_player_running, is_player_walking};
    use crate::core::collision::Rect2DCollider;
    use crate::core::input::Action;
    use bevy::math::Vec2;
    use leafwing_input_manager::action_state::ActionState;
    use seldom_state::machine::StateMachine;
    use seldom_state::prelude::IntoTrigger;

    let initial_clip = match SpriteAnimationClip::new(
        &mut sprite_params.create_sprite_context(),
        "overworld",
        "frisk_walk_down",
    ) {
        Ok(clip) => clip,
        Err(e) => {
            error!(
                "Failed to load initial player animation 'frisk_walk_down': {}. Player spawn aborted.",
                e
            );
            return;
        }
    };

    commands.spawn((
        Name::new("OverworldPlayer"),
        StateIdle,
        PlayerControlled,
        StateMachine::default()
            .trans::<StateIdle, _>(is_player_walking, StateWalking)
            .trans::<StateWalking, _>(is_player_walking.not(), StateIdle)
            .trans::<StateRunning, _>(is_player_walking.not(), StateIdle)
            .trans::<StateWalking, _>(is_player_running, StateRunning)
            .trans::<StateRunning, _>(is_player_running.not(), StateWalking),
        player_input.get_merged_map(),
        ActionState::<Action>::default(),
        Rect2DCollider::new(Vec2::new(20.0, 12.0), Vec2::new(0.0, -9.0)),
        PlayerBundle::new(Vec2::new(0.0, 0.0), Direction::Down, initial_clip),
    ));
}
