//! Standalone egui window for the View layout observer.
//!
//! View 布局观察器的独立 egui 窗口。

use bevy::camera::RenderTarget;
use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::*;
use bevy::window::{Window, WindowClosed, WindowRef, WindowResolution};
use bevy_inspector_egui::bevy_egui::{EguiContext, EguiMultipassSchedule};
use bevy_inspector_egui::egui;

use super::format::{build_selection_text, display_label, format_layout_rect, mode_label};
use super::state::{
    ViewLayoutObserverMode, ViewLayoutObserverSelection, ViewLayoutObserverSnapshot,
    ViewLayoutObserverState,
};
use crate::extra::debug::DebugCamera;

#[derive(Component)]
struct ViewLayoutObserverWindow;

#[derive(Component)]
struct ViewLayoutObserverCamera;

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub(super) struct ViewLayoutObserverContextPass;

pub(super) fn ensure_view_layout_observer_window(
    commands: &mut Commands,
    state: &mut ViewLayoutObserverState,
) {
    if state.window_entity.is_none() {
        spawn_view_layout_observer_window(commands, state);
    }
}

pub(super) fn spawn_view_layout_observer_window(
    commands: &mut Commands,
    state: &mut ViewLayoutObserverState,
) {
    if state.window_entity.is_some() {
        return;
    }

    let window_entity = commands
        .spawn((
            Name::new("Debug: View Observer Window"),
            Window {
                title: "Souprune View Observer".into(),
                resolution: WindowResolution::new(1080, 720),
                resizable: true,
                decorations: true,
                ..default()
            },
            ViewLayoutObserverWindow,
        ))
        .id();

    let camera_entity = commands
        .spawn((
            Name::new("Debug: View Observer Camera"),
            Camera2d,
            Camera::default(),
            RenderTarget::Window(WindowRef::Entity(window_entity)),
            EguiMultipassSchedule::new(ViewLayoutObserverContextPass),
            ViewLayoutObserverCamera,
            DebugCamera,
        ))
        .id();

    state.window_entity = Some(window_entity);
    state.camera_entity = Some(camera_entity);
    state.show_box_model = true;
    state.show_flex_guides = true;
    state.show_grid_guides = true;
    state.show_spatial_guides = true;
    info!("View Observer window opened");
}

pub(super) fn close_view_layout_observer_window(
    commands: &mut Commands,
    state: &mut ViewLayoutObserverState,
) {
    if let Some(camera_entity) = state.camera_entity.take() {
        commands.entity(camera_entity).despawn();
    }
    if let Some(window_entity) = state.window_entity.take() {
        commands.entity(window_entity).despawn();
    }
    state.locked_entity = None;
    info!("View Observer window closed");
}

pub(super) fn view_layout_observer_window_closed_system(
    mut commands: Commands,
    mut window_events: MessageReader<WindowClosed>,
    mut state: ResMut<ViewLayoutObserverState>,
) {
    let Some(window_entity) = state.window_entity else {
        return;
    };

    for event in window_events.read() {
        if event.window != window_entity {
            continue;
        }
        state.window_entity = None;
        if let Some(camera_entity) = state.camera_entity.take() {
            commands.entity(camera_entity).despawn();
        }
        state.mode = ViewLayoutObserverMode::Off;
        state.locked_entity = None;
        info!("View Observer window closed");
        break;
    }
}

pub(super) fn view_layout_observer_window_ui_system(world: &mut World) {
    let Some(camera_entity) = world
        .get_resource::<ViewLayoutObserverState>()
        .and_then(|state| state.camera_entity)
    else {
        return;
    };

    let mut contexts = world.query_filtered::<&mut EguiContext, With<ViewLayoutObserverCamera>>();
    let mut egui_context = match contexts.get_mut(world, camera_entity) {
        Ok(context) => context.clone(),
        Err(_) => return,
    };

    let state = world.resource::<ViewLayoutObserverState>().clone();
    let snapshot = world.resource::<ViewLayoutObserverSnapshot>().clone();
    egui::CentralPanel::default().show(egui_context.get_mut(), |ui| {
        render_observer_toolbar(ui, world, &state);
        ui.separator();
        ui.columns(2, |columns| {
            render_observer_tree(&mut columns[0], world, &state, &snapshot);
            render_observer_details(&mut columns[1], &state, &snapshot);
        });
    });
}

fn render_observer_toolbar(ui: &mut egui::Ui, world: &mut World, state: &ViewLayoutObserverState) {
    let mut next_mode = state.mode;
    let mut show_box_model = state.show_box_model;
    let mut show_flex_guides = state.show_flex_guides;
    let mut show_grid_guides = state.show_grid_guides;
    let mut show_spatial_guides = state.show_spatial_guides;

    ui.horizontal_wrapped(|ui| {
        ui.label("Mode");
        if ui
            .selectable_label(state.mode == ViewLayoutObserverMode::Off, "Off")
            .clicked()
        {
            next_mode = ViewLayoutObserverMode::Off;
        }
        if ui
            .selectable_label(state.mode == ViewLayoutObserverMode::Hover, "Hover")
            .clicked()
        {
            next_mode = ViewLayoutObserverMode::Hover;
        }
        if ui
            .selectable_label(state.mode == ViewLayoutObserverMode::Locked, "Locked")
            .clicked()
        {
            next_mode = ViewLayoutObserverMode::Locked;
        }
        if ui
            .selectable_label(state.mode == ViewLayoutObserverMode::All, "All")
            .clicked()
        {
            next_mode = ViewLayoutObserverMode::All;
        }
        ui.separator();
        ui.checkbox(&mut show_box_model, "Box");
        ui.checkbox(&mut show_flex_guides, "Flex");
        ui.checkbox(&mut show_grid_guides, "Grid");
        ui.checkbox(&mut show_spatial_guides, "3D");
    });

    if let Some(mut writable) = world.get_resource_mut::<ViewLayoutObserverState>() {
        writable.mode = next_mode;
        if next_mode != ViewLayoutObserverMode::Locked {
            writable.locked_entity = None;
        }
        writable.show_box_model = show_box_model;
        writable.show_flex_guides = show_flex_guides;
        writable.show_grid_guides = show_grid_guides;
        writable.show_spatial_guides = show_spatial_guides;
    }
}

