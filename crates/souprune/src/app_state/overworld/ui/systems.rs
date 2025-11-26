use crate::app_state::overworld::ui::components::{
    BoxCursor, BoxCursorOwner, BoxCursorPosition, BoxCursorReady, BoxCursorSprite,
    BoxCursorVisibility, CameraAnchored, OverworldUI, OverworldUIBox, UIBoxFiller, UIFont, UILayer,
    UILayerNavigationConfig, UITextConfig,
};
use crate::app_state::overworld::{OverworldState, character};
use crate::core::data::PlayerData;
use crate::core::input::Action;
use crate::core::sprite::params::SpriteParams;
use bevy::ecs::relationship::Relationship;
use bevy::prelude::*;
use bevy::sprite_render::AlphaMode2d;
use bevy_rich_text3d::*;
use bevy_smud::prelude::SdfAssets;
use bevy_smud::{Frame, SmudShape};
use leafwing_input_manager::action_state::ActionState;
use std::collections::VecDeque;

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

pub(crate) fn update_overworld_ui_navigation_system(
    overworld_state: Res<State<OverworldState>>,
    navigation: Res<UILayerNavigationConfig>,
    mut ui_query: Query<&mut OverworldUI>,
    query: Query<&ActionState<Action>, With<character::components::PlayerControlled>>,
) {
    if overworld_state.get() != &OverworldState::Backpack {
        return;
    }

    let Ok(action_state) = query.single() else {
        return;
    };

    for mut overworld_ui in ui_query.iter_mut() {
        let Some(rule) = navigation.get(overworld_ui.layer()) else {
            continue;
        };

        let mut delta: isize = 0;
        for action in [Action::Up, Action::Down, Action::Left, Action::Right] {
            if action_state.just_pressed(&action)
                && let Some(change) = rule.delta_for(action)
            {
                delta += change;
            }
        }

        if delta != 0 {
            let mut next_index = overworld_ui.index() as isize + delta;
            let max_index = overworld_ui.max_index() as isize;
            if next_index < 0 {
                next_index = 0;
            } else if next_index > max_index {
                next_index = max_index;
            }
            overworld_ui.set_index(next_index as usize);
        }
    }
}

