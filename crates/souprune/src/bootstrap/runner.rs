//! Owns the top-level boot sequence that turns configuration into a running app.
//!
//! 负责最顶层的启动流程，把配置真正变成一个可运行的应用。
//!
//! Souprune crosses from "startup description" into an actual `App` here:
//! it loads config, prepares asset sources and hot reload
//! watchers, installs startup resources, and finally runs the assembled plugin
//! graph. If something only matters while the process is being constructed, it
//! belongs here rather than in gameplay code.
//!
//! 这里负责把 Souprune 从“启动描述”推进到实际 `App` 实例：读取配置、
//! 准备资源源和热重载监听器、注入启动资源，并最终运行装配好的插件图。
//! 只在进程构建阶段有意义的逻辑，应当放在这里，而不是混进玩法代码里。

use crate::app_state::app_setup;
use crate::bootstrap::logging::setup_logging;
use crate::bootstrap::plugins::{
    get_bevy_default_plugins, get_file_importer_plugins, get_game_plugins, get_third_plugins,
};
use crate::bootstrap::resources::load_touch_layout;
use crate::config;
use crate::core::input;
use crate::extra;
use crate::extra::multi_source::MultiSourceAssetReader;
use crate::init_game_state;
use bevy::asset::io::file::{FileAssetReader, FileWatcher};
use bevy::asset::io::{AssetSourceBuilder, AssetSourceId};
use bevy::prelude::*;

pub fn run() {
    #[cfg(target_os = "android")]
    {
        std::panic::set_hook(Box::new(|info| {
            let msg = format!(
                "[SoupRune PANIC] {}\n  at: {:?}\n  thread: {:?}\n---\n",
                info,
                info.location(),
                std::thread::current().name()
            );
            eprintln!("{}", msg);
            use std::io::Write;
            if let Ok(mut f) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open("/sdcard/SoupRune/panic.log")
            {
                let _ = f.write_all(msg.as_bytes());
            }
        }));
        let _ = std::fs::remove_file("/sdcard/SoupRune/panic.log");
        eprintln!("[SoupRune] run() started on Android");
        eprintln!(
            "[SoupRune] projects base: {:?}",
            config::get_projects_base_path()
        );
    }

    let _log_guard = setup_logging().expect("Failed to initialize logging");

    #[cfg(feature = "unsafe_gpu")]
    info!("Starting SoupRune with [unsafe_gpu] feature enabled.");

    #[cfg(target_os = "android")]
    eprintln!("[SoupRune] Loading config...");

    let config = config::load_config();

    #[cfg(target_os = "android")]
    eprintln!("[SoupRune] Config loaded: mod={}", config.project.mod_name);

    let resolution_scale = config.window.resolution_scale;
    let project_name = config.project.mod_name.clone();
    let language = config.project.language.clone();
    let render_config = config.render.clone();

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

    let touch_layout = load_touch_layout(&input_config, &projects_base, &config.project.mod_name);
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
                    let config = config::load_config();
                    let project_root =
                        config::get_projects_base_path().join(&config.project.mod_name);
                    let watch_paths = vec![
                        project_root.clone(),
                        dunce::canonicalize(&project_root).unwrap_or(project_root.clone()),
                    ];

                    for path in &watch_paths {
                        if !path.exists() {
                            continue;
                        }
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
        .insert_resource(bevy_bitmap_text::FontDirectories {
            directories: vec![
                projects_base
                    .join(&config.project.mod_name)
                    .join("assets/fonts")
                    .to_string_lossy()
                    .into_owned(),
            ],
        })
        .insert_resource(action_registry)
        .insert_resource(player_input_settings)
        .insert_resource(input_behavior_config);

    init_game_state(&mut app);
    app.add_plugins(get_game_plugins()).run();
}
