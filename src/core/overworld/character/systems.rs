use crate::core::basic_components::{Facing, Position, Speed};
use crate::core::input::Action;
use crate::core::overworld::character::components::{StateRunning, StateWalking};
use bevy::prelude::*;
use leafwing_input_manager::action_state::*;
fn apply_walking_step(
    mut pos: Mut<Position>,
    mut facing: Mut<Facing>,
    speed: f32,
    action_state: &ActionState<Action>,
    delta_secs: f32,
) {
    use crate::core::basic_components::*;

    let mut direction_vec = Vec2::ZERO;

    // 防止同时按下相反方向的按键
    let up_pressed = action_state.pressed(&Action::Up);
    let down_pressed = action_state.pressed(&Action::Down);
    let left_pressed = action_state.pressed(&Action::Left);
    let right_pressed = action_state.pressed(&Action::Right);

    // 垂直方向：如果同时按下上下，则忽略垂直移动
    if up_pressed && !down_pressed {
        direction_vec.y += 1.0;
    } else if down_pressed && !up_pressed {
        direction_vec.y -= 1.0;
    }

    // 水平方向：如果同时按下左右，则忽略水平移动
    if left_pressed && !right_pressed {
        direction_vec.x -= 1.0;
    } else if right_pressed && !left_pressed {
        direction_vec.x += 1.0;
    }

    let new_direction = match (direction_vec.x as i32, direction_vec.y as i32) {
        (0, 1) => Some(Direction::Up),
        (0, -1) => Some(Direction::Down),
        (-1, 0) => Some(Direction::Left),
        (1, 0) => Some(Direction::Right),
        (-1, 1) => Some(Direction::UpLeft),
        (1, 1) => Some(Direction::UpRight),
        (-1, -1) => Some(Direction::DownLeft),
        (1, -1) => Some(Direction::DownRight),
        _ => None,
    };

    if let Some(direction) = new_direction {
        facing.value = direction;
        pos.value += facing.value.as_vec2() * speed * delta_secs;
    }
}

pub(crate) fn update_walking_system(
    time: Res<Time>,
    mut query: Query<
        (&mut Position, &mut Facing, &Speed, &ActionState<Action>),
        With<StateWalking>,
    >,
) {
    for (pos, facing, speed, action_state) in query.iter_mut() {
        apply_walking_step(pos, facing, speed.value, action_state, time.delta_secs());
    }
}
pub(crate) fn update_running_system(
    time: Res<Time>,
    mut query: Query<
        (&mut Position, &mut Facing, &Speed, &ActionState<Action>),
        With<StateRunning>,
    >,
) {
    for (pos, facing, speed, action_state) in query.iter_mut() {
        apply_walking_step(
            pos,
            facing,
            speed.value * 2.0,
            action_state,
            time.delta_secs(),
        );
    }
}
