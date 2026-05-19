//! Builds the plugin groups that make up a Souprune app.
//!
//! 组装构成 Souprune 应用的各类插件集合。
//!
//! Acts as the plugin-selection layer for startup. It decides which Bevy
//! defaults are patched, which third-party integrations are enabled, and which
//! game plugins form the actual runtime. Keeping that choice here prevents the
//! crate root and runner from turning into a long list of unrelated plugin
//! details.
//!
//! 启动期的插件选择层。它决定 Bevy 默认插件如何调整、哪些第三方
//! 集成要启用，以及哪些游戏插件组成真正的运行时。把这些选择集中在这里，
//! 可以避免 crate 根入口和 runner 被一长串互不相干的插件细节淹没。

use crate::config;
use crate::core::input::Action;
use crate::core::*;
use crate::extra;
use bevy::app::PluginGroupBuilder;
#[cfg(feature = "unsafe_gpu")]
use bevy::prelude::info;
use bevy::prelude::{ImagePlugin, PluginGroup, default};
#[cfg(any(feature = "unsafe_gpu", target_os = "android"))]
use bevy::render::RenderPlugin;
#[cfg(feature = "unsafe_gpu")]
use bevy::render::settings::InstanceFlags;
#[cfg(any(feature = "unsafe_gpu", target_os = "android"))]
use bevy::render::settings::{RenderCreation, WgpuSettings};
#[cfg(not(target_os = "android"))]
use bevy::window::WindowResolution;
use bevy::window::{Window, WindowPlugin};

pub mod app_setup;

pub(crate) fn get_bevy_default_plugins(
    resolution_scale: u32,
    render_config: &config::RenderConfig,
) -> PluginGroupBuilder {
    #[cfg(not(target_os = "android"))]
    let (base_width, base_height) = (
        render_config.base_resolution_width * resolution_scale,
        render_config.base_resolution_height * resolution_scale,
    );
    #[cfg(target_os = "android")]
    let _ = (resolution_scale, render_config);

    #[allow(unused_mut)]
    let mut plugins = bevy::DefaultPlugins
        .set(ImagePlugin::default_nearest())
        .set(bevy::asset::AssetPlugin {
            unapproved_path_mode: bevy::asset::UnapprovedPathMode::Allow,
            ..default()
        })
        .set(WindowPlugin {
            primary_window: Some(Window {
                #[cfg(not(target_os = "android"))]
                resolution: WindowResolution::new(base_width, base_height),
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

    plugins = plugins.disable::<bevy::log::LogPlugin>();

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
            .disable::<bevy::gizmos::GizmoPlugin>()
            .disable::<bevy::gizmos_render::GizmoRenderPlugin>();
    }

    plugins
}

pub fn get_file_importer_plugins() -> (
    extra::markdown::MarkdownPlugin,
    extra::mortar::MortarExtraPlugin,
) {
    (
        extra::markdown::MarkdownPlugin,
        extra::mortar::MortarExtraPlugin,
    )
}

pub fn get_third_plugins() -> (
    leafwing_input_manager::prelude::InputManagerPlugin<Action>,
    bevy_ecs_tiled::prelude::TiledPlugin,
    bevy_bitmap_text::BitmapTextPlugin,
    bevy_alight_motion::prelude::AlightMotionPlugin,
    bevy_tween::DefaultTweenPlugins<()>,
) {
    (
        leafwing_input_manager::prelude::InputManagerPlugin::<Action>::default(),
        bevy_ecs_tiled::prelude::TiledPlugin::default(),
        bevy_bitmap_text::BitmapTextPlugin {
            atlas_config: bevy_bitmap_text::GlyphCacheConfig {
                alpha_mode: bevy_bitmap_text::GlyphAlphaMode::Binary { threshold: 128 },
                ..default()
            },
            register_animation_systems: false,
        },
        bevy_alight_motion::prelude::AlightMotionPlugin,
        bevy_tween::DefaultTweenPlugins::default(),
    )
}

pub fn get_game_plugins() -> (
    CorePlugin,
    app_setup::AppSetupPlugin,
    GlobalPlugin,
    mod_system::ModPlugin,
) {
    (
        CorePlugin,
        app_setup::AppSetupPlugin,
        GlobalPlugin,
        mod_system::ModPlugin,
    )
}
