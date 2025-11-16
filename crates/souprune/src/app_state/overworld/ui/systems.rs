use crate::app_state::overworld::character::components::PlayerControlled;
use crate::app_state::overworld::ui::components::{OverworldUI, UILayer};
use crate::core::input::Action;
use bevy::prelude::*;
use bevy_smud::prelude::SdfAssets;
use bevy_smud::{Frame, SmudShape};
use leafwing_input_manager::action_state::ActionState;

pub(crate) fn spawn_backpack_ui_system(
    mut commands: Commands,
    mut query: Query<(&ActionState<Action>, &PlayerControlled, &Transform)>,
) {
    for (action_state, _player, player_transform) in query.iter_mut() {
        if action_state.just_pressed(&Action::Menu) {
            //TODO: 把 硬编码的 2 改为 动态获取 UILayer 的总数 - 1
            commands.spawn((
                OverworldUI::new(UILayer::BACKPACK_MENU, 2),
                *player_transform,
            ));
        }
    }
}

// UT 风格
pub(crate) fn draw_backpack_ui_system(
    mut shaders: ResMut<Assets<Shader>>,
    mut commands: Commands,
    overworld_ui_query: Query<(Entity, &OverworldUI, &Transform), Added<OverworldUI>>,
) {
    for (entity, overworld_ui, transform) in overworld_ui_query.iter() {
        if *overworld_ui.layer() == UILayer::BACKPACK_MENU {
            info!(
                "DRAW!!! Creating UI at position: {:?}",
                transform.translation
            );

            let box_width: f32 = 100.0;
            let box_height: f32 = 50.0;
            let border_width: f32 = 2.0;

            // 白色外框
            let outer_sdf = shaders.add_sdf_expr(format!(
                "smud::sd_box(p, vec2<f32>({}, {}))",
                (box_width + border_width * 2.0) / 2.0,
                (box_height + border_width * 2.0) / 2.0
            ));

            // 黑色内填充
            let inner_sdf = shaders.add_sdf_expr(format!(
                "smud::sd_box(p, vec2<f32>({}, {}))",
                box_width / 2.0,
                box_height / 2.0
            ));

            // 创建没有抗锯齿的填充着色器
            let sharp_fill = shaders.add_fill_body(
                r#"
                // 使用 sd_fill_alpha_nearest 避免抗锯齿 
                let a = smud::sd_fill_alpha_nearest(input.distance);
                return vec4<f32>(input.color.rgb, a);
                "#,
            );

            let final_position = transform.translation + Vec3::new(0.0, 0.0, 0.5);

            info!("Spawning SmudShape at position: {:?}", final_position);

            // 先生成白色外框
            commands.spawn((
                SmudShape {
                    color: Color::WHITE,
                    sdf: outer_sdf,
                    frame: Frame::Quad((box_width + border_width * 2.0) + 10.0),
                    fill: sharp_fill.clone(),
                    ..default()
                },
                Transform::from_translation(final_position),
            ));

            // 然后生成黑色内框，稍微向前一点
            commands.spawn((
                SmudShape {
                    color: Color::BLACK,
                    sdf: inner_sdf,
                    frame: Frame::Quad(box_width.max(box_height) + 10.0),
                    fill: sharp_fill,
                    ..default()
                },
                Transform::from_translation(final_position + Vec3::new(0.0, 0.0, 0.1)),
            ));

            //TODO: 目前生成的两个矩形 都是 **边框**，没有填充。我们要求必须两者都是填充的。需要修改 SDF 表达式，生成一个填充的矩形。
        }
    }
}
