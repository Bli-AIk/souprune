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
    use bevy::ecs::schedule::ScheduleLabel;
    use bevy::prelude::*;
    use bevy::window::{Window, WindowClosed, WindowFocused, WindowRef, WindowResolution};
    use bevy_inspector_egui::bevy_egui::{EguiContext, EguiMultipassSchedule, EguiPlugin};
    use bevy_inspector_egui::{DefaultInspectorConfigPlugin, bevy_inspector, egui};
    use bevy_tween::interpolate::Interpolator;
    use bevy_tween::prelude::*;
    use iyes_perf_ui::prelude::*;
    use leafwing_input_manager::action_state::ActionState;
    use leafwing_input_manager::plugin::InputManagerSystem;
    use std::time::Duration;

    #[derive(Component)]
    struct StandaloneInspectorWindow;

    #[derive(Component)]
    struct StandaloneInspectorCamera;

    #[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
    struct InspectorWindowContextPass;

    #[derive(Resource, Default)]
    pub(in crate::extra::debug) struct InspectorUiState {
        inspector_window: Option<Entity>,
        inspector_camera: Option<Entity>,
        window_focused: bool,
    }

    #[derive(Component)]
    pub(in crate::extra::debug) struct DebugHelpText {
        timer: Timer,
        visible: bool,
        text_entities: Vec<Entity>,
        fade_out_started: bool,
    }

    #[derive(Debug, Clone)]
    struct TextColorInterpolator {
        start: Color,
        end: Color,
    }

    impl Interpolator for TextColorInterpolator {
        type Item = TextColor;

        fn interpolate(&self, item: &mut Self::Item, value: f32, _previous_value: f32) {
            item.0 = self.start.mix(&self.end, value);
        }
    }

    fn text_color_interpolator(start: Color, end: Color) -> TextColorInterpolator {
        TextColorInterpolator { start, end }
    }

    pub(in crate::extra::debug) fn setup_debug_features(app: &mut App) {
        app.init_resource::<InspectorUiState>();

        app.add_plugins(EguiPlugin::default());
        app.add_plugins(DefaultInspectorConfigPlugin);

        app.add_plugins((
            bevy::diagnostic::FrameTimeDiagnosticsPlugin::default(),
            bevy::diagnostic::EntityCountDiagnosticsPlugin::default(),
            bevy::diagnostic::SystemInformationDiagnosticsPlugin,
            bevy::render::diagnostic::RenderDiagnosticsPlugin,
            PerfUiPlugin,
            DefaultTweenPlugins,
        ));

        app.add_tween_systems(bevy_tween::tween::component_tween_system::<
            TextColorInterpolator,
        >());

        app.add_systems(Startup, setup_debug_help_text_system);
        app.add_systems(
            Update,
            (
                handle_inspector_hotkeys_system,
                inspector_window_closed_system,
                inspector_window_focus_system,
                toggle_perf_ui_system.before(iyes_perf_ui::PerfUiSet::Setup),
                toggle_debug_help_text_system,
                fade_debug_help_text_system,
                handle_fade_out_complete_system,
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
            .spawn((Node {
                display: Display::Flex,
                position_type: PositionType::Absolute,
                bottom: Val::Px(10.),
                left: Val::Px(10.),
                flex_direction: FlexDirection::Column,
                ..default()
            },))
            .with_children(|builder| {
                let texts = [
                    "You are now running the game in Debug feature: ",
                    "Inspector: [F1]",
                    "Performance monitoring: [F2]",
                    "Show colliders: [F3]",
                    "Debug image overlay: [F4]",
                    "Cycle Player HP (1/Half/Full): [F5]",
                    "Toggle debug help: [F12]",
                ];

                for text in texts {
                    let text_entity = builder
                        .spawn((Text::new(text), TextColor(Color::srgba(1.0, 1.0, 1.0, 0.0))))
                        .id();
                    text_entities.push(text_entity);
                }
            })
            .id();

        commands.entity(debug_entity).insert(DebugHelpText {
            timer: Timer::new(Duration::from_secs(3), TimerMode::Once),
            visible: true,
            text_entities: text_entities.clone(),
            fade_out_started: false,
        });

        fade_in_text(&mut commands, &text_entities);
    }

    fn toggle_perf_ui_system(
        mut commands: Commands,
        q_perf_ui: Query<Entity, With<PerfUiRoot>>,
        keyboard_input: Res<ButtonInput<KeyCode>>,
    ) {
        if keyboard_input.just_pressed(KeyCode::F2) {
            let message = if let Ok(e) = q_perf_ui.single() {
                commands.entity(e).despawn();
                "OFF"
            } else {
                commands.spawn(PerfUiAllEntries::default());
                "ON"
            };
            info!("Performance monitoring: {}", message);
        }
    }

    fn toggle_debug_help_text_system(
        keyboard_input: Res<ButtonInput<KeyCode>>,
        mut commands: Commands,
        mut q_debug_text: Query<(&mut DebugHelpText, &mut Node)>,
    ) {
        if keyboard_input.just_pressed(KeyCode::F12)
            && let Ok((mut debug_help, mut style)) = q_debug_text.single_mut()
        {
            debug_help.visible = !debug_help.visible;

            if debug_help.visible {
                style.display = Display::Flex;
                debug_help.timer.reset();
                debug_help.fade_out_started = false;
                fade_in_text(&mut commands, &debug_help.text_entities);
                info!("Debug help text: ON");
            } else {
                debug_help.fade_out_started = true;
                fade_out_text(&mut commands, &debug_help.text_entities);
                info!("Debug help text: OFF");
            }
        }
    }

    fn fade_debug_help_text_system(
        time: Res<Time>,
        mut commands: Commands,
        mut q_debug_text: Query<(&mut DebugHelpText, &mut Node)>,
    ) {
        if let Ok((mut debug_help, _style)) = q_debug_text.single_mut()
            && debug_help.visible
            && !debug_help.fade_out_started
        {
            debug_help.timer.tick(time.delta());

            if debug_help.timer.is_finished() {
                debug_help.visible = false;
                debug_help.fade_out_started = true;

                fade_out_text(&mut commands, &debug_help.text_entities);
            }
        }
    }

    fn fade_in_text(commands: &mut Commands, text_entities: &[Entity]) {
        for &entity in text_entities {
            commands.entity(entity).animation().insert_tween_here(
                Duration::from_millis(400),
                EaseKind::QuadraticOut,
                entity.into_target().with(text_color_interpolator(
                    Color::srgba(1.0, 1.0, 1.0, 0.0),
                    Color::srgba(1.0, 1.0, 1.0, 1.0),
                )),
            );
        }
    }

    fn fade_out_text(commands: &mut Commands, text_entities: &[Entity]) {
        for &entity in text_entities {
            commands.entity(entity).animation().insert_tween_here(
                Duration::from_millis(400),
                EaseKind::QuadraticIn,
                entity.into_target().with(text_color_interpolator(
                    Color::srgba(1.0, 1.0, 1.0, 1.0),
                    Color::srgba(1.0, 1.0, 1.0, 0.0),
                )),
            );
        }
    }

    fn handle_inspector_hotkeys_system(
        keyboard_input: Res<ButtonInput<KeyCode>>,
        mut ui_state: ResMut<InspectorUiState>,
        mut commands: Commands,
    ) {
        if !keyboard_input.just_pressed(KeyCode::F1) {
            return;
        }

        if ui_state.inspector_window.is_some() {
            close_inspector_window(&mut commands, &mut ui_state);
        } else {
            spawn_inspector_window(&mut commands, &mut ui_state);
        }
    }

    fn spawn_inspector_window(commands: &mut Commands, ui_state: &mut InspectorUiState) {
        if ui_state.inspector_window.is_some() {
            return;
        }

        let window_entity = commands
            .spawn((
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
                Camera3d::default(),
                Transform::from_xyz(0.0, 0.0, 1.0),
                Camera {
                    target: RenderTarget::Window(WindowRef::Entity(window_entity)),
                    ..default()
                },
                EguiMultipassSchedule::new(InspectorWindowContextPass),
                StandaloneInspectorCamera,
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
            if event.window == window_entity {
                ui_state.inspector_window = None;
                if let Some(camera_entity) = ui_state.inspector_camera.take() {
                    commands.entity(camera_entity).despawn();
                }
                ui_state.window_focused = false;
                info!("Standalone inspector window closed");
                break;
            }
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

    fn inspector_window_ui_system(world: &mut World) {
        let inspector_camera = world
            .get_resource::<InspectorUiState>()
            .and_then(|state| state.inspector_camera);
        let Some(camera_entity) = inspector_camera else {
            return;
        };

        let mut contexts =
            world.query_filtered::<&mut EguiContext, With<StandaloneInspectorCamera>>();
        let mut egui_context = match contexts.get_mut(world, camera_entity) {
            Ok(ctx) => {
                // Clone so we can drop the world borrow before running the UI, mirroring the quick plugin.
                ctx.clone()
            }
            Err(_) => return,
        };

        egui::CentralPanel::default().show(egui_context.get_mut(), |ui| {
            egui::ScrollArea::both().show(ui, |ui| {
                bevy_inspector::ui_for_world(world, ui);
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
            if should_disable {
                if !action_state.disabled() {
                    action_state.disable();
                }
            } else if action_state.disabled() {
                action_state.enable();
            }
        }
    }

    fn handle_fade_out_complete_system(
        _time: Res<Time>,
        mut q_debug_text: Query<(&mut DebugHelpText, &mut Node)>,
        q_text_colors: Query<&TextColor>,
    ) {
        if let Ok((mut debug_help, mut style)) = q_debug_text.single_mut()
            && debug_help.fade_out_started
        {
            let all_transparent = debug_help.text_entities.iter().all(|&entity| {
                if let Ok(text_color) = q_text_colors.get(entity) {
                    text_color.0.alpha() < 0.01
                } else {
                    true
                }
            });

            if all_transparent {
                style.display = Display::None;
                debug_help.fade_out_started = false;
            }
        }
    }
}
