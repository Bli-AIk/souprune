//! # SoupRune Editor
//!
//! # SoupRune 编辑器
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Sequence-driven editor built on bevy_workbench.
//!
//! 基于 bevy_workbench 的序列驱动编辑器。

mod data;
mod editors;
mod i18n;
pub mod icons;
mod panels;
mod platform;
mod sequencer_bridge;
pub mod widgets;

use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::*;
use bevy_workbench::prelude::*;

/// SoupRune 编辑器主插件。
pub struct SoupRuneEditorPlugin;

impl Plugin for SoupRuneEditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(WorkbenchPlugin {
            config: WorkbenchConfig {
                show_menu_bar: true,
                show_toolbar: true,
                show_console: true,
                enable_game_view: true,
                ..default()
            },
        });

        // 平台适配
        app.add_plugins(platform::PlatformPlugin);

        // 游戏系统注册到 GameSchedule（由 bevy_workbench 在 Play 模式下执行）
        app.insert_resource(souprune::GameUpdateSchedule(GameSchedule.intern()));

        // 从配置读取分辨率参数
        let (resolution_scale, base_w, base_h) = {
            let config = app
                .world()
                .get_resource::<souprune::config::SoupruneConfig>();
            (
                config.map_or(2, |c| c.window.resolution_scale),
                config.map_or(320u32, |c| c.render.base_resolution_width),
                config.map_or(240u32, |c| c.render.base_resolution_height),
            )
        };

        // Render target 分辨率 = base * scale，与独立游戏窗口一致，保证像素完美。
        app.world_mut()
            .resource_mut::<bevy_workbench::game_view::GameViewState>()
            .resolution = UVec2::new(base_w * resolution_scale, base_h * resolution_scale);

        // 应用状态（游戏插件依赖）— 共享初始化
        souprune::init_game_state(app);
        app.insert_resource(souprune::core::input::touch::TouchOverlayEnabled(false))
            .insert_resource(souprune::app_state::app_setup::ResolutionScale(
                resolution_scale,
            ));

        // 从配置加载完整的输入资源（ActionRegistry, PlayerInputSettings, InputBehaviorConfig）
        souprune::insert_input_resources(app);

        // 从配置加载字体目录（bevy_rich_text3d 需要）
        souprune::insert_font_resources(app);

        // 完整的游戏插件栈（所有系统注册到 GameSchedule）
        app.add_plugins(souprune::get_third_plugins());
        app.add_plugins(souprune::get_game_plugins());
        app.add_plugins(souprune::get_file_importer_plugins());

        // 进入 Play 模式时激活游戏状态并设置 SequenceMode
        app.add_systems(OnEnter(EditorMode::Play), enter_play_mode);

        // 注册编辑器面板
        app.register_panel(panels::AssetBrowserPanel::new());
        app.register_panel(panels::SequenceTimelinePanel::new());
        app.register_panel(panels::ChapterInspectorPanel::new());
        app.register_panel(panels::PlaybackPanel::new());
        app.register_panel(panels::FrePanel::new());
        app.register_panel(panels::ViewEditorPanel::new());

        // View 编辑器状态
        app.init_resource::<panels::view_editor::ViewEditorState>();

        // View 预览渲染
        app.init_resource::<panels::view_preview::ViewPreviewState>();
        app.add_systems(Startup, panels::view_preview::setup_view_preview);
        app.add_systems(
            Update,
            (
                panels::view_preview::sync_preview_texture,
                panels::view_preview::rebuild_preview_entities,
                panels::view_preview::sync_preview_camera,
                panels::view_preview::propagate_preview_render_layers,
            ),
        );
        // SDF/Text 渲染系统 — 仅在编辑模式下运行
        // Play 模式下由 GameSchedule 中的 CoreViewPlugin 驱动
        app.add_systems(
            Update,
            (
                souprune::core::view::sdf_view_shape::update_sdf_view_shape_system,
                souprune::core::view::text::assign_text_material_system,
                souprune::core::view::text::show_text_when_ready_system,
            )
                .run_if(in_state(EditorMode::Edit)),
        );

        // i18n 在 Startup 时注册（I18n 资源由 WorkbenchPlugin 创建）
        app.add_systems(Startup, register_i18n);

        // Collider gizmos（编辑器中注册，debug 模式下由 DebugPlugin 注册）
        souprune::extra::debug::setup_collider_debug(app);
        app.init_resource::<souprune::extra::debug::RuleTriggerHistory>();
        app.init_resource::<panels::fre_panel::EditorFactEventHistory>();
        app.insert_resource(bevy_workbench::game_view::GameViewToolbar {
            toggles: vec![bevy_workbench::game_view::ToolbarToggle {
                id: "gizmos".into(),
                label: "Gizmos".into(),
                enabled: false,
            }],
        });
        app.add_systems(
            Update,
            (
                panels::fre_panel::track_fact_events_system,
                sync_gizmo_toggle_system,
            ),
        );

        // 发现游戏相机并注册为外部相机（由 GameViewPlugin 在 Play 模式劫持）
        // 使用 Update 而非 PostStartup，因为 OnEnter(Loading) 在首次 Update 时才触发
        app.add_systems(
            Update,
            register_external_game_camera.run_if(not(resource_exists::<ExternalGameCamera>)),
        );

        // 编辑器核心资源
        app.init_resource::<panels::sequence_timeline::EditorSequenceState>();
        app.init_resource::<panels::playback::PlaybackState>();
        app.init_resource::<editors::SubEditorManager>();

        // 自动保存系统
        app.add_systems(Update, data::auto_save_system);

        // Play/Edit 模式切换钩子
        app.add_systems(OnEnter(EditorMode::Play), sequencer_bridge::on_enter_play);
        app.add_systems(
            OnEnter(EditorMode::Edit),
            (souprune::reset_game_state, sequencer_bridge::on_exit_play).chain(),
        );

        // 序列回放 UI 同步（仅在 Play 模式下执行）
        app.add_systems(GameSchedule, panels::playback::playback_sync_system);
    }
}

