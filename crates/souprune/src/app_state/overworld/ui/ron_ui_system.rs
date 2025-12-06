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
) {
    commands.entity(parent_entity).with_children(|parent| {
        if let Some(ui_box_logic) = &node_def.ui_box_logic {
            let visibility_rule = node_def
                .visibility_rule
                .as_ref()
                .map(|v| parse_visibility_rule(v))
                .unwrap_or(UILayerVisibilityRule::Always);

            let texts = node_def
                .texts
                .iter()
                .map(|text_def| UITextConfig {
                    name: Name::new(text_def.id.clone()),
                    content: text_def.default_content.clone(),
                    font: text_def.font.clone().into(),
                    world_scale: Vec2::splat(text_def.font_size),
                    color: bevy::color::Srgba::new(
                        text_def.color.r,
                        text_def.color.g,
                        text_def.color.b,
                        text_def.color.a,
                    ),
                    transform: Transform::from_xyz(
                        text_def.transform_x.unwrap_or(-9.5),
                        text_def.transform_y.unwrap_or(28.25),
                        text_def.transform_z.unwrap_or(1.0),
                    ),
                    line_height: text_def.line_height.unwrap_or(1.0),
                    ..Default::default()
                })
                .collect::<Vec<_>>();

            let offset = Vec3::new(
                ui_box_logic.offset_x.unwrap_or(-108.5),
                ui_box_logic.offset_y.unwrap_or(-1.0),
                ui_box_logic.offset_z.unwrap_or(0.0),
            );

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
                    BoxCursorPositionDef::Static { x, y, z } => {
                        BoxCursorPosition::Static(Vec3::new(*x, *y, *z))
                    }
                    BoxCursorPositionDef::Linear {
                        origin_x,
                        origin_y,
                        origin_z,
                        step_x,
                        step_y,
                        step_z,
                    } => BoxCursorPosition::Linear {
                        origin: Vec3::new(*origin_x, *origin_y, *origin_z),
                        step: Vec3::new(*step_x, *step_y, *step_z),
                    },
                    BoxCursorPositionDef::Custom { positions } => BoxCursorPosition::Custom(
                        positions
                            .iter()
                            .map(|(x, y, z)| Vec3::new(*x, *y, *z))
                            .collect(),
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
