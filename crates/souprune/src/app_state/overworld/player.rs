//! # player.rs
//!
//! ## Module Overview
//! This module manages player-specific logic and systems within the overworld, including input-based movement, direction control, and animation states.
//!
//! ## Source File Overview
//! This file defines the `PlayerPlugin`, which integrates systems for controlling player direction and various animation states (idle, walk, run).
//!
//! ## 模块概述
//! 该模块管理着 Overworld 中玩家特有的逻辑和系统，包括基于输入的移动、方向控制和动画状态。
//!
//! ## 源文件概述
//! 该文件定义了 `PlayerPlugin`，它集成了用于控制玩家方向和各种动画状态（空闲、行走、奔跑）的系统。

use crate::core::animation::components::{
    SpriteAnimationClip, SpriteAnimationCurrentFrame, SpriteAnimationTimer,
};
use crate::core::basic_components::{Direction, Facing};
use crate::core::input::PlayerInputSettings;
use crate::core::sprite::params::SpriteParams;
use bevy::app::{App, Plugin, Update};
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
        *clip = change_sprite_animation(sprite_params, frame, timer, "overworld", &clip_name);
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

    SpriteAnimationClip::new(
        &mut sprite_params.create_sprite_context(),
        module_name,
        clip_name,
    )
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
    commands.spawn((
        Name::new("Overworld Player"),
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
        PlayerBundle::new(
            Vec2::new(0.0, 0.0),
            Direction::Down,
            SpriteAnimationClip::new(
                &mut sprite_params.create_sprite_context(),
                "overworld",
                "frisk_walk_down",
            ),
        ),
    ));
}
