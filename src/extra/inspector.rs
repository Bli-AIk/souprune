use bevy::app::{App, Plugin};
use bevy::input::common_conditions::input_toggle_active;
use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

#[cfg(debug_assertions)]
use iyes_perf_ui::prelude::*;

pub struct InspectorPlugin;

impl Plugin for InspectorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            EguiPlugin::default(),
            WorldInspectorPlugin::default().run_if(input_toggle_active(false, KeyCode::F1)),
        ));

        #[cfg(debug_assertions)]
        {
            app.add_plugins((
                bevy::diagnostic::FrameTimeDiagnosticsPlugin::default(),
                bevy::diagnostic::EntityCountDiagnosticsPlugin::default(),
                bevy::diagnostic::SystemInformationDiagnosticsPlugin,
                bevy::render::diagnostic::RenderDiagnosticsPlugin,
                PerfUiPlugin,
            ));

            app.add_systems(Startup, setup_debug_help_text);
            app.add_systems(
                Update,
                toggle_perf_ui.before(iyes_perf_ui::PerfUiSet::Setup),
            );
        }
    }
}

#[cfg(debug_assertions)]
fn setup_debug_help_text(mut commands: Commands) {
    commands
        .spawn(Node {
            display: Display::Flex,
            position_type: PositionType::Absolute,
            top: Val::Px(10.),
            right: Val::Px(10.),
            flex_direction: FlexDirection::Column,
            ..default()
        })
        .with_children(|builder| {
            builder.spawn(Text::new("Debug: "));
            builder.spawn(Text::new("Inspector: [F1]"));
            builder.spawn(Text::new("Performance monitoring: [F2]"));
        });
}

#[cfg(debug_assertions)]
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
