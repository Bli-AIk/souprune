use bevy::prelude::*;

#[derive(Component)]
pub(crate) struct Position {
    pub value: Vec2,
}

#[derive(Component)]
pub(crate) struct Rotation {
    pub angle: f32,
}
#[derive(Component)]
pub(crate) struct Speed {
    pub value: f32,
}

#[derive(Component)]
pub(crate) struct Facing {
    pub value: Direction,
}
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
pub(crate) struct StateTimer {
    pub value: f32,
}

#[derive(Default)]
pub(crate) enum Direction {
    Up,
    #[default]
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
