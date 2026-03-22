mod facts_ui;
mod rules_ui;
mod states_ui;

use crate::app_state::overworld::character::components::PlayerControlled;
use crate::core::input::Action;
use bevy::camera::RenderTarget;
use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::*;
use bevy::window::{
    PrimaryWindow, Window, WindowClosed, WindowFocused, WindowRef, WindowResolution,
};
use bevy_fact_rule_event::FactEvent;
use bevy_inspector_egui::bevy_egui::{EguiContext, EguiMultipassSchedule};
use bevy_inspector_egui::egui;
use leafwing_input_manager::action_state::ActionState;
use leafwing_input_manager::plugin::InputManagerSystem;
use std::collections::VecDeque;

/// Maximum number of events to keep in history.
const MAX_EVENT_HISTORY: usize = 100;

/// Marker component for the FRE panel window.
#[derive(Component)]
struct FREPanelWindow;

/// Marker component for the FRE panel camera.
#[derive(Component)]
struct FREPanelCamera;

/// Schedule label for the FRE panel UI pass.
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
struct FREPanelContextPass;

/// Refresh phase for two-frame refresh process.
#[derive(Default, Clone, Copy, PartialEq, Eq)]
enum RefreshPhase {
    #[default]
    None,
    /// Window should be closed this frame.
    CloseWindow,
    /// Window should be reopened this frame.
    ReopenWindow,
}

/// UI state resource for the FRE debug panel.
#[derive(Resource, Default)]
struct FREPanelState {
    window_entity: Option<Entity>,
    camera_entity: Option<Entity>,
    /// Whether the FRE panel window is focused.
    window_focused: bool,
    /// Two-phase refresh state for state change handling.
    refresh_phase: RefreshPhase,
    /// Currently selected tab.
    current_tab: FREPanelTab,
    /// New fact input state.
    new_fact_key: String,
    new_fact_value_str: String,
    new_fact_type: FactTypeSelection,
    new_fact_layer: FactLayerSelection,
    /// Search filter.
    search_filter: String,
}

/// Tabs in the FRE debug panel.
#[derive(Default, Clone, Copy, PartialEq, Eq)]
enum FREPanelTab {
    #[default]
    Facts,
    ViewFacts,
    Rules,
    EventHistory,
    States,
}

/// Fact type selection for adding new facts.
#[derive(Default, Clone, Copy, PartialEq, Eq)]
enum FactTypeSelection {
    #[default]
    Int,
    Float,
    Bool,
    String,
}

/// Fact layer selection for adding new facts.
#[derive(Default, Clone, Copy, PartialEq, Eq)]
enum FactLayerSelection {
    #[default]
    Local,
    Global,
}

/// Resource to track recent FactEvents.
#[derive(Resource, Default)]
struct FactEventHistory {
    events: VecDeque<FactEventRecord>,
}

/// Record of a single FactEvent.
struct FactEventRecord {
    event_id: String,
    timestamp: f64,
    data_keys: Vec<String>,
}

pub(crate) fn setup_fre_panel_debug(app: &mut App) {
    app.init_resource::<FREPanelState>()
        .init_resource::<FactEventHistory>()
        .add_systems(
            Update,
            (
                handle_fre_panel_hotkeys_system,
                fre_panel_window_closed_system,
                fre_panel_window_focus_system,
                primary_window_closed_system,
                app_state_changed_refresh_fre_panel_system,
                fre_panel_refresh_system,
                track_fact_events_system,
            ),
        )
        .add_systems(
            PreUpdate,
            block_player_actions_when_fre_panel_focused_system
                .after(InputManagerSystem::ManualControl),
        )
        .add_systems(FREPanelContextPass, fre_panel_ui_system);
}

