//! Manual-acceptance harness for the View layout observer.
//!
//! View 布局观察器的手工验收示例。
//!
//! ```bash
//! cargo run -p souprune --example view_layout_observer --features debug
//! ```

use std::path::PathBuf;

use bevy::asset::UnapprovedPathMode;
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::prelude::*;
use bevy::window::WindowResolution;
use souprune::core::camera::MainGameCamera;
use souprune::core::view::SpawnViewRequest;

const VIEW_PATH: &str = "view/layout_observer_demo.view.ron";

fn main() {
    let asset_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/assets");

    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(bevy::image::ImagePlugin::default_nearest())
            .set(bevy::asset::AssetPlugin {
                file_path: asset_root.to_string_lossy().into_owned(),
                unapproved_path_mode: UnapprovedPathMode::Allow,
                ..default()
            })
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "SoupRune View Layout Observer".into(),
                    resolution: WindowResolution::new(960, 540),
                    resizable: false,
                    ..default()
                }),
                ..default()
            }),
    );

    app.add_plugins((
        souprune::get_file_importer_plugins(),
        souprune::get_third_plugins(),
        souprune::core::CorePlugin,
    ));

    #[cfg(feature = "debug")]
    {
        app.add_plugins(bevy_brp_extras::BrpExtrasPlugin);
        app.add_plugins(souprune::extra::debug::DebugPlugin);
    }

    app.insert_resource(souprune::config::load_config());
    souprune::init_game_state(&mut app);
    souprune::insert_input_resources(&mut app);
    app.insert_resource(ClearColor(Color::srgb(0.02, 0.03, 0.05)));
    app.add_systems(Startup, setup);
    app.run();
}

fn setup(
    mut commands: Commands,
    mut next_state: ResMut<NextState<souprune::app_state::AppState>>,
    mut spawn_writer: MessageWriter<SpawnViewRequest>,
) {
    commands.spawn((
        Name::new("View Layout Observer Camera"),
        Camera3d::default(),
        Tonemapping::None,
        Transform::from_xyz(0.0, 0.25, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        MainGameCamera,
    ));

    next_state.set(souprune::app_state::AppState::Running);
    spawn_writer.write(SpawnViewRequest {
        path: VIEW_PATH.to_string(),
        mode_scope: None,
        pre_spawn_events: Vec::new(),
        bindings: None,
    });
}
