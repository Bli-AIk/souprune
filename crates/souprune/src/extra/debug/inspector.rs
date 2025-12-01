#[cfg(feature = "debug")]
pub mod debug_inspector {
    use bevy::app::App;
    use bevy::camera::RenderTarget;
    use bevy::ecs::schedule::ScheduleLabel;
    use bevy::prelude::*;
    use bevy::window::{Window, WindowClosed, WindowRef, WindowResolution};
    use bevy_inspector_egui::bevy_egui::{EguiContext, EguiMultipassSchedule, EguiPlugin};
    use bevy_inspector_egui::bevy_inspector;
    use bevy_inspector_egui::egui;
    use bevy_inspector_egui::quick::WorldInspectorPlugin;
    use bevy_tween::interpolate::Interpolator;
    use bevy_tween::prelude::*;
    use iyes_perf_ui::prelude::*;
    use std::time::Duration;

    const F1_DOUBLE_PRESS_THRESHOLD: f32 = 0.3;

    #[derive(Component)]
    struct StandaloneInspectorWindow;

    #[derive(Component)]
    struct StandaloneInspectorCamera;

    #[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
    struct InspectorWindowContextPass;

    #[derive(Resource, Default)]
    pub(in crate::extra::debug) struct InspectorUiState {
        overlay_enabled: bool,
        last_f1_press: Option<f32>,
        inspector_window: Option<Entity>,
        inspector_camera: Option<Entity>,
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

        app.add_plugins((
            EguiPlugin::default(),
            WorldInspectorPlugin::default().run_if(inspector_overlay_is_active),
        ));

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

        app.add_systems(Startup, setup_debug_help_text);
        app.add_systems(
            Update,
            (
                handle_inspector_hotkeys_system,
                inspector_window_closed_system,
                toggle_perf_ui_system.before(iyes_perf_ui::PerfUiSet::Setup),
                toggle_debug_help_text_system,
                fade_debug_help_text_system,
                handle_fade_out_complete_system,
            ),
        );
        app.add_systems(InspectorWindowContextPass, inspector_window_ui_system);
    }

    fn setup_debug_help_text(mut commands: Commands) {
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
                    "Cycle UI layout: [F5]",
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
        time: Res<Time>,
        keyboard_input: Res<ButtonInput<KeyCode>>,
        mut ui_state: ResMut<InspectorUiState>,
        mut commands: Commands,
    ) {
        if !keyboard_input.just_pressed(KeyCode::F1) {
            return;
        }

        if ui_state.inspector_window.is_some() {
            ui_state.last_f1_press = None;
            return;
        }

        let now = time.elapsed_secs();
        if let Some(last_press) = ui_state.last_f1_press {
            if now - last_press <= F1_DOUBLE_PRESS_THRESHOLD {
                ui_state.last_f1_press = None;

                if ui_state.overlay_enabled {
                    ui_state.overlay_enabled = false;
                    info!("Inspector overlay: OFF");
                }

                spawn_inspector_window(&mut commands, &mut ui_state);
                return;
            }
        }

        ui_state.last_f1_press = Some(now);
        ui_state.overlay_enabled = !ui_state.overlay_enabled;

        if ui_state.overlay_enabled {
            info!("Inspector overlay: ON");
        } else {
            info!("Inspector overlay: OFF");
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
        info!("Standalone inspector window opened");
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
                ui_state.overlay_enabled = false;
                ui_state.last_f1_press = None;
                info!("Standalone inspector window closed");
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

    fn inspector_overlay_is_active(ui_state: Option<Res<InspectorUiState>>) -> bool {
        ui_state
            .map(|state| state.overlay_enabled && state.inspector_window.is_none())
            .unwrap_or(false)
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
