#[cfg(feature = "debug")]
pub mod debug_player_hp {
    use crate::core::data::PlayerData;
    use bevy::prelude::*;

    pub(crate) fn setup_player_hp_debug(app: &mut App) {
        app.add_systems(Update, debug_player_hp_system);
    }

    fn debug_player_hp_system(
        input: Res<ButtonInput<KeyCode>>,
        mut player_data: ResMut<PlayerData>,
    ) {
        if input.just_pressed(KeyCode::F5) {
            let max = player_data.hp_max;
            let current = player_data.hp;

            // Logic: 1 -> Max/2 -> Max -> 1
            // 逻辑：1 -> 半血 -> 满血 -> 1
            let new_hp = if current == 1 {
                max / 2
            } else if current == max / 2 {
                max
            } else {
                1
            };

            player_data.hp = new_hp;
            info!("Debug: Player HP set to {}/{}", new_hp, max);
        }
    }
}
