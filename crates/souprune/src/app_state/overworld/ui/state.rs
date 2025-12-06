use super::components::{OverworldUI, UILayer, UILayerNavigationConfig};
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

                match overworld_ui.layer() {
                    layer if layer == &UILayer::BACKPACK_MENU => {
                        if action_state.just_pressed(&Action::Menu)
                            || action_state.just_pressed(&Action::Cancel)
                        {
                            info!("Leaving Backpack menu and returning to Normal state");
                            next_state.set(OverworldState::Normal);
                            return;
                        }

                        if action_state.just_pressed(&Action::Confirm) {
                            match UILayer::BACKPACK_MENU_OPTIONS.get(overworld_ui.index()) {
                                Some(layer) if layer == &UILayer::BACKPACK_ITEM => {
                                    if player_data.inventory.is_empty() {
                                        return;
                                    }
                                    info!("Opening Backpack item layer");
                                    overworld_ui.set_layer(
                                        UILayer::BACKPACK_ITEM,
                                        if player_data.inventory.len()
                                            < player_data.inventory_capacity
                                        {
                                            player_data.inventory.len()
                                        } else {
                                            player_data.inventory_capacity
                                        },
                                    );
                                }
                                Some(layer) if layer == &UILayer::BACKPACK_STATUS => {
                                    info!("Opening Backpack status layer");
                                    overworld_ui.set_layer(UILayer::BACKPACK_STATUS, 1);
                                }
                                _ => {
                                    warn!(
                                        "Unhandled Backpack menu index {} when confirming",
                                        overworld_ui.index()
                                    );
                                }
                            }
                        }
                    }
                    layer if layer == &UILayer::BACKPACK_ITEM => {
                        if action_state.just_pressed(&Action::Cancel) {
                            info!("Returning to Backpack menu layer from Item layer");
                            overworld_ui.set_layer(
                                UILayer::BACKPACK_MENU,
                                UILayer::BACKPACK_MENU_OPTIONS.len(),
                            );
                            return;
                        }

                        if action_state.just_pressed(&Action::Confirm) {
                            let item_index = overworld_ui.index();
                            if item_index >= player_data.inventory.len() {
                                warn!(
                                    "Confirmed item index {} out of bounds (max {})",
                                    item_index,
                                    player_data.inventory.len()
                                );
                                return;
                            }
                            let item_id = &player_data.inventory[item_index];
                            info!("TODO: Use item {:?} from backpack", item_id);

                            overworld_ui.set_layer(
                                UILayer::BACKPACK_ITEM_CHOOSES,
                                3, //TODO: 不使用硬编码
                            );
                        }
                    }
                    _ => {
                        if action_state.just_pressed(&Action::Cancel) {
                            info!(
                                "Returning to Backpack menu layer from {}",
                                overworld_ui.layer()
                            );
                            overworld_ui.set_layer(
                                UILayer::BACKPACK_MENU,
                                UILayer::BACKPACK_MENU_OPTIONS.len(),
                            );
                            return;
                        }

                        if action_state.just_pressed(&Action::Confirm) {
                            info!(
                                "TODO: confirm action handling for Backpack layer {}",
                                overworld_ui.layer()
                            );
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
