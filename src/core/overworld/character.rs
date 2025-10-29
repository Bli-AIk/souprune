use crate::core::components::*;
use crate::core::input::Action;
use crate::core::overworld::character::components::PlayerControlled;
use bevy::math::Vec2;
use bevy::prelude;
use bevy::prelude::{Bundle, GlobalTransform, Query, Sprite, Transform, With};
use leafwing_input_manager::action_state::ActionState;

pub(crate) mod components;
pub(crate) mod systems;

#[derive(Bundle)]
pub struct CharacterBundle {
    position: Position,
    rotation: Rotation,
    facing: Facing,
    speed: Speed,
    sprite: Sprite,
    health: Health,
    anim: AnimationState,
    transform: Transform,
    global_transform: GlobalTransform,
}

impl CharacterBundle {
    pub fn new(spawn_pos: Vec2, facing: Direction, sprite: Sprite) -> Self {
        Self {
            position: Position { value: spawn_pos },
            rotation: Rotation { angle: 0.0 },
            facing: Facing { value: facing },
            speed: Speed { value: 50.0 },
            sprite,
            health: Health {
                current: 20,
                max: 20,
            },
            anim: AnimationState {
                clip: "idle_down".into(),
            },
            transform: Transform::from_translation(spawn_pos.extend(0.0)),
            global_transform: GlobalTransform::default(),
        }
    }
}

pub fn is_walking(
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

pub fn is_running(
    query: Query<&ActionState<Action>, With<PlayerControlled>>,
) -> prelude::Result<(), ()> {
    let action_state = query.single().map_err(|_| ())?;
    if action_state.pressed(&Action::Cancel) {
        Ok(())
    } else {
        Err(())
    }
}
