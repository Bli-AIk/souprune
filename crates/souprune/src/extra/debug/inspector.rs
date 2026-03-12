//! # inspector.rs
//!
//! # inspector.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Sets up the integrated `bevy-inspector-egui` for debugging, including a standalone inspector window, performance UI, and debug help text overlay.
//!
//! 设置集成的 `bevy-inspector-egui` 以进行调试，包括独立的检查器窗口、性能 UI 和调试帮助文本覆盖层。

#[cfg(feature = "debug")]
pub mod debug_inspector {
    use crate::app_state::overworld::character::components::PlayerControlled;
    use crate::core::input::Action;
    use bevy::app::App;
    use bevy::camera::RenderTarget;

    use bevy::ecs::message::MessageReader;
    use bevy::ecs::schedule::ScheduleLabel;
    use bevy::ecs::system::SystemIdMarker;
    use bevy::prelude::*;
    use bevy::window::{
        PrimaryWindow, Window, WindowClosed, WindowFocused, WindowRef, WindowResolution,
    };
    use bevy_inspector_egui::bevy_egui::{EguiContext, EguiMultipassSchedule, EguiPlugin};
    use bevy_inspector_egui::{DefaultInspectorConfigPlugin, bevy_inspector, egui};
    use iyes_perf_ui::prelude::*;
    use leafwing_input_manager::action_state::ActionState;
    use leafwing_input_manager::plugin::InputManagerSystem;
    use std::marker::PhantomData;
    use std::time::Duration;

    #[derive(Component)]
    struct StandaloneInspectorWindow;

    #[derive(Component)]
    struct StandaloneInspectorCamera;

    #[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
    struct InspectorWindowContextPass;

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

    #[derive(Resource, Default)]
    pub(in crate::extra::debug) struct InspectorUiState {
        inspector_window: Option<Entity>,
        inspector_camera: Option<Entity>,
        window_focused: bool,
        /// Two-phase refresh state for state change handling.
        refresh_phase: RefreshPhase,
        /// Whether to show all entities including BRP/system internals.
        /// 是否显示所有实体，包括 BRP/系统内部实体。
        show_all_entities: bool,
        /// Search filter for entity names.
        /// 实体名称搜索过滤器。
        search_query: String,
    }

    /// Custom entity filter that excludes BRP system entities by default.
    /// 自定义实体过滤器，默认排除 BRP 系统实体。
    struct BrpEntityFilter {
        show_all: bool,
        /// Search query for filtering by entity name.
        /// 用于按实体名称过滤的搜索查询。
        search_query: String,
        _marker: PhantomData<Without<ChildOf>>,
    }

    impl BrpEntityFilter {
        fn new(show_all: bool, search_query: &str) -> Self {
            Self {
                show_all,
                search_query: search_query.to_lowercase(),
                _marker: PhantomData,
            }
        }
    }

    impl bevy_inspector::EntityFilter for BrpEntityFilter {
        type StaticFilter = Without<ChildOf>;

        fn is_active(&self) -> bool {
            !self.show_all || !self.search_query.is_empty()
        }

        fn filter_entity(&self, world: &mut World, entity: Entity) -> bool {
            // Filter out BRP system entities and observers
            if !self.show_all
                && (world.get::<SystemIdMarker>(entity).is_some()
                    || world.get::<bevy::ecs::observer::Observer>(entity).is_some())
            {
                return false;
            }

            // Apply search filter if provided
            if self.search_query.is_empty() {
                return true;
            }

            world
                .get::<Name>(entity)
                .is_some_and(|name| name.to_lowercase().contains(&self.search_query))
        }
    }

    #[derive(Component)]
    pub(in crate::extra::debug) struct DebugHelpText {
        timer: Timer,
        visible: bool,
        text_entities: Vec<Entity>,
        fade_out_started: bool,
    }

    /// Toast notification shown when debug features are toggled.
    #[derive(Component)]
    struct DebugToast {
        timer: Timer,
        fade_out_started: bool,
    }