/// System to handle F7 hotkey for opening/closing the FRE panel.
fn handle_fre_panel_hotkeys_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<FREPanelState>,
    mut commands: Commands,
    mut toast_events: MessageWriter<super::super::DebugToastEvent>,
) {
    if !keyboard_input.just_pressed(KeyCode::F2) {
        return;
    }

    if state.window_entity.is_some() {
        close_fre_panel(&mut commands, &mut state);
        toast_events.write(super::super::DebugToastEvent {
            message: "FRE Panel: OFF".into(),
        });
    } else {
        spawn_fre_panel(&mut commands, &mut state);
        toast_events.write(super::super::DebugToastEvent {
            message: "FRE Panel: ON".into(),
        });
    }
}

/// Spawn the FRE debug panel window.
fn spawn_fre_panel(commands: &mut Commands, state: &mut FREPanelState) {
    if state.window_entity.is_some() {
        return;
    }

    let window_entity = commands
        .spawn((
            Name::new("Debug: FRE Panel Window"),
            Window {
                title: "FRE Debug Panel".into(),
                resolution: WindowResolution::new(600, 700),
                resizable: true,
                decorations: true,
                ..default()
            },
            FREPanelWindow,
        ))
        .id();

    let camera_entity = commands
        .spawn((
            Name::new("Debug: FRE Panel Camera"),
            Camera2d,
            Camera::default(),
            RenderTarget::Window(WindowRef::Entity(window_entity)),
            EguiMultipassSchedule::new(FREPanelContextPass),
            FREPanelCamera,
            super::super::DebugCamera,
        ))
        .id();

    state.window_entity = Some(window_entity);
    state.camera_entity = Some(camera_entity);
    state.window_focused = false;
    info!("FRE Debug Panel opened");
}

/// Close the FRE debug panel window.
fn close_fre_panel(commands: &mut Commands, state: &mut FREPanelState) {
    if let Some(camera_entity) = state.camera_entity.take() {
        commands.entity(camera_entity).despawn();
    }
    if let Some(window_entity) = state.window_entity.take() {
        commands.entity(window_entity).despawn();
    }
    state.window_focused = false;
    info!("FRE Debug Panel closed");
}

/// System to handle window close events.
fn fre_panel_window_closed_system(
    mut commands: Commands,
    mut window_events: MessageReader<WindowClosed>,
    mut state: ResMut<FREPanelState>,
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
        state.window_focused = false;
        info!("FRE Debug Panel closed");
        break;
    }
}

/// System to close FRE panel when primary window is closed.
/// Uses RemovedComponents to detect when PrimaryWindow component is removed.
fn primary_window_closed_system(
    mut commands: Commands,
    mut state: ResMut<FREPanelState>,
    mut removed: RemovedComponents<PrimaryWindow>,
) {
    if removed.read().next().is_some() && state.window_entity.is_some() {
        close_fre_panel(&mut commands, &mut state);
        info!("FRE Debug Panel closed (primary window closed)");
    }
}

/// System to track FRE panel window focus state.
fn fre_panel_window_focus_system(
    mut focus_events: MessageReader<WindowFocused>,
    mut state: ResMut<FREPanelState>,
) {
    let Some(window_entity) = state.window_entity else {
        state.window_focused = false;
        return;
    };

    for event in focus_events.read() {
        if event.window == window_entity {
            state.window_focused = event.focused;
            break;
        }
    }
}

/// System to detect AppState changes and trigger FRE panel refresh.
fn app_state_changed_refresh_fre_panel_system(
    mut state: ResMut<FREPanelState>,
    app_state: Res<State<crate::app_state::AppState>>,
) {
    if state.window_entity.is_some()
        && app_state.is_changed()
        && state.refresh_phase == RefreshPhase::None
    {
        state.refresh_phase = RefreshPhase::CloseWindow;
        info!("AppState changed, scheduling FRE panel window refresh (phase 1: close)");
    }
}

