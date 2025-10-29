mod core;
mod extra;

use crate::core::input::{Action, PlayerInputSettings};
use crate::core::overworld::*;
use crate::core::setup::*;
use crate::core::*;
use bevy::app::PluginGroupBuilder;
use bevy::prelude::*;
use bevy::window::*;
use extra::markdown_asset_loader::*;
use leafwing_input_manager::prelude::*;
use seldom_state::prelude::*;

fn main() {
    let default_plugins = get_default_plugins();

    App::new()
        .add_plugins((
            default_plugins,
            MarkdownPlugin,
            InputManagerPlugin::<Action>::default(),
            StateMachinePlugin::default(),
        ))
        .init_resource::<ResolutionScale>()
        .init_resource::<PlayerInputSettings>()
        .init_state::<AppState>()
        .add_plugins((CorePlugin, SetupPlugin, OverworldPlugin))
        .run();
}

fn get_default_plugins() -> PluginGroupBuilder {
    let resolution_scale = ResolutionScale::default();
    DefaultPlugins
        .set(ImagePlugin::default_nearest())
        .set(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(
                    320 * resolution_scale.get(),
                    240 * resolution_scale.get(),
                ),
                resizable: false,
                ..default()
            }),
            ..default()
        })
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, States)]
enum AppState {
    #[default]
    Setup,
    Menu,
    Overworld,
    Battle,
}
