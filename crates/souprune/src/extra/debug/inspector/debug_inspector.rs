//! Implements the standalone inspector window used by Souprune's debug mode.
//!
//! 实现 Souprune 调试模式下使用的独立 Inspector 窗口。
//!
//! Owns the debug inspector shell: the window and camera markers,
//! filter state, startup wiring, and the systems that coordinate the inspector
//! with help overlays, toast notifications, and gameplay input blocking. The
//! concrete helper behaviors are split into sibling modules, but this module turns
//! them into one coherent debug tool.
//!
//! 负责调试 Inspector 的外壳：窗口与相机标记、过滤状态、启动装配，
//! 以及让 Inspector 与帮助覆盖层、提示气泡和玩法输入屏蔽协同工作的系统。
//! 具体辅助行为拆在旁边的子模块里，而把它们收拢成完整调试工具的是这里。

mod help_overlay;
mod toast;
mod window_lifecycle;

use crate::app_state::overworld::character::components::PlayerControlled;
use crate::core::input::Action;
use crate::extra::debug::DebugToastEvent;
use bevy::app::App;
use bevy::ecs::schedule::ScheduleLabel;
use bevy::ecs::system::SystemIdMarker;
use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::{EguiContext, EguiPlugin};
use bevy_inspector_egui::{DefaultInspectorConfigPlugin, bevy_inspector, egui};
use iyes_perf_ui::prelude::*;
use leafwing_input_manager::action_state::ActionState;
use leafwing_input_manager::plugin::InputManagerSystem;
use std::marker::PhantomData;

#[derive(Component)]
pub(super) struct StandaloneInspectorWindow;

#[derive(Component)]
pub(super) struct StandaloneInspectorCamera;

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub(super) struct InspectorWindowContextPass;

/// Refresh phase for two-frame refresh process.
#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub(super) enum RefreshPhase {
    #[default]
    None,
    /// Window should be closed this frame.
    CloseWindow,
    /// Window should be reopened this frame.
    ReopenWindow,
}

#[derive(Resource, Default)]
pub(in crate::extra::debug) struct InspectorUiState {
    inspector_window: Option<Entity>,
    inspector_camera: Option<Entity>,
    window_focused: bool,
    /// Two-phase refresh state for state change handling.
    refresh_phase: RefreshPhase,
    /// Whether to show all entities including BRP/system internals.
    /// 是否显示所有实体，包括 BRP/系统内部实体。
    show_all_entities: bool,
    /// Search filter for entity names.
    /// 实体名称搜索过滤器。
    search_query: String,
}

/// Custom entity filter that excludes BRP system entities by default.
/// 自定义实体过滤器，默认排除 BRP 系统实体。
struct BrpEntityFilter {
    show_all: bool,
    /// Search query for filtering by entity name.
    /// 用于按实体名称过滤的搜索查询。
    search_query: String,
    _marker: PhantomData<Without<ChildOf>>,
}

impl BrpEntityFilter {
    fn new(show_all: bool, search_query: &str) -> Self {
        Self {
            show_all,
            search_query: search_query.to_lowercase(),
            _marker: PhantomData,
        }
    }
}

impl bevy_inspector::EntityFilter for BrpEntityFilter {
    type StaticFilter = Without<ChildOf>;

    fn is_active(&self) -> bool {
        !self.show_all || !self.search_query.is_empty()
    }

    fn filter_entity(&self, world: &mut World, entity: Entity) -> bool {
        if !self.show_all
            && (world.get::<SystemIdMarker>(entity).is_some()
                || world.get::<bevy::ecs::observer::Observer>(entity).is_some())
        {
            return false;
        }

        if self.search_query.is_empty() {
            return true;
        }

        world
            .get::<Name>(entity)
            .is_some_and(|name| name.to_lowercase().contains(&self.search_query))
    }
}

pub(in crate::extra::debug) fn setup_debug_features(app: &mut App) {
    app.init_resource::<InspectorUiState>();

    app.add_plugins(EguiPlugin::default());
    app.add_plugins(DefaultInspectorConfigPlugin);

    app.add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default());
    app.add_plugins(bevy::diagnostic::EntityCountDiagnosticsPlugin::default());
    app.add_plugins(bevy::diagnostic::SystemInformationDiagnosticsPlugin);

    if !app.is_plugin_added::<bevy::render::diagnostic::RenderDiagnosticsPlugin>() {
        app.add_plugins(bevy::render::diagnostic::RenderDiagnosticsPlugin);
    }

    app.add_plugins(PerfUiPlugin);

    app.add_systems(
        Startup,
        (
            help_overlay::setup_debug_help_text_system,
            toast::setup_debug_toast_system,
        ),
    );

    app.add_systems(
        Update,
        (
            window_lifecycle::handle_inspector_hotkeys_system,
            window_lifecycle::inspector_window_closed_system,
            window_lifecycle::inspector_window_focus_system,
            window_lifecycle::primary_window_closed_system,
            window_lifecycle::app_state_changed_refresh_inspector_system,
            window_lifecycle::inspector_refresh_system,
            toggle_perf_ui_system.before(iyes_perf_ui::PerfUiSet::Setup),
            help_overlay::toggle_debug_help_text_system,
            help_overlay::fade_debug_help_text_system,
            toast::handle_debug_toast_event_system,
            toast::fade_debug_toast_system,
        ),
    );
    app.add_systems(InspectorWindowContextPass, inspector_window_ui_system);
    app.add_systems(
        PreUpdate,
        block_player_actions_when_inspector_focused_system.after(InputManagerSystem::ManualControl),
    );
}