/// System to perform FRE panel window refresh in two phases.
/// Phase 1: Close the window.
/// Phase 2: Reopen the window.
fn fre_panel_refresh_system(mut commands: Commands, mut state: ResMut<FREPanelState>) {
    match state.refresh_phase {
        RefreshPhase::None => {}
        RefreshPhase::CloseWindow => {
            if state.window_entity.is_some() {
                close_fre_panel(&mut commands, &mut state);
                info!("FRE panel window closed for refresh (phase 1 complete)");
            }
            state.refresh_phase = RefreshPhase::ReopenWindow;
        }
        RefreshPhase::ReopenWindow => {
            if state.window_entity.is_none() {
                spawn_fre_panel(&mut commands, &mut state);
                info!("FRE panel window reopened after refresh (phase 2 complete)");
            }
            state.refresh_phase = RefreshPhase::None;
        }
    }
}

/// System to block player actions when FRE panel window is focused.
fn block_player_actions_when_fre_panel_focused_system(
    state: Option<Res<FREPanelState>>,
    mut query: Query<&mut ActionState<Action>, With<PlayerControlled>>,
) {
    let should_disable = state.map(|s| s.window_focused).unwrap_or(false);

    for mut action_state in query.iter_mut() {
        if should_disable && !action_state.disabled() {
            action_state.disable();
        }
    }
}

/// System to track FactEvents for history display.
fn track_fact_events_system(
    mut events: MessageReader<FactEvent>,
    mut history: ResMut<FactEventHistory>,
    time: Res<Time>,
) {
    for event in events.read() {
        let record = FactEventRecord {
            event_id: event.id.0.clone(),
            timestamp: time.elapsed_secs_f64(),
            data_keys: event.data.keys().cloned().collect(),
        };

        history.events.push_front(record);

        while history.events.len() > MAX_EVENT_HISTORY {
            history.events.pop_back();
        }
    }
}

/// Main UI system for the FRE debug panel.
fn fre_panel_ui_system(world: &mut World) {
    let camera_entity = world
        .get_resource::<FREPanelState>()
        .and_then(|state| state.camera_entity);

    let Some(camera_entity) = camera_entity else {
        return;
    };

    let mut contexts = world.query_filtered::<&mut EguiContext, With<FREPanelCamera>>();
    let mut egui_context = match contexts.get_mut(world, camera_entity) {
        Ok(context) => context.clone(),
        Err(_) => return,
    };

    let current_tab = world.resource::<FREPanelState>().current_tab;

    egui::CentralPanel::default().show(egui_context.get_mut(), |ui| {
        render_tab_bar(ui, world, current_tab);

        ui.separator();

        match current_tab {
            FREPanelTab::Facts => facts_ui::render_facts_tab(ui, world),
            FREPanelTab::ViewFacts => facts_ui::render_view_facts_tab(ui, world),
            FREPanelTab::Rules => rules_ui::render_rules_tab(ui, world),
            FREPanelTab::EventHistory => facts_ui::render_events_tab(ui, world),
            FREPanelTab::States => states_ui::render_states_tab(ui, world),
        }
    });
}

/// Render the tab bar for the FRE debug panel.
fn render_tab_bar(ui: &mut egui::Ui, world: &mut World, current_tab: FREPanelTab) {
    let mut new_tab = current_tab;
    ui.horizontal(|ui| {
        if ui
            .selectable_label(current_tab == FREPanelTab::Facts, "📊 Facts")
            .clicked()
        {
            new_tab = FREPanelTab::Facts;
        }
        if ui
            .selectable_label(current_tab == FREPanelTab::ViewFacts, "🖼 View")
            .clicked()
        {
            new_tab = FREPanelTab::ViewFacts;
        }
        if ui
            .selectable_label(current_tab == FREPanelTab::Rules, "📜 Rules")
            .clicked()
        {
            new_tab = FREPanelTab::Rules;
        }
        if ui
            .selectable_label(current_tab == FREPanelTab::EventHistory, "📨 Events")
            .clicked()
        {
            new_tab = FREPanelTab::EventHistory;
        }
        if ui
            .selectable_label(current_tab == FREPanelTab::States, "🎮 States")
            .clicked()
        {
            new_tab = FREPanelTab::States;
        }
    });
    if new_tab != current_tab {
        world.resource_mut::<FREPanelState>().current_tab = new_tab;
    }
}
