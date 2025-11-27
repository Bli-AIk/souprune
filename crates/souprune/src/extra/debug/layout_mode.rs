#[cfg(feature = "debug")]
pub mod debug_layout_mode {
    use bevy::prelude::*;

    use crate::app_state::overworld::ui::components::UILayoutMode;

    /// Register the layout-mode debug controls.
    ///
    /// 注册布局模式调试控制。
    pub fn setup_layout_mode_debug(app: &mut App) {
        app.add_systems(Update, cycle_layout_mode_system);
    }

    fn cycle_layout_mode_system(
        keyboard: Res<ButtonInput<KeyCode>>,
        layout_mode: Option<ResMut<UILayoutMode>>,
    ) {
        if keyboard.just_pressed(KeyCode::F5)
            && let Some(mut layout_mode) = layout_mode
        {
            let new_mode = match *layout_mode {
                UILayoutMode::Original => UILayoutMode::Unified,
                UILayoutMode::Unified => UILayoutMode::Original,
            };

            *layout_mode = new_mode;
            info!("Overworld UI layout mode: {:?}", new_mode);
        }
    }
}
