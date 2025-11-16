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
    overworld_ui_query: Query<(&OverworldUI, &Transform), Added<OverworldUI>>,
) {
    for (overworld_ui, transform) in overworld_ui_query.iter() {
        if *overworld_ui.layer() == UILayer::BACKPACK_MENU {
            info!(
                "Overworld UI spawned at position: {:?}",
                transform.translation
            );

            let box_width: f32 = 100.0;
            let box_height: f32 = 50.0;
            let border_width: f32 = 2.0;

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

            // 创建实心填充着色器
            let solid_fill = shaders.add_fill_body(
                r#"
                let a = select(0.0, 1.0, input.distance <= 0.0);
                return vec4<f32>(input.color.rgb, a);
                "#,
            );

            let final_position = transform.translation + Vec3::new(0.0, 0.0, 5.0);

            info!("Spawning SmudShape at position: {:?}", final_position);

            commands.spawn((
                SmudShape {
                    color: Color::WHITE,
                    sdf: outer_sdf,
                    frame: Frame::Quad((box_width + border_width * 2.0) + 10.0),
                    fill: solid_fill.clone(),
                    ..default()
                },
                Transform::from_translation(final_position),
            ));

            commands.spawn((
                SmudShape {
                    color: Color::BLACK,
                    sdf: inner_sdf,
                    frame: Frame::Quad(box_width.max(box_height) + 10.0),
                    fill: solid_fill,
                    ..default()
                },
                Transform::from_translation(final_position + Vec3::new(0.0, 0.0, 0.1)),
            ));
        }
    }
}
