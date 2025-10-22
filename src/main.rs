mod core;
mod extra;

use crate::core::overworld::*;
use crate::core::resource::*;
use crate::core::setup::*;
use bevy::prelude::*;
use bevy::window::*;
use extra::markdown_asset_loader::*;

fn main() {
    let resolution_scale = ResolutionScale::default();
    
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resolution: WindowResolution::new(320 * resolution_scale.get(), 240 * resolution_scale.get()),
                        resizable: false,
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(MarkdownPlugin)
        .init_resource::<ResolutionScale>()
        .init_state::<AppState>()
        .add_plugins((SetupPlugin, OverworldPlugin))
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
