//! # main.rs
//!
//! # main.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This standalone file houses the entry point without any submodules.
//!
//! main.rs 是一个没有子模块的独立入口文件。
//!
//! ## Source File Overview
//!
//! ## 源文件概述
//!
//! It initializes the Bevy app, registers resources and states, and manages every plugin.
//!
//! 文件负责初始化 Bevy 应用、注册资源与状态，并管理所有插件。

mod app_state;
mod core;
mod extra;

use std::default::*;

use crate::core::*;
use crate::extra::multi_source::MultiSourceAssetReader;
use app_state::{app_setup, overworld};
use bevy::app::PluginGroupBuilder;
use bevy::asset::io::file::FileAssetReader;
use bevy::asset::io::{AssetSource, AssetSourceId};
use bevy::prelude::*;
use bevy::window::*;
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
fn get_bevy_default_plugins() -> PluginGroupBuilder {
    let resolution_scale = app_setup::ResolutionScale::default();
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
        // Disable the default LogPlugin because we initialize our own tracing subscriber
        //
        // 禁用默认的 LogPlugin，因为我们初始化了自己的追踪订阅者
        .disable::<bevy::log::LogPlugin>()
}

/// Get the file importer plugins used in the application.
///
/// 获取应用程序中使用的文件导入器插件。
macro_rules! get_file_importer_plugins {
    () => {
        (
            extra::markdown::MarkdownPlugin,
            extra::toml::TomlPlugin,
            extra::mortar::MortarExtraPlugin,
        )
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
            bevy_smud::SmudPlugin,
            bevy_rich_text3d::Text3dPlugin{
                default_atlas_dimension: (1024, 1024),
                load_system_fonts: false,
                ..Default::default()
            }

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
            app_setup::AppSetupPlugin,
            overworld::OverworldPlugin,
            GlobalPlugin,
        )
    };
}
fn main() {
    // Initialize logging and keep the guard alive
    //
    // 初始化日志记录并保持 guard 存活
    let _log_guard = setup_logging().expect("Failed to initialize logging");

    App::new()
        // TODO: 读取 mod 配置并加载正确的项目
        .register_asset_source(
            AssetSourceId::Default,
            AssetSource::build().with_reader(|| {
                let project_name = "example_mod";

                let project_path = format!("projects/{}", project_name);

                let mut readers = vec![
                    // Priority 1: Distribution / Standalone (folder next to executable)
                    //
                    // 优先级 1：分发/独立（可执行文件旁边的文件夹）
                    FileAssetReader::new(&project_path),
                ];

                // Priority 2: Development (absolute path based on source location, strictly for debug builds)
                //
                // 优先级 2：开发（基于源位置的绝对路径，严格用于调试构建）
                #[cfg(feature = "debug")]
                {
                    readers.push(FileAssetReader::new(
                        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                            .join("../../")
                            .join(&project_path),
                    ));
                }

                // Priority 3: Core Fallback (embedded assets)
                //
                // 优先级 3：核心回退（嵌入式资产）
                readers.push(FileAssetReader::new("assets"));
                readers.push(FileAssetReader::new("crates/souprune/assets"));

                Box::new(MultiSourceAssetReader::new(readers))
            }),
        )
        .init_resource::<app_setup::ResolutionScale>()
        .add_plugins((
            get_bevy_default_plugins(),
            get_file_importer_plugins!(),
            get_third_plugins!(),
            #[cfg(feature = "debug")]
            extra::debug::DebugPlugin,
        ))
        .insert_resource(bevy_rich_text3d::LoadFonts {
            font_directories: vec!["crates/souprune/assets/fonts".to_owned()],
            ..Default::default()
        })
        .init_resource::<input::PlayerInputSettings>()
        .init_state::<app_state::AppState>()
        .add_plugins(get_game_plugins!())
        .run();
}
