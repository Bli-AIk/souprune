use super::components::{OverworldUI, UILayerNavigationConfig};
use crate::app_state::overworld::{OverworldState, character};
use crate::core::input::Action;
use bevy::prelude::*;
use leafwing_input_manager::action_state::ActionState;

/// Handle transitions between overworld sub-states driven by menu actions.
///
/// 处理菜单输入驱动的 Overworld 子状态间转换。
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

/// Update UI focus navigation while the overworld backpack is active.
///
/// 在背包界面激活时更新 UI 焦点导航。
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