pub(crate) fn spawn_backpack_ui_system(
    mut commands: Commands,
    overworld_ui_query: Query<&OverworldUI>,
) {
    // Only create the UI if it does not exist yet and we are in the menu state.
    //
    // 仅在处于菜单状态且 UI 尚未存在时才创建 UI。
    if !overworld_ui_query.is_empty() {
        return;
    }

    // Dynamically compute `UILayer` total count minus one.
    //
    // 动态获取 UILayer 的总数减一。
    let max_index = UILayer::total_count().saturating_sub(1);

    commands.spawn((
        OverworldUI::new(UILayer::BACKPACK_MENU, max_index),
        // Add a Transform so the UI entity can be positioned.
        //
        // 添加 Transform 组件以便控制 UI 实体的位置。
        Transform::from_translation(Vec3::ZERO),
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

// UT style: only add the `OverworldUIBox` component.
//
// UT 风格：只负责添加 OverworldUIBox 组件。
pub(crate) fn draw_backpack_ui_system(
    mut commands: Commands,
    overworld_ui_query: OverworldUIQuery,
    camera_query: Query<&Transform, With<Camera2d>>,
    player_data: Res<PlayerData>,
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
                            // TODO: 取消硬编码文本
                            content: "ITEM\nSTAT".to_string(),
                            font: UIFont::DeterminationSans,
                            world_scale: Vec2::splat(13.25),
                            transform: Transform::from_xyz(-9.5, 28.5, 1.0),
                            line_height: 1.4,
                            ..Default::default()
                        }],
                    ),
                    BoxCursor::new(
                        "common",
                        "heart",
                        BoxCursorVisibility::OnlyIn(vec![UILayer::BACKPACK_MENU]),
                        BoxCursorPosition::linear(
                            Vec3::new(-34.0, 28.0, 2.0),
                            Vec3::new(0.0, -20.0, 0.0),
                        ),
                    ),
                    CameraAnchored::new(Vec3::new(-108.5, -1.0, 0.0)),
                    Transform::from_translation(
                        camera_transform.translation + Vec3::new(-108.5, -1.0, 0.0),
                    ),
                    Visibility::default(),
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
                                content: {
                                    let hud_text = format!(
                                        "LV  {}\nhp  {}/{}\ng   {}",
                                        player_data.lv,
                                        player_data.hp,
                                        player_data.hp_max,
                                        player_data.gold
                                    );
                                    hud_text
                                },
                                font: UIFont::Hud,
                                world_scale: Vec2::splat(8.),
                                transform: Transform::from_xyz(-28.5, 5.75, 1.0),
                                line_height: 1.125,
                                ..Default::default()
                            },
                        ],
                    ),
                    BoxCursor::new(
                        "common",
                        "heart",
                        BoxCursorVisibility::AlwaysHidden,
                        BoxCursorPosition::fixed(Vec3::ZERO),
                    ),
                    CameraAnchored::new(Vec3::new(-108.5, 66.5, 0.0)),
                    Transform::from_translation(
                        camera_transform.translation + Vec3::new(-108.5, 66.5, 0.0),
                    ),
                    Visibility::default(),
                    Name::new("InfoBox"),
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
                        outer_shape.sdf = outer_sdf;
                        outer_shape.frame = Frame::Quad((box_width + border_width * 2.0) + 10.0);
                    }

                    if let Ok(mut inner_shape) = smud_shape_query.get_mut(smud_shape_entities[1]) {
                        inner_shape.sdf = inner_sdf;
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

fn find_ui_box_filler_entity(
    root: Entity,
    children_query: &Query<&Children>,
    filler_query: &Query<(), With<UIBoxFiller>>,
) -> Option<Entity> {
    let mut queue: VecDeque<Entity> = VecDeque::new();
    if let Ok(children) = children_query.get(root) {
        for child in children.iter() {
            queue.push_back(child);
        }
    }

    while let Some(child) = queue.pop_front() {
        if filler_query.get(child).is_ok() {
            return Some(child);
        }

        if let Ok(children) = children_query.get(child) {
            for grandchild in children.iter() {
                queue.push_back(grandchild);
            }
        }
    }

    None
}

pub(crate) fn spawn_box_cursor_visual_system(
    mut commands: Commands,
    mut sprite_params: SpriteParams,
    query: Query<(Entity, &BoxCursor), (With<OverworldUIBox>, Without<BoxCursorReady>)>,
    children_query: Query<&Children>,
    filler_query: Query<(), With<UIBoxFiller>>,
) {
    for (entity, cursor) in query.iter() {
        let Some(filler_entity) = find_ui_box_filler_entity(entity, &children_query, &filler_query)
        else {
            continue;
        };

        let (module_name, sprite_name) = cursor.sprite_config();
        let sprite = sprite_params
            .create_sprite_context()
            .get_sprite(module_name, sprite_name);

        commands.entity(filler_entity).with_children(|parent| {
            parent.spawn((
                Name::new("BoxCursorSprite"),
                BoxCursorSprite,
                BoxCursorOwner(entity),
                sprite,
                Transform::from_translation(Vec3::ZERO),
                Visibility::Hidden,
            ));
        });

        commands.entity(entity).insert(BoxCursorReady);
    }
}

pub(crate) fn update_box_cursor_state_system(
    overworld_state: Res<State<OverworldState>>,
    ui_query: Query<&OverworldUI>,
    box_query: Query<&BoxCursor, With<OverworldUIBox>>,
    parent_query: Query<&ChildOf>,
    mut sprite_query: Query<
        (&BoxCursorOwner, &mut Transform, &mut Visibility),
        With<BoxCursorSprite>,
    >,
) {
    for (owner, mut transform, mut visibility) in sprite_query.iter_mut() {
        let Ok(cursor) = box_query.get(owner.0) else {
            if *visibility != Visibility::Hidden {
                *visibility = Visibility::Hidden;
            }
            continue;
        };

        let Ok(parent) = parent_query.get(owner.0) else {
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

        let mut should_show = overworld_state.get() == &OverworldState::Backpack;
        should_show &= !cursor.is_hidden();
        should_show &= cursor.visibility().is_visible_for(overworld_ui.layer());

        if should_show {
            let translation = cursor.desired_translation(overworld_ui.index());
            if transform.translation != translation {
                transform.translation = translation;
            }
            if *visibility != Visibility::Inherited {
                *visibility = Visibility::Inherited;
            }
        } else if *visibility != Visibility::Hidden {
            *visibility = Visibility::Hidden;
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
        // Trigger glyph reload by modifying the text.
        //
        // 通过修改文本来触发字形重新加载。
        if let Some(s) = text.get_single_mut() {
            let current = s.clone();
            s.clear();
            *s = current;
        }

        // Remove the marker.
        //
        // 移除标记。
        commands.entity(entity).remove::<NeedsGlyphRefresh>();
        info!("Refreshed glyphs for text entity {:?}", entity);
    }
}

/// Show text once mesh is generated
///
/// 网格生成后显示文本
type TextMeshQuery<'w, 's> =
    Query<'w, 's, (&'static Mesh2d, &'static mut Visibility), (With<Text3d>, Changed<Mesh2d>)>;

pub(crate) fn show_text_when_ready_system(mut text_query: TextMeshQuery) {
    for (mesh, mut visibility) in text_query.iter_mut() {
        if mesh.0 != Handle::default() && *visibility == Visibility::Hidden {
            *visibility = Visibility::Inherited;
            info!("Text mesh ready, showing text");
        }
    }
}

/// Keep camera-anchored UI in place even if the camera is still interpolating.
///
/// 在摄像机插值移动时保持 UI 的相对位置不漂移
pub(crate) fn update_camera_anchored_ui_system(
    overworld_state: Res<State<OverworldState>>,
    camera_query: Query<&Transform, With<Camera2d>>,
    mut anchored_ui_query: Query<(&CameraAnchored, &mut Transform), Without<Camera2d>>,
) {
    if overworld_state.get() != &OverworldState::Backpack {
        return;
    }

    let Ok(camera_transform) = camera_query.single() else {
        warn!("No Camera2d available for anchoring UI");
        return;
    };

    for (anchor, mut transform) in anchored_ui_query.iter_mut() {
        let new_translation = camera_transform.translation + anchor.offset;
        if transform.translation != new_translation {
            transform.translation = new_translation;
        }
    }
}
