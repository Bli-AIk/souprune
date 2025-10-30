use crate::core::basic_components::{Facing, Position, Speed};
use crate::core::input::Action;
use crate::core::overworld::character::components::{Idle, Running, Walking};
use bevy::prelude::*;
use leafwing_input_manager::action_state::*;

pub(crate) fn update_idle_system(_query: Query<(&mut Position, &mut Facing, &Speed), With<Idle>>) {
    // TODO: 实现 Idle 状态逻辑
}
fn apply_walking_step(
    mut pos: Mut<Position>,
    mut facing: Mut<Facing>,
    speed: f32,
    action_state: &ActionState<Action>,
    delta_secs: f32,
) {
    use crate::core::basic_components::*;

    let mut direction_vec = Vec2::ZERO;
    if action_state.pressed(&Action::Up) {
        direction_vec.y += 1.0;
    }
    if action_state.pressed(&Action::Down) {
        direction_vec.y -= 1.0;
    }
    if action_state.pressed(&Action::Left) {
        direction_vec.x -= 1.0;
    }
    if action_state.pressed(&Action::Right) {
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
    mut query: Query<(&mut Position, &mut Facing, &Speed, &ActionState<Action>), With<Walking>>,
) {
    for (pos, facing, speed, action_state) in query.iter_mut() {
        apply_walking_step(pos, facing, speed.value, action_state, time.delta_secs());
    }
}
pub(crate) fn update_running_system(
    time: Res<Time>,
    mut query: Query<(&mut Position, &mut Facing, &Speed, &ActionState<Action>), With<Running>>,
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
