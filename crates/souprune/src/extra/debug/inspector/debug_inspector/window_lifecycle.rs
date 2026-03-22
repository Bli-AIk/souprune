use super::{
    InspectorUiState, InspectorWindowContextPass, RefreshPhase, StandaloneInspectorCamera,
    StandaloneInspectorWindow,
};
use bevy::camera::RenderTarget;
use bevy::ecs::message::MessageReader;
use bevy::prelude::*;
use bevy::window::{
    PrimaryWindow, Window, WindowClosed, WindowFocused, WindowRef, WindowResolution,
};
use bevy_inspector_egui::bevy_egui::EguiMultipassSchedule;

pub(super) fn handle_inspector_hotkeys_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut ui_state: ResMut<InspectorUiState>,
    mut commands: Commands,
    mut toast_events: MessageWriter<super::super::DebugToastEvent>,
) {
    if !keyboard_input.just_pressed(KeyCode::F1) {
        return;
    }

    if ui_state.inspector_window.is_some() {
        close_inspector_window(&mut commands, &mut ui_state);
        toast_events.write(super::super::DebugToastEvent {
            message: "Inspector: OFF".into(),
        });
    } else {
        spawn_inspector_window(&mut commands, &mut ui_state);
        toast_events.write(super::super::DebugToastEvent {
            message: "Inspector: ON".into(),
        });
    }
}

fn spawn_inspector_window(commands: &mut Commands, ui_state: &mut InspectorUiState) {
    if ui_state.inspector_window.is_some() {
        return;
    }

    let window_entity = commands
        .spawn((
            Name::new("Debug: Inspector Window"),
            Window {
                title: "Souprune Inspector".into(),
                resolution: WindowResolution::new(960, 640),
                resizable: true,
                decorations: true,
                ..default()
            },
            StandaloneInspectorWindow,
        ))
        .id();

    let camera_entity = commands
        .spawn((
            Name::new("Debug: Inspector Camera"),
            Camera2d,
            Camera::default(),
            RenderTarget::Window(WindowRef::Entity(window_entity)),
            EguiMultipassSchedule::new(InspectorWindowContextPass),
            StandaloneInspectorCamera,
            super::super::DebugCamera,
        ))
        .id();

    ui_state.inspector_window = Some(window_entity);
    ui_state.inspector_camera = Some(camera_entity);
    ui_state.window_focused = false;
    info!("Standalone inspector window opened");
}

fn close_inspector_window(commands: &mut Commands, ui_state: &mut InspectorUiState) {
    if let Some(camera_entity) = ui_state.inspector_camera.take() {
        commands.entity(camera_entity).despawn();
    }
    if let Some(window_entity) = ui_state.inspector_window.take() {
        commands.entity(window_entity).despawn();
    }
    ui_state.window_focused = false;
    info!("Standalone inspector window closed");
}

pub(super) fn inspector_window_closed_system(
    mut commands: Commands,
    mut window_events: MessageReader<WindowClosed>,
    mut ui_state: ResMut<InspectorUiState>,
) {
    let Some(window_entity) = ui_state.inspector_window else {
        return;
    };

    for event in window_events.read() {
        if event.window != window_entity {
            continue;
        }
        ui_state.inspector_window = None;
        if let Some(camera_entity) = ui_state.inspector_camera.take() {
            commands.entity(camera_entity).despawn();
        }
        ui_state.window_focused = false;
        info!("Standalone inspector window closed");
        break;
    }
}

/// System to close inspector when primary window is closed.
pub(super) fn primary_window_closed_system(
    mut commands: Commands,
    mut ui_state: ResMut<InspectorUiState>,
    mut removed: RemovedComponents<PrimaryWindow>,
) {
    if removed.read().next().is_some() && ui_state.inspector_window.is_some() {
        close_inspector_window(&mut commands, &mut ui_state);
        info!("Standalone inspector window closed (primary window closed)");
    }
}

pub(super) fn inspector_window_focus_system(
    mut focus_events: MessageReader<WindowFocused>,
    mut ui_state: ResMut<InspectorUiState>,
) {
    let Some(window_entity) = ui_state.inspector_window else {
        ui_state.window_focused = false;
        return;
    };

    for event in focus_events.read() {
        if event.window == window_entity {
            ui_state.window_focused = event.focused;
            break;
        }
    }
}

/// System to detect AppState changes and trigger inspector refresh.
pub(super) fn app_state_changed_refresh_inspector_system(
    mut ui_state: ResMut<InspectorUiState>,
    app_state: Res<State<crate::app_state::AppState>>,
) {
    if ui_state.inspector_window.is_some()
        && app_state.is_changed()
        && ui_state.refresh_phase == RefreshPhase::None
    {
        ui_state.refresh_phase = RefreshPhase::CloseWindow;
        info!("AppState changed, scheduling inspector window refresh (phase 1: close)");
    }
}

/// System to perform inspector window refresh in two phases.
pub(super) fn inspector_refresh_system(
    mut commands: Commands,
    mut ui_state: ResMut<InspectorUiState>,
) {
    match ui_state.refresh_phase {
        RefreshPhase::None => {}
        RefreshPhase::CloseWindow => {
            if ui_state.inspector_window.is_some() {
                close_inspector_window(&mut commands, &mut ui_state);
                info!("Inspector window closed for refresh (phase 1 complete)");
            }
            ui_state.refresh_phase = RefreshPhase::ReopenWindow;
        }
        RefreshPhase::ReopenWindow => {
            if ui_state.inspector_window.is_none() {
                spawn_inspector_window(&mut commands, &mut ui_state);
                info!("Inspector window reopened after refresh (phase 2 complete)");
            }
            ui_state.refresh_phase = RefreshPhase::None;
        }
    }
}
