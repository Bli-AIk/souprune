use super::components::{
    BoxCursor, BoxCursorPosition, BoxCursorVisibility, CameraAnchored, OverworldUI, OverworldUIBox,
    OverworldUIBoxVisibility, UIBoxFiller, UIFont, UILayer, UILayerVisibilityRule, UITextConfig,
};
use super::text::NeedsGlyphRefresh;
use crate::app_state::overworld::OverworldState;
use crate::core::data::PlayerData;
use crate::core::sprite::params::SpriteParams;
use crate::extra::mortar::MortarStringTable;
use bevy::ecs::relationship::Relationship;
use bevy::prelude::*;
use bevy::sprite_render::AlphaMode2d;
use bevy_rich_text3d::{Text3d, Text3dStyling, TextAtlas};
use bevy_smud::prelude::SdfAssets;
use bevy_smud::{Frame, SmudShape};
use std::collections::VecDeque;

type OverworldUIQuery<'w, 's> =
    Query<'w, 's, (Entity, &'static OverworldUI), (Added<OverworldUI>, Without<OverworldUIBox>)>;

/// Handle Undertale-style UI container creation whenever the backpack UI entity spawns.
///
/// 当背包 UI 实体生成时，负责创建 Undertale 风格的 UI 容器。
pub(crate) fn draw_backpack_ui_system(
    mut commands: Commands,
    overworld_ui_query: OverworldUIQuery,
    camera_query: Query<&Transform, With<Camera2d>>,
    player_data: Res<PlayerData>,
    mut sprite_params: SpriteParams,
    mortar_strings: Res<MortarStringTable>,
) {
    for (ui_entity, overworld_ui) in overworld_ui_query.iter() {
        if *overworld_ui.layer() != UILayer::BACKPACK_MENU {
            continue;
        }

        info!("Adding OverworldUIBox component to UI entity");

        let camera_transform = match camera_query.single() {
            Ok(transform) => transform,
            Err(_) => {
                warn!("No Camera2d found for UI spawning!");
                return;
            }
        };

        let cursor_sprite = {
            let mut sprite_context = sprite_params.create_sprite_context();
            let mut sprite = sprite_context.get_sprite("common", "heartsmall");
            sprite.color = Color::srgb(1.0, 0.0, 0.0);
            sprite
        };

        // Only add the `OverworldUIBox` component; rendering happens in `update_overworld_ui_box_system`.
        //
        // 只负责添加 OverworldUIBox 组件，具体绘制交由 `update_overworld_ui_box_system`。
        commands.entity(ui_entity).with_children(|parent| {
            parent.spawn((
                OverworldUIBox::new_with_texts(
                    65.0,
                    68.0,
                    3.0,
                    vec![UITextConfig {
                        name: "Text".into(),
                        content: format!(
                            "{}\n{}",
                            mortar_strings.get("ITEM").unwrap_or("ITEM"),
                            mortar_strings.get("STAT").unwrap_or("STAT")
                        ),
                        font: UIFont::DeterminationSans,
                        world_scale: Vec2::splat(13.25),
                        transform: Transform::from_xyz(-9.5, 28.25, 1.0),
                        line_height: 1.375,
                        ..Default::default()
                    }],
                ),
                OverworldUIBoxVisibility::new(UILayerVisibilityRule::Always),
                Visibility::default(),
                BoxCursor::new(
                    cursor_sprite,
                    BoxCursorVisibility::OnlyIn(vec![UILayer::BACKPACK_MENU]),
                    BoxCursorPosition::linear(
                        Vec3::new(-19.0, 18.5, 2.0),
                        Vec3::new(0.0, -18.0, 0.0),
                    ),
                    Transform::from_scale(Vec3::splat(1.0)),
                ),
                CameraAnchored::new(Vec3::new(-108.5, -1.0, 0.0)),
                Transform::from_translation(
                    camera_transform.translation + Vec3::new(-108.5, -1.0, 0.0),
                ),
                Name::new("MenuBox"),
            ));
        });

        commands.entity(ui_entity).with_children(|parent| {
            parent.spawn((
                OverworldUIBox::new_with_texts(
                    65.0,
                    49.0,
                    3.0,
                    vec![
                        UITextConfig {
                            name: "NameText".into(),
                            content: player_data.name.clone(),
                            font: UIFont::DeterminationSans,
                            world_scale: Vec2::splat(13.),
                            transform: Transform::from_xyz(-28.5, 22.0, 1.0),
                            ..Default::default()
                        },
                        UITextConfig {
                            name: "HUDText".into(),
                            // TODO: 取消硬编码文本
                            content: format!(
                                "LV  {}\nhp  {}/{}\ng   {}",
                                player_data.lv,
                                player_data.hp,
                                player_data.hp_max,
                                player_data.gold
                            ),
                            font: UIFont::Hud,
                            world_scale: Vec2::splat(8.),
                            transform: Transform::from_xyz(-28.5, 5.75, 1.0),
                            line_height: 1.125,
                            ..Default::default()
                        },
                    ],
                ),
                OverworldUIBoxVisibility::new(UILayerVisibilityRule::Always),
                Visibility::default(),
                CameraAnchored::new(Vec3::new(-108.5, 66.5, 0.0)),
                Transform::from_translation(
                    camera_transform.translation + Vec3::new(-108.5, 66.5, 0.0),
                ),
                Name::new("InfoBox"),
            ));
        });

        commands.entity(ui_entity).with_children(|parent| {
            parent.spawn((
                OverworldUIBox::new_with_texts(
                    105.0,
                    68.0,
                    3.0,
                    vec![UITextConfig {
                        name: "ItemLayerText".into(),
                        content: "* TODO: Items go here".to_string(),
                        font: UIFont::DeterminationSans,
                        world_scale: Vec2::splat(10.5),
                        transform: Transform::from_xyz(-48.5, 23.0, 1.0),
                        line_height: 1.2,
                        ..Default::default()
                    }],
                ),
                OverworldUIBoxVisibility::new(UILayerVisibilityRule::OnlyIn(vec![
                    UILayer::BACKPACK_ITEM.clone(),
                ])),
                Visibility::default(),
                CameraAnchored::new(Vec3::new(-32.5, -1.0, 0.0)),
                Transform::from_translation(
                    camera_transform.translation + Vec3::new(-32.5, -1.0, 0.0),
                ),
                Name::new("ItemBox"),
            ));
        });

        commands.entity(ui_entity).with_children(|parent| {
            parent.spawn((
                OverworldUIBox::new_with_texts(
                    105.0,
                    68.0,
                    3.0,
                    vec![UITextConfig {
                        name: "StatusLayerText".into(),
                        content: format!(
                            "* LV: {}\n* ATK: {}\n* DEF: {}\n* HP: {}/{}",
                            player_data.lv,
                            player_data.attack,
                            player_data.defense,
                            player_data.hp,
                            player_data.hp_max
                        ),
                        font: UIFont::DeterminationSans,
                        world_scale: Vec2::splat(9.5),
                        transform: Transform::from_xyz(-48.5, 23.0, 1.0),
                        line_height: 1.25,
                        ..Default::default()
                    }],
                ),
                OverworldUIBoxVisibility::new(UILayerVisibilityRule::OnlyIn(vec![
                    UILayer::BACKPACK_STATUS.clone(),
                ])),
                Visibility::default(),
                CameraAnchored::new(Vec3::new(-32.5, 66.5, 0.0)),
                Transform::from_translation(
                    camera_transform.translation + Vec3::new(-32.5, 66.5, 0.0),
                ),
                Name::new("StatusBox"),
            ));
        });
    }
}

type OverworldUIBoxQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static OverworldUIBox,
        &'static Transform,
        Option<&'static Children>,
    ),
    Or<(
        Added<OverworldUIBox>,
        Changed<OverworldUIBox>,
        Changed<Transform>,
    )>,
