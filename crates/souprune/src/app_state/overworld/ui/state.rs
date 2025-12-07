use super::components::{
    BackpackItemOption, OverworldUI, TransitionAction, UILayer, UILayerNavigationConfig,
    UILayerTransitionConfig,
};
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
    mut overworld_ui_query: Query<&mut OverworldUI>,
    query: Query<&ActionState<Action>, With<character::components::PlayerControlled>>,
    player_data: Res<crate::core::data::PlayerData>,
    transition_config: Res<UILayerTransitionConfig>,
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
                let Ok(mut overworld_ui) = overworld_ui_query.single_mut() else {
                    warn!("Backpack menu open but no OverworldUI entity found");
                    return;
                };

                let current_layer = overworld_ui.layer().clone();

                if current_layer == UILayer::BACKPACK_MENU
                    && action_state.just_pressed(&Action::Menu)
                {
                    info!("Leaving Backpack menu and returning to Normal state");
                    next_state.set(OverworldState::Normal);
                    return;
                }

                if let Some(transitions) = transition_config.get(&current_layer) {
                    if action_state.just_pressed(&Action::Confirm) {
                        for rule in &transitions.on_confirm {
                            if let Some(condition) = &rule.condition {
                                if !evaluate_transition_condition(
                                    condition,
                                    overworld_ui.index(),
                                    &player_data,
                                ) {
                                    continue;
                                }
                            }

                            execute_transition_action(
                                &rule.action,
                                &mut overworld_ui,
                                &mut next_state,
                                &player_data,
                            );
                            return;
                        }
                    }

                    if action_state.just_pressed(&Action::Cancel) {
                        if let Some(cancel_action) = &transitions.on_cancel {
                            execute_transition_action(
                                cancel_action,
                                &mut overworld_ui,
                                &mut next_state,
                                &player_data,
                            );
                            return;
                        }
                    }
                }
            }
            OverworldState::Cutscene => {
                info!("Menu key pressed during cutscene, ignoring");
            }
        }
    }
}

fn evaluate_transition_condition(
    condition: &str,
    index: usize,
    player_data: &crate::core::data::PlayerData,
) -> bool {
    let condition = condition.trim();

    if condition.starts_with("index == ") {
        if let Some(num_str) = condition.strip_prefix("index == ") {
            let parts: Vec<&str> = num_str.split("&&").map(|s| s.trim()).collect();
            let index_part = parts[0];
            if let Ok(target_index) = index_part.parse::<usize>() {
                if index != target_index {
                    return false;
                }
                for part in parts.iter().skip(1) {
                    if *part == "!player.inventory.is_empty" && player_data.inventory.is_empty() {
                        return false;
                    }
                }
                return true;
            }
        }
    }

    true
}

fn execute_transition_action(
    action: &TransitionAction,
    overworld_ui: &mut OverworldUI,
    next_state: &mut ResMut<NextState<OverworldState>>,
    player_data: &crate::core::data::PlayerData,
) {
    match action {
        TransitionAction::GotoLayer(target_layer) => {
            let max_index = calculate_max_index_for_layer(target_layer, player_data);
            info!(
                "Transitioning to layer {} with max_index {}",
                target_layer, max_index
            );
            overworld_ui.set_layer(target_layer.clone(), max_index);
        }
        TransitionAction::PopState => {
            info!("Popping state, returning to Normal");
            next_state.set(OverworldState::Normal);
        }
        TransitionAction::PushState(state_name) => {
            info!("TODO: Push state {}", state_name);
        }
    }
}

fn calculate_max_index_for_layer(
    layer: &UILayer,
    player_data: &crate::core::data::PlayerData,
) -> usize {
    if layer == &UILayer::BACKPACK_MENU {
        UILayer::BACKPACK_MENU_OPTIONS.len()
    } else if layer == &UILayer::BACKPACK_ITEM {
        if player_data.inventory.len() < player_data.inventory_capacity {
            player_data.inventory.len()
        } else {
            player_data.inventory_capacity
        }
    } else if layer == &UILayer::BACKPACK_ITEM_CHOOSES {
        BackpackItemOption::count()
    } else if layer == &UILayer::BACKPACK_STATUS {
        1
    } else {
        1
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
            let max_index = overworld_ui.max_index() as isize - 1;
            if next_index < 0 {
                next_index = max_index;
            } else if next_index > max_index {
                next_index = 0;
            }
            overworld_ui.set_index(next_index as usize);
        }
    }
}
