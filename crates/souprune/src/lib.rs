pub mod app_state;
pub mod config;
pub mod core;
pub mod extra;

pub use crate::app_state::overworld::player::config::PlayerBehavior;
pub use crate::app_state::overworld::ui::layout::{
    BoxCursorPositionDef, FloatOrExpr, IndexBoundDef, LayerTransitionsDef, NavigationRuleDef,
    SerializableVec3, TransitionActionDef, TransitionRuleDef, UIBoxLogicDef, UILayoutAsset,
    UINodeDef,
};
pub use crate::core::basic_components::Direction;
pub use crate::core::character_asset::{
    AnimationConfigAsset, CharacterAsset, StateAnimationMapping,
};
pub use crate::core::input::actions::Action;
pub use crate::core::item::{Item, ItemAsset, ItemEffect, ItemRegistry, ItemType};

use std::default::Default;

use crate::core::*;
use crate::extra::multi_source::MultiSourceAssetReader;
use app_state::{app_setup, overworld};
use bevy::app::PluginGroupBuilder;
use bevy::asset::io::file::FileAssetReader;
use bevy::asset::io::{AssetSource, AssetSourceId};
use bevy::prelude::*;
use bevy::window::{Window, WindowPlugin, WindowResolution};
use chrono::Local;
use tracing_subscriber::{EnvFilter, Layer, layer::SubscriberExt, util::SubscriberInitExt};

/// Sets up the logging system with both stdout and file output.
///
/// 设置同时包含标准输出和文件输出的日志系统。
fn setup_logging() -> anyhow::Result<tracing_appender::non_blocking::WorkerGuard> {
    // Generate a timestamped filename for this run
    //
    // 为本次运行生成带时间戳的文件名
    let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S");
    let filename = format!("souprune_{}.log", timestamp);
    let log_dir = std::path::Path::new("logs");

    // Ensure the logs directory exists
    //
    // 确保 logs 目录存在
    std::fs::create_dir_all(log_dir)?;

    let file_path = log_dir.join(filename);
    let file = std::fs::File::create(file_path)?;

    // Create a non-blocking writer for the file
    //
    // 为文件创建一个非阻塞写入器
    let (non_blocking, guard) = tracing_appender::non_blocking(file);

    // Filter configuration
    // Default to INFO, but silence noisy libraries
    //
    // 过滤器配置
    // 默认为 INFO，但静音嘈杂的库
    let filter = EnvFilter::builder()
        .with_default_directive(tracing::Level::INFO.into())
        .from_env_lossy()
        .add_directive("souprune=info".parse()?)
        .add_directive("wgpu=error".parse()?)
        .add_directive("naga=warn".parse()?)
        .add_directive("bevy=info".parse()?)
        .add_directive("bevy_render=warn".parse()?)
        .add_directive("bevy_app=warn".parse()?)
        .add_directive("bevy_ecs=warn".parse()?);

    // File layer: writes to the log file
    //
    // 文件层：写入日志文件
    let file_layer = tracing_subscriber::fmt::layer()
        .with_ansi(false)
        .with_writer(non_blocking)
        .with_filter(filter.clone());

    // Stdout layer: writes to the console (standard Bevy behavior)
    //
    // 标准输出层：写入控制台（标准 Bevy 行为）
    let stdout_layer = tracing_subscriber::fmt::layer()
        .pretty()
        .with_filter(filter);

    // Initialize the registry with both layers
    //
    // 使用两个层初始化注册表
    tracing_subscriber::registry()
        .with(stdout_layer)
        .with(file_layer)
        .init();

    Ok(guard)
}

/// Get the default Bevy plugins with custom window size and image plugin settings.
///
/// 获取具有自定义窗口大小和图像插件设置的默认 Bevy 插件。
fn get_bevy_default_plugins(resolution_scale: u32) -> PluginGroupBuilder {
    DefaultPlugins
        .set(ImagePlugin::default_nearest())
        .set(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(320 * resolution_scale, 240 * resolution_scale),
                resizable: false,
                ..default()
            }),
            ..default()
        })
        // Disable the default LogPlugin because we initialize our own tracing subscriber
        //
        // 禁用默认的 LogPlugin，因为我们初始化了自己的追踪订阅者
        .disable::<bevy::log::LogPlugin>()
}

/// Get the file importer plugins used in the application.
///
/// 获取应用程序中使用的文件导入器插件。
fn get_file_importer_plugins() -> (
    extra::markdown::MarkdownPlugin,
    extra::toml::TomlPlugin,
    extra::mortar::MortarExtraPlugin,
) {
    (
        extra::markdown::MarkdownPlugin,
        extra::toml::TomlPlugin,
        extra::mortar::MortarExtraPlugin,
    )
}

/// Get the third-party plugins used in the application.
///
/// 获取应用程序中使用的第三方插件。
fn get_third_plugins() -> (
    leafwing_input_manager::prelude::InputManagerPlugin<crate::core::input::Action>,
    seldom_state::prelude::StateMachinePlugin,
    bevy_ecs_tiled::prelude::TiledPlugin,
    bevy_smud::SmudPlugin,
    bevy_rich_text3d::Text3dPlugin,
) {
    (
        leafwing_input_manager::prelude::InputManagerPlugin::<crate::core::input::Action>::default(
        ),
        seldom_state::prelude::StateMachinePlugin::default(),
        bevy_ecs_tiled::prelude::TiledPlugin::default(),
        bevy_smud::SmudPlugin,
        bevy_rich_text3d::Text3dPlugin {
            default_atlas_dimension: (1024, 1024),
            load_system_fonts: false,
            ..Default::default()
        },
    )
}

/// Get the game-specific plugins.
///
/// 获取特定于游戏的插件。
fn get_game_plugins() -> (
    CorePlugin,
    app_setup::AppSetupPlugin,
    overworld::OverworldPlugin,
    GlobalPlugin,
) {
    (
        CorePlugin,
        app_setup::AppSetupPlugin,
        overworld::OverworldPlugin,
        GlobalPlugin,
    )
}

pub fn run() {
    // Initialize logging and keep the guard alive
    //
    // 初始化日志记录并保持 guard 存活
    let _log_guard = setup_logging().expect("Failed to initialize logging");

    let config = config::load_config();
    let resolution_scale = config.window.resolution_scale;
    let project_name = config.project.mod_name.clone();
    let language = config.project.language.clone();

    App::new()
        // TODO: 读取 mod 配置并加载正确的项目
        .register_asset_source(
            AssetSourceId::Default,
            AssetSource::build().with_reader(move || {
                let roots = config::get_asset_roots(&project_name);
                let readers = roots.into_iter().map(FileAssetReader::new).collect();
                Box::new(MultiSourceAssetReader::new(readers))
            }),
        )
        .insert_resource(app_setup::ResolutionScale(resolution_scale as u32))
        .insert_resource(extra::mortar::CurrentLocale(language))
        .add_plugins((
            get_bevy_default_plugins(resolution_scale),
            get_file_importer_plugins(),
            get_third_plugins(),
            #[cfg(feature = "debug")]
            extra::debug::DebugPlugin,
        ))
        .insert_resource(bevy_rich_text3d::LoadFonts {
            font_directories: vec!["crates/souprune/assets/fonts".to_owned()],
            ..Default::default()
        })
        .init_resource::<input::PlayerInputSettings>()
        .init_state::<app_state::AppState>()
        .add_plugins(get_game_plugins())
        .run();
}
