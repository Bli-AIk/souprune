mod app_states;
mod core;
mod extra;

use std::default::*;

use bevy::app::PluginGroupBuilder;
use bevy::prelude::*;
use bevy::window::*;

use crate::core::*;
use app_states::AppState;

/// Get the default Bevy plugins with custom window size and image plugin settings.
///
/// 获取具有自定义窗口大小和图像插件设置的默认 Bevy 插件。
fn get_bevy_default_plugins() -> PluginGroupBuilder {
    let resolution_scale = setup::ResolutionScale::default();
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
        (extra::markdown::MarkdownPlugin, extra::toml::TomlPlugin)
    };
}

/// Get the third-party plugins used in the application.
///
/// 获取应用程序中使用的第三方插件。
macro_rules! get_third_plugins {
    () => {
        (
            leafwing_input_manager::prelude::InputManagerPlugin::<crate::core::input::Action>::default(),
            seldom_state::prelude::StateMachinePlugin::default(),
            bevy_ecs_tiled::prelude::TiledPlugin::default(),
        )
    };
}

/// Get the game-specific plugins.
///
/// 获取特定于游戏的插件。
macro_rules! get_game_plugins {
    () => {
        (
            CorePlugin,
            setup::SetupPlugin,
            overworld::OverworldPlugin,
            GlobalPlugin,
        )
    };
}
fn main() {
    App::new()
        .init_resource::<setup::ResolutionScale>()
        .add_plugins((
            get_bevy_default_plugins(),
            get_file_importer_plugins!(),
            get_third_plugins!(),
            #[cfg(feature = "debug")]
            extra::debug::DebugPlugin,
        ))
        .init_resource::<input::PlayerInputSettings>()
        .init_state::<AppState>()
        .add_plugins(get_game_plugins!())
        .run();
}
