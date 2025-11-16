use crate::app_state::overworld::ui::components::{OverworldUI, UILayer};
use crate::core::input::Action;
use bevy::prelude::*;
use leafwing_input_manager::action_state::ActionState;

pub(crate) fn spawn_backpack_ui_system(
    mut commands: Commands,
    mut query: Query<&ActionState<Action>>,
) {
    for action_state in query.iter_mut() {
        if action_state.pressed(&Action::Menu) {
            //TODO: 把 硬编码的 2 改为 动态获取 UILayer 的总数 - 1
            commands.spawn(OverworldUI::new(UILayer::BACKPACK_MENU, 2));
        }
    }
}

// UT 风格
pub(crate) fn draw_backpack_ui_system(
    mut commands: Commands,
    overworld_ui_query: Query<(Entity, &OverworldUI), Added<OverworldUI>>,
) {
    for (entity, overworld_ui) in overworld_ui_query.iter() {
        if *overworld_ui.layer() == UILayer::BACKPACK_MENU {
            info!("Spawned backpack UI");
        }
    }
}
