//! # lib.rs
//!
//! # lib.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This is the main library entry point for the Souprune framework. It orchestrates the application startup, including logging initialization, plugin registration (Bevy defaults, third-party, and game-specific), and configuring the asset system for multi-source loading.
//!
//! 这是 Souprune 框架的主要库入口点。它负责协调应用程序的启动，包括日志初始化、插件注册（Bevy 默认插件、第三方插件和游戏特定插件），以及配置用于多源加载的资产系统。

pub mod app_state;
pub mod config;
pub mod core;
pub mod extra;

pub use crate::app_state::overworld::player::config::PlayerBehavior;
pub use crate::core::basic_components::Direction;
pub use crate::core::character_asset::{
    AnimationConfigAsset, CharacterAsset, StateAnimationMapping,
};
pub use crate::core::input::actions::Action;
pub use crate::core::item::{Item, ItemAsset, ItemEffect, ItemRegistry, ItemType};
pub use crate::core::save::{
    LoadCompleteEvent, LoadGameEvent, SaveCompleteEvent, SaveConfig, SaveData, SaveGameEvent,
    SaveMetadata, SaveSlot, Saveable,
};
pub use crate::core::view::layout::{
    BoxCursorPositionDef, FloatOrExpr, IndexBoundDef, LayerTransitionsDef, NavigationRuleDef,
    SerializableVec3, TransitionActionDef, TransitionRuleDef, UIBoxLogicDef, ViewLayoutAsset,
    ViewNodeDef,
};

use std::default::Default;

use crate::core::*;
use crate::extra::multi_source::MultiSourceAssetReader;
use app_state::{app_setup, battle, overworld};
use bevy::app::PluginGroupBuilder;
use bevy::asset::io::file::{FileAssetReader, FileWatcher};
use bevy::asset::io::{AssetSource, AssetSourceId};
use bevy::prelude::*;
use bevy::render::RenderPlugin;
use bevy::render::settings::{InstanceFlags, RenderCreation, WgpuSettings};
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
        .add_directive("bevy_ecs=warn".parse()?)
        .add_directive("bevy_alight_motion=warn".parse()?);

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
fn get_bevy_default_plugins(
    resolution_scale: u32,
    render_config: &config::RenderConfig,
) -> PluginGroupBuilder {
    let base_width = render_config.base_resolution_width;
    let base_height = render_config.base_resolution_height;

    let mut plugins = DefaultPlugins
        .set(ImagePlugin::default_nearest())
        .set(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(
                    base_width * resolution_scale,
                    base_height * resolution_scale,
                ),
                resizable: false,
                title: "SoupRune".into(),
                ..default()
            }),
            ..default()
        })
        .disable::<bevy::log::LogPlugin>();

    // On some devices, enabling the WGPU verification layer can cause a panic.
    // This may be caused by driver incompatibility or resource limitations.
    // Therefore, if the unsafe_gpu feature is enabled, we will forcibly disable the verification layer
    // to improve compatibility, although this will sacrifice some debugging information and stability.
    // We only recommend using this feature when running on known safe GPUs, and it is not
    // recommended for production environments.
    //
    // 在某些设备上，启用 WGPU 验证层会导致 panic。这可能是由于驱动程序不兼容或资源限制引起的。
    // 因此，如果启用了 unsafe_gpu 特性，我们将强制关闭验证层以提高兼容性，尽管这会牺牲一些调试信息和稳定性。
    // 我们仅建议在已知安全的 GPU 上运行时使用此特性，并且不建议在生产环境中使用。
    #[cfg(feature = "unsafe_gpu")]
    {
        info!("【SYSTEM】Unsafe GPU Mode Detected: Forcing WGPU Validation Layers OFF.");

        plugins = plugins.set(RenderPlugin {
            render_creation: RenderCreation::Automatic(WgpuSettings {
                instance_flags: InstanceFlags::empty(),
                ..default()
            }),
            ..default()
        });
    }

    plugins
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
    leafwing_input_manager::prelude::InputManagerPlugin<Action>,
    seldom_state::prelude::StateMachinePlugin,
    bevy_ecs_tiled::prelude::TiledPlugin,
    bevy_smud::SmudPlugin,
    bevy_rich_text3d::Text3dPlugin,
    bevy_alight_motion::prelude::AlightMotionPlugin,
) {
    (
        leafwing_input_manager::prelude::InputManagerPlugin::<Action>::default(),
        seldom_state::prelude::StateMachinePlugin::default(),
        bevy_ecs_tiled::prelude::TiledPlugin::default(),
        bevy_smud::SmudPlugin,
        bevy_rich_text3d::Text3dPlugin {
            default_atlas_dimension: (1024, 1024),
            load_system_fonts: false,
            ..Default::default()
        },
        bevy_alight_motion::prelude::AlightMotionPlugin,
    )
}

