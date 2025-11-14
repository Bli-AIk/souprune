pub(crate) mod load_context;
pub(crate) mod params;
pub(crate) mod resources;
mod utils;

pub(crate) use resources::*;

use bevy::app::{App, Plugin};

pub(crate) struct SpritePlugin;

impl Plugin for SpritePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ModuleSpriteRegistry>();
    }
}
