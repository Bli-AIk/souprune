use super::set_text_entities_color;
use bevy::prelude::*;
use std::time::Duration;

#[derive(Component)]
pub(super) struct DebugHelpText {
    timer: Timer,
    visible: bool,
    text_entities: Vec<Entity>,
    fade_out_started: bool,
}

pub(super) fn setup_debug_help_text_system(mut commands: Commands) {
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
                "Restart Game: [F9]",
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

pub(super) fn toggle_debug_help_text_system(
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
            set_text_entities_color(&debug_help.text_entities, Color::WHITE, &mut q_text_colors);
            info!("Debug help text: ON");
        } else {
            set_text_entities_color(
                &debug_help.text_entities,
                Color::srgba(1.0, 1.0, 1.0, 0.0),
                &mut q_text_colors,
            );
            style.display = Display::None;
            debug_help.fade_out_started = false;
            info!("Debug help text: OFF");
        }
    }
}

pub(super) fn fade_debug_help_text_system(
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
            set_text_entities_color(
                &debug_help.text_entities,
                Color::srgba(1.0, 1.0, 1.0, 0.0),
                &mut q_text_colors,
            );
            style.display = Display::None;
        }
    }
}
