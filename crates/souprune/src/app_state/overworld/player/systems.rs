use super::update_player_animation;
use crate::app_state::overworld::character::components::{
    PlayerControlled, StateIdle, StateRunning, StateWalking,
};
use crate::app_state::overworld::OverworldState;
use crate::core::animation::components::{
    SpriteAnimationClip, SpriteAnimationCurrentFrame, SpriteAnimationTimer,
};
use crate::core::basic_components::{Direction, Facing};
use crate::core::input::Action;
use crate::core::sprite::params::SpriteParams;
use bevy::prelude::{Query, Res, Sprite, State, With};
use leafwing_input_manager::action_state::ActionState;

pub(crate) fn player_direction_control_system(
    mut query: Query<(&mut Facing, &ActionState<Action>), With<PlayerControlled>>,
    overworld_state: Res<State<OverworldState>>,
) {
    // 只在Normal状态下允许方向控制
    if *overworld_state != OverworldState::Normal {
        return;
    }

    for (mut facing, action_state) in query.iter_mut() {
        let up_pressed = action_state.pressed(&Action::Up);
        let down_pressed = action_state.pressed(&Action::Down);
        let left_pressed = action_state.pressed(&Action::Left);
        let right_pressed = action_state.pressed(&Action::Right);

        // Only updates heading when opposite direction is pressed at different times
        if up_pressed && !down_pressed {
            facing.value = Direction::Up;
        } else if down_pressed && !up_pressed {
            facing.value = Direction::Down;
        } else if left_pressed && !right_pressed {
            facing.value = Direction::Left;
        } else if right_pressed && !left_pressed {
            facing.value = Direction::Right;
        }
    }
}

macro_rules! create_animation_system {
    ($func_name:ident, $state:ty, $animation_prefix:expr) => {
        pub(crate) fn $func_name(
            mut sprite_params: SpriteParams,
            mut query: Query<
                (
                    &Facing,
                    &mut Sprite,
                    &mut SpriteAnimationClip,
                    &mut SpriteAnimationCurrentFrame,
                    &mut SpriteAnimationTimer,
                ),
                (With<PlayerControlled>, With<$state>),
            >,
        ) {
            for (facing, mut sprite, mut clip, mut frame, mut timer) in query.iter_mut() {
                update_player_animation(
                    &mut sprite_params,
                    facing,
                    &mut sprite,
                    &mut clip,
                    &mut frame,
                    &mut timer,
                    $animation_prefix,
                );
            }
        }
    };
}

create_animation_system!(player_idle_anim_control_system, StateIdle, "frisk_idle");
create_animation_system!(player_walk_anim_control_system, StateWalking, "frisk_walk");
create_animation_system!(player_run_anim_control_system, StateRunning, "frisk_run");
