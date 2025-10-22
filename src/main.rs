mod core;
mod extra;

use crate::core::overworld_plugin::*;
use crate::core::resource_plugin::*;
use bevy::prelude::*;
use extra::markdown_asset_loader::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(MarkdownPlugin)
        .init_state::<AppState>()
        .add_plugins(ResourcePlugin)
        .add_plugins(OverworldPlugin)
        .run();
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States)]
enum AppState {
    #[default]
    Setup,
    Menu,
    Overworld,
    Battle,
}
