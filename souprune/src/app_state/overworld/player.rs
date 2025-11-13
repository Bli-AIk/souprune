use crate::app_state::overworld::character::components::*;
use crate::core::animation::component::{
    SpriteAnimationClip, SpriteAnimationCurrentFrame, SpriteAnimationTimer,
};
use crate::core::basic_components::{BasicAttributes, Direction, Facing, Speed};
use crate::core::input::Action;
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

    // 检查各方向按键状态
    let up_pressed = action_state.pressed(&Action::Up);
    let down_pressed = action_state.pressed(&Action::Down);
    let left_pressed = action_state.pressed(&Action::Left);
    let right_pressed = action_state.pressed(&Action::Right);

    // 检查是否有有效的移动输入（不能同时按下相反方向）
    let has_vertical_input = (up_pressed && !down_pressed) || (down_pressed && !up_pressed);
    let has_horizontal_input = (left_pressed && !right_pressed) || (right_pressed && !left_pressed);

    if has_vertical_input || has_horizontal_input {
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
        // 防止同时按下相反方向的按键
        let up_pressed = action_state.pressed(&Action::Up);
        let down_pressed = action_state.pressed(&Action::Down);
        let left_pressed = action_state.pressed(&Action::Left);
        let right_pressed = action_state.pressed(&Action::Right);

        // 只有在不同时按下相反方向时才更新朝向
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

create_animation_system!(player_idle_anim_control_system, StateIdle, "frisk_idle");
create_animation_system!(player_walk_anim_control_system, StateWalking, "frisk_walk");
create_animation_system!(player_run_anim_control_system, StateRunning, "frisk_run");
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