    pub(in crate::extra::debug) fn setup_debug_features(app: &mut App) {
        app.init_resource::<InspectorUiState>();

        app.add_plugins(EguiPlugin::default());
        app.add_plugins(DefaultInspectorConfigPlugin);

        app.add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default());
        app.add_plugins(bevy::diagnostic::EntityCountDiagnosticsPlugin::default());
        app.add_plugins(bevy::diagnostic::SystemInformationDiagnosticsPlugin);

        // Only add RenderDiagnosticsPlugin if not already added (e.g., by trace_tracy)
        if !app.is_plugin_added::<bevy::render::diagnostic::RenderDiagnosticsPlugin>() {
            app.add_plugins(bevy::render::diagnostic::RenderDiagnosticsPlugin);
        }

        app.add_plugins(PerfUiPlugin);

        app.add_systems(
            Startup,
            (setup_debug_help_text_system, setup_debug_toast_system),
        );

        app.add_systems(
            Update,
            (
                handle_inspector_hotkeys_system,
                inspector_window_closed_system,
                inspector_window_focus_system,
                primary_window_closed_system,
                app_state_changed_refresh_inspector_system,
                inspector_refresh_system,
                toggle_perf_ui_system.before(iyes_perf_ui::PerfUiSet::Setup),
                toggle_debug_help_text_system,
                fade_debug_help_text_system,
                handle_debug_toast_event_system,
                fade_debug_toast_system,
            ),
        );
        app.add_systems(InspectorWindowContextPass, inspector_window_ui_system);
        app.add_systems(
            PreUpdate,
            block_player_actions_when_inspector_focused_system
                .after(InputManagerSystem::ManualControl),
        );
    }

    fn setup_debug_help_text_system(mut commands: Commands) {
        let mut text_entities = Vec::new();

        let debug_entity = commands
            .spawn((
                Node {
                    display: Display::Flex,
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(10.),
                    left: Val::Px(10.),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(6.)),
                    ..default()
                },
                GlobalZIndex(i32::MAX - 1),
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
            ))
            .with_children(|builder| {
                let texts = [
                    "Debug mode active: ",
                    "Inspector Window: [F1]",
                    "FRE Debug Panel: [F2]",
                    "Performance UI: [F3]",
                    "Collider Gizmos: [F4]",
                    "Image Overlay: [F5]",
                    "Battle Test: [F6]",
                    "Game Freeze: [F7]",
                    "Debug Camera: [F8]",
                    "Toggle this help: [F12]",
                ];

                for text in texts {
                    let text_entity = builder
                        .spawn((
                            Text::new(text),
                            TextColor(Color::WHITE),
                            TextFont::from_font_size(14.0),
                        ))
                        .id();
                    text_entities.push(text_entity);
                }
            })
            .id();

        eprintln!("[DEBUG] setup_debug_help_text_system: spawned root={debug_entity:?}");

        commands.entity(debug_entity).insert(DebugHelpText {
            timer: Timer::new(Duration::from_secs(3), TimerMode::Once),
            visible: true,
            text_entities: text_entities.clone(),
            fade_out_started: false,
        });
    }

    fn toggle_perf_ui_system(
        mut commands: Commands,
        q_perf_ui: Query<Entity, With<PerfUiRoot>>,
        keyboard_input: Res<ButtonInput<KeyCode>>,
        mut toast_events: MessageWriter<super::super::DebugToastEvent>,
    ) {
        if keyboard_input.just_pressed(KeyCode::F3) {
            let message = if let Ok(e) = q_perf_ui.single() {
                commands.entity(e).despawn();
                "Performance UI: OFF"
            } else {
                commands.spawn(PerfUiAllEntries::default());
                "Performance UI: ON"
            };
            info!("{}", message);
            toast_events.write(super::super::DebugToastEvent {
                message: message.into(),
            });
        }
    }

    fn toggle_debug_help_text_system(
        keyboard_input: Res<ButtonInput<KeyCode>>,
        mut q_debug_text: Query<(&mut DebugHelpText, &mut Node)>,
        mut q_text_colors: Query<&mut TextColor>,
    ) {
        if keyboard_input.just_pressed(KeyCode::F12)
            && let Ok((mut debug_help, mut style)) = q_debug_text.single_mut()
        {
            debug_help.visible = !debug_help.visible;

            if debug_help.visible {
                style.display = Display::Flex;
                debug_help.timer.reset();
                debug_help.fade_out_started = false;
                for &entity in &debug_help.text_entities {
                    if let Ok(mut tc) = q_text_colors.get_mut(entity) {
                        tc.0 = Color::WHITE;
                    }
                }
                info!("Debug help text: ON");
            } else {
                for &entity in &debug_help.text_entities {
                    if let Ok(mut tc) = q_text_colors.get_mut(entity) {
                        tc.0 = Color::srgba(1.0, 1.0, 1.0, 0.0);
                    }
                }
                style.display = Display::None;
                debug_help.fade_out_started = false;
                info!("Debug help text: OFF");
            }
        }
    }

    fn fade_debug_help_text_system(
        time: Res<Time>,
        mut q_debug_text: Query<(&mut DebugHelpText, &mut Node)>,
        mut q_text_colors: Query<&mut TextColor>,
    ) {
        if let Ok((mut debug_help, mut style)) = q_debug_text.single_mut()
            && debug_help.visible
            && !debug_help.fade_out_started
        {
            debug_help.timer.tick(time.delta());

            if debug_help.timer.is_finished() {
                debug_help.visible = false;
                debug_help.fade_out_started = false;
                for &entity in &debug_help.text_entities {
                    if let Ok(mut tc) = q_text_colors.get_mut(entity) {
                        tc.0 = Color::srgba(1.0, 1.0, 1.0, 0.0);
                    }
                }
                style.display = Display::None;
            }
        }
    }

    fn handle_inspector_hotkeys_system(
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

    fn inspector_window_closed_system(
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
    /// Uses RemovedComponents to detect when PrimaryWindow component is removed.
    fn primary_window_closed_system(
        mut commands: Commands,
        mut ui_state: ResMut<InspectorUiState>,
        mut removed: RemovedComponents<PrimaryWindow>,
    ) {
        // If PrimaryWindow component was removed from any entity, close inspector
        if removed.read().next().is_some() && ui_state.inspector_window.is_some() {
            close_inspector_window(&mut commands, &mut ui_state);
            info!("Standalone inspector window closed (primary window closed)");
        }
    }

    fn inspector_window_focus_system(
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
    fn app_state_changed_refresh_inspector_system(
        mut ui_state: ResMut<InspectorUiState>,
        app_state: Res<State<crate::app_state::AppState>>,
    ) {
        // Only trigger refresh if inspector window is open and state just changed
        if ui_state.inspector_window.is_some()
            && app_state.is_changed()
            && ui_state.refresh_phase == RefreshPhase::None
        {
            ui_state.refresh_phase = RefreshPhase::CloseWindow;
            info!("AppState changed, scheduling inspector window refresh (phase 1: close)");
        }
    }

    /// System to perform inspector window refresh in two phases.
    /// Phase 1: Close the window.
    /// Phase 2: Reopen the window.
    fn inspector_refresh_system(mut commands: Commands, mut ui_state: ResMut<InspectorUiState>) {
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

    fn render_inspector_search_box(ui: &mut egui::Ui, world: &mut World) {
        ui.label("🔍");
        let mut search_query = world
            .get_resource::<InspectorUiState>()
            .map(|s| s.search_query.clone())
            .unwrap_or_default();

        let response = ui.add(
            egui::TextEdit::singleline(&mut search_query)
                .hint_text("Search entities by name...")
                .desired_width(300.0),
        );

        if response.changed()
            && let Some(mut state) = world.get_resource_mut::<InspectorUiState>()
        {
            state.search_query = search_query.clone();
        }

        if ui.button("✕").clicked()
            && let Some(mut state) = world.get_resource_mut::<InspectorUiState>()
        {
            state.search_query.clear();
        }
    }

    fn render_inspector_entity_filter(ui: &mut egui::Ui, world: &mut World) {
        let mut show_all = world
            .get_resource::<InspectorUiState>()
            .map(|s| s.show_all_entities)
            .unwrap_or(false);

        if ui
            .checkbox(&mut show_all, "Show all entities (BRP/System internals)")
            .changed()
            && let Some(mut state) = world.get_resource_mut::<InspectorUiState>()
        {
            state.show_all_entities = show_all;
        }
    }

    fn inspector_window_ui_system(world: &mut World) {
        // Get inspector state
        let (inspector_camera, show_all_entities) = {
            let state = world.get_resource::<InspectorUiState>();
            (
                state.and_then(|s| s.inspector_camera),
                state.map(|s| s.show_all_entities).unwrap_or(false),
            )
        };
        let Some(camera_entity) = inspector_camera else {
            return;
        };

        let mut contexts =
            world.query_filtered::<&mut EguiContext, With<StandaloneInspectorCamera>>();
        let mut egui_context = match contexts.get_mut(world, camera_entity) {
            Ok(context) => {
                // Clone so we can drop the world borrow before running the UI, mirroring the quick plugin.
                context.clone()
            }
            Err(_) => return,
        };

        // Top panel for controls
        egui::TopBottomPanel::top("inspector_controls").show(egui_context.get_mut(), |ui| {
            ui.horizontal(|ui| render_inspector_search_box(ui, world));
            ui.horizontal(|ui| render_inspector_entity_filter(ui, world));
        });

        // Get search query for filter
        let search_query = world
            .get_resource::<InspectorUiState>()
            .map(|s| s.search_query.clone())
            .unwrap_or_default();

        egui::CentralPanel::default().show(egui_context.get_mut(), |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                // Use filtered entity display
                // 使用过滤后的实体显示
                let filter = BrpEntityFilter::new(show_all_entities, &search_query);
                bevy_inspector::ui_for_entities_filtered(world, ui, true, &filter);
                ui.allocate_space(ui.available_size());
            });
        });
    }

    fn block_player_actions_when_inspector_focused_system(
        ui_state: Option<Res<InspectorUiState>>,
        mut query: Query<&mut ActionState<Action>, With<PlayerControlled>>,
    ) {
        let should_disable = ui_state.map(|state| state.window_focused).unwrap_or(false);

        for mut action_state in query.iter_mut() {
            if should_disable && !action_state.disabled() {
                action_state.disable();
            } else if !should_disable && action_state.disabled() {
                action_state.enable();
            }
        }
    }

    // --- Debug Toast Notification System ---

    fn setup_debug_toast_system(mut commands: Commands) {
        commands
            .spawn((
                Node {
                    display: Display::None,
                    position_type: PositionType::Absolute,
                    top: Val::Px(10.),
                    left: Val::Px(10.),
                    ..default()
                },
                GlobalZIndex(i32::MAX - 1),
                DebugToast {
                    timer: Timer::new(Duration::from_secs(2), TimerMode::Once),
                    fade_out_started: false,
                },
            ))
            .with_children(|builder| {
                builder.spawn((
                    Text::new(""),
                    TextColor(Color::WHITE),
                    TextFont::from_font_size(14.0),
                ));
            });
    }

    fn handle_debug_toast_event_system(
        mut events: MessageReader<super::super::DebugToastEvent>,
        mut q_toast: Query<(&mut DebugToast, &mut Node, &Children)>,
        mut q_text: Query<(&mut Text, &mut TextColor)>,
    ) {
        let Some(event) = events.read().last() else {
            return;
        };

        let Ok((mut toast, mut style, children)) = q_toast.single_mut() else {
            return;
        };

        for child in children.iter() {
            if let Ok((mut text, mut tc)) = q_text.get_mut(child) {
                **text = event.message.clone();
                tc.0 = Color::WHITE;
            }
        }

        style.display = Display::Flex;
        toast.timer.reset();
        toast.fade_out_started = false;
    }

    fn fade_debug_toast_system(
        time: Res<Time>,
        mut q_toast: Query<(&mut DebugToast, &mut Node, &Children)>,
        mut q_text_colors: Query<&mut TextColor>,
    ) {
        let Ok((mut toast, mut style, children)) = q_toast.single_mut() else {
            return;
        };

        if toast.fade_out_started {
            return;
        }

        toast.timer.tick(time.delta());

        if toast.timer.is_finished() {
            toast.fade_out_started = false;
            for child in children.iter() {
                if let Ok(mut tc) = q_text_colors.get_mut(child) {
                    tc.0 = Color::srgba(1.0, 1.0, 1.0, 0.0);
                }
            }
            style.display = Display::None;
        }
    }
}
