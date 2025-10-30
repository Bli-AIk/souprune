use crate::core::animation::{
    SpriteAnimationClip, SpriteAnimationCurrentFrame, SpriteAnimationTimer,
};
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
        // 立即应用新的sprite，消除延迟
        *sprite = clip.get_current_sprite().clone();
    }
}

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

create_animation_system!(player_idle_anim_control_system, Idle, "frisk_idle");
create_animation_system!(player_walk_anim_control_system, Walking, "frisk_walk");
create_animation_system!(player_run_anim_control_system, Running, "frisk_run");
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
