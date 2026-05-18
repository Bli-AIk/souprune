//! # view_taffy_layout
//!
//! Minimal manual-acceptance harness for loading a View layout that exercises
//! the staged Taffy style fields.
//!
//! 用于手工验收 View 布局的最小示例，加载覆盖阶段性 Taffy 样式字段的
//! `.view.ron` 资产。
//!
//! ## Usage
//!
//! ## 运行方式
//!
//! ```bash
//! cargo run -p souprune --example view_taffy_layout
//! ```

use std::path::PathBuf;

use bevy::asset::UnapprovedPathMode;
use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy_fact_rule_event::FREPlugin;
use souprune::core::camera::MainGameCamera;
use souprune::core::game_action::GameActionDef;
use souprune::core::view::{CoreViewPlugin, SpawnViewRequest};

const VIEW_PATH: &str = "view/taffy_minimal.view.ron";

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
                    title: "SoupRune View Taffy Layout".into(),
                    resolution: WindowResolution::new(640, 480),
                    resizable: false,
                    ..default()
                }),
                ..default()
            }),
    );

    app.add_plugins((
        souprune::get_file_importer_plugins(),
        souprune::get_third_plugins(),
        FREPlugin::<GameActionDef>::default(),
        souprune::core::CorePlugin,
        CoreViewPlugin,
    ));

    souprune::init_game_state(&mut app);
    souprune::insert_input_resources(&mut app);

    app.insert_resource(ClearColor(Color::BLACK));
    app.add_systems(Startup, setup);
    app.run();
}

fn setup(
    mut commands: Commands,
    mut next_state: ResMut<NextState<souprune::app_state::AppState>>,
    mut spawn_writer: MessageWriter<SpawnViewRequest>,
) {
    commands.spawn((
        Name::new("View Taffy Layout Camera"),
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: bevy::camera::ScalingMode::Fixed {
                width: 640.0,
                height: 480.0,
            },
            ..OrthographicProjection::default_2d()
        }),
        Transform::default(),
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
