use crate::core::overworld::character::character_components::{Idle, Running, Walking};
use bevy::prelude::*;

pub(crate) fn update_idle_system(query: Query<Entity, With<Idle>>) {
    // TODO
}

pub(crate) fn update_walking_system(query: Query<Entity, With<Walking>>) {
    // TODO
}

pub(crate) fn update_running_system(query: Query<Entity, With<Running>>) {
    // TODO
}
