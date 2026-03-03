#![allow(
    dead_code,
    clippy::too_many_arguments,
    clippy::type_complexity,
    unexpected_cfgs
)]
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
pub use crate::core::item::{Item, ItemEffect, ItemRegistry, ItemType};
pub use crate::core::view::layout::{
    FloatOrExpr, SerializableVec3, ViewBoxLogicDef, ViewLayoutAsset, ViewNodeDef,
};

use std::default::Default;

use crate::core::*;
use crate::extra::multi_source::MultiSourceAssetReader;
use app_state::{app_setup, battle, overworld};
use bevy::app::PluginGroupBuilder;
use bevy::asset::io::file::{FileAssetReader, FileWatcher};
use bevy::asset::io::{AssetSourceBuilder, AssetSourceId};
use bevy::ecs::schedule::{InternedScheduleLabel, ScheduleLabel};
use bevy::prelude::*;
#[cfg(any(feature = "unsafe_gpu", target_os = "android"))]
use bevy::render::RenderPlugin;
#[cfg(feature = "unsafe_gpu")]
use bevy::render::settings::InstanceFlags;
#[cfg(any(feature = "unsafe_gpu", target_os = "android"))]
use bevy::render::settings::{RenderCreation, WgpuSettings};
use bevy::window::{Window, WindowPlugin, WindowResolution};

use chrono::Local;
use tracing_subscriber::EnvFilter;

/// 游戏逻辑系统的目标调度器。
///
/// 游戏本体使用 `Update`，编辑器使用 `GameSchedule`（由 bevy_workbench 控制执行时机）。
/// 所有游戏 Plugin 在 `build()` 中读取此资源，将系统注册到指定的调度器。
#[derive(Resource, Clone)]
pub struct GameUpdateSchedule(pub InternedScheduleLabel);

impl Default for GameUpdateSchedule {
    fn default() -> Self {
        Self(Update.intern())
    }
}

/// 从 App 中获取游戏逻辑调度器标签。
/// 如果未设置 `GameUpdateSchedule` 资源，返回 `Update`。
pub fn game_schedule(app: &App) -> InternedScheduleLabel {
    app.world()
        .get_resource::<GameUpdateSchedule>()
        .map_or(Update.intern(), |s| s.0)
}

