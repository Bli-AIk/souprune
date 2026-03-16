//! # bevy_bitmap_text demo
//!
//! Renders text using bevy_bitmap_text with preset size buttons.
//!
//! Font: Source Han Sans CN Light (思源黑体 CN Light)
//! Copyright 2014-2021 Adobe — SIL Open Font License 1.1
//! https://github.com/adobe-fonts/source-han-sans

use bevy::prelude::*;
use bevy_bitmap_text::*;

const DEMO_TEXT: &str = "Hello 你好世界!\nbevy_bitmap_text 位图文字渲染";
const FONT_NAME: &str = "SourceHanSansCN-Light";
const DEFAULT_SIZE: u32 = 32;
const SIZE_PRESETS: &[u32] = &[16, 24, 32, 48, 64, 96];

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "bevy_bitmap_text demo".into(),
                resolution: (960, 640).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(BitmapTextPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, size_button_system)
        .run();
}

/// Marker for the text entity we want to update.
#[derive(Component)]
struct DemoText;

/// Marker for a size-preset button, storing its target size.
#[derive(Component)]
struct SizeButton(u32);

fn setup(mut commands: Commands, mut cache: ResMut<DynamicGlyphCache>) {
    commands.spawn(Camera2d);

    // Load font from example assets
    let font_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples/assets/SourceHanSansCN-Light.otf");
    let font_data = std::fs::read(&font_path).expect("Failed to read font file");
    cache
        .add_font(FontId::from_name(FONT_NAME), &font_data)
        .expect("Failed to load font");
    info!("Loaded font: {FONT_NAME}");

    // Spawn demo text
    commands.spawn((
        DemoText,
        TextBlock::from_segments(parse_text_to_segments(DEMO_TEXT).segments),
        TextBlockStyling {
            font: FontId::from_name(FONT_NAME),
            size_px: DEFAULT_SIZE,
            world_scale: DEFAULT_SIZE as f32,
            color: bevy::color::Srgba::WHITE,
            align: TextAlign::Left,
            anchor: TextAnchor::CENTER,
            line_height: 1.4,
            ..default()
        },
        Transform::from_xyz(0.0, 80.0, 0.0),
    ));

    // === UI: row of preset-size buttons at bottom ===
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::End,
            align_items: AlignItems::Center,
            padding: UiRect::all(Val::Px(20.0)),
            row_gap: Val::Px(10.0),
            ..default()
        })
        .with_children(|parent| {
            // License notice
            parent.spawn((
                Text::new("Font: Source Han Sans CN Light - Adobe - SIL OFL 1.1"),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgba(0.6, 0.6, 0.6, 0.8)),
            ));

            // Button row
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(8.0),
                    ..default()
                })
                .with_children(spawn_size_buttons);
        });
}

fn spawn_size_buttons(row: &mut bevy::ecs::relationship::RelatedSpawnerCommands<ChildOf>) {
    for &size in SIZE_PRESETS {
        let bg = if size == DEFAULT_SIZE {
            Color::srgba(0.2, 0.7, 0.2, 1.0)
        } else {
            Color::srgba(0.3, 0.3, 0.4, 1.0)
        };
        row.spawn((
            SizeButton(size),
            Button,
            Node {
                width: Val::Px(64.0),
                height: Val::Px(36.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(bg),
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(format!("{size}px")),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
    }
}

fn size_button_system(
    button_query: Query<(&Interaction, &SizeButton), Changed<Interaction>>,
    mut text_query: Query<(&mut TextBlock, &mut TextBlockStyling), With<DemoText>>,
    mut bg_query: Query<(&SizeButton, &mut BackgroundColor)>,
) {
    for (&interaction, size_btn) in button_query.iter() {
        if interaction != Interaction::Pressed {
            continue;
        }

        let target = size_btn.0;
        info!("Switching to {target}px");

        for (mut block, mut styling) in text_query.iter_mut() {
            styling.size_px = target;
            styling.world_scale = target as f32;
            block.set_changed();
        }

        // Highlight active button
        for (sb, mut bg) in bg_query.iter_mut() {
            *bg = if sb.0 == target {
                BackgroundColor(Color::srgba(0.2, 0.7, 0.2, 1.0))
            } else {
                BackgroundColor(Color::srgba(0.3, 0.3, 0.4, 1.0))
            };
        }
    }
}
