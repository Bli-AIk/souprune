use crate::{bootstrap::mode, data, editors, i18n, panels, platform};
use bevy::ecs::schedule::ScheduleLabel;
use bevy::prelude::*;
use bevy_workbench::prelude::*;
use souprune::editor_api as api;

pub(crate) fn build_editor_app(app: &mut App) {
    app.add_plugins(WorkbenchPlugin {
        config: WorkbenchConfig {
            show_menu_bar: true,
            show_toolbar: true,
            show_console: true,
            enable_game_view: true,
            ..default()
        },
    });

    app.add_plugins(platform::PlatformPlugin);
    app.insert_resource(souprune::GameUpdateSchedule(GameSchedule.intern()));

    let (resolution_scale, base_w, base_h) = load_resolution_config(app);
    app.world_mut()
        .resource_mut::<bevy_workbench::game_view::GameViewState>()
        .resolution = UVec2::new(base_w * resolution_scale, base_h * resolution_scale);

    souprune::init_game_state(app);
    app.insert_resource(api::input::TouchOverlayEnabled(false))
        .insert_resource(api::app::ResolutionScale(resolution_scale));

    souprune::insert_input_resources(app);
    insert_preview_key_map(app);
    souprune::insert_font_resources(app);

    app.add_plugins(souprune::get_third_plugins());
    app.add_plugins(souprune::get_game_plugins());
    app.add_plugins(souprune::get_file_importer_plugins());

    register_panels(app);
    configure_view_preview(app);
    configure_i18n(app);
    configure_debug_tools(app);
    configure_editor_resources(app);
    configure_editor_systems(app);
    mode::add_mode_systems(app);
}

fn load_resolution_config(app: &mut App) -> (u32, u32, u32) {
    let config = app
        .world()
        .get_resource::<souprune::config::SoupruneConfig>();

    (
        config.map_or(2, |c| c.window.resolution_scale),
        config.map_or(320u32, |c| c.render.base_resolution_width),
        config.map_or(240u32, |c| c.render.base_resolution_height),
    )
}

fn insert_preview_key_map(app: &mut App) {
    let config = app
        .world()
        .get_resource::<souprune::config::SoupruneConfig>()
        .expect("SoupruneConfig required");
    let projects_base = souprune::config::get_projects_base_path();
    let input_config_path = projects_base
        .join(&config.project.mod_name)
        .join(&config.game.input_config_path);
    let input_config = api::input::InputConfig::load_from_file(&input_config_path);
    let key_map =
        panels::view_preview::ViewPreviewKeyMap(input_config.build_keycode_to_action_map());
    app.insert_resource(key_map);
}

fn register_panels(app: &mut App) {
    app.register_panel(panels::AssetBrowserPanel::new());
    app.register_panel(panels::SequenceTimelinePanel::new());
    app.register_panel(panels::ChapterInspectorPanel::new());
    app.register_panel(panels::PlaybackPanel::new());
    app.register_panel(panels::FrePanel::new());
    app.register_panel(panels::ViewEditorPanel::new());
}

fn configure_view_preview(app: &mut App) {
    app.init_resource::<panels::view_editor::ViewEditorState>();
    app.init_resource::<panels::view_preview::ViewPreviewState>();
    app.init_resource::<panels::view_fre_panel::ViewFreState>();
    app.add_systems(Startup, panels::view_preview::setup_view_preview);
    app.add_systems(
        Update,
        (
            panels::view_preview::sync_preview_texture,
            panels::view_preview::rebuild_preview_entities,
            ApplyDeferred,
            panels::view_preview::sync_preview_camera,
            panels::view_preview::propagate_preview_render_layers,
            api::view::update_sdf_view_shape_system,
            api::view::show_text_when_ready_system,
        )
            .chain()
            .run_if(in_state(EditorMode::Edit)),
    );
    app.add_systems(
        Update,
        panels::view_preview::preview_play_control_system.run_if(in_state(EditorMode::Edit)),
    );
    app.add_systems(
        Update,
        (
            panels::view_preview::preview_input_to_fre_system,
            api::fre_bridge::process_view_actions_system,
            api::view::evaluate_visible_when_system,
            api::view::update_fact_dependent_ui_elements,
        )
            .chain()
            .run_if(in_state(EditorMode::Edit))
            .run_if(|state: Res<panels::view_preview::ViewPreviewState>| state.playing),
    );
}

fn configure_i18n(app: &mut App) {
    app.add_systems(Startup, register_i18n);
}

fn configure_debug_tools(app: &mut App) {
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

fn configure_editor_resources(app: &mut App) {
    app.init_resource::<panels::sequence_timeline::EditorSequenceState>();
    app.init_resource::<panels::playback::PlaybackState>();
    app.init_resource::<editors::SubEditorManager>();
}

fn configure_editor_systems(app: &mut App) {
    app.add_systems(Update, data::auto_save_system);
    app.add_systems(GameSchedule, panels::playback::playback_sync_system);
}

fn register_i18n(mut i18n: ResMut<bevy_workbench::i18n::I18n>) {
    i18n::register_editor_i18n(&mut i18n);
}
