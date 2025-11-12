use bevy::app::{App, Plugin};

#[cfg(feature = "debug")]
use bevy::prelude::*;

#[cfg(feature = "debug")]
use std::time::Duration;

#[cfg(feature = "debug")]
use bevy::input::common_conditions::input_toggle_active;

#[cfg(feature = "debug")]
use bevy_inspector_egui::bevy_egui::EguiPlugin;

#[cfg(feature = "debug")]
use bevy_inspector_egui::quick::WorldInspectorPlugin;

#[cfg(feature = "debug")]
use iyes_perf_ui::prelude::*;

#[cfg(feature = "debug")]
#[derive(Component)]
struct DebugHelpText {
    timer: Timer,
    visible: bool,
}

pub struct InspectorPlugin;

impl Plugin for InspectorPlugin {
    fn build(&self, _app: &mut App) {
        #[cfg(feature = "debug")]
        {
            _app.add_plugins((
                EguiPlugin::default(),
                WorldInspectorPlugin::default().run_if(input_toggle_active(false, KeyCode::F1)),
            ));

            _app.add_plugins((
                bevy::diagnostic::FrameTimeDiagnosticsPlugin::default(),
                bevy::diagnostic::EntityCountDiagnosticsPlugin::default(),
                bevy::diagnostic::SystemInformationDiagnosticsPlugin,
                bevy::render::diagnostic::RenderDiagnosticsPlugin,
                PerfUiPlugin,
            ));

            _app.add_systems(Startup, setup_debug_help_text);
            _app.add_systems(
                Update,
                (
                    toggle_perf_ui.before(iyes_perf_ui::PerfUiSet::Setup),
                    toggle_debug_help_text,
                    fade_debug_help_text,
                ),
            );
        }
    }
}

#[cfg(feature = "debug")]
fn setup_debug_help_text(mut commands: Commands) {
    commands
        .spawn((
            Node {
                display: Display::Flex,
                position_type: PositionType::Absolute,
                bottom: Val::Px(10.),
                left: Val::Px(10.),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            DebugHelpText {
                timer: Timer::new(Duration::from_secs(3), TimerMode::Once),
                visible: true,
            },
        ))
        .with_children(|builder| {
            builder.spawn(Text::new("Debug: "));
            builder.spawn(Text::new("Inspector: [F1]"));
            builder.spawn(Text::new("Performance monitoring: [F2]"));
            builder.spawn(Text::new("Toggle debug help: [F12]"));
        });
}

#[cfg(feature = "debug")]
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

#[cfg(feature = "debug")]
fn toggle_debug_help_text(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut q_debug_text: Query<(&mut DebugHelpText, &mut Node)>,
) {
    if keyboard_input.just_pressed(KeyCode::F12) {
        if let Ok((mut debug_help, mut style)) = q_debug_text.single_mut() {
            debug_help.visible = !debug_help.visible;
            style.display = if debug_help.visible {
                Display::Flex
            } else {
                Display::None
            };

            // 重置计时器，重新开始淡出倒计时
            if debug_help.visible {
                debug_help.timer.reset();
            }
        }
    }
}

#[cfg(feature = "debug")]
fn fade_debug_help_text(time: Res<Time>, mut q_debug_text: Query<(&mut DebugHelpText, &mut Node)>) {
    if let Ok((mut debug_help, mut style)) = q_debug_text.single_mut() {
        if debug_help.visible {
            debug_help.timer.tick(time.delta());

            if debug_help.timer.is_finished() {
                debug_help.visible = false;
                style.display = Display::None;
            }
        }
    }
}
