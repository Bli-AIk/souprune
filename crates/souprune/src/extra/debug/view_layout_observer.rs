//! View layout observer for debug builds.
//!
//! 调试构建下的 View 布局观察器。
//!
//! The observer inspects spawned View entities, follows their computed Taffy
//! metadata, renders game-window gizmos for the selected element, and presents
//! a standalone egui inspector window for detailed layout inspection.
//!
//! 观察器会检查已生成的 View 实体，读取其计算后的 Taffy 元数据，在游戏窗口中
//! 绘制选中元素的 gizmos，并提供独立 egui 检查窗口展示详细布局信息。

mod format;
mod gizmos;
mod selection;
mod state;
mod window;

use bevy::prelude::*;
use bevy::window::{PrimaryWindow, Window};

use self::selection::{
    ViewLayoutElementQuery, ViewRootObserverQuery, collect_all_selections, collect_hover_selection,
    collect_locked_selection, cursor_world_2d, selected_selection_for_mode,
};
use self::state::{
    ViewLayoutObserverMode, ViewLayoutObserverSelection, ViewLayoutObserverSnapshot,
    ViewLayoutObserverState,
};
use crate::core::camera::MainGameCamera;
use crate::core::view::components::ViewRoot;
use crate::core::view::layout::ViewClipRect;
use crate::extra::debug::{DebugCamera, DebugToastEvent};

pub(super) fn setup_view_layout_observer_debug(app: &mut App) {
    app.init_resource::<ViewLayoutObserverState>()
        .init_resource::<ViewLayoutObserverSnapshot>()
        .add_systems(
            Update,
            (
                handle_view_layout_observer_hotkeys_system,
                window::view_layout_observer_window_closed_system,
                update_view_layout_observer_snapshot_system.after(crate::core::view::ViewUpdate),
                gizmos::draw_view_layout_observer_gizmos_system
                    .after(update_view_layout_observer_snapshot_system),
            ),
        )
        .add_systems(
            window::ViewLayoutObserverContextPass,
            window::view_layout_observer_window_ui_system,
        );
}

fn update_view_layout_observer_snapshot_system(
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_2d: Query<
        (&Camera, &GlobalTransform),
        (With<Camera2d>, With<MainGameCamera>, Without<DebugCamera>),
    >,
    view_roots: ViewRootObserverQuery,
    view_elements: ViewLayoutElementQuery,
    view_root_lookup: Query<&ViewRoot>,
    child_of_query: Query<&ChildOf>,
    clip_rect_query: Query<&ViewClipRect>,
    mut state: ResMut<ViewLayoutObserverState>,
    mut snapshot: ResMut<ViewLayoutObserverSnapshot>,
) {
    if !state.overlay_active() {
        snapshot.hover_selection = None;
        snapshot.selected_selection = None;
        snapshot.all_selections.clear();
        return;
    }

    let hover_selection = collect_hover_selection(
        cursor_world_2d(&windows, &camera_2d),
        &view_roots,
        &view_elements,
        &view_root_lookup,
        &child_of_query,
        &clip_rect_query,
    );
    let locked_selection = state.locked_entity.and_then(|locked_entity| {
        collect_locked_selection(
            locked_entity,
            &view_roots,
            &view_elements,
            &view_root_lookup,
            &child_of_query,
        )
    });

    if state.locked_entity.is_some() && locked_selection.is_none() {
        state.locked_entity = None;
        if state.mode == ViewLayoutObserverMode::Locked {
            state.mode = ViewLayoutObserverMode::Hover;
        }
    }

    snapshot.hover_selection = hover_selection.clone();
    snapshot.selected_selection =
        selected_selection_for_mode(state.mode, hover_selection, locked_selection);
    snapshot.all_selections = collect_all_selections(
        &view_roots,
        &view_elements,
        &view_root_lookup,
        &child_of_query,
    );
}

fn handle_view_layout_observer_hotkeys_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<ViewLayoutObserverState>,
    snapshot: Res<ViewLayoutObserverSnapshot>,
    mut commands: Commands,
    mut toast_events: MessageWriter<DebugToastEvent>,
) {
    if keyboard.just_pressed(KeyCode::Escape) && state.locked_entity.take().is_some() {
        if state.mode == ViewLayoutObserverMode::Locked {
            state.mode = ViewLayoutObserverMode::Hover;
        }
        toast_events.write(DebugToastEvent {
            message: "View Layout Observer: lock cleared".into(),
        });
        return;
    }

    if !keyboard.just_pressed(KeyCode::F11) {
        return;
    }

    let shift = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);
    let control = keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight);

    if control {
        handle_lock_hotkey(
            &mut state,
            snapshot.hover_selection.as_ref(),
            &mut toast_events,
        );
        return;
    }

    if shift {
        window::ensure_view_layout_observer_window(&mut commands, &mut state);
        state.mode = if state.mode == ViewLayoutObserverMode::All {
            ViewLayoutObserverMode::Hover
        } else {
            ViewLayoutObserverMode::All
        };
        toast_events.write(DebugToastEvent {
            message: format!(
                "View Layout Observer mode: {}",
                format::mode_label(state.mode)
            ),
        });
        return;
    }

    if state.window_entity.is_some() {
        window::close_view_layout_observer_window(&mut commands, &mut state);
        state.mode = ViewLayoutObserverMode::Off;
        toast_events.write(DebugToastEvent {
            message: "View Layout Observer: OFF".into(),
        });
    } else {
        window::spawn_view_layout_observer_window(&mut commands, &mut state);
        state.mode = ViewLayoutObserverMode::Hover;
        toast_events.write(DebugToastEvent {
            message: "View Layout Observer: ON".into(),
        });
    }
}

fn handle_lock_hotkey(
    state: &mut ViewLayoutObserverState,
    hover_selection: Option<&ViewLayoutObserverSelection>,
    toast_events: &mut MessageWriter<DebugToastEvent>,
) {
    if !state.overlay_active() {
        return;
    }

    let Some(selection) = hover_selection else {
        if state.locked_entity.take().is_some() {
            toast_events.write(DebugToastEvent {
                message: "View Layout Observer: lock cleared".into(),
            });
        }
        return;
    };

    if state.locked_entity == Some(selection.entity) {
        state.locked_entity = None;
        state.mode = ViewLayoutObserverMode::Hover;
        toast_events.write(DebugToastEvent {
            message: "View Layout Observer: lock cleared".into(),
        });
    } else {
        state.locked_entity = Some(selection.entity);
        state.mode = ViewLayoutObserverMode::Locked;
        toast_events.write(DebugToastEvent {
            message: format!("View Layout Observer locked: {}", selection.element_name),
        });
    }
}
