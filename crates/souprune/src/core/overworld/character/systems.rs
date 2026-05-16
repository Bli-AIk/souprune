//! # systems.rs
//!
//! # systems.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This module implements systems for character movement and animation updates.
//!
//! 本模块实现角色移动和动画更新的系统。
//!
//! ## Source File Overview
//!
//! ## 源文件概述
//!
//! It manages state transitions based on input and game state.
//!
//! 根据输入和游戏状态管理状态转换。

use crate::core::basic_components::{Facing, Speed};
use crate::core::input::{Direction as InputDirection, InputCommandState};
use crate::core::overworld::character::components::{StateRunning, StateWalking};
use crate::core::overworld::player::config::PlayerBehavior;
use bevy::prelude::*;

pub(crate) fn update_walking_system(
    time: Res<Time>,
    input_state: Res<InputCommandState>,
    mut query: Query<(&mut Transform, &mut Facing, &Speed), With<StateWalking>>,
) {
    for (mut transform, facing, speed) in query.iter_mut() {
        apply_walking_step(
            &mut transform,
            facing,
            speed.value,
            &input_state,
            time.delta_secs(),
        );
    }
}
pub(crate) fn update_running_system(
    time: Res<Time>,
    input_state: Res<InputCommandState>,
    behavior: Res<PlayerBehavior>,
    mut query: Query<(&mut Transform, &mut Facing, &Speed), With<StateRunning>>,
) {
    for (mut transform, facing, speed) in query.iter_mut() {
        apply_walking_step(
            &mut transform,
            facing,
            speed.value * behavior.run_speed_multiplier,
            &input_state,
            time.delta_secs(),
        );
    }
}

fn apply_walking_step(
    transform: &mut Transform,
    mut facing: Mut<Facing>,
    speed: f32,
    input_state: &InputCommandState,
    delta_secs: f32,
) {
    use crate::core::basic_components::*;

    let mut direction_vec = Vec2::ZERO;

    let up_pressed = input_state.navigation_pressed(InputDirection::Up);
    let down_pressed = input_state.navigation_pressed(InputDirection::Down);
    let left_pressed = input_state.navigation_pressed(InputDirection::Left);
    let right_pressed = input_state.navigation_pressed(InputDirection::Right);

    if up_pressed && !down_pressed {
        direction_vec.y += 1.0;
    } else if down_pressed && !up_pressed {
        direction_vec.y -= 1.0;
    }

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

        // The SDF collision system handles arbitrary movement magnitudes without splitting steps.
        //
        // SDF 碰撞系统可以处理任意大小的移动，无需分解步骤。
        let movement = facing.value.as_vec2() * speed * delta_secs;
        transform.translation += movement.extend(0.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::input::{Direction as InputDirection, InputCommand, InputCommandState};

    #[test]
    fn walking_step_reads_direction_from_input_command_state() {
        let mut transform = Transform::default();
        let facing = Facing {
            value: crate::core::basic_components::Direction::Down,
        };
        let mut input_state = InputCommandState::default();
        input_state.set_pressed_command(InputCommand::Navigate(InputDirection::Right), "Right");

        let mut world = World::new();
        let entity = world.spawn(facing).id();
        let mut entity_mut = world.entity_mut(entity);
        let facing_mut = entity_mut
            .get_mut::<Facing>()
            .expect("test entity should have Facing");

        apply_walking_step(&mut transform, facing_mut, 10.0, &input_state, 0.5);

        let facing = world.get::<Facing>(entity).expect("facing should remain");
        assert_eq!(
            facing.value,
            crate::core::basic_components::Direction::Right
        );
        assert_eq!(transform.translation.x, 5.0);
        assert_eq!(transform.translation.y, 0.0);
    }
}
