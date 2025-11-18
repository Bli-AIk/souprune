use crate::app_state::overworld::ui::components::{
    OverworldUI, OverworldUIBox, UIFont, UILayer, UITextConfig,
};
use crate::app_state::overworld::{OverworldState, character};
use crate::core::input::Action;
use bevy::prelude::*;
use bevy::sprite_render::AlphaMode2d;
use bevy_rich_text3d::*;
use bevy_smud::prelude::SdfAssets;
use bevy_smud::{Frame, SmudShape};
use leafwing_input_manager::action_state::ActionState;

/// Marker component for newly spawned text that needs glyph refresh
///
/// 新生成文本的标记组件，需要刷新字形
#[derive(Component)]
pub(crate) struct NeedsGlyphRefresh;

/// Handle transitions between overworld sub-states
///
/// 处理 Menu 对 Overworld 子状态之间的转换
pub(crate) fn menu_overworld_state_transitions_system(
    mut next_state: ResMut<NextState<OverworldState>>,
    current_state: Res<State<OverworldState>>,
    query: Query<&ActionState<Action>, With<character::components::PlayerControlled>>,
) {
    if let Ok(action_state) = query.single() {
        match current_state.get() {
            OverworldState::Normal => {
                if !action_state.just_pressed(&Action::Menu) {
                    return;
                }
                info!("Transitioning from Normal to Menu state");
                next_state.set(OverworldState::Backpack);
            }
            OverworldState::Backpack => {
                if !(action_state.just_pressed(&Action::Menu)
                    || action_state.just_pressed(&Action::Cancel))
                {
                    return;
                }
                info!("Transitioning from Menu to Normal state");
                next_state.set(OverworldState::Normal);
            }
            OverworldState::Cutscene => {
                info!("Menu key pressed during cutscene, ignoring");
            }
        }
    }
}

pub(crate) fn spawn_backpack_ui_system(
    mut commands: Commands,
    overworld_ui_query: Query<&OverworldUI>,
) {
    // Only create UI if it doesn't already exist and we're in menu state
    if !overworld_ui_query.is_empty() {
        return;
    }

    // 动态获取 UILayer 的总数 - 1
    let max_index = UILayer::total_count().saturating_sub(1);

    commands.spawn((
        OverworldUI::new(UILayer::BACKPACK_MENU, max_index),
        Transform::from_translation(Vec3::ZERO), // 添加Transform组件
        Name::new("Backpack Menu UI"),
    ));

    info!("Spawned backpack UI in Menu state");
}

pub(crate) fn destroy_backpack_ui_system(
    mut commands: Commands,
    ui_query: Query<(Entity, &OverworldUI)>,
) {
    for (entity, overworld_ui) in ui_query.iter() {
        if *overworld_ui.layer() == UILayer::BACKPACK_MENU {
            commands.entity(entity).despawn();
            info!("Destroyed backpack UI when leaving Menu state");
        }
    }
}

