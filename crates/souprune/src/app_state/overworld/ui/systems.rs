use crate::app_state::overworld::ui::components::{OverworldUI, OverworldUIBox, UILayer};
use crate::app_state::overworld::{OverworldState, character};
use crate::core::input::Action;
use bevy::prelude::*;
use bevy_smud::prelude::SdfAssets;
use bevy_smud::{Frame, SmudShape};
use leafwing_input_manager::action_state::ActionState;

/// Handle transitions between overworld sub-states
///
/// 处理 Menu 对 Overworld 子状态之间的转换
pub(crate) fn menu_overworld_state_transitions_system(
    mut next_state: ResMut<NextState<OverworldState>>,
    current_state: Res<State<OverworldState>>,
    query: Query<&ActionState<Action>, With<character::components::PlayerControlled>>,
) {
    if let Ok(action_state) = query.single()
        && action_state.just_pressed(&Action::Menu)
    {
        match current_state.get() {
            OverworldState::Normal => {
                info!("Transitioning from Normal to Menu state");
                next_state.set(OverworldState::Backpack);
            }
            OverworldState::Backpack => {
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
    camera_query: Query<&Transform, With<Camera2d>>,
    overworld_ui_query: Query<&OverworldUI>,
) {
    // Only create UI if it doesn't already exist and we're in menu state
    if !overworld_ui_query.is_empty() {
        return;
    }

    let camera_transform = match camera_query.single() {
        Ok(transform) => transform,
        Err(_) => {
            warn!("No Camera2d found for UI spawning!");
            return;
        }
    };

    // 动态获取 UILayer 的总数 - 1
    let max_index = UILayer::total_count().saturating_sub(1);

    let mut ui_transform = *camera_transform;
    ui_transform.translation += Vec3::new(-108.5, -1.0, 0.0);

    commands.spawn((
        OverworldUI::new(UILayer::BACKPACK_MENU, max_index),
        ui_transform,
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
) {
    for (ui_entity, overworld_ui) in overworld_ui_query.iter() {
        if *overworld_ui.layer() == UILayer::BACKPACK_MENU {
            info!("Adding OverworldUIBox component to UI entity");

            // 只负责添加 OverworldUIBox 组件，具体绘制交给 update_overworld_ui_box_system
            let ui_box = OverworldUIBox::new(325.0, 340.0, 15.0);
            commands.entity(ui_entity).insert(ui_box);
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
    (
        With<OverworldUI>,
        Or<(Added<OverworldUIBox>, Changed<OverworldUIBox>)>,
    ),
>;
pub(crate) fn update_overworld_ui_box_system(
    mut shaders: ResMut<Assets<Shader>>,
    mut commands: Commands,
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
            // if there are no child entities,
            // it means it's the first time adding, need to create child entities
            //
            // 如果没有子实体，说明是首次添加，需要创建子实体
            None => {
                info!(
                    "Creating new SmudShape children for UI box at position: {:?}",
                    transform.translation
                );

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
                    ));
                });
            }
            // if there are child entities,
            // it means it's an update, just need to modify existing shapes
            //
            // 如果有子实体，说明是更新，只需要修改现有的形状
            Some(children) => {
                if children.len() >= 2 {
                    info!("Updating existing SmudShape children for UI box");

                    if let Ok(mut outer_shape) = smud_shape_query.get_mut(children[0]) {
                        outer_shape.sdf = outer_sdf;
                        outer_shape.frame = Frame::Quad((box_width + border_width * 2.0) + 10.0);
                    }

                    if let Ok(mut inner_shape) = smud_shape_query.get_mut(children[1]) {
                        inner_shape.sdf = inner_sdf;
                        inner_shape.frame = Frame::Quad(box_width.max(box_height) + 10.0);
                    }
                }
            }
        }
    }
}
