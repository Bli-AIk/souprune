use bevy::prelude::*;

#[derive(Component)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Component)]
struct Rotation {
    angle: f32,
}

#[derive(Component)]
struct Facing(Direction);
#[derive(Component)]
struct Health {
    current: i32,
    max: i32,
}
#[derive(Component)]
struct AnimationState {
    clip: String,
}
// Current state duration
#[derive(Component)]
struct StateTimer(f32);

#[derive(Component)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}
