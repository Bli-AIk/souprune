//! SoupRune Editor 独立入口。

use bevy::prelude::*;
use bevy_workbench::console::console_log_layer;
use souprune_editor::SoupRuneEditorPlugin;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "SoupRune Editor".to_string(),
                        resolution: (1280u32, 720u32).into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(bevy::log::LogPlugin {
                    custom_layer: console_log_layer,
                    ..default()
                }),
        )
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins(SoupRuneEditorPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}