>;

/// Create SmudShape child entities for each UI box.
///
/// 为 UI 框创建 SmudShape 子实体。
fn spawn_ui_box_children(
    commands: &mut Commands,
    entity: Entity,
    ui_box: &OverworldUIBox,
    outer_sdf: Handle<Shader>,
    inner_sdf: Handle<Shader>,
    shaders: &mut ResMut<Assets<Shader>>,
    color_materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    info!("Spawning SmudShape children for UI box");

    let box_width = ui_box.width();
    let box_height = ui_box.height();
    let border_width = ui_box.border_width();

    let solid_fill = shaders.add_fill_body(
        r#"
                        let a = select(0.0, 1.0, input.distance <= 0.0);
                        return vec4<f32>(input.color.rgb, a);
                        "#,
    );

    let mut filler_entity: Option<Entity> = None;

    commands.entity(entity).with_children(|parent| {
        parent
            .spawn((
                SmudShape {
                    color: Color::WHITE,
                    sdf: outer_sdf.clone(),
                    frame: Frame::Quad((box_width + border_width * 2.0) + 10.0),
                    fill: solid_fill.clone(),
                    ..default()
                },
                Transform::from_translation(Vec3::new(0.0, 0.0, 5.0)),
                Name::new("UIBoxBorder"),
            ))
            .with_children(|border_parent| {
                let filler = border_parent
                    .spawn((
                        SmudShape {
                            color: Color::BLACK,
                            sdf: inner_sdf.clone(),
                            frame: Frame::Quad(box_width.max(box_height) + 10.0),
                            fill: solid_fill.clone(),
                            ..default()
                        },
                        Transform::from_translation(Vec3::new(0.0, 0.0, 0.1)),
                        Name::new("UIBoxFiller"),
                        UIBoxFiller,
                    ))
                    .id();

                filler_entity = Some(filler);
            });
    });

    let Some(filler_entity) = filler_entity else {
        warn!("Failed to spawn UI box filler for entity {:?}", entity);
        return;
    };

    commands
        .entity(filler_entity)
        .with_children(|filler_parent| {
            for text_config in &ui_box.texts {
                info!("Spawning text for UI box: {}", text_config.content);

                let mat = color_materials.add(ColorMaterial {
                    texture: Some(TextAtlas::DEFAULT_IMAGE.clone()),
                    alpha_mode: AlphaMode2d::Blend,
                    ..Default::default()
                });

                filler_parent.spawn((
                    text_config.name.clone(),
                    Text3d::new(text_config.content.clone()),
                    Text3dStyling {
                        font: text_config.font.font_name().into(),
                        size: text_config.font.default_size(),
                        world_scale: Some(text_config.world_scale),
                        color: text_config.color,
                        align: text_config.align,
                        anchor: text_config.anchor,
                        line_height: text_config.line_height,
                        ..Default::default()
                    },
                    Mesh2d::default(),
                    MeshMaterial2d(mat.clone()),
                    text_config.transform,
                    Visibility::Hidden,
                    NeedsGlyphRefresh,
                ));
            }
        });
}