fn register_i18n(mut i18n: ResMut<bevy_workbench::i18n::I18n>) {
    i18n::register_editor_i18n(&mut i18n);
}

/// Play 模式启动：设置 AppState 和 SequenceMode。
fn enter_play_mode(
    mut next: ResMut<NextState<souprune::app_state::AppState>>,
    mut sequence_mode: ResMut<souprune::app_state::SequenceMode>,
    config: Res<souprune::config::SoupruneConfig>,
) {
    next.set(souprune::app_state::AppState::Running);

    // 从配置推断 SequenceMode（与 check_textures_system 逻辑一致）
    if sequence_mode.0.is_none() {
        if config.game.initial_sequence_path.is_none()
            && config.game.initial_map_path.is_empty()
            && !config.game.initial_battle_path.is_empty()
        {
            sequence_mode.0 = Some("battle".to_string());
        } else {
            sequence_mode.0 = Some("overworld".to_string());
        }
    }
}

/// 查找游戏主摄像机并注册为外部摄像机，Play 模式时 GameViewPlugin 会劫持它。
fn register_external_game_camera(
    mut commands: Commands,
    cameras: Query<Entity, With<souprune::core::camera::MainGameCamera>>,
) {
    if let Some(entity) = cameras.iter().next() {
        commands.insert_resource(ExternalGameCamera(entity));
        info!("[编辑器] 已注册外部游戏相机: {:?}", entity);
    }
}

/// Syncs the GameViewToolbar "gizmos" toggle to ColliderGizmos config.
fn sync_gizmo_toggle_system(
    toolbar: Res<bevy_workbench::game_view::GameViewToolbar>,
    mut gizmo_store: ResMut<GizmoConfigStore>,
) {
    if toolbar.is_changed()
        && let Some(enabled) = toolbar.is_enabled("gizmos")
    {
        let (config, _) = gizmo_store.config_mut::<souprune::extra::debug::ColliderGizmos>();
        config.enabled = enabled;
    }
}