type OverworldUIQuery<'w, 's> =
    Query<'w, 's, (Entity, &'static OverworldUI), (Added<OverworldUI>, Without<OverworldUIBox>)>;

// UT 风格 - 只负责添加 OverworldUIBox 组件
pub(crate) fn draw_backpack_ui_system(
    mut commands: Commands,
    overworld_ui_query: OverworldUIQuery,
    camera_query: Query<&Transform, With<Camera2d>>,
) {
    for (ui_entity, overworld_ui) in overworld_ui_query.iter() {
        if *overworld_ui.layer() == UILayer::BACKPACK_MENU {
            info!("Adding OverworldUIBox component to UI entity");

            let camera_transform = match camera_query.single() {
                Ok(transform) => transform,
                Err(_) => {
                    warn!("No Camera2d found for UI spawning!");
                    return;
                }
            };

            // TODO: 修复跑动时打开菜单位置偏移问题
            // 只负责添加 OverworldUIBox 组件，具体绘制交给 update_overworld_ui_box_system

            commands.entity(ui_entity).with_children(|parent| {
                parent.spawn((
                    OverworldUIBox::new_with_texts(
                        65.0,
                        68.0,
                        3.0,
                        vec![UITextConfig {
                            name: "Menu Box Text".into(),
                            content: "ITEM\nSTAT".to_string(),
                            font: UIFont::DeterminationSans,
                            world_scale: Vec2::splat(13.),
                            color: Srgba::WHITE,
                            // TODO: 调整父子关系以让 z 使用 1 而不是 6
                            transform: Transform::from_xyz(-9.5, -8.0, 6.0),
                            line_height: 1.4,
                            ..Default::default()
                        }],
                    ),
                    Transform::from_translation(
                        camera_transform.translation + Vec3::new(-108.5, -1.0, 0.0),
                    ),
                    Visibility::default(),
                    Name::new("Menu Box"),
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
                                name: "Info Box Text".into(),
                                content: "Name".to_string(),
                                font: UIFont::DeterminationSans,
                                world_scale: Vec2::splat(13.),
                                color: Srgba::WHITE,
                                transform: Transform::from_xyz(-28.5, 9.0, 6.0),
                                ..Default::default()
                            },
                            UITextConfig {
                                transform: Transform::from_xyz(0.0, 0.0, 6.0),
                                ..Default::default()
                            },
                        ],
                    ),
                    Transform::from_translation(
                        camera_transform.translation + Vec3::new(-108.5, 66.5, 0.0),
                    ),
                    Visibility::default(),
                    Name::new("Info Box"),
                ));
            });
        }
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
/// 为UI框创建SmudShape子实体
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

    commands.entity(entity).with_children(|parent| {
        parent.spawn((
            SmudShape {
                color: Color::WHITE,
                sdf: outer_sdf,
                frame: Frame::Quad((box_width + border_width * 2.0) + 10.0),
                fill: solid_fill.clone(),
                ..default()
            },
            Transform::from_translation(Vec3::new(0.0, 0.0, 5.0)),
            Name::new("UI Box Border"),
        ));

        parent.spawn((
            SmudShape {
                color: Color::BLACK,
                sdf: inner_sdf,
                frame: Frame::Quad(box_width.max(box_height) + 10.0),
                fill: solid_fill,
                ..default()
            },
            Transform::from_translation(Vec3::new(0.0, 0.0, 5.1)),
            Name::new("UI Box Background"),
        ));

        // Spawn all configured texts
        //
        // 生成所有配置的文本
        for text_config in &ui_box.texts {
            info!("Spawning text for UI box: {}", text_config.content);

            let mat = color_materials.add(ColorMaterial {
                texture: Some(TextAtlas::DEFAULT_IMAGE.clone()),
                alpha_mode: AlphaMode2d::Blend,
                ..Default::default()
            });

            parent.spawn((
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

pub(crate) fn update_overworld_ui_box_system(
    mut shaders: ResMut<Assets<Shader>>,
    mut commands: Commands,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
    overworld_ui_box_query: OverworldUIBoxQuery,
    mut smud_shape_query: Query<&mut SmudShape>,
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
            // Check if there are already SmudShape children
            // 检查是否已经有SmudShape子实体
            Some(children) => {
                // Filter out SmudShape children
                // 过滤出SmudShape子实体
                let smud_shape_children: Vec<_> = children
                    .iter()
                    .filter(|&child| smud_shape_query.get(child).is_ok())
                    .collect();

                if smud_shape_children.len() >= 2 {
                    // Update existing SmudShapes
                    // 更新现有的SmudShape
                    info!("Updating existing SmudShape children for UI box");

                    if let Ok(mut outer_shape) = smud_shape_query.get_mut(smud_shape_children[0]) {
                        outer_shape.sdf = outer_sdf;
                        outer_shape.frame = Frame::Quad((box_width + border_width * 2.0) + 10.0);
                    }

                    if let Ok(mut inner_shape) = smud_shape_query.get_mut(smud_shape_children[1]) {
                        inner_shape.sdf = inner_sdf;
                        inner_shape.frame = Frame::Quad(box_width.max(box_height) + 10.0);
                    }
                } else {
                    // Has children but no SmudShapes, need to add SmudShapes
                    // 有子实体但没有SmudShape，需要添加SmudShape
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
            // No children, first time creating SmudShapes
            // 没有子实体，首次创建SmudShape
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

/// After spawning, changing the Text3d string in PreUpdate phase to immediately render glyphs
///
/// 在spawn后，在PreUpdate阶段修改Text3d字符串，以立刻渲染字形
pub(crate) fn refresh_text_glyphs_system(
    mut commands: Commands,
    mut text_query: Query<(Entity, &mut Text3d), Added<NeedsGlyphRefresh>>,
) {
    for (entity, mut text) in text_query.iter_mut() {
        // Trigger glyph reload by modifying the text
        // 通过修改文本来触发字形重新加载
        if let Some(s) = text.get_single_mut() {
            let current = s.clone();
            s.clear();
            *s = current;
        }

        // Remove the marker
        // 移除标记
        commands.entity(entity).remove::<NeedsGlyphRefresh>();
        info!("Refreshed glyphs for text entity {:?}", entity);
    }
}

/// Show text once mesh is generated
///
/// 网格生成后显示文本
pub(crate) fn show_text_when_ready_system(
    mut text_query: Query<(&Mesh2d, &mut Visibility), (With<Text3d>, Changed<Mesh2d>)>,
) {
    for (mesh, mut visibility) in text_query.iter_mut() {
        if mesh.0 != Handle::default() && *visibility == Visibility::Hidden {
            *visibility = Visibility::Inherited;
            info!("Text mesh ready, showing text");
        }
    }
}
