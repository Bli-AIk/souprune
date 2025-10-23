use crate::core::core_components::{Facing, Position, Speed};
use crate::core::input::Action;
use crate::core::overworld::character::character_components::{Idle, Running, Walking};
use bevy::prelude::*;
use leafwing_input_manager::action_state::*;

pub(crate) fn update_idle_system(
    mut query: Query<(&mut Position, &mut Facing, &Speed), With<Idle>>,
) {
    // TODO: 实现 Idle 状态逻辑
}

pub(crate) fn update_walking_system(
    time: Res<Time>,
    mut query: Query<(&mut Position, &mut Facing, &Speed, &ActionState<Action>), With<Walking>>,
) {
    for (mut pos, mut facing, speed, action_state) in query.iter_mut() {
        use crate::core::core_components::*;
        for (action, direction) in [
            (Action::Up, Direction::Up),
            (Action::Down, Direction::Down),
            (Action::Left, Direction::Left),
            (Action::Right, Direction::Right),
        ] {
            if action_state.pressed(&action) {
                facing.value = direction;
                break;
            }
        }

        pos.value += facing.value.as_vec2() * speed.value * time.delta_secs();
    }
}

pub(crate) fn update_running_system(
    mut query: Query<(&mut Position, &mut Facing, &Speed), With<Running>>,
) {
    for (mut pos, mut facing, speed) in query.iter_mut() {
        // TODO: 实现 Running 状态逻辑
    }
}
