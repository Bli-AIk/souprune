use super::components::*;
use super::layout::*;
use crate::core::sprite::params::SpriteParams;
use bevy::prelude::*;

#[derive(Resource)]
pub struct UILayoutHandle {
    pub handle: Handle<UILayoutAsset>,
}

#[derive(Component)]
pub struct RonDrivenUI;

#[derive(Component)]
pub struct UITextId(pub String);

pub fn load_ui_layout_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    let handle: Handle<UILayoutAsset> = asset_server.load("ui/backpack.ui.ron");
    commands.insert_resource(UILayoutHandle { handle });
    info!("Loading UI layout from RON file");
}

pub fn spawn_ron_ui_system(
    mut commands: Commands,
    ui_layout_handle: Option<Res<UILayoutHandle>>,
    ui_layouts: Res<Assets<UILayoutAsset>>,
    overworld_ui_query: Query<
        (Entity, &super::components::OverworldUI),
        (
            Added<super::components::OverworldUI>,
            Without<super::components::OverworldUIBox>,
        ),
    >,
    camera_query: Query<&Transform, With<Camera2d>>,
    mut sprite_params: SpriteParams,
    mortar_strings: Res<crate::extra::mortar::MortarStringTable>,
) {
    let Some(ui_layout_handle) = ui_layout_handle else {
        return;
    };

    let Some(ui_layout) = ui_layouts.get(&ui_layout_handle.handle) else {
        return;
    };

    for (ui_entity, overworld_ui) in overworld_ui_query.iter() {
        if *overworld_ui.layer() != super::components::UILayer::BACKPACK_MENU {
            continue;
        }

        info!("Spawning UI from RON layout for backpack menu");

        let camera_transform = match camera_query.single() {
            Ok(transform) => transform,
            Err(_) => {
                warn!("No Camera2d found for UI spawning!");
                return;
            }
        };

        for root in &ui_layout.roots {
            spawn_ui_node(
                &mut commands,
                ui_entity,
                root,
                camera_transform,
                &mut sprite_params,
                &mortar_strings,
            );
        }
    }
}

fn spawn_ui_node(
    commands: &mut Commands,
    parent_entity: Entity,
    node_def: &UINodeDef,
    camera_transform: &Transform,
    sprite_params: &mut SpriteParams,
    mortar_strings: &crate::extra::mortar::MortarStringTable,
) {
    commands.entity(parent_entity).with_children(|parent| {
        if let Some(ui_box_logic) = &node_def.ui_box_logic {
            let visibility_rule = node_def
                .visibility_rule
                .as_ref()
                .map(parse_visibility_rule)
                .unwrap_or(UILayerVisibilityRule::Always);

            let texts = node_def
                .texts
                .iter()
                .map(|text_def| {
                    let content = resolve_text_content(
                        text_def.content.as_deref().unwrap_or(""),
                        mortar_strings,
                    );

                    UITextConfig {
                        name: Name::new(text_def.id.clone()),
                        content,
                        font: text_def.font.clone().into(),
                        world_scale: text_def.world_scale.clone().into(),
                        color: bevy::color::Srgba::new(
                            text_def.color.r,
                            text_def.color.g,
                            text_def.color.b,
                            text_def.color.a,
                        ),
                        transform: Transform::from_translation(text_def.transform.clone().into()),
                        line_height: text_def.line_height.unwrap_or(1.0),
                        ..Default::default()
                    }
                })
                .collect::<Vec<_>>();

            let offset: Vec3 = ui_box_logic.offset.clone().into();

            let mut box_entity = parent.spawn((
                OverworldUIBox::new_with_texts(
                    ui_box_logic.width,
                    ui_box_logic.height,
                    ui_box_logic.border_width,
                    texts,
                ),
                OverworldUIBoxVisibility::new(visibility_rule),
                Visibility::default(),
                CameraAnchoredBundle::from_camera_transform(camera_transform, offset),
                Name::new(node_def.name.clone()),
            ));

            if let Some(cursor_def) = &node_def.cursor {
                let mut sprite_context = sprite_params.create_sprite_context();
                let mut sprite = sprite_context.get_sprite("common", "heartsmall");
                sprite.color = Color::srgb(1.0, 0.0, 0.0);

                let cursor_position = match &cursor_def.default_position {
                    BoxCursorPositionDef::Static(vec) => {
                        BoxCursorPosition::Static(vec.clone().into())
                    }
                    BoxCursorPositionDef::Linear { origin, step } => BoxCursorPosition::Linear {
                        origin: origin.clone().into(),
                        step: step.clone().into(),
                    },
                    BoxCursorPositionDef::Custom { positions } => BoxCursorPosition::Custom(
                        positions.iter().map(|v| v.clone().into()).collect(),
                    ),
                };

                let cursor_visibility = BoxCursorVisibility::OnlyIn(vec![UILayer::BACKPACK_MENU]);

                box_entity.insert(BoxCursor::new(
                    sprite,
                    cursor_visibility,
                    BoxCursorPlacement::new(cursor_position),
                    Transform::from_scale(Vec3::splat(1.0)),
                ));
            }
        }
    });
}

fn parse_visibility_rule(rule_def: &UIVisibilityRuleDef) -> UILayerVisibilityRule {
    match rule_def.rule_type.as_str() {
        "Always" => UILayerVisibilityRule::Always,
        "AlwaysHidden" => UILayerVisibilityRule::AlwaysHidden,
        "OnlyIn" => {
            if let Some(layers) = &rule_def.layers {
                let ui_layers = layers
                    .iter()
                    .map(|name| UILayer::new(name.clone()))
                    .collect();
                UILayerVisibilityRule::OnlyIn(ui_layers)
            } else {
                UILayerVisibilityRule::Always
            }
        }
        "Except" => {
            if let Some(layers) = &rule_def.layers {
                let ui_layers = layers
                    .iter()
                    .map(|name| UILayer::new(name.clone()))
                    .collect();
                UILayerVisibilityRule::Except(ui_layers)
            } else {
                UILayerVisibilityRule::Always
            }
        }
        _ => UILayerVisibilityRule::Always,
    }
}

fn resolve_text_content(
    template: &str,
    mortar_strings: &crate::extra::mortar::MortarStringTable,
) -> String {
    let mut result = String::new();
    let mut chars = template.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '{' {
            if let Some(&next_ch) = chars.peek() {
                if next_ch == '{' {
                    chars.next();
                    let mut key = String::new();
                    let mut found_closing = false;

                    while let Some(ch) = chars.next() {
                        if ch == '}' {
                            if let Some(&next_ch) = chars.peek() {
                                if next_ch == '}' {
                                    chars.next();
                                    found_closing = true;
                                    break;
                                }
                            }
                        }
                        key.push(ch);
                    }

                    if found_closing {
                        let resolved = mortar_strings.resolve(&key);
                        result.push_str(resolved);
                    } else {
                        result.push_str("{{");
                        result.push_str(&key);
                    }
                    continue;
                }
            }
        }
        result.push(ch);
    }

    result
}
