mod inspector;

use bevy::app::{App, Plugin};

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, _app: &mut App) {
        #[cfg(feature = "debug")]
        {
            use inspector::debug_inspector;
            debug_inspector::setup_debug_features(_app);
        }
    }
}
