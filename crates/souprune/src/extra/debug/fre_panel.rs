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
    use crate::app_state::overworld::character::components::PlayerControlled;
    use crate::core::input::Action;
    use bevy::camera::RenderTarget;
    use bevy::ecs::schedule::ScheduleLabel;
    use bevy::prelude::*;
    use bevy::window::{
        PrimaryWindow, Window, WindowClosed, WindowFocused, WindowRef, WindowResolution,
    };
    use bevy_fact_rule_event::{FactEvent, FactValue, LayeredFactDatabase, LayeredRuleRegistry};
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
                Camera {
                    target: RenderTarget::Window(WindowRef::Entity(window_entity)),
                    ..default()
                },
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
                FREPanelTab::Facts => render_facts_tab(ui, world),
                FREPanelTab::ViewFacts => render_view_facts_tab(ui, world),
                FREPanelTab::Rules => render_rules_tab(ui, world),
                FREPanelTab::EventHistory => render_events_tab(ui, world),
                FREPanelTab::States => render_states_tab(ui, world),
            }
        });
    }

    /// Render the Facts tab.
    fn render_facts_tab(ui: &mut egui::Ui, world: &mut World) {
        // Search filter
        let mut search_filter = world.resource::<FREPanelState>().search_filter.clone();
        ui.horizontal(|ui| {
            ui.label("🔍");
            if ui.text_edit_singleline(&mut search_filter).changed() {
                world.resource_mut::<FREPanelState>().search_filter = search_filter.clone();
            }
        });

        ui.separator();

        // Check if LayeredFactDatabase exists
        let has_layered = world.get_resource::<LayeredFactDatabase>().is_some();

        egui::ScrollArea::vertical().show(ui, |ui| {
            if !has_layered {
                ui.label("⚠️ No LayeredFactDatabase found.");
                ui.label("FRE system may not be initialized.");
                return;
            }

            // Show LayeredFactDatabase with Global/Local layers
            egui::CollapsingHeader::new("🌍 Global Layer")
                .default_open(true)
                .show(ui, |ui| {
                    render_layered_facts(ui, world, FactLayerSelection::Global, &search_filter);
                });

            egui::CollapsingHeader::new("🎮 Local Layer")
                .default_open(true)
                .show(ui, |ui| {
                    render_layered_facts(ui, world, FactLayerSelection::Local, &search_filter);
                });

            ui.separator();

            // Add new fact form
            egui::CollapsingHeader::new("➕ Add New Fact").show(ui, |ui| {
                render_add_fact_form(ui, world, true);
            });
        });
    }

    /// Render the View Local Facts tab.
    /// Shows local facts from all active ViewRoot components.
    ///
    /// 渲染 View 局部事实选项卡。
    /// 显示所有活跃 ViewRoot 组件的局部事实。
    fn render_view_facts_tab(ui: &mut egui::Ui, world: &mut World) {
        use crate::core::view::components::ViewRoot;

        // Search filter
        let search_filter = world.resource::<FREPanelState>().search_filter.clone();
        let mut new_filter = search_filter.clone();
        ui.horizontal(|ui| {
            ui.label("🔍");
            if ui.text_edit_singleline(&mut new_filter).changed() {
                world.resource_mut::<FREPanelState>().search_filter = new_filter.clone();
            }
        });

        ui.separator();

        // Query all ViewRoot components
        let mut view_roots: Vec<(Entity, String, String, Vec<(String, FactValue)>)> = Vec::new();

        // Use a scope to avoid borrowing world mutably while iterating
        {
            let mut query = world.query::<(Entity, &ViewRoot, Option<&Name>)>();
            for (entity, view_root, name) in query.iter(world) {
                let display_name = name
                    .map(|n| n.to_string())
                    .unwrap_or_else(|| format!("Entity {:?}", entity));

                let facts: Vec<_> = view_root
                    .local_facts
                    .iter()
                    .filter(|(k, _)| search_filter.is_empty() || k.0.contains(&search_filter))
                    .map(|(k, v)| (k.0.clone(), v.clone()))
                    .collect();

                view_roots.push((entity, display_name, view_root.namespace.clone(), facts));
            }
        }

        egui::ScrollArea::vertical().show(ui, |ui| {
            if view_roots.is_empty() {
                ui.label("⚠ No active View instances found.");
                ui.label("Views with local_facts will appear here when loaded.");
                return;
            }

            ui.label(format!("📊 {} active View instance(s)", view_roots.len()));
            ui.separator();

            // Collect modifications to apply after iteration
            let mut modifications: Vec<(Entity, String, FactValue)> = Vec::new();

            for (entity, display_name, namespace, facts) in &view_roots {
                let header_text = format!("🖼 {} ({})", display_name, namespace);
                egui::CollapsingHeader::new(header_text)
                    .default_open(true)
                    .show(ui, |ui| {
                        if facts.is_empty() {
                            ui.label("(no local facts)");
                            return;
                        }

                        for (key, value) in facts {
                            ui.horizontal(|ui| {
                                // Key label with namespace prefix indicator
                                let key_label = if key.starts_with("view.") {
                                    format!("  {}", key)
                                } else {
                                    format!("  ${}", key)
                                };
                                ui.label(&key_label);

                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| match value {
                                        FactValue::Int(v) => {
                                            let mut val = *v;
                                            if ui.add(egui::DragValue::new(&mut val)).changed() {
                                                modifications.push((
                                                    *entity,
                                                    key.clone(),
                                                    FactValue::Int(val),
                                                ));
                                            }
                                        }
                                        FactValue::Float(v) => {
                                            let mut val = *v;
                                            if ui
                                                .add(egui::DragValue::new(&mut val).speed(0.1))
                                                .changed()
                                            {
                                                modifications.push((
                                                    *entity,
                                                    key.clone(),
                                                    FactValue::Float(val),
                                                ));
                                            }
                                        }
                                        FactValue::Bool(v) => {
                                            let mut checked = *v;
                                            if ui.checkbox(&mut checked, "").changed() {
                                                modifications.push((
                                                    *entity,
                                                    key.clone(),
                                                    FactValue::Bool(checked),
                                                ));
                                            }
                                        }
                                        FactValue::String(s) => {
                                            let mut text = s.clone();
                                            let response = ui.add(
                                                egui::TextEdit::singleline(&mut text)
                                                    .desired_width(150.0),
                                            );
                                            if response.changed() {
                                                modifications.push((
                                                    *entity,
                                                    key.clone(),
                                                    FactValue::String(text.clone()),
                                                ));
                                            }
                                        }
                                        FactValue::StringList(list) => {
                                            // Show list with editable elements
                                            ui.push_id(format!("strlist_{}", key), |ui| {
                                                ui.collapsing(format!("[{}]", list.len()), |ui| {
                                                    let mut new_list = list.clone();
                                                    let mut changed = false;
                                                    let mut to_remove: Option<usize> = None;

                                                    for (idx, item) in
                                                        new_list.iter_mut().enumerate()
                                                    {
                                                        ui.horizontal(|ui| {
                                                            ui.label(format!("  {}:", idx));
                                                            let response = ui.add(
                                                                egui::TextEdit::singleline(item)
                                                                    .desired_width(120.0),
                                                            );
                                                            if response.changed() {
                                                                changed = true;
                                                            }
                                                            if ui.small_button("🗑").clicked() {
                                                                to_remove = Some(idx);
                                                            }
                                                        });
                                                    }

                                                    // Handle removal
                                                    if let Some(idx) = to_remove {
                                                        new_list.remove(idx);
                                                        changed = true;
                                                    }

                                                    // Add new element button
                                                    if ui.small_button("➕ Add").clicked() {
                                                        new_list.push(String::new());
                                                        changed = true;
                                                    }

                                                    if changed {
                                                        modifications.push((
                                                            *entity,
                                                            key.clone(),
                                                            FactValue::StringList(new_list),
                                                        ));
                                                    }
                                                });
                                            });
                                        }
                                        FactValue::IntList(list) => {
                                            // Show int list with editable elements
                                            ui.push_id(format!("intlist_{}", key), |ui| {
                                                ui.collapsing(format!("[{}]", list.len()), |ui| {
                                                    let mut new_list = list.clone();
                                                    let mut changed = false;
                                                    let mut to_remove: Option<usize> = None;

                                                    for (idx, item) in
                                                        new_list.iter_mut().enumerate()
                                                    {
                                                        ui.horizontal(|ui| {
                                                            ui.label(format!("  {}:", idx));
                                                            if ui
                                                                .add(egui::DragValue::new(item))
                                                                .changed()
                                                            {
                                                                changed = true;
                                                            }
                                                            if ui.small_button("🗑").clicked() {
                                                                to_remove = Some(idx);
                                                            }
                                                        });
                                                    }

                                                    // Handle removal
                                                    if let Some(idx) = to_remove {
                                                        new_list.remove(idx);
                                                        changed = true;
                                                    }

                                                    // Add new element button
                                                    if ui.small_button("➕ Add").clicked() {
                                                        new_list.push(0);
                                                        changed = true;
                                                    }

                                                    if changed {
                                                        modifications.push((
                                                            *entity,
                                                            key.clone(),
                                                            FactValue::IntList(new_list),
                                                        ));
                                                    }
                                                });
                                            });
                                        }
                                    },
                                );
                            });
                        }
                    });
            }

            // Apply modifications to ViewRoot local_facts
            for (entity, key, value) in modifications {
                if let Some(mut view_root) = world.get_mut::<ViewRoot>(entity) {
                    view_root.local_facts.set(key.as_str(), value);
                }
            }
        });
    }

    /// Render facts from LayeredFactDatabase.
    fn render_layered_facts(
        ui: &mut egui::Ui,
        world: &mut World,
        layer: FactLayerSelection,
        filter: &str,
    ) {
        let db = world.resource::<LayeredFactDatabase>();
        let facts: Vec<_> = match layer {
            FactLayerSelection::Global => db
                .global()
                .iter()
                .filter(|(k, _)| filter.is_empty() || k.0.contains(filter))
                .map(|(k, v)| (k.0.clone(), v.clone()))
                .collect(),
            FactLayerSelection::Local => db
                .local()
                .iter()
                .filter(|(k, _)| filter.is_empty() || k.0.contains(filter))
                .map(|(k, v)| (k.0.clone(), v.clone()))
                .collect(),
        };

        if facts.is_empty() {
            ui.label("(empty)");
            return;
        }

        let mut modifications: Vec<(String, FactValue, FactLayerSelection)> = Vec::new();

        for (key, value) in facts {
            ui.horizontal(|ui| {
                ui.label(&key);
                ui.with_layout(
                    egui::Layout::right_to_left(egui::Align::Center),
                    |ui| match value {
                        FactValue::Int(v) => {
                            let mut val = v;
                            if ui.add(egui::DragValue::new(&mut val)).changed() {
                                modifications.push((key.clone(), FactValue::Int(val), layer));
                            }
                        }
                        FactValue::Float(v) => {
                            let mut val = v;
                            if ui.add(egui::DragValue::new(&mut val).speed(0.1)).changed() {
                                modifications.push((key.clone(), FactValue::Float(val), layer));
                            }
                        }
                        FactValue::Bool(v) => {
                            let mut checked = v;
                            if ui.checkbox(&mut checked, "").changed() {
                                modifications.push((key.clone(), FactValue::Bool(checked), layer));
                            }
                        }
                        FactValue::String(ref s) => {
                            let mut text = s.clone();
                            let response =
                                ui.add(egui::TextEdit::singleline(&mut text).desired_width(150.0));
                            if response.changed() {
                                modifications.push((
                                    key.clone(),
                                    FactValue::String(text.clone()),
                                    layer,
                                ));
                            }
                        }
                        FactValue::StringList(ref list) => {
                            // Show list with editable elements
                            ui.push_id(format!("layered_strlist_{}", key), |ui| {
                                ui.collapsing(format!("[{}]", list.len()), |ui| {
                                    let mut new_list = list.clone();
                                    let mut changed = false;
                                    let mut to_remove: Option<usize> = None;

                                    for (idx, item) in new_list.iter_mut().enumerate() {
                                        ui.horizontal(|ui| {
                                            ui.label(format!("  {}:", idx));
                                            let response = ui.add(
                                                egui::TextEdit::singleline(item)
                                                    .desired_width(120.0),
                                            );
                                            if response.changed() {
                                                changed = true;
                                            }
                                            if ui.small_button("🗑").clicked() {
                                                to_remove = Some(idx);
                                            }
                                        });
                                    }

                                    // Handle removal
                                    if let Some(idx) = to_remove {
                                        new_list.remove(idx);
                                        changed = true;
                                    }

                                    // Add new element button
                                    if ui.small_button("➕ Add").clicked() {
                                        new_list.push(String::new());
                                        changed = true;
                                    }

                                    if changed {
                                        modifications.push((
                                            key.clone(),
                                            FactValue::StringList(new_list),
                                            layer,
                                        ));
                                    }
                                });
                            });
                        }
                        FactValue::IntList(ref list) => {
                            // Show int list with editable elements
                            ui.push_id(format!("layered_intlist_{}", key), |ui| {
                                ui.collapsing(format!("[{}]", list.len()), |ui| {
                                    let mut new_list = list.clone();
                                    let mut changed = false;
                                    let mut to_remove: Option<usize> = None;

                                    for (idx, item) in new_list.iter_mut().enumerate() {
                                        ui.horizontal(|ui| {
                                            ui.label(format!("  {}:", idx));
                                            if ui.add(egui::DragValue::new(item)).changed() {
                                                changed = true;
                                            }
                                            if ui.small_button("🗑").clicked() {
                                                to_remove = Some(idx);
                                            }
                                        });
                                    }

                                    // Handle removal
                                    if let Some(idx) = to_remove {
                                        new_list.remove(idx);
                                        changed = true;
                                    }

                                    // Add new element button
                                    if ui.small_button("➕ Add").clicked() {
                                        new_list.push(0);
                                        changed = true;
                                    }

                                    if changed {
                                        modifications.push((
                                            key.clone(),
                                            FactValue::IntList(new_list),
                                            layer,
                                        ));
                                    }
                                });
                            });
                        }
                    },
                );
            });
        }

        // Apply modifications to LayeredFactDatabase
        for (key, value, layer) in modifications {
            let mut db = world.resource_mut::<LayeredFactDatabase>();
            match layer {
                FactLayerSelection::Global => db.set_global(key.as_str(), value),
                FactLayerSelection::Local => db.set_local(key.as_str(), value),
            }
        }
    }

    /// Render the add fact form.
    fn render_add_fact_form(ui: &mut egui::Ui, world: &mut World, _has_layered: bool) {
        let state = world.resource::<FREPanelState>();
        let mut key = state.new_fact_key.clone();
        let mut value_str = state.new_fact_value_str.clone();
        let mut fact_type = state.new_fact_type;
        let mut fact_layer = state.new_fact_layer;

        ui.horizontal(|ui| {
            ui.label("Key:");
            ui.text_edit_singleline(&mut key);
        });

        ui.horizontal(|ui| {
            ui.label("Type:");
            egui::ComboBox::from_id_salt("fact_type")
                .selected_text(match fact_type {
                    FactTypeSelection::Int => "Int",
                    FactTypeSelection::Float => "Float",
                    FactTypeSelection::Bool => "Bool",
                    FactTypeSelection::String => "String",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut fact_type, FactTypeSelection::Int, "Int");
                    ui.selectable_value(&mut fact_type, FactTypeSelection::Float, "Float");
                    ui.selectable_value(&mut fact_type, FactTypeSelection::Bool, "Bool");
                    ui.selectable_value(&mut fact_type, FactTypeSelection::String, "String");
                });
        });

        ui.horizontal(|ui| {
            ui.label("Value:");
            ui.text_edit_singleline(&mut value_str);
        });

        // Layer selection for LayeredFactDatabase
        ui.horizontal(|ui| {
            ui.label("Layer:");
            egui::ComboBox::from_id_salt("fact_layer")
                .selected_text(match fact_layer {
                    FactLayerSelection::Local => "Local",
                    FactLayerSelection::Global => "Global",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut fact_layer, FactLayerSelection::Local, "Local");
                    ui.selectable_value(&mut fact_layer, FactLayerSelection::Global, "Global");
                });
        });

        // Update state
        {
            let mut state = world.resource_mut::<FREPanelState>();
            state.new_fact_key = key.clone();
            state.new_fact_value_str = value_str.clone();
            state.new_fact_type = fact_type;
            state.new_fact_layer = fact_layer;
        }

        if ui.button("Add Fact").clicked() && !key.is_empty() {
            let value = match fact_type {
                FactTypeSelection::Int => value_str.parse::<i64>().ok().map(FactValue::Int),
                FactTypeSelection::Float => value_str.parse::<f64>().ok().map(FactValue::Float),
                FactTypeSelection::Bool => value_str.parse::<bool>().ok().map(FactValue::Bool),
                FactTypeSelection::String => Some(FactValue::String(value_str.clone())),
            };

            if let Some(value) = value {
                let mut db = world.resource_mut::<LayeredFactDatabase>();
                match fact_layer {
                    FactLayerSelection::Local => db.set_local(key.as_str(), value),
                    FactLayerSelection::Global => db.set_global(key.as_str(), value),
                }

                // Clear input
                let mut state = world.resource_mut::<FREPanelState>();
                state.new_fact_key.clear();
                state.new_fact_value_str.clear();

                info!("Added fact: {} = {:?}", key, value_str);
            }
        }
    }

    /// Render the Rules tab.
    fn render_rules_tab(ui: &mut egui::Ui, world: &mut World) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.label("📜 Registered Rules");
            ui.separator();

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
                                    show_rule_entry(ui, rule);
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
                                    show_rule_entry(ui, rule);
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
                                                show_rule_entry(ui, rule);
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

    /// Helper function to display a single rule entry.
    /// 显示单个规则条目的辅助函数。
    fn show_rule_entry(ui: &mut egui::Ui, rule: &bevy_fact_rule_event::Rule) {
        let status_icon = if rule.enabled { "✅" } else { "❌" };
        let header_text = format!("{} {} [Priority: {}]", status_icon, rule.id, rule.priority);

        egui::CollapsingHeader::new(header_text)
            .default_open(false)
            .show(ui, |ui| {
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

                // Condition
                egui::CollapsingHeader::new("Condition")
                    .default_open(false)
                    .show(ui, |ui| {
                        ui.monospace(format!("{:?}", rule.condition));
                    });

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

                // Actions
                if !rule.actions.is_empty() {
                    egui::CollapsingHeader::new(format!("Actions ({})", rule.actions.len()))
                        .default_open(false)
                        .show(ui, |ui| {
                            for i in 0..rule.actions.len() {
                                ui.monospace(format!("{}: <action function>", i + 1));
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

    /// Render the Event History tab.
    fn render_events_tab(ui: &mut egui::Ui, world: &mut World) {
        let event_count = world.resource::<FactEventHistory>().events.len();

        ui.horizontal(|ui| {
            ui.label(format!(
                "📨 Recent Events ({}/{})",
                event_count, MAX_EVENT_HISTORY
            ));
            if ui.button("Clear").clicked() {
                world.resource_mut::<FactEventHistory>().events.clear();
            }
        });

        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            let history = world.resource::<FactEventHistory>();

            if history.events.is_empty() {
                ui.label("No events recorded yet.");
                ui.label("Events will appear here when FactEvents are emitted.");
                return;
            }

            for record in history.events.iter() {
                ui.horizontal(|ui| {
                    ui.label(format!("[{:.2}s]", record.timestamp));
                    ui.strong(&record.event_id);
                    if !record.data_keys.is_empty() {
                        ui.label(format!("(data: {})", record.data_keys.join(", ")));
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
                                    let is_ui_interactive = config.is_ui_interactive(state_name);
                                    let is_player_movable = config.is_player_movable(state_name);
                                    let view_layout = config.get_view_layout(state_name);
                                    let chase_config = config.get_chase_config_path(state_name);

                                    ui.indent(state_name, |ui| {
                                        ui.horizontal(|ui| {
                                            ui.label("UI Interactive:");
                                            if is_ui_interactive {
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