fn toggle_perf_ui_system(
    mut commands: Commands,
    q_perf_ui: Query<Entity, With<PerfUiRoot>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut toast_events: MessageWriter<DebugToastEvent>,
) {
    if keyboard_input.just_pressed(KeyCode::F3) {
        let message = if let Ok(e) = q_perf_ui.single() {
            commands.entity(e).despawn();
            "Performance UI: OFF"
        } else {
            commands.spawn(PerfUiAllEntries::default());
            "Performance UI: ON"
        };
        info!("{}", message);
        toast_events.write(DebugToastEvent {
            message: message.into(),
        });
    }
}

pub(super) fn set_text_entities_color(
    entities: &[Entity],
    color: Color,
    q_text_colors: &mut Query<&mut TextColor>,
) {
    for &entity in entities {
        if let Ok(mut tc) = q_text_colors.get_mut(entity) {
            tc.0 = color;
        }
    }
}

fn render_inspector_search_box(ui: &mut egui::Ui, world: &mut World) {
    ui.label("🔍");
    let mut search_query = world
        .get_resource::<InspectorUiState>()
        .map(|s| s.search_query.clone())
        .unwrap_or_default();

    let response = ui.add(
        egui::TextEdit::singleline(&mut search_query)
            .hint_text("Search entities by name...")
            .desired_width(300.0),
    );

    if response.changed()
        && let Some(mut state) = world.get_resource_mut::<InspectorUiState>()
    {
        state.search_query = search_query.clone();
    }

    if ui.button("✕").clicked()
        && let Some(mut state) = world.get_resource_mut::<InspectorUiState>()
    {
        state.search_query.clear();
    }
}

fn render_inspector_entity_filter(ui: &mut egui::Ui, world: &mut World) {
    let mut show_all = world
        .get_resource::<InspectorUiState>()
        .map(|s| s.show_all_entities)
        .unwrap_or(false);

    if ui
        .checkbox(&mut show_all, "Show all entities (BRP/System internals)")
        .changed()
        && let Some(mut state) = world.get_resource_mut::<InspectorUiState>()
    {
        state.show_all_entities = show_all;
    }
}

fn inspector_window_ui_system(world: &mut World) {
    let (inspector_camera, show_all_entities) = {
        let state = world.get_resource::<InspectorUiState>();
        (
            state.and_then(|s| s.inspector_camera),
            state.map(|s| s.show_all_entities).unwrap_or(false),
        )
    };
    let Some(camera_entity) = inspector_camera else {
        return;
    };

    let mut contexts = world.query_filtered::<&mut EguiContext, With<StandaloneInspectorCamera>>();
    let mut egui_context = match contexts.get_mut(world, camera_entity) {
        Ok(context) => context.clone(),
        Err(_) => return,
    };

    egui::TopBottomPanel::top("inspector_controls").show(egui_context.get_mut(), |ui| {
        ui.horizontal(|ui| render_inspector_search_box(ui, world));
        ui.horizontal(|ui| render_inspector_entity_filter(ui, world));
    });

    let search_query = world
        .get_resource::<InspectorUiState>()
        .map(|s| s.search_query.clone())
        .unwrap_or_default();

    egui::CentralPanel::default().show(egui_context.get_mut(), |ui| {
        egui::ScrollArea::both().show(ui, |ui| {
            let filter = BrpEntityFilter::new(show_all_entities, &search_query);
            bevy_inspector::ui_for_entities_filtered(world, ui, true, &filter);
            ui.allocate_space(ui.available_size());
        });
    });
}

fn block_player_actions_when_inspector_focused_system(
    ui_state: Option<Res<InspectorUiState>>,
    mut query: Query<&mut ActionState<Action>, With<PlayerControlled>>,
) {
    let should_disable = ui_state.map(|state| state.window_focused).unwrap_or(false);

    for mut action_state in query.iter_mut() {
        if should_disable && !action_state.disabled() {
            action_state.disable();
        } else if !should_disable && action_state.disabled() {
            action_state.enable();
        }
    }
}
