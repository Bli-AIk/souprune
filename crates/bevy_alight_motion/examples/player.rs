//! Example player for Alight Motion projects.
//!
//! Controls:
//! - Space: Play/Pause toggle
//! - R: Reset to beginning (keeps current play state)
//! - P: Replay from beginning (resets and plays)
//! - Left/Right: Seek backward/forward by 50ms
//! - Up/Down: Speed up/slow down playback
//! - L: Toggle loop mode

use bevy::prelude::*;
use bevy_alight_motion::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Alight Motion Player".to_string(),
                resolution: (1280, 960).into(),
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        // Black background matching AM project
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins(AlightMotionPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, (handle_input, update_ui, debug_sprites))
        .run();
}

/// UI text component for status display.
#[derive(Component)]
struct StatusText;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Spawn camera
    commands.spawn(Camera2d);

    // Load the AM project from assets folder
    load_am_project(&mut commands, &asset_server, "am/project.amproj");

    // Spawn UI for status display
    commands.spawn((
        Text::new("Loading..."),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
        StatusText,
    ));

    // Instructions - clear English key descriptions
    commands.spawn((
        Text::new("[Space] Play/Pause | [R] Reset | [P] Replay | [Left/Right] Seek | [Up/Down] Speed | [L] Loop"),
        TextFont {
            font_size: 16.0,
            ..default()
        },
        TextColor(Color::srgba(0.8, 0.8, 0.8, 1.0)),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        },
    ));
}

/// Debug system to print sprite info once
fn debug_sprites(query: Query<(&AmLayerMarker, &Transform, &Sprite), Added<Sprite>>) {
    for (marker, transform, sprite) in query.iter() {
        println!(
            "Sprite added: '{}' at ({:.1},{:.1},{:.1}) scale=({:.2},{:.2}) alpha={:.2} size={:?}",
            marker.label,
            transform.translation.x,
            transform.translation.y,
            transform.translation.z,
            transform.scale.x,
            transform.scale.y,
            sprite.color.alpha(),
            sprite.custom_size
        );
    }
}

fn handle_input(keyboard: Res<ButtonInput<KeyCode>>, mut playback: ResMut<AmPlayback>) {
    // Play/Pause toggle
    if keyboard.just_pressed(KeyCode::Space) {
        playback.toggle();
    }

    // Reset (keeps current play/pause state)
    if keyboard.just_pressed(KeyCode::KeyR) {
        playback.reset();
    }

    // Replay (reset and start playing)
    if keyboard.just_pressed(KeyCode::KeyP) {
        playback.reset();
        playback.playing = true;
    }

    // Seek backward/forward by 50ms
    if keyboard.pressed(KeyCode::ArrowLeft) {
        playback.current_time_ms = (playback.current_time_ms - 50.0).max(0.0);
    }
    if keyboard.pressed(KeyCode::ArrowRight) {
        playback.current_time_ms = (playback.current_time_ms + 50.0).min(playback.total_time_ms);
    }

    // Speed control (up = faster, down = slower)
    if keyboard.just_pressed(KeyCode::ArrowUp) {
        playback.speed = (playback.speed * 1.5).min(4.0);
    }
    if keyboard.just_pressed(KeyCode::ArrowDown) {
        playback.speed = (playback.speed / 1.5).max(0.25);
    }

    // Loop mode toggle
    if keyboard.just_pressed(KeyCode::KeyL) {
        playback.looping = !playback.looping;
    }
}

fn update_ui(playback: Res<AmPlayback>, mut query: Query<&mut Text, With<StatusText>>) {
    for mut text in query.iter_mut() {
        let status = if playback.playing {
            "Playing"
        } else {
            "Paused"
        };
        let loop_status = if playback.looping { "Loop" } else { "Once" };

        **text = format!(
            "{} | Time: {:.0}/{:.0}ms | Speed: {:.2}x | {}",
            status, playback.current_time_ms, playback.total_time_ms, playback.speed, loop_status
        );
    }
}
