pub(crate) mod actions;
pub(crate) mod resources;

pub(crate) use actions::*;
pub(crate) use resources::*;

use bevy::app::{App, Plugin};

pub(crate) struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerInputSettings>();
    }
}
