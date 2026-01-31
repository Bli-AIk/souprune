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
    use bevy::camera::RenderTarget;
    use bevy::ecs::schedule::ScheduleLabel;
    use bevy::prelude::*;
    use bevy::window::{Window, WindowClosed, WindowRef, WindowResolution};
    use bevy_fact_rule_event::{FactEvent, FactValue, LayeredFactDatabase};
    use bevy_inspector_egui::bevy_egui::{EguiContext, EguiMultipassSchedule};
    use bevy_inspector_egui::egui;
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

    /// UI state resource for the FRE debug panel.
    #[derive(Resource, Default)]
    struct FREPanelState {
        window_entity: Option<Entity>,
        camera_entity: Option<Entity>,
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
        Rules,
        EventHistory,
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
                    track_fact_events_system,
                ),
            )
            .add_systems(FREPanelContextPass, fre_panel_ui_system);
    }

    /// System to handle F7 hotkey for opening/closing the FRE panel.
    fn handle_fre_panel_hotkeys_system(
        keyboard_input: Res<ButtonInput<KeyCode>>,
        mut state: ResMut<FREPanelState>,
        mut commands: Commands,
    ) {
        if !keyboard_input.just_pressed(KeyCode::F7) {
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
            ))
            .id();

        state.window_entity = Some(window_entity);
        state.camera_entity = Some(camera_entity);
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
                info!("FRE Debug Panel closed");
                break;
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
            });

            ui.separator();

            // Content based on selected tab
            match current_tab {
                FREPanelTab::Facts => render_facts_tab(ui, world),
                FREPanelTab::Rules => render_rules_tab(ui, world),
                FREPanelTab::EventHistory => render_events_tab(ui, world),
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
    fn render_rules_tab(ui: &mut egui::Ui, _world: &mut World) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.label("📜 Registered Rules");
            ui.separator();

            // Get all registered rules by checking if the registry has any
            // Note: RuleRegistry doesn't expose an iterator, so we show stats
            ui.label("Rules are managed through .rules.ron files.");
            ui.label("Load rules via FRE asset system.");

            ui.separator();

            // Show some helpful info
            egui::CollapsingHeader::new("ℹ️ How Rules Work").show(ui, |ui| {
                ui.label("• Rules are defined in .rules.ron files");
                ui.label("• Each rule has: trigger, condition, modifications, actions, outputs");
                ui.label("• Rules are evaluated when their trigger event fires");
                ui.label("• Conditions use facts from the LayeredFactDatabase");
            });

            // Note: To properly display rules, we would need to add an iter() method to RuleRegistry
            // For now, this serves as a placeholder
            ui.label("");
            ui.label("(Rule inspection requires RuleRegistry.iter() - to be added)");
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
}
