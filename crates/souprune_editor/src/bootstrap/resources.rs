use bevy::prelude::*;
use bevy_workbench::prelude::*;
use souprune::editor_api as api;

use crate::{data, editors, i18n, panels};

pub(super) fn configure_i18n(app: &mut App) {
    app.add_systems(Startup, register_i18n);
}

pub(super) fn configure_debug_tools(app: &mut App) {
    api::debug::setup_collider_debug(app);
    app.init_resource::<api::debug::RuleTriggerHistory>();
    app.init_resource::<panels::fre_panel::EditorFactEventHistory>();
    app.insert_resource(bevy_workbench::game_view::GameViewToolbar {
        toggles: vec![bevy_workbench::game_view::ToolbarToggle {
            id: "gizmos".into(),
            label: "Gizmos".into(),
            enabled: false,
        }],
    });
}

pub(super) fn configure_editor_resources(app: &mut App) {
    app.init_resource::<panels::sequence_timeline::EditorSequenceState>();
    app.init_resource::<panels::playback::PlaybackState>();
    app.init_resource::<editors::SubEditorManager>();
}

pub(super) fn configure_editor_systems(app: &mut App) {
    app.add_systems(Update, data::auto_save_system);
    app.add_systems(GameSchedule, panels::playback::playback_sync_system);
}

fn register_i18n(mut i18n: ResMut<bevy_workbench::i18n::I18n>) {
    i18n::register_editor_i18n(&mut i18n);
}
