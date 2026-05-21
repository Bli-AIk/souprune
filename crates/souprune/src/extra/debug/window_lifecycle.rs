//! Shared teardown helpers for standalone debug windows.
//!
//! 独立调试窗口的共享关闭助手。

use bevy::ecs::message::MessageReader;
use bevy::prelude::*;
use bevy::window::{PrimaryWindow, WindowClosed};

pub(crate) trait DebugWindowLifecycleState {
    fn window_entity(&self) -> Option<Entity>;
    fn window_entity_mut(&mut self) -> &mut Option<Entity>;
    fn camera_entity_mut(&mut self) -> &mut Option<Entity>;

    fn on_window_closed(&mut self) {}
}

pub(crate) fn close_debug_window<S: DebugWindowLifecycleState>(
    commands: &mut Commands,
    state: &mut S,
) {
    if let Some(camera_entity) = state.camera_entity_mut().take() {
        commands.entity(camera_entity).despawn();
    }
    if let Some(window_entity) = state.window_entity_mut().take() {
        commands.entity(window_entity).despawn();
    }
    state.on_window_closed();
}

pub(crate) fn close_debug_window_on_child_window_closed<S: DebugWindowLifecycleState>(
    commands: &mut Commands,
    window_events: &mut MessageReader<WindowClosed>,
    state: &mut S,
) -> bool {
    let Some(window_entity) = state.window_entity() else {
        return false;
    };

    for event in window_events.read() {
        if event.window != window_entity {
            continue;
        }
        close_debug_window(commands, state);
        return true;
    }

    false
}

pub(crate) fn close_debug_window_on_primary_window_removed<S: DebugWindowLifecycleState>(
    commands: &mut Commands,
    removed: &mut RemovedComponents<PrimaryWindow>,
    state: &mut S,
) -> bool {
    if removed.read().next().is_some() && state.window_entity().is_some() {
        close_debug_window(commands, state);
        return true;
    }

    false
}
