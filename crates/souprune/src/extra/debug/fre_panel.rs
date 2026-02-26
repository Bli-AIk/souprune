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

    /// Resource to track recently triggered rules for visual feedback.
    /// 跟踪最近触发的规则以提供视觉反馈的资源。
    #[derive(Resource, Default)]
    pub struct RuleTriggerHistory {
        /// Map from rule_id to last trigger timestamp (in seconds)
        /// 规则ID到上次触发时间戳（秒）的映射
        pub triggered_rules: std::collections::HashMap<String, f64>,
    }

    impl RuleTriggerHistory {
        /// Record that a rule was triggered at the current time.
        /// 记录规则在当前时间被触发。
        pub fn record_trigger(&mut self, rule_id: &str, current_time: f64) {
            self.triggered_rules
                .insert(rule_id.to_string(), current_time);
        }

        /// Check if a rule was triggered within the last N seconds.
        /// 检查规则是否在最近 N 秒内被触发。
        pub fn was_recently_triggered(
            &self,
            rule_id: &str,
            current_time: f64,
            duration: f64,
        ) -> bool {
            if let Some(&trigger_time) = self.triggered_rules.get(rule_id) {
                current_time - trigger_time < duration
            } else {
                false
            }
        }

        /// Clean up old triggers (older than 5 seconds).
        /// 清理旧的触发记录（超过5秒的）。
        pub fn cleanup_old_triggers(&mut self, current_time: f64) {
            self.triggered_rules
                .retain(|_, &mut trigger_time| current_time - trigger_time < 5.0);
        }
    }

    pub(crate) fn setup_fre_panel_debug(app: &mut App) {
        app.init_resource::<FREPanelState>()
            .init_resource::<FactEventHistory>()
            .init_resource::<RuleTriggerHistory>()
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
                    cleanup_rule_trigger_history_system,
                ),
            )
            .add_systems(
                PreUpdate,
                block_player_actions_when_fre_panel_focused_system
                    .after(InputManagerSystem::ManualControl),
            )
            .add_systems(FREPanelContextPass, fre_panel_ui_system);
    }

    /// System to clean up old rule trigger history entries.
    /// 清理旧的规则触发历史记录的系统。
    fn cleanup_rule_trigger_history_system(
        mut history: ResMut<RuleTriggerHistory>,
        time: Res<Time>,
    ) {
        history.cleanup_old_triggers(time.elapsed_secs_f64());
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
            if event.window == window_entity {
                state.window_entity = None;
                if let Some(camera_entity) = state.camera_entity.take() {
                    commands.entity(camera_entity).despawn();
                }
                state.window_focused = false;
                info!("FRE Debug Panel closed");
                break;
            }
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
            if should_disable {
                if !action_state.disabled() {
                    action_state.disable();
                }
            } else if action_state.disabled() {
                // Note: Don't enable here if other systems might have disabled it
                // We rely on the Inspector's system to re-enable
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
            // Tab bar
            ui.horizontal(|ui| {
                if ui
                    .selectable_label(current_tab == FREPanelTab::Facts, "📊 Facts")
                    .clicked()
                {
                    world.resource_mut::<FREPanelState>().current_tab = FREPanelTab::Facts;
                }
                if ui
                    .selectable_label(current_tab == FREPanelTab::ViewFacts, "🖼 View")
                    .clicked()
                {
                    world.resource_mut::<FREPanelState>().current_tab = FREPanelTab::ViewFacts;
                }
                if ui
                    .selectable_label(current_tab == FREPanelTab::Rules, "📜 Rules")
                    .clicked()
                {
                    world.resource_mut::<FREPanelState>().current_tab = FREPanelTab::Rules;
                }
                if ui
                    .selectable_label(current_tab == FREPanelTab::EventHistory, "📨 Events")
                    .clicked()
                {
                    world.resource_mut::<FREPanelState>().current_tab = FREPanelTab::EventHistory;
                }
                if ui
                    .selectable_label(current_tab == FREPanelTab::States, "🎮 States")
                    .clicked()
                {
                    world.resource_mut::<FREPanelState>().current_tab = FREPanelTab::States;
                }
            });

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

    /// Render the Rules tab.
    fn render_rules_tab(ui: &mut egui::Ui, world: &mut World) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.label("📜 Registered Rules");
            ui.separator();

            // Get time and trigger history for highlight calculation
            // 获取时间和触发历史用于高亮计算
            let current_time = world
                .get_resource::<Time>()
                .map(|t| t.elapsed_secs_f64())
                .unwrap_or(0.0);
            let trigger_history = world.get_resource::<RuleTriggerHistory>();

            let rule_registry = world.get_resource::<LayeredRuleRegistry>();

            match rule_registry {
                Some(registry) => {
                    // Count rules across all layers
                    let global_count = registry.global_iter().count();
                    let local_count = registry.local_iter().count();
                    let view_count: usize =
                        registry.view_iter().map(|(_, r)| r.iter().count()).sum();
                    let total_count = global_count + local_count + view_count;

                    ui.label(format!(
                        "Total rules: {} (Global: {}, Local: {}, View: {})",
                        total_count, global_count, local_count, view_count
                    ));
                    ui.separator();

                    if total_count == 0 {
                        ui.label("No rules registered.");
                        ui.label("Rules are loaded from .fre.ron files.");
                    } else {
                        // Show rules grouped by scope
                        // 按作用域分组显示规则

                        // Global rules
                        if global_count > 0 {
                            egui::CollapsingHeader::new(format!(
                                "🌍 Global Rules ({})",
                                global_count
                            ))
                            .default_open(true)
                            .show(ui, |ui| {
                                let mut global_rules: Vec<_> = registry.global_iter().collect();
                                global_rules.sort_by(|a, b| b.priority.cmp(&a.priority));
                                for rule in global_rules {
                                    let is_triggered = trigger_history
                                        .map(|h| {
                                            h.was_recently_triggered(&rule.id, current_time, 1.0)
                                        })
                                        .unwrap_or(false);
                                    show_rule_entry(ui, rule, is_triggered);
                                }
                            });
                        }

                        // Local rules
                        if local_count > 0 {
                            egui::CollapsingHeader::new(format!(
                                "📍 Local Rules ({})",
                                local_count
                            ))
                            .default_open(true)
                            .show(ui, |ui| {
                                let mut local_rules: Vec<_> = registry.local_iter().collect();
                                local_rules.sort_by(|a, b| b.priority.cmp(&a.priority));
                                for rule in local_rules {
                                    let is_triggered = trigger_history
                                        .map(|h| {
                                            h.was_recently_triggered(&rule.id, current_time, 1.0)
                                        })
                                        .unwrap_or(false);
                                    show_rule_entry(ui, rule, is_triggered);
                                }
                            });
                        }

                        // View rules
                        if view_count > 0 {
                            egui::CollapsingHeader::new(format!("👁 View Rules ({})", view_count))
                                .default_open(true)
                                .show(ui, |ui| {
                                    for (entity, view_registry) in registry.view_iter() {
                                        let view_rule_count = view_registry.iter().count();
                                        egui::CollapsingHeader::new(format!(
                                            "Entity {:?} ({} rules)",
                                            entity, view_rule_count
                                        ))
                                        .default_open(true)
                                        .show(ui, |ui| {
                                            let mut view_rules: Vec<_> =
                                                view_registry.iter().collect();
                                            view_rules.sort_by(|a, b| b.priority.cmp(&a.priority));
                                            for rule in view_rules {
                                                let is_triggered = trigger_history
                                                    .map(|h| {
                                                        h.was_recently_triggered(
                                                            &rule.id,
                                                            current_time,
                                                            1.0,
                                                        )
                                                    })
                                                    .unwrap_or(false);
                                                show_rule_entry(ui, rule, is_triggered);
                                            }
                                        });
                                    }
                                });
                        }
                    }
                }
                None => {
                    ui.label("LayeredRuleRegistry not available.");
                    ui.label("Make sure FREPlugin is installed.");
                }
            }

            ui.separator();

            // Show some helpful info
            egui::CollapsingHeader::new("How Rules Work").show(ui, |ui| {
                ui.label("• Rules are defined in .fre.ron files");
                ui.label("• Each rule has: trigger, condition, modifications, actions, outputs");
                ui.label("• Rules are evaluated when their trigger event fires");
                ui.label("• Conditions use facts from the LayeredFactDatabase");
            });
        });
    }

    /// Helper function to display a single rule entry with optional trigger highlight.
    /// 显示单个规则条目的辅助函数，可选触发高亮。
    ///
    /// # Arguments
    /// * `ui` - The egui UI context
    /// * `rule` - The rule to display
    /// * `is_recently_triggered` - Whether this rule was triggered in the last second
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

        // Use green color for recently triggered rules
        // 为最近触发的规则使用绿色
        let header_color = if is_recently_triggered {
            egui::Color32::from_rgb(100, 255, 100) // Bright green
        } else {
            ui.visuals().text_color()
        };

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
                ui.label(if rule.enabled { "Yes" } else { "No" });
            });

            ui.horizontal(|ui| {
                ui.label("Consume Event:");
                ui.label(if rule.consume_event { "Yes" } else { "No" });
            });

            // Condition expressions
            if !rule.condition_expressions.is_empty() {
                egui::CollapsingHeader::new(format!(
                    "Conditions ({})",
                    rule.condition_expressions.len()
                ))
                .default_open(false)
                .show(ui, |ui| {
                    for (i, expr) in rule.condition_expressions.iter().enumerate() {
                        ui.monospace(format!("{}: {}", i + 1, expr));
                    }
                });
            }

            // Modifications
            if !rule.modifications.is_empty() {
                egui::CollapsingHeader::new(format!(
                    "Modifications ({})",
                    rule.modifications.len()
                ))
                .default_open(false)
                .show(ui, |ui| {
                    for (i, modification) in rule.modifications.iter().enumerate() {
                        ui.monospace(format!("{}: {:?}", i + 1, modification));
                    }
                });
            }

            // Outputs
            if !rule.outputs.is_empty() {
                egui::CollapsingHeader::new(format!("Outputs ({})", rule.outputs.len()))
                    .default_open(false)
                    .show(ui, |ui| {
                        for output in &rule.outputs {
                            ui.monospace(&output.0);
                        }
                    });
            }
        });
    }

    /// Render the States tab.
    ///
    /// 渲染状态标签页，显示 AppState 和 OverworldSubState。
    fn render_states_tab(ui: &mut egui::Ui, world: &mut World) {
        use crate::app_state::AppState;
        use crate::app_state::overworld::OverworldSubState;

        ui.heading("Game States");
        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            // AppState section
            egui::CollapsingHeader::new("AppState")
                .default_open(true)
                .show(ui, |ui| {
                    let current_app_state =
                        world.get_resource::<State<AppState>>().map(|s| *s.get());

                    // List all AppState variants
                    let all_states = [
                        (AppState::AppSetup, "App initialization"),
                        (AppState::Menu, "Main menu"),
                        (AppState::Overworld, "Overworld exploration"),
                        (AppState::Battle, "Battle mode"),
                    ];

                    for (state, description) in all_states {
                        let is_current = current_app_state == Some(state);
                        let state_name = format!("{:?}", state);

                        ui.horizontal(|ui| {
                            if is_current {
                                ui.colored_label(egui::Color32::GREEN, "> ");
                                ui.colored_label(egui::Color32::GREEN, &state_name);
                                ui.small(description);
                            } else {
                                ui.label("  ");
                                ui.colored_label(egui::Color32::GRAY, &state_name);
                            }
                        });
                    }
                });

            ui.add_space(10.0);

            // OverworldSubState section - only show when in Overworld
            let is_in_overworld = world
                .get_resource::<State<AppState>>()
                .map(|s| *s.get() == AppState::Overworld)
                .unwrap_or(false);

            if is_in_overworld {
                egui::CollapsingHeader::new("OverworldSubState")
                    .default_open(true)
                    .show(ui, |ui| {
                        let current_sub_state = world
                            .get_resource::<State<OverworldSubState>>()
                            .map(|s| s.name().to_string());

                        // Get all available states from LoadedStateConfig
                        let state_config =
                            world.get_resource::<crate::core::state_config::LoadedStateConfig>();

                        if let Some(config) = state_config {
                            // Get all state names and sort them
                            let mut state_names: Vec<&String> = config.0.states.keys().collect();
                            state_names.sort();

                            for state_name in state_names {
                                let is_current =
                                    current_sub_state.as_deref() == Some(state_name.as_str());

                                ui.horizontal(|ui| {
                                    if is_current {
                                        ui.colored_label(egui::Color32::GREEN, "> ");
                                        ui.colored_label(egui::Color32::GREEN, state_name);
                                    } else {
                                        ui.label("  ");
                                        ui.colored_label(egui::Color32::GRAY, state_name);
                                    }
                                });

                                // Show details only for current state
                                if is_current {
                                    let is_view_interactive =
                                        config.is_view_interactive(state_name);
                                    let is_player_movable = config.is_player_movable(state_name);
                                    let view_layout = config.get_view_layout(state_name);
                                    let chase_config = config.get_chase_config_path(state_name);

                                    ui.indent(state_name, |ui| {
                                        ui.horizontal(|ui| {
                                            ui.label("UI Interactive:");
                                            if is_view_interactive {
                                                ui.colored_label(egui::Color32::GREEN, "Yes");
                                            } else {
                                                ui.colored_label(egui::Color32::GRAY, "No");
                                            }
                                        });

                                        ui.horizontal(|ui| {
                                            ui.label("Player Movable:");
                                            if is_player_movable {
                                                ui.colored_label(egui::Color32::GREEN, "Yes");
                                            } else {
                                                ui.colored_label(egui::Color32::GRAY, "No");
                                            }
                                        });

                                        ui.horizontal(|ui| {
                                            ui.label("View Layout:");
                                            if let Some(layout) = view_layout {
                                                ui.small(layout);
                                            } else {
                                                ui.colored_label(egui::Color32::GRAY, "None");
                                            }
                                        });

                                        if let Some(chase_path) = chase_config {
                                            ui.horizontal(|ui| {
                                                ui.label("Chase Config:");
                                                ui.small(chase_path);
                                            });
                                        }
                                    });
                                }
                            }
                        } else {
                            ui.label("StateConfig not loaded");
                        }
                    });
            }

            // Chase state info - only show when in Overworld
            if is_in_overworld {
                ui.add_space(10.0);

                egui::CollapsingHeader::new("Chase State Info")
                    .default_open(true)
                    .show(ui, |ui| {
                        // ChaseEnabled
                        if let Some(chase_enabled) =
                            world.get_resource::<crate::app_state::overworld::chase::ChaseEnabled>()
                        {
                            ui.horizontal(|ui| {
                                ui.label("Chase Enabled:");
                                if chase_enabled.0 {
                                    ui.colored_label(egui::Color32::GREEN, "Yes");
                                } else {
                                    ui.colored_label(egui::Color32::GRAY, "No");
                                }
                            });
                        }

                        // ChaseStateName
                        if let Some(chase_state_name) =
                            world
                                .get_resource::<crate::app_state::overworld::chase::ChaseStateName>(
                                )
                        {
                            ui.horizontal(|ui| {
                                ui.label("Chase State Name:");
                                if let Some(name) = &chase_state_name.0 {
                                    ui.strong(name);
                                } else {
                                    ui.colored_label(egui::Color32::GRAY, "Not configured");
                                }
                            });
                        }
                    });
            }
        });
    }
}