/// Sets up the logging system with both stdout and file output.
/// When `trace_tracy` feature is enabled, also adds Tracy profiler layer.
///
/// 设置同时包含标准输出和文件输出的日志系统。
/// 当启用 `trace_tracy` feature 时，还会添加 Tracy profiler 层。
#[cfg(not(feature = "trace_tracy"))]
fn setup_logging() -> anyhow::Result<tracing_appender::non_blocking::WorkerGuard> {
    use tracing_subscriber::{Layer, layer::SubscriberExt, util::SubscriberInitExt};
    // Generate a timestamped filename for this run
    //
    // 为本次运行生成带时间戳的文件名
    let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S");
    let filename = format!("souprune_{}.log", timestamp);
    let log_dir: std::path::PathBuf = if cfg!(target_os = "android") {
        "/sdcard/SoupRune".into()
    } else {
        "logs".into()
    };

    // Ensure the logs directory exists (only works on desktop)
    #[cfg(not(target_os = "android"))]
    std::fs::create_dir_all(&log_dir)?;

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

/// Sets up the logging system with Tracy profiler support.
/// This version is used when `trace_tracy` feature is enabled.
///
/// 设置带有 Tracy profiler 支持的日志系统。
/// 当启用 `trace_tracy` feature 时使用此版本。
#[cfg(feature = "trace_tracy")]
fn setup_logging() -> anyhow::Result<tracing_appender::non_blocking::WorkerGuard> {
    use tracing_subscriber::prelude::*;

    // Generate a timestamped filename for this run
    let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S");
    let filename = format!("souprune_{}.log", timestamp);
    let log_dir: std::path::PathBuf = if cfg!(target_os = "android") {
        "/sdcard/SoupRune".into()
    } else {
        "logs".into()
    };
    #[cfg(not(target_os = "android"))]
    std::fs::create_dir_all(&log_dir)?;

    let file_path = log_dir.join(filename);
    let file = std::fs::File::create(file_path)?;
    let (non_blocking, guard) = tracing_appender::non_blocking(file);

    // Filter configuration - use INFO level for profiling
    let filter = EnvFilter::builder()
        .with_default_directive(tracing::Level::INFO.into())
        .from_env_lossy()
        .add_directive("wgpu=error".parse()?)
        .add_directive("naga=warn".parse()?);

    // File layer
    let file_layer = tracing_subscriber::fmt::layer()
        .with_ansi(false)
        .with_writer(non_blocking)
        .with_filter(filter.clone());

    // Stdout layer
    let stdout_layer = tracing_subscriber::fmt::layer()
        .pretty()
        .with_filter(filter);

    // Tracy layer for profiling
    let tracy_layer = tracing_tracy::TracyLayer::default();

    // Initialize with all layers including Tracy
    tracing_subscriber::registry()
        .with(stdout_layer)
        .with(file_layer)
        .with(tracy_layer)
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

    #[allow(unused_mut)]
    let mut plugins = DefaultPlugins
        .set(ImagePlugin::default_nearest())
        .set(bevy::asset::AssetPlugin {
            // Mod system loads from external storage (/sdcard/SoupRune/...).
            // Must allow absolute paths for asset loading.
            unapproved_path_mode: bevy::asset::UnapprovedPathMode::Allow,
            ..default()
        })
        .set(WindowPlugin {
            primary_window: Some(Window {
                #[cfg(not(target_os = "android"))]
                resolution: WindowResolution::new(
                    base_width * resolution_scale,
                    base_height * resolution_scale,
                ),
                resizable: false,
                title: "SoupRune".into(),
                #[cfg(target_os = "android")]
                mode: bevy::window::WindowMode::BorderlessFullscreen(
                    bevy::window::MonitorSelection::Primary,
                ),
                ..default()
            }),
            ..default()
        });

    // On all platforms: disable LogPlugin, use our own file-based logging
    //
    // 所有平台：禁用 LogPlugin，使用自定义文件日志
    plugins = plugins.disable::<bevy::log::LogPlugin>();

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

    // Android: use GLES3 backend with minimal feature set. Disable PBR entirely.
    // Huawei BiSheng GPU driver: stack corruption in libGLES_v200.so during queue submit.
    // Mitigations:
    //   - Compatibility priority: minimal wgpu features (avoid enabling buggy GL extensions)
    //   - constrained_limits: no storage buffers (VERTEX_STORAGE not supported)
    //   - InstanceFlags::empty(): disable wgpu validation (reduces GL API calls)
    //   - Gles3MinorVersion::Version0: force GLES 3.0 (avoid 3.1+ compute/storage features)
    //   - No PBR/Gizmo plugins
    #[cfg(target_os = "android")]
    #[cfg(not(feature = "unsafe_gpu"))]
    {
        use bevy::render::settings::InstanceFlags;
        use bevy::render::settings::{
            Backends, Gles3MinorVersion, WgpuLimits, WgpuSettingsPriority,
        };
        let mut no_storage = WgpuLimits::default();
        no_storage.max_storage_buffers_per_shader_stage = 0;
        no_storage.max_storage_textures_per_shader_stage = 0;
        no_storage.max_dynamic_storage_buffers_per_pipeline_layout = 0;
        no_storage.max_storage_buffer_binding_size = 0;
        plugins = plugins
            .set(RenderPlugin {
                render_creation: RenderCreation::Automatic(WgpuSettings {
                    backends: Some(Backends::GL),
                    priority: WgpuSettingsPriority::Compatibility,
                    instance_flags: InstanceFlags::empty(),
                    gles3_minor_version: Gles3MinorVersion::Version0,
                    constrained_limits: Some(no_storage),
                    ..default()
                }),
                ..default()
            })
            .disable::<bevy::pbr::PbrPlugin>()
            .disable::<bevy::gizmos::GizmoPlugin>()
            .disable::<bevy::gizmos_render::GizmoRenderPlugin>();
    }

    plugins
}

/// Get the file importer plugins used in the application.
///
/// 获取应用程序中使用的文件导入器插件。
pub fn get_file_importer_plugins() -> (
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
pub fn get_third_plugins() -> (
    leafwing_input_manager::prelude::InputManagerPlugin<Action>,
    bevy_ecs_tiled::prelude::TiledPlugin,
    bevy_rich_text3d::Text3dPlugin,
    bevy_alight_motion::prelude::AlightMotionPlugin,
    bevy_tween::DefaultTweenPlugins,
) {
    (
        leafwing_input_manager::prelude::InputManagerPlugin::<Action>::default(),
        bevy_ecs_tiled::prelude::TiledPlugin::default(),
        bevy_rich_text3d::Text3dPlugin {
            default_atlas_dimension: (1024, 1024),
            load_system_fonts: false,
            ..Default::default()
        },
        bevy_alight_motion::prelude::AlightMotionPlugin,
        bevy_tween::DefaultTweenPlugins,
    )
}

/// Get the game-specific plugins.
///
/// 获取特定于游戏的插件。
pub fn get_game_plugins() -> (
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

/// 为编辑器初始化游戏基础设施。
///
/// 添加核心插件、视图系统、第三方插件和必要的状态资源，
/// 使 Sequencer 能在编辑器的 Play 模式下正常运行。
///
/// 调用方需要在此之前插入 `SoupruneConfig` 资源。
pub fn init_editor_game_systems(app: &mut App) {
    let schedule = game_schedule(app);

    // 第三方游戏插件
    app.add_plugins((
        leafwing_input_manager::prelude::InputManagerPlugin::<Action>::default(),
        bevy_ecs_tiled::prelude::TiledPlugin::default(),
        bevy_rich_text3d::Text3dPlugin {
            default_atlas_dimension: (1024, 1024),
            load_system_fonts: false,
            ..Default::default()
        },
        bevy_alight_motion::prelude::AlightMotionPlugin,
    ));

    // 核心游戏系统（音频、摄像机、碰撞、动画、对话、FRE 桥接等）
    app.add_plugins(core::CorePlugin);

    // Mod 系统（DanmakuRegistry、BehaviorRegistry）
    app.add_plugins(core::mod_system::ModPlugin);

    // View 系统（SpawnView / ModifyViewElement 章节依赖）
    app.add_plugins(core::view::CoreViewPlugin);

    // 应用状态（FRE 桥接依赖）
    // 编辑器直接跳到 Running，游戏系统（ViewUpdate 等）才会执行
    app.init_state::<app_state::AppState>()
        .init_state::<app_state::SequenceSubState>()
        .init_resource::<app_state::SequenceMode>()
        .init_resource::<app_state::overworld::trigger::RuleActionDefs>()
        .insert_resource(core::input::ActionRegistry::new())
        .add_message::<app_state::ModeChanged>()
        .add_systems(
            PreUpdate,
            (
                app_state::detect_mode_changes,
                app_state::cleanup_mode_scoped_entities,
            )
                .chain(),
        )
        .configure_sets(
            schedule,
            core::view::ViewUpdate.run_if(in_state(app_state::AppState::Running)),
        )
        .configure_sets(
            schedule,
            core::sequencer::SequencerUpdate.run_if(in_state(app_state::AppState::Running)),
        )
        .add_systems(
            Startup,
            |mut next: ResMut<NextState<app_state::AppState>>| {
                next.set(app_state::AppState::Running);
            },
        );

    // 文件导入器插件
    app.add_plugins(get_file_importer_plugins());
}

/// 从 SoupruneConfig 加载输入配置并插入所有输入相关资源。
///
/// 插入：ActionRegistry, PlayerInputSettings, InputBehaviorConfig。
/// 调用方需要先插入 `SoupruneConfig` 资源。
pub fn insert_input_resources(app: &mut App) {
    let config = app
        .world()
        .get_resource::<config::SoupruneConfig>()
        .expect("SoupruneConfig must be inserted before calling insert_input_resources");
    let projects_base = config::get_projects_base_path();
    let input_config_path = projects_base
        .join(&config.project.mod_name)
        .join(&config.game.input_config_path);
    let input_config = input::InputConfig::load_from_file(&input_config_path);
    let action_registry = input_config.build_registry();
    let player_input_settings =
        input::PlayerInputSettings::from_config(&input_config, &action_registry);
    let input_behavior_config = input::InputBehaviorConfig::from_config(&input_config);
    app.insert_resource(action_registry)
        .insert_resource(player_input_settings)
        .insert_resource(input_behavior_config);
}

pub fn run() {
    // On Android, print early debug info before any potential panic
    //
    // 在 Android 上，在任何潜在 panic 之前输出早期调试信息
    #[cfg(target_os = "android")]
    {
        // Install custom panic hook that writes to a file on /sdcard/
        // because eprintln! may not appear in logcat for all threads
        std::panic::set_hook(Box::new(|info| {
            let msg = format!(
                "[SoupRune PANIC] {}\n  at: {:?}\n  thread: {:?}\n---\n",
                info,
                info.location(),
                std::thread::current().name()
            );
            eprintln!("{}", msg);
            // Append to file so both the original panic and cleanup panic are captured
            use std::io::Write;
            if let Ok(mut f) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open("/sdcard/SoupRune/panic.log")
            {
                let _ = f.write_all(msg.as_bytes());
            }
        }));
        // Clear old panic log on startup
        let _ = std::fs::remove_file("/sdcard/SoupRune/panic.log");
        eprintln!("[SoupRune] run() started on Android");
        eprintln!(
            "[SoupRune] projects base: {:?}",
            config::get_projects_base_path()
        );
    }

    // Initialize logging and keep the guard alive
    //
    // 初始化日志记录并保持 guard 存活
    let _log_guard = setup_logging().expect("Failed to initialize logging");

    #[cfg(feature = "unsafe_gpu")]
    info!("Starting SoupRune with [unsafe_gpu] feature enabled.");

    #[cfg(target_os = "android")]
    eprintln!("[SoupRune] Loading config...");

    let config = config::load_config();

    #[cfg(target_os = "android")]
    eprintln!("[SoupRune] Config loaded: mod={}", config.project.mod_name);

    // Data
    let resolution_scale = config.window.resolution_scale;
    let project_name = config.project.mod_name.clone();
    let language = config.project.language.clone();

    // Config
    let render_config = config.render.clone();

    // Load input configuration from RON file
    // 从 RON 文件加载输入配置
    let projects_base = config::get_projects_base_path();
    #[cfg(target_os = "android")]
    eprintln!(
        "[SoupRune] input_config_path parts: base={:?}, mod={:?}, input={:?}",
        projects_base, config.project.mod_name, config.game.input_config_path
    );
    let input_config_path = projects_base
        .join(&config.project.mod_name)
        .join(&config.game.input_config_path);
    #[cfg(target_os = "android")]
    eprintln!(
        "[SoupRune] input_config_path joined: {:?}",
        input_config_path
    );
    let input_config = input::InputConfig::load_from_file(&input_config_path);
    let action_registry = input_config.build_registry();
    let player_input_settings =
        input::PlayerInputSettings::from_config(&input_config, &action_registry);
    let input_behavior_config = input::InputBehaviorConfig::from_config(&input_config);

    // Load touch layout config if specified
    // 如果指定了触控布局配置则加载
    let touch_layout = if let Some(ref touch_cfg) = input_config.touch_overlay {
        if let Some(ref layout_path) = touch_cfg.layout {
            let full_path = projects_base
                .join(&config.project.mod_name)
                .join(layout_path);
            match input::TouchLayoutDef::load_from_file(&full_path) {
                Ok(mut layout) => {
                    info!("Loaded touch layout from {:?}", full_path);
                    // Apply overlay-level opacity/scale if explicitly set in touch_overlay config
                    if let Some(opacity) = touch_cfg.opacity {
                        layout.opacity = opacity;
                    }
                    if let Some(scale) = touch_cfg.scale {
                        layout.scale = scale;
                    }
                    Some(layout)
                }
                Err(e) => {
                    warn!("Failed to load touch layout: {}", e);
                    None
                }
            }
        } else {
            None
        }
    } else {
        None
    };

    // Determine touch overlay enabled state: check if current OS is in platforms list
    let touch_enabled = input_config
        .touch_overlay
        .as_ref()
        .map(|cfg| {
            cfg.platforms
                .iter()
                .any(|p| p.eq_ignore_ascii_case(std::env::consts::OS))
        })
        .unwrap_or(false);

    let mut app = App::new();
    if let Some(layout) = touch_layout {
        app.insert_resource(layout);
    }
    app.insert_resource(input::touch::TouchOverlayEnabled(touch_enabled));
    app.insert_resource(ClearColor(Color::BLACK))
        .register_asset_source(
            AssetSourceId::Default,
            AssetSourceBuilder::new(move || {
                let roots = config::get_asset_roots(&project_name);
                let readers = roots.into_iter().map(FileAssetReader::new).collect();
                Box::new(MultiSourceAssetReader::new(readers))
            })
            .with_watcher(
                |sender: async_channel::Sender<bevy::asset::io::AssetSourceEvent>| {
                    // Watch the project root directory for hot reloading
                    // This allows hot reloading of view_layout.ron files outside of assets/
                    //
                    // 监听项目根目录以实现热重载
                    // 这允许热重载 assets/ 目录之外的 view_layout.ron 文件
                    let config = config::load_config();
                    let project_root =
                        config::get_projects_base_path().join(&config.project.mod_name);

                    // Try to watch the project root directory (not just assets/)
                    // 尝试监视项目根目录（不仅仅是 assets/）
                    let watch_paths = vec![
                        project_root.clone(),
                        dunce::canonicalize(&project_root).unwrap_or(project_root.clone()),
                    ];

                    for path in &watch_paths {
                        if path.exists() {
                            info!(
                                "[Hot Reload] Setting up file watcher for project root: {:?}",
                                path
                            );
                            match FileWatcher::new(
                                path.clone(),
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
                                        path, e
                                    );
                                }
                            }
                        }
                    }
                    error!("[Hot Reload] No valid project root found for file watching");
                    None
                },
            ),
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
        .insert_resource(config.clone())
        .insert_resource(bevy_rich_text3d::LoadFonts {
            font_directories: vec![
                projects_base
                    .join(&config.project.mod_name)
                    .join("assets/fonts")
                    .to_string_lossy()
                    .into_owned(),
            ],
            ..Default::default()
        })
        .insert_resource(action_registry)
        .insert_resource(player_input_settings)
        .insert_resource(input_behavior_config)
        .init_state::<app_state::AppState>()
        .init_state::<app_state::SequenceSubState>()
        .init_resource::<app_state::SequenceMode>()
        .add_message::<app_state::ModeChanged>()
        .add_systems(
            PreUpdate,
            (
                app_state::detect_mode_changes,
                app_state::cleanup_mode_scoped_entities,
            )
                .chain(),
        );

    // GameUpdateSchedule defaults to Update for standalone game
    let schedule = game_schedule(&app);
    app.configure_sets(
        schedule,
        view::ViewUpdate.run_if(in_state(app_state::AppState::Running)),
    )
    .configure_sets(
        schedule,
        sequencer::SequencerUpdate.run_if(in_state(app_state::AppState::Running)),
    )
    .add_plugins(get_game_plugins())
    .run();
}

#[cfg(target_os = "android")]
#[bevy_main]
fn main() {
    run();
}
