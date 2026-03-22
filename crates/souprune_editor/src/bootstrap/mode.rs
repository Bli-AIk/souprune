//! Defines the editor-side mode transitions between editing and playing the game.
//!
//! 定义编辑器在编辑态与试玩态之间切换时需要执行的模式逻辑。
//!
//! This file owns the behavior that is specific to `bevy_workbench` editor
//! modes: entering play mode, registering the external game camera, resetting
//! runtime state when returning to edit mode, and keeping editor gizmo toggles
//! in sync with the actual debug configuration.
//!
//! 这个文件负责 `bevy_workbench` 编辑器模式独有的切换行为：进入试玩态、
//! 注册外部游戏相机、回到编辑态时重置运行时状态，以及让编辑器工具栏里的
//! gizmo 开关和真实调试配置保持同步。

use crate::panels;
use bevy::prelude::*;
use bevy_workbench::prelude::*;
use souprune::editor_api as api;

pub(crate) fn add_mode_systems(app: &mut App) {
    app.add_systems(OnEnter(EditorMode::Play), enter_play_mode);
    app.add_systems(
        Update,
        register_external_game_camera.run_if(not(resource_exists::<ExternalGameCamera>)),
    );
    app.add_systems(
        Update,
        (
            panels::fre_panel::track_fact_events_system,
            sync_gizmo_toggle_system,
        ),
    );
    app.add_systems(
        OnEnter(EditorMode::Play),
        crate::sequencer_bridge::on_enter_play,
    );
    app.add_systems(
        OnEnter(EditorMode::Edit),
        (
            souprune::reset_game_state,
            crate::sequencer_bridge::on_exit_play,
        )
            .chain(),
    );
}

fn enter_play_mode(
    mut next: ResMut<NextState<api::app::AppState>>,
    mut sequence_mode: ResMut<api::app::SequenceMode>,
    config: Res<souprune::config::SoupruneConfig>,
) {
    next.set(api::app::AppState::Running);

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

fn register_external_game_camera(
    mut commands: Commands,
    cameras: Query<Entity, With<api::camera::MainGameCamera>>,
) {
    if let Some(entity) = cameras.iter().next() {
        commands.insert_resource(ExternalGameCamera(entity));
        info!("[编辑器] 已注册外部游戏相机: {:?}", entity);
    }
}

fn sync_gizmo_toggle_system(
    toolbar: Res<bevy_workbench::game_view::GameViewToolbar>,
    mut gizmo_store: ResMut<GizmoConfigStore>,
) {
    if toolbar.is_changed()
        && let Some(enabled) = toolbar.is_enabled("gizmos")
    {
        let (config, _) = gizmo_store.config_mut::<api::debug::ColliderGizmos>();
        config.enabled = enabled;
    }
}
