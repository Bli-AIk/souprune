use crate::app_state::{app_setup, battle, overworld};
use crate::config;
use crate::core::input::Action;
use crate::core::*;
use crate::extra;
use bevy::app::PluginGroupBuilder;
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
    extra::toml::TomlPlugin,
    extra::mortar::MortarExtraPlugin,
) {
    (
        extra::markdown::MarkdownPlugin,
        extra::toml::TomlPlugin,
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
        bevy_bitmap_text::BitmapTextPlugin::default(),
        bevy_alight_motion::prelude::AlightMotionPlugin,
        bevy_tween::DefaultTweenPlugins::default(),
    )
}

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
