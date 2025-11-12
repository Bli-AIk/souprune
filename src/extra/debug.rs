use bevy::app::{App, Plugin};

pub struct InspectorPlugin;

impl Plugin for InspectorPlugin {
    fn build(&self, _app: &mut App) {
        #[cfg(feature = "debug")]
        debug_inspector::setup_debug_features(_app);
    }
}

#[cfg(feature = "debug")]
mod debug_inspector {
    use bevy::app::App;
    use bevy::input::common_conditions::input_toggle_active;
    use bevy::prelude::*;
    use bevy_inspector_egui::bevy_egui::EguiPlugin;
    use bevy_inspector_egui::quick::WorldInspectorPlugin;
    use bevy_tween::interpolate::Interpolator;
    use bevy_tween::prelude::*;
    use iyes_perf_ui::prelude::*;
    use std::time::Duration;

    #[derive(Component)]
    pub(super) struct DebugHelpText {
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

    pub(super) fn setup_debug_features(app: &mut App) {
        app.add_plugins((
            EguiPlugin::default(),
            WorldInspectorPlugin::default().run_if(input_toggle_active(false, KeyCode::F1)),
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
                toggle_perf_ui.before(iyes_perf_ui::PerfUiSet::Setup),
                toggle_debug_help_text,
                fade_debug_help_text,
                handle_fade_out_complete,
            ),
        );
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

    fn toggle_perf_ui(
        mut commands: Commands,
        q_perf_ui: Query<Entity, With<PerfUiRoot>>,
        keyboard_input: Res<ButtonInput<KeyCode>>,
    ) {
        if keyboard_input.just_pressed(KeyCode::F2) {
            if let Ok(e) = q_perf_ui.single() {
                commands.entity(e).despawn();
            } else {
                commands.spawn(PerfUiAllEntries::default());
            }
        }
    }

    fn toggle_debug_help_text(
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
            } else {
                debug_help.fade_out_started = true;
                fade_out_text(&mut commands, &debug_help.text_entities);
            }
        }
    }

    fn fade_debug_help_text(
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

    fn handle_fade_out_complete(
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
