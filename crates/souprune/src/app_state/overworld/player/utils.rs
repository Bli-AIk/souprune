use crate::app_state::overworld::character::components::PlayerControlled;
use crate::core::input::Action;
use bevy::prelude;
use bevy::prelude::{Query, With};
use leafwing_input_manager::action_state::ActionState;

pub fn is_player_walking(
    query: Query<&ActionState<Action>, With<PlayerControlled>>,
) -> prelude::Result<(), ()> {
    let action_state = query.single().map_err(|_| ())?;

    let up_pressed = action_state.pressed(&Action::Up);
    let down_pressed = action_state.pressed(&Action::Down);
    let left_pressed = action_state.pressed(&Action::Left);
    let right_pressed = action_state.pressed(&Action::Right);

    let has_vertical_input = (up_pressed && !down_pressed) || (down_pressed && !up_pressed);
    let has_horizontal_input = (left_pressed && !right_pressed) || (right_pressed && !left_pressed);

    if has_vertical_input || has_horizontal_input {
        Ok(())
    } else {
        Err(())
    }
}

pub fn is_player_running(
    query: Query<&ActionState<Action>, With<PlayerControlled>>,
) -> prelude::Result<(), ()> {
    let action_state = query.single().map_err(|_| ())?;
    if action_state.pressed(&Action::Cancel) {
        Ok(())
    } else {
        Err(())
    }
}