/// Update UI box geometry each time layout components change.
///
/// 当布局组件变化时更新 UI 框的几何数据。
pub(crate) fn update_overworld_ui_box_system(
    mut shaders: ResMut<Assets<Shader>>,
    mut commands: Commands,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
    overworld_ui_box_query: OverworldUIBoxQuery,
    mut smud_shape_query: Query<&mut SmudShape>,
    children_query: Query<&Children>,
) {
    for (entity, ui_box, transform, children_opt) in overworld_ui_box_query.iter() {
        let box_width = ui_box.width();
        let box_height = ui_box.height();
        let border_width = ui_box.border_width();

        let outer_sdf = shaders.add_sdf_expr(format!(
            "smud::sd_box(p, vec2<f32>({}, {}))",
            (box_width + border_width * 2.0) / 2.0,
            (box_height + border_width * 2.0) / 2.0
        ));

        let inner_sdf = shaders.add_sdf_expr(format!(
            "smud::sd_box(p, vec2<f32>({}, {}))",
            box_width / 2.0,
            box_height / 2.0
        ));

        match children_opt {
            Some(children) => {
                let mut queue: VecDeque<Entity> = VecDeque::from(children.to_vec());
                let mut smud_shape_entities: Vec<Entity> = Vec::new();

                while let Some(child) = queue.pop_front() {
                    if smud_shape_query.get(child).is_ok() {
                        smud_shape_entities.push(child);
                        if smud_shape_entities.len() >= 2 {
                            break;
                        }
                    }

                    if let Ok(grandchildren) = children_query.get(child) {
                        queue.extend(grandchildren.to_vec());
                    }
                }

                if smud_shape_entities.len() >= 2 {
                    info!("Updating existing SmudShape children for UI box");

                    if let Ok(mut outer_shape) = smud_shape_query.get_mut(smud_shape_entities[0]) {
                        outer_shape.sdf = outer_sdf.clone();
                        outer_shape.frame = Frame::Quad((box_width + border_width * 2.0) + 10.0);
                    }

                    if let Ok(mut inner_shape) = smud_shape_query.get_mut(smud_shape_entities[1]) {
                        inner_shape.sdf = inner_sdf.clone();
                        inner_shape.frame = Frame::Quad(box_width.max(box_height) + 10.0);
                    }
                } else {
                    info!(
                        "Adding SmudShape children to existing UI box at position: {:?}",
                        transform.translation
                    );

                    spawn_ui_box_children(
                        &mut commands,
                        entity,
                        ui_box,
                        outer_sdf,
                        inner_sdf,
                        &mut shaders,
                        &mut color_materials,
                    );
                }
            }
            None => {
                info!(
                    "Creating new SmudShape children for UI box at position: {:?}",
                    transform.translation
                );

                spawn_ui_box_children(
                    &mut commands,
                    entity,
                    ui_box,
                    outer_sdf,
                    inner_sdf,
                    &mut shaders,
                    &mut color_materials,
                );
            }
        }
    }
}

/// Toggle UI box visibility according to the active [`UILayer`].
///
/// 根据当前激活的 [`UILayer`] 切换 UI 框可见性。
pub(crate) fn update_overworld_ui_box_visibility_system(
    overworld_state: Res<State<OverworldState>>,
    ui_query: Query<&OverworldUI>,
    parent_query: Query<&ChildOf>,
    mut box_query: Query<
        (Entity, &OverworldUIBoxVisibility, &mut Visibility),
        With<OverworldUIBox>,
    >,
) {
    let in_backpack = overworld_state.get() == &OverworldState::Backpack;

    for (entity, layer_visibility, mut visibility) in box_query.iter_mut() {
        if !in_backpack {
            if *visibility != Visibility::Hidden {
                *visibility = Visibility::Hidden;
            }
            continue;
        }

        let Ok(parent) = parent_query.get(entity) else {
            if *visibility != Visibility::Hidden {
                *visibility = Visibility::Hidden;
            }
            continue;
        };

        let Ok(overworld_ui) = ui_query.get(parent.get()) else {
            if *visibility != Visibility::Hidden {
                *visibility = Visibility::Hidden;
            }
            continue;
        };

        let should_show = layer_visibility.is_visible_for(overworld_ui.layer());
        if should_show {
            if *visibility != Visibility::Inherited {
                *visibility = Visibility::Inherited;
            }
        } else if *visibility != Visibility::Hidden {
            *visibility = Visibility::Hidden;
        }
    }
}