fn render_observer_tree(
    ui: &mut egui::Ui,
    world: &mut World,
    state: &ViewLayoutObserverState,
    snapshot: &ViewLayoutObserverSnapshot,
) {
    ui.heading("View Tree");
    ui.label(format!(
        "nodes={} target={}",
        snapshot.all_selections.len(),
        snapshot
            .selected_selection
            .as_ref()
            .map(|selection| selection.element_name.clone())
            .unwrap_or_else(|| "none".to_string())
    ));
    ui.separator();

    let mut lock_request = None;
    egui::ScrollArea::vertical()
        .id_salt("view_observer_tree_scroll")
        .show(ui, |ui| {
            for selection in &snapshot.all_selections {
                let selected = snapshot
                    .selected_selection
                    .as_ref()
                    .is_some_and(|selected| selected.entity == selection.entity);
                let indent = (selection.depth as f32 * 14.0).min(84.0);
                if render_tree_selection_row(ui, selection, selected, indent) {
                    lock_request = Some(selection.entity);
                }
            }
        });

    if let Some(entity) = lock_request
        && let Some(mut writable) = world.get_resource_mut::<ViewLayoutObserverState>()
    {
        writable.locked_entity = Some(entity);
        writable.mode = ViewLayoutObserverMode::Locked;
    }

    ui.separator();
    ui.label(format!("current_mode={}", mode_label(state.mode)));
}

fn render_tree_selection_row(
    ui: &mut egui::Ui,
    selection: &ViewLayoutObserverSelection,
    selected: bool,
    indent: f32,
) -> bool {
    let mut clicked = false;
    ui.horizontal(|ui| {
        ui.add_space(indent);
        let label = format!(
            "{}  [{}]",
            selection.element_name,
            selection
                .debug
                .as_ref()
                .map(|metadata| display_label(metadata.display))
                .unwrap_or("view")
        );
        clicked = ui.selectable_label(selected, label).clicked();
    });
    clicked
}

fn render_observer_details(
    ui: &mut egui::Ui,
    state: &ViewLayoutObserverState,
    snapshot: &ViewLayoutObserverSnapshot,
) {
    ui.heading("Layout");
    render_box_model(ui, snapshot.selected_selection.as_ref());
    ui.separator();
    egui::ScrollArea::vertical()
        .id_salt("view_observer_details_scroll")
        .show(ui, |ui| {
            ui.monospace(build_selection_text(
                state,
                snapshot.selected_selection.as_ref(),
            ));
        });
}

fn render_box_model(ui: &mut egui::Ui, selection: Option<&ViewLayoutObserverSelection>) {
    let desired_size = egui::vec2(ui.available_width().min(430.0), 240.0);
    let (rect, _) = ui.allocate_exact_size(desired_size, egui::Sense::hover());
    let painter = ui.painter_at(rect);

    let margin_color = egui::Color32::from_rgb(110, 38, 38);
    let border_color = egui::Color32::from_rgb(130, 96, 28);
    let padding_color = egui::Color32::from_rgb(32, 100, 58);
    let content_color = egui::Color32::from_rgb(28, 82, 112);
    let stroke_color = egui::Color32::from_rgb(220, 230, 240);

    painter.rect_filled(rect, 4, margin_color);
    let border_rect = rect.shrink(24.0);
    painter.rect_filled(border_rect, 4, border_color);
    let padding_rect = border_rect.shrink(24.0);
    painter.rect_filled(padding_rect, 4, padding_color);
    let content_rect = padding_rect.shrink(30.0);
    painter.rect_filled(content_rect, 4, content_color);
    painter.rect_stroke(
        rect,
        4,
        egui::Stroke::new(1.0, stroke_color),
        egui::StrokeKind::Inside,
    );
    painter.rect_stroke(
        border_rect,
        4,
        egui::Stroke::new(1.0, stroke_color),
        egui::StrokeKind::Inside,
    );
    painter.rect_stroke(
        padding_rect,
        4,
        egui::Stroke::new(1.0, stroke_color),
        egui::StrokeKind::Inside,
    );
    painter.rect_stroke(
        content_rect,
        4,
        egui::Stroke::new(1.0, stroke_color),
        egui::StrokeKind::Inside,
    );

    let text_color = egui::Color32::from_rgb(245, 248, 252);
    painter.text(
        rect.left_top() + egui::vec2(10.0, 8.0),
        egui::Align2::LEFT_TOP,
        "margin",
        egui::FontId::monospace(12.0),
        text_color,
    );
    painter.text(
        border_rect.left_top() + egui::vec2(10.0, 8.0),
        egui::Align2::LEFT_TOP,
        "border",
        egui::FontId::monospace(12.0),
        text_color,
    );
    painter.text(
        padding_rect.left_top() + egui::vec2(10.0, 8.0),
        egui::Align2::LEFT_TOP,
        "padding",
        egui::FontId::monospace(12.0),
        text_color,
    );
    painter.text(
        content_rect.center(),
        egui::Align2::CENTER_CENTER,
        selection
            .map(|selection| format_layout_rect(&selection.rect))
            .unwrap_or_else(|| "content".to_string()),
        egui::FontId::monospace(12.0),
        text_color,
    );
}
