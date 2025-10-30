use crate::core::animation::{SpriteAnimationClip, SpriteAnimationCurrentFrame};
use crate::core::basic_components::{
    BasicAttributes, Direction, Facing, Position, Rotation, Speed,
};
use crate::core::input::Action;
use crate::core::overworld::character::components::*;
use crate::core::sprite::params::SpriteParams;
use bevy::app::{App, Plugin, Update};
use bevy::math::Vec2;
use bevy::prelude;
use bevy::prelude::{Bundle, GlobalTransform, Query, Sprite, Transform, With};
use leafwing_input_manager::action_state::ActionState;

pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
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
#[derive(Bundle)]
pub struct PlayerBundle {
    position: Position,
    rotation: Rotation,
    facing: Facing,
    speed: Speed,
    sprite: Sprite,
    health: BasicAttributes,
    anim: SpriteAnimationClip,
    transform: Transform,
    global_transform: GlobalTransform,
}

impl PlayerBundle {
    pub fn new(spawn_pos: Vec2, facing: Direction, anim: SpriteAnimationClip) -> Self {
        Self {
            position: Position { value: spawn_pos },
            rotation: Rotation { angle: 0.0 },
            facing: Facing { value: facing },
            speed: Speed { value: 100.0 },
            sprite: Sprite::default(),
            health: BasicAttributes {
                hp_current: 20,
                hp_max: 20,
                atk: 10,
                def: 10,
            },
            anim,
            transform: Transform::from_translation(spawn_pos.extend(0.0)),
            global_transform: GlobalTransform::default(),
        }
    }
}

pub fn is_player_walking(
    query: Query<&ActionState<Action>, With<PlayerControlled>>,
) -> prelude::Result<(), ()> {
    let action_state = query.single().map_err(|_| ())?;
    if action_state.pressed(&Action::Left)
        || action_state.pressed(&Action::Right)
        || action_state.pressed(&Action::Up)
        || action_state.pressed(&Action::Down)
    {
        Ok(())
    } else {
        Err(())
    }
}

pub fn is_player_running(
    query: Query<&ActionState<Action>, With<PlayerControlled>>,
) -> prelude::Result<(), ()> {
    let action_state = query.single().map_err(|_| ())?;
    if action_state.pressed(&Action::Cancel) {
        Ok(())
    } else {
        Err(())
    }
}
pub(crate) fn player_direction_control_system(
    mut query: Query<(&mut Facing, &ActionState<Action>), With<PlayerControlled>>,
) {
    for (mut facing, action_state) in query.iter_mut() {
        if action_state.pressed(&Action::Up) {
            facing.value = Direction::Up;
        } else if action_state.pressed(&Action::Down) {
            facing.value = Direction::Down;
        } else if action_state.pressed(&Action::Left) {
            facing.value = Direction::Left;
        } else if action_state.pressed(&Action::Right) {
            facing.value = Direction::Right;
        }
    }
}

pub(crate) fn player_idle_anim_control_system(
    mut sprite_params: SpriteParams,
    mut query: Query<
        (
            &Facing,
            &mut SpriteAnimationClip,
            &mut SpriteAnimationCurrentFrame,
        ),
        (With<PlayerControlled>, With<Idle>),
    >,
) {
    for (facing, mut clip, mut frame) in query.iter_mut() {
        let clip_name = match facing.value {
            Direction::Up => "frisk_idle_up",
            Direction::Down => "frisk_idle_down",
            Direction::Left => "frisk_idle_left",
            Direction::Right => "frisk_idle_right",
            _ => {
                continue;
            }
        };

        *clip = change_sprite_animation(&mut sprite_params, &mut frame, "overworld", clip_name);
    }
}

pub(crate) fn player_walk_anim_control_system(
    mut sprite_params: SpriteParams,
    mut query: Query<
        (
            &Facing,
            &mut SpriteAnimationClip,
            &mut SpriteAnimationCurrentFrame,
        ),
        (With<PlayerControlled>, With<Walking>),
    >,
) {
    for (facing, mut clip, mut frame) in query.iter_mut() {
        let clip_name = match facing.value {
            Direction::Up => "frisk_walk_up",
            Direction::Down => "frisk_walk_down",
            Direction::Left => "frisk_walk_left",
            Direction::Right => "frisk_walk_right",
            _ => {
                continue;
            }
        };

        *clip = change_sprite_animation(&mut sprite_params, &mut frame, "overworld", clip_name);
    }
}

pub(crate) fn player_run_anim_control_system(
    mut sprite_params: SpriteParams,
    mut query: Query<
        (
            &Facing,
            &mut SpriteAnimationClip,
            &mut SpriteAnimationCurrentFrame,
        ),
        (With<PlayerControlled>, With<Running>),
    >,
) {
    for (facing, mut clip, mut frame) in query.iter_mut() {
        let clip_name = match facing.value {
            Direction::Up => "frisk_run_up",
            Direction::Down => "frisk_run_down",
            Direction::Left => "frisk_run_left",
            Direction::Right => "frisk_run_right",
            _ => {
                continue;
            }
        };

        *clip = change_sprite_animation(&mut sprite_params, &mut frame, "overworld", clip_name);
    }
}
fn change_sprite_animation(
    sprite_params: &mut SpriteParams,
    current_frame: &mut SpriteAnimationCurrentFrame,
    module_name: &str,
    clip_name: &str,
) -> SpriteAnimationClip {
    println!("change_sprite_animation");
    current_frame.value = 0;
    SpriteAnimationClip::new(
        &mut sprite_params.create_sprite_context(),
        module_name,
        clip_name,
    )
}