/// Get the game-specific plugins.
///
/// 获取特定于游戏的插件。
fn get_game_plugins() -> (
    CorePlugin,
    app_setup::AppSetupPlugin,
    overworld::OverworldPlugin,
    battle::BattlePlugin,
    GlobalPlugin,
    mod_system::ModPlugin,
) {
    (
        CorePlugin,
        app_setup::AppSetupPlugin,
        overworld::OverworldPlugin,
        battle::BattlePlugin,
        GlobalPlugin,
        mod_system::ModPlugin,
    )
}

pub fn run() {
    // Initialize logging and keep the guard alive
    //
    // 初始化日志记录并保持 guard 存活
    let _log_guard = setup_logging().expect("Failed to initialize logging");

    #[cfg(feature = "unsafe_gpu")]
    info!("Starting SoupRune with [unsafe_gpu] feature enabled.");

    let config = config::load_config();

    // Data
    let resolution_scale = config.window.resolution_scale;
    let project_name = config.project.mod_name.clone();
    let language = config.project.language.clone();

    // Config
    let render_config = config.render.clone();

    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .register_asset_source(
            AssetSourceId::Default,
            AssetSource::build()
                .with_reader(move || {
                    let roots = config::get_asset_roots(&project_name);
                    let readers = roots.into_iter().map(FileAssetReader::new).collect();
                    Box::new(MultiSourceAssetReader::new(readers))
                })
                .with_watcher(|sender| {
                    // Watch the primary project directory for hot reloading
                    //
                    // 监听主项目目录以实现热重载
                    let config = config::load_config();
                    let roots = config::get_asset_roots(&config.project.mod_name);

                    // Find the first root that actually exists on disk
                    //
                    // 找到磁盘上实际存在的第一个根目录
                    for root in &roots {
                        if root.exists() {
                            info!("[Hot Reload] Setting up file watcher for: {:?}", root);
                            match FileWatcher::new(
                                root.clone(),
                                sender.clone(),
                                std::time::Duration::from_millis(300),
                            ) {
                                Ok(watcher) => {
                                    return Some(
                                        Box::new(watcher) as Box<dyn bevy::asset::io::AssetWatcher>
                                    );
                                }
                                Err(e) => {
                                    warn!(
                                        "[Hot Reload] Failed to create file watcher for {:?}: {:?}",
                                        root, e
                                    );
                                }
                            }
                        }
                    }
                    error!("[Hot Reload] No valid asset root found for file watching");
                    None
                }),
        )
        .insert_resource(app_setup::ResolutionScale(resolution_scale))
        .insert_resource(extra::mortar::CurrentLocale(language))
        .add_plugins((
            get_bevy_default_plugins(resolution_scale, &render_config),
            get_file_importer_plugins(),
            get_third_plugins(),
            #[cfg(feature = "debug")]
            extra::debug::DebugPlugin,
            #[cfg(feature = "debug")]
            bevy_brp_extras::BrpExtrasPlugin,
        ))
        .insert_resource(config)
        .insert_resource(bevy_rich_text3d::LoadFonts {
            font_directories: vec!["crates/souprune/assets/fonts".to_owned()],
            ..Default::default()
        })
        .init_resource::<input::PlayerInputSettings>()
        .init_state::<app_state::AppState>()
        .configure_sets(
            Update,
            view::UIUpdate.run_if(
                in_state(app_state::AppState::Overworld).or(in_state(app_state::AppState::Battle)),
            ),
        )
        .add_plugins(get_game_plugins())
        .run();
}
