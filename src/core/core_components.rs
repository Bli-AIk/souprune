use bevy::prelude::*;

#[derive(Component)]
pub(crate) struct Position(pub Vec2);

#[derive(Component)]
pub(crate) struct Rotation(pub f32);

#[derive(Component)]
pub(crate) struct Facing(pub Direction);
#[derive(Component)]
pub(crate) struct Health {
    pub(crate) current: i32,
    pub(crate) max: i32,
}
#[derive(Component)]
pub(crate) struct AnimationState {
    pub(crate) clip: String,
}
// Current state duration
#[derive(Component)]
pub(crate) struct StateTimer(pub f32);

pub(crate) enum Direction {
    Up,
    Down,
    Left,
    Right,
}
impl Direction {
    pub fn as_vec2(&self) -> Vec2 {
        match self {
            Direction::Up => Vec2::Y,
            Direction::Down => -Vec2::Y,
            Direction::Left => -Vec2::X,
            Direction::Right => Vec2::X,
        }
    }
}