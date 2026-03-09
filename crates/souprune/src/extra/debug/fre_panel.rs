//! # fre_panel.rs
//!
//! # fre_panel.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! FRE Debug Panel for viewing and modifying FRE (Fact-Rule-Event) data at runtime.
//! Press F7 to open/close the debug panel in a standalone window.
//!
//! FRE 调试面板，用于在运行时查看和修改 FRE（事实-规则-事件）数据。
//! 按 F7 键打开/关闭独立窗口中的调试面板。

#[cfg(feature = "debug")]
pub mod debug_fre_panel {
    mod facts_ui;

    use crate::app_state::overworld::character::components::PlayerControlled;
    use crate::core::input::Action;
    use bevy::camera::RenderTarget;
    use bevy::ecs::schedule::ScheduleLabel;
    use bevy::prelude::*;
    use bevy::window::{
        PrimaryWindow, Window, WindowClosed, WindowFocused, WindowRef, WindowResolution,
    };
    use bevy_fact_rule_event::{FactEvent, LayeredRuleRegistry};
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

    use crate::extra::debug::RuleTriggerHistory;

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
    ) {
        if !keyboard_input.just_pressed(KeyCode::F2) {
            return;
        }

        if state.window_entity.is_some() {
            close_fre_panel(&mut commands, &mut state);
        } else {
            spawn_fre_panel(&mut commands, &mut state);
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
        // If PrimaryWindow component was removed from any entity, close FRE panel
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
        // Only trigger refresh if FRE panel window is open and state just changed
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
            // Note: Don't enable here if other systems might have disabled it
            // We rely on the Inspector's system to re-enable
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

            // Keep only the most recent events
            while history.events.len() > MAX_EVENT_HISTORY {
                history.events.pop_back();
            }
        }
    }

    /// Main UI system for the FRE debug panel.
    fn fre_panel_ui_system(world: &mut World) {
        // Get the camera entity
        let camera_entity = world
            .get_resource::<FREPanelState>()
            .and_then(|state| state.camera_entity);

        let Some(camera_entity) = camera_entity else {
            return;
        };

        // Get egui context
        let mut contexts = world.query_filtered::<&mut EguiContext, With<FREPanelCamera>>();
        let mut egui_context = match contexts.get_mut(world, camera_entity) {
            Ok(context) => context.clone(),
            Err(_) => return,
        };

        // Get current tab first (before mutable borrow)
        let current_tab = world.resource::<FREPanelState>().current_tab;

        // Render the panel
        egui::CentralPanel::default().show(egui_context.get_mut(), |ui| {
            render_tab_bar(ui, world, current_tab);

            ui.separator();

            // Content based on selected tab
            match current_tab {
                FREPanelTab::Facts => facts_ui::render_facts_tab(ui, world),
                FREPanelTab::ViewFacts => facts_ui::render_view_facts_tab(ui, world),
                FREPanelTab::Rules => render_rules_tab(ui, world),
                FREPanelTab::EventHistory => facts_ui::render_events_tab(ui, world),
                FREPanelTab::States => render_states_tab(ui, world),
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

    /// Render the Rules tab.
    fn render_rules_tab(ui: &mut egui::Ui, world: &mut World) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.label("📜 Registered Rules");
            ui.separator();

            let current_time = world
                .get_resource::<Time>()
                .map(|t| t.elapsed_secs_f64())
                .unwrap_or(0.0);
            let trigger_history = world.get_resource::<RuleTriggerHistory>();

            let Some(registry) = world.get_resource::<LayeredRuleRegistry>() else {
                ui.label("LayeredRuleRegistry not available.");
                ui.label("Make sure FREPlugin is installed.");
                return;
            };

            let global_count = registry.global_iter().count();
            let local_count = registry.local_iter().count();
            let view_count: usize = registry.view_iter().map(|(_, r)| r.iter().count()).sum();
            let total_count = global_count + local_count + view_count;

            ui.label(format!(
                "Total rules: {} (Global: {}, Local: {}, View: {})",
                total_count, global_count, local_count, view_count
            ));
            ui.separator();

            if total_count == 0 {
                ui.label("No rules registered.");
                ui.label("Rules are loaded from .fre.ron files.");
                return;
            }

            if global_count > 0 {
                let mut rules: Vec<_> = registry.global_iter().collect();
                render_rule_group(
                    ui,
                    "🌍 Global Rules",
                    &mut rules,
                    trigger_history,
                    current_time,
                );
            }

            if local_count > 0 {
                let mut rules: Vec<_> = registry.local_iter().collect();
                render_rule_group(
                    ui,
                    "📍 Local Rules",
                    &mut rules,
                    trigger_history,
                    current_time,
                );
            }

            if view_count > 0 {
                render_view_rules_group(ui, registry, trigger_history, current_time, view_count);
            }

            ui.separator();

            egui::CollapsingHeader::new("How Rules Work").show(ui, |ui| {
                ui.label("• Rules are defined in .fre.ron files");
                ui.label("• Each rule has: trigger, condition, modifications, actions, outputs");
                ui.label("• Rules are evaluated when their trigger event fires");
                ui.label("• Conditions use facts from the LayeredFactDatabase");
            });
        });
    }

    /// Render a group of rules under a collapsing header.
    fn render_rule_group(
        ui: &mut egui::Ui,
        title: &str,
        rules: &mut [&bevy_fact_rule_event::Rule],
        trigger_history: Option<&RuleTriggerHistory>,
        current_time: f64,
    ) {
        rules.sort_by(|a, b| b.priority.cmp(&a.priority));
        egui::CollapsingHeader::new(format!("{} ({})", title, rules.len()))
            .default_open(true)
            .show(ui, |ui| {
                for rule in rules.iter() {
                    let is_triggered = trigger_history
                        .is_some_and(|h| h.was_recently_triggered(&rule.id, current_time, 1.0));
                    show_rule_entry(ui, rule, is_triggered);
                }
            });
    }

    /// Render the view rules group with per-entity sub-groups.
    fn render_view_rules_group(
        ui: &mut egui::Ui,
        registry: &LayeredRuleRegistry,
        trigger_history: Option<&RuleTriggerHistory>,
        current_time: f64,
        view_count: usize,
    ) {
        egui::CollapsingHeader::new(format!("👁 View Rules ({})", view_count))
            .default_open(true)
            .show(ui, |ui| {
                for (entity, view_registry) in registry.view_iter() {
                    render_view_entity_rules(
                        ui,
                        entity,
                        view_registry,
                        trigger_history,
                        current_time,
                    );
                }
            });
    }

    /// Render rules for a single view entity.
    fn render_view_entity_rules(
        ui: &mut egui::Ui,
        entity: Entity,
        view_registry: &bevy_fact_rule_event::RuleRegistry,
        trigger_history: Option<&RuleTriggerHistory>,
        current_time: f64,
    ) {
        let view_rule_count = view_registry.iter().count();
        let mut view_rules: Vec<_> = view_registry.iter().collect();
        view_rules.sort_by(|a, b| b.priority.cmp(&a.priority));
        egui::CollapsingHeader::new(format!("Entity {:?} ({} rules)", entity, view_rule_count))
            .default_open(true)
            .show(ui, |ui| {
                for rule in view_rules {
                    let is_triggered = trigger_history
                        .is_some_and(|h| h.was_recently_triggered(&rule.id, current_time, 1.0));
                    show_rule_entry(ui, rule, is_triggered);
                }
            });
    }

    /// Helper function to display a single rule entry with optional trigger highlight.
    fn show_rule_entry(
        ui: &mut egui::Ui,
        rule: &bevy_fact_rule_event::Rule,
        is_recently_triggered: bool,
    ) {
        let status_icon = if rule.enabled { "✅" } else { "❌" };
        let trigger_indicator = if is_recently_triggered { "🔥 " } else { "" };
        let header_text = format!(
            "{}{} {} [Priority: {}]",
            trigger_indicator, status_icon, rule.id, rule.priority
        );

        let header_color = if is_recently_triggered {
            egui::Color32::from_rgb(100, 255, 100)
        } else {
            ui.visuals().text_color()
        };

        let enabled_text = if rule.enabled { "Yes" } else { "No" };
        let consume_text = if rule.consume_event { "Yes" } else { "No" };

        let header =
            egui::CollapsingHeader::new(egui::RichText::new(header_text).color(header_color))
                .default_open(false);

        header.show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("Trigger:");
                ui.monospace(&rule.trigger.0);
            });

            ui.horizontal(|ui| {
                ui.label("Enabled:");
                ui.label(enabled_text);
            });

            ui.horizontal(|ui| {
                ui.label("Consume Event:");
                ui.label(consume_text);
            });

            show_rule_conditions(ui, &rule.condition_expressions);
            show_rule_modifications(ui, &rule.modifications);
            show_rule_outputs(ui, &rule.outputs);
        });
    }

    /// Render condition expressions for a rule.
    fn show_rule_conditions(ui: &mut egui::Ui, expressions: &[String]) {
        if expressions.is_empty() {
            return;
        }
        egui::CollapsingHeader::new(format!("Conditions ({})", expressions.len()))
            .default_open(false)
            .show(ui, |ui| {
                for (i, expr) in expressions.iter().enumerate() {
                    ui.monospace(format!("{}: {}", i + 1, expr));
                }
            });
    }

    /// Render modifications for a rule.
    fn show_rule_modifications(
        ui: &mut egui::Ui,
        modifications: &[bevy_fact_rule_event::FactModification],
    ) {
        if modifications.is_empty() {
            return;
        }
        egui::CollapsingHeader::new(format!("Modifications ({})", modifications.len()))
            .default_open(false)
            .show(ui, |ui| {
                for (i, modification) in modifications.iter().enumerate() {
                    ui.monospace(format!("{}: {:?}", i + 1, modification));
                }
            });
    }

    /// Render output events for a rule.
    fn show_rule_outputs(ui: &mut egui::Ui, outputs: &[bevy_fact_rule_event::FactEventId]) {
        if outputs.is_empty() {
            return;
        }
        egui::CollapsingHeader::new(format!("Outputs ({})", outputs.len()))
            .default_open(false)
            .show(ui, |ui| {
                for output in outputs {
                    ui.monospace(&output.0);
                }
            });
    }

    /// Render the States tab.
    fn render_states_tab(ui: &mut egui::Ui, world: &mut World) {
        ui.heading("Game States");
        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            render_app_state_section(ui, world);
            ui.add_space(5.0);
            render_sequence_mode_section(ui, world);
            ui.add_space(10.0);
            render_sequence_sub_state_section(ui, world);
            render_chase_state_section(ui, world);
        });
    }

    /// Render a state indicator row with current/inactive styling.
    fn render_state_row(
        ui: &mut egui::Ui,
        is_current: bool,
        state_name: &str,
        description: Option<&str>,
    ) {
        let desc_to_show = if is_current { description } else { None };
        ui.horizontal(|ui| {
            if is_current {
                ui.colored_label(egui::Color32::GREEN, "> ");
                ui.colored_label(egui::Color32::GREEN, state_name);
            } else {
                ui.label("  ");
                ui.colored_label(egui::Color32::GRAY, state_name);
            }
            if let Some(desc) = desc_to_show {
                ui.small(desc);
            }
        });
    }

    /// Render the AppState section.
    fn render_app_state_section(ui: &mut egui::Ui, world: &mut World) {
        use crate::app_state::AppState;

        egui::CollapsingHeader::new("AppState")
            .default_open(true)
            .show(ui, |ui| {
                let current_app_state = world.get_resource::<State<AppState>>().map(|s| *s.get());

                let all_states = [
                    (AppState::Loading, "Resource loading"),
                    (AppState::Running, "Running"),
                ];

                for (state, description) in all_states {
                    let is_current = current_app_state == Some(state);
                    render_state_row(ui, is_current, &format!("{:?}", state), Some(description));
                }
            });
    }

    /// Render the SequenceMode section.
    fn render_sequence_mode_section(ui: &mut egui::Ui, world: &mut World) {
        use crate::app_state::SequenceMode;

        egui::CollapsingHeader::new("SequenceMode")
            .default_open(true)
            .show(ui, |ui| {
                let current_mode = world
                    .get_resource::<SequenceMode>()
                    .and_then(|m| m.0.clone());

                let (text, color) = match &current_mode {
                    Some(mode) => (mode.as_str(), egui::Color32::GREEN),
                    None => ("None", egui::Color32::GRAY),
                };

                ui.horizontal(|ui| {
                    ui.label("Current:");
                    ui.colored_label(color, text);
                });
            });
    }

    /// Render the SequenceSubState section.
    fn render_sequence_sub_state_section(ui: &mut egui::Ui, world: &mut World) {
        use crate::app_state::{SequenceMode, SequenceSubState};

        let has_mode = world
            .get_resource::<SequenceMode>()
            .map(|m| m.0.is_some())
            .unwrap_or(false);

        if !has_mode {
            return;
        }

        egui::CollapsingHeader::new("SequenceSubState")
            .default_open(true)
            .show(ui, |ui| {
                let current_sub_state = world
                    .get_resource::<State<SequenceSubState>>()
                    .map(|s| s.name().to_string());

                let state_config =
                    world.get_resource::<crate::core::state_config::LoadedStateConfig>();

                let Some(config) = state_config else {
                    ui.label("StateConfig not loaded");
                    return;
                };

                let mut state_names: Vec<&String> = config.0.states.keys().collect();
                state_names.sort();

                for state_name in state_names {
                    let is_current = current_sub_state.as_deref() == Some(state_name.as_str());
                    render_sub_state_row(ui, state_name, is_current, config);
                }
            });
    }

    /// Render a single sub-state row with details when current.
    fn render_sub_state_row(
        ui: &mut egui::Ui,
        state_name: &str,
        is_current: bool,
        config: &crate::core::state_config::LoadedStateConfig,
    ) {
        render_state_row(ui, is_current, state_name, None);

        if !is_current {
            return;
        }

        let is_view_interactive = config.is_view_interactive(state_name);
        let is_player_movable = config.is_player_movable(state_name);
        let view_layout = config.get_view_layout(state_name);
        let chase_config = config.get_chase_config_path(state_name);

        let interactive_color = if is_view_interactive {
            egui::Color32::GREEN
        } else {
            egui::Color32::GRAY
        };
        let interactive_text = if is_view_interactive { "Yes" } else { "No" };
        let movable_color = if is_player_movable {
            egui::Color32::GREEN
        } else {
            egui::Color32::GRAY
        };
        let movable_text = if is_player_movable { "Yes" } else { "No" };

        ui.indent(state_name, |ui| {
            ui.horizontal(|ui| {
                ui.label("UI Interactive:");
                ui.colored_label(interactive_color, interactive_text);
            });

            ui.horizontal(|ui| {
                ui.label("Player Movable:");
                ui.colored_label(movable_color, movable_text);
            });

            render_view_layout_row(ui, view_layout);
            render_chase_config_row(ui, chase_config);
        });
    }

    /// Render the view layout row.
    fn render_view_layout_row(ui: &mut egui::Ui, view_layout: Option<&str>) {
        ui.horizontal(|ui| {
            ui.label("View Layout:");
            if let Some(layout) = view_layout {
                ui.small(layout);
            } else {
                ui.colored_label(egui::Color32::GRAY, "None");
            }
        });
    }

    /// Render the chase config path row.
    fn render_chase_config_row(ui: &mut egui::Ui, chase_config: Option<&str>) {
        let Some(chase_path) = chase_config else {
            return;
        };
        ui.horizontal(|ui| {
            ui.label("Chase Config:");
            ui.small(chase_path);
        });
    }

    /// Render the Chase State Info section.
    fn render_chase_state_section(ui: &mut egui::Ui, world: &mut World) {
        let has_mode = world
            .get_resource::<crate::app_state::SequenceMode>()
            .map(|m| m.0.is_some())
            .unwrap_or(false);

        if !has_mode {
            return;
        }

        ui.add_space(10.0);

        let chase_enabled_display: Option<(&str, egui::Color32)> = world
            .get_resource::<crate::app_state::overworld::chase::ChaseEnabled>()
            .map(|c| {
                if c.0 {
                    ("Yes", egui::Color32::GREEN)
                } else {
                    ("No", egui::Color32::GRAY)
                }
            });

        let chase_name_val = world
            .get_resource::<crate::app_state::overworld::chase::ChaseStateName>()
            .and_then(|c| c.0.clone());

        egui::CollapsingHeader::new("Chase State Info")
            .default_open(true)
            .show(ui, |ui| {
                render_chase_enabled_row(ui, chase_enabled_display);
                render_chase_name_row(ui, &chase_name_val);
            });
    }

    /// Render the chase enabled status row.
    fn render_chase_enabled_row(ui: &mut egui::Ui, display: Option<(&str, egui::Color32)>) {
        let Some((text, color)) = display else {
            return;
        };
        ui.horizontal(|ui| {
            ui.label("Chase Enabled:");
            ui.colored_label(color, text);
        });
    }

    /// Render the chase state name row.
    fn render_chase_name_row(ui: &mut egui::Ui, name: &Option<String>) {
        let (display_text, use_strong) = match name {
            Some(n) => (n.as_str(), true),
            None => ("Not configured", false),
        };
        ui.horizontal(|ui| {
            ui.label("Chase State Name:");
            if use_strong {
                ui.strong(display_text);
            } else {
                ui.colored_label(egui::Color32::GRAY, display_text);
            }
        });
    }
}
