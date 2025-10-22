mod core;
mod extra;

use crate::core::overworld::*;
use crate::core::setup::*;
use bevy::prelude::*;
use extra::markdown_asset_loader::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(MarkdownPlugin)
        .init_state::<AppState>()
        .add_plugins(SetupPlugin)
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
