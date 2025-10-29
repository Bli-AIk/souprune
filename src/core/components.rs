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

#[derive(Default)]
pub(crate) enum Direction {
    Up,
    #[default]
    Down,
    Left,
    Right,
    UpLeft,
    UpRight,
    DownLeft,
    DownRight,
}
impl Direction {
    pub fn as_vec2(&self) -> Vec2 {
        match self {
            Direction::Up => Vec2::Y,
            Direction::Down => -Vec2::Y,
            Direction::Left => -Vec2::X,
            Direction::Right => Vec2::X,
            Direction::UpLeft => Vec2::new(-1.0, 1.0).normalize(),
            Direction::UpRight => Vec2::new(1.0, 1.0).normalize(),
            Direction::DownLeft => Vec2::new(-1.0, -1.0).normalize(),
            Direction::DownRight => Vec2::new(1.0, -1.0).normalize(),
        }
    }
}
