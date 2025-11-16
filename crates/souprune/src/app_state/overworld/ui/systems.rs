use crate::app_state::overworld::character::components::PlayerControlled;
use crate::app_state::overworld::ui::components::{OverworldUI, UILayer};
use crate::core::input::Action;
use bevy::prelude::*;
use bevy_smud::prelude::SdfAssets;
use bevy_smud::{Frame, SIMPLE_FILL_HANDLE, SmudShape};
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

            let border_sdf = shaders.add_sdf_expr(format!(
                "abs(smud::sd_box(p, vec2<f32>({}, {}))) - {}",
                50.0, 50.0, 2.0
            ));
            let final_position = transform.translation + Vec3::new(0.0, 0.0, 0.5);

            info!("Spawning SmudShape at position: {:?}", final_position);

            commands.spawn((
                SmudShape {
                    color: Color::hsl(210.0, 0.75, 0.5),
                    sdf: border_sdf,
                    frame: Frame::Quad(500.0),
                    fill: SIMPLE_FILL_HANDLE,
                    ..default()
                },
                Transform::from_translation(final_position),
            ));
        }
    }
}
