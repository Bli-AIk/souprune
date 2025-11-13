mod app_states;
mod core;
mod extra;

use crate::core::input::{Action, PlayerInputSettings};
use crate::core::overworld::*;
use crate::core::setup::*;
use crate::core::*;
use crate::extra::debug::DebugPlugin;
use crate::extra::toml::TomlPlugin;
use app_states::AppState;
use bevy::app::PluginGroupBuilder;
use bevy::prelude::*;
use bevy::window::*;
use bevy_ecs_tiled::prelude::*;
use extra::markdown::*;
use leafwing_input_manager::prelude::*;
use seldom_state::prelude::*;

/// Get the default Bevy plugins with custom window size and image plugin settings.
///
/// 获取具有自定义窗口大小和图像插件设置的默认 Bevy 插件。
fn get_bevy_default_plugins() -> PluginGroupBuilder {
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

/// Get the file importer plugins used in the application.
///
/// 获取应用程序中使用的文件导入器插件。
macro_rules! get_file_importer_plugins {
    () => {
        (
            MarkdownPlugin,
            TomlPlugin,
        )
    };
}

/// Get the third-party plugins used in the application.
///
/// 获取应用程序中使用的第三方插件。
macro_rules! get_third_plugins {
    () => {
        (
            get_file_importer_plugins!(),
            InputManagerPlugin::<Action>::default(),
            StateMachinePlugin::default(),
            TiledPlugin::default(),
        )
    };
}

/// Get the game-specific plugins.
///
/// 获取特定于游戏的插件。
macro_rules! get_game_plugins {
    () => {
        (CorePlugin, SetupPlugin, OverworldPlugin, GlobalPlugin)
    };
}
fn main() {
    App::new()
        .add_plugins((
            get_bevy_default_plugins(),
            get_third_plugins!(),
            #[cfg(feature = "debug")]
            DebugPlugin,
        ))
        .init_resource::<ResolutionScale>()
        .init_resource::<PlayerInputSettings>()
        .init_state::<AppState>()
        .add_plugins(get_game_plugins!())
        .run();
}
