//! # view_taffy_layout
//!
//! Minimal manual-acceptance harness for loading a View layout that exercises
//! the staged Taffy style fields.
//!
//! 用于手工验收 View 布局的最小示例，加载覆盖阶段性 Taffy 样式字段的
//! `.view.ron` 资产。
//!
//! ## Usage
//!
//! ## 运行方式
//!
//! ```bash
//! cargo run -p souprune --example view_taffy_layout
//! ```

use std::collections::HashMap;
use std::path::PathBuf;

use bevy::asset::UnapprovedPathMode;
use bevy::prelude::*;
use bevy::window::WindowResolution;
use bevy_fact_rule_event::{FREPlugin, FactValue};
use souprune::core::camera::MainGameCamera;
use souprune::core::game_action::GameActionDef;
use souprune::core::sequencer::chapter_schema::DataBinding;
use souprune::core::view::ViewRoot;
use souprune::core::view::{CoreViewPlugin, SpawnViewRequest};

const VIEW_PATH: &str = "view/taffy_minimal.view.ron";
const SCROLL_STEP: i64 = 12;
const SCROLL_MIN: i64 = -48;
const SCROLL_MAX: i64 = 48;
const FOCUS_SLOT_COUNT: i64 = 3;
const FOCUS_STACK_MAX: i64 = 3;

struct AcceptanceState {
    compact: bool,
    hidden_leaf_visible: bool,
    hidden_container_visible: bool,
    scroll_offset: i64,
    focus_index: i64,
    focus_stack_depth: i64,
    focus_active: bool,
}

impl Default for AcceptanceState {
    fn default() -> Self {
        Self {
            compact: false,
            hidden_leaf_visible: true,
            hidden_container_visible: true,
            scroll_offset: 0,
            focus_index: 0,
            focus_stack_depth: 0,
            focus_active: true,
        }
    }
}

impl AcceptanceState {
    fn reset_focus(&mut self) {
        self.focus_index = 0;
        self.focus_stack_depth = 0;
        self.focus_active = true;
    }

    fn focus_label(&self) -> String {
        format!(
            "focus={} stack={} active={}",
            self.focus_index, self.focus_stack_depth, self.focus_active
        )
    }
}

fn main() {
    let asset_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/assets");

    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(bevy::image::ImagePlugin::default_nearest())
            .set(bevy::asset::AssetPlugin {
                file_path: asset_root.to_string_lossy().into_owned(),
                unapproved_path_mode: UnapprovedPathMode::Allow,
                ..default()
            })
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "SoupRune View Taffy Layout".into(),
                    resolution: WindowResolution::new(640, 480),
                    resizable: false,
                    ..default()
                }),
                ..default()
            }),
    );

    app.add_plugins((
        souprune::get_file_importer_plugins(),
        souprune::get_third_plugins(),
        FREPlugin::<GameActionDef>::default(),
        souprune::core::CorePlugin,
        CoreViewPlugin,
    ));

    souprune::init_game_state(&mut app);
    souprune::insert_input_resources(&mut app);

    app.insert_resource(ClearColor(Color::BLACK));
    app.add_systems(Startup, setup);
    app.add_systems(Update, drive_dynamic_acceptance);
    app.run();
}

fn setup(
    mut commands: Commands,
    mut next_state: ResMut<NextState<souprune::app_state::AppState>>,
    mut spawn_writer: MessageWriter<SpawnViewRequest>,
) {
    commands.spawn((
        Name::new("View Taffy Layout Camera"),
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: bevy::camera::ScalingMode::Fixed {
                width: 640.0,
                height: 480.0,
            },
            ..OrthographicProjection::default_2d()
        }),
        Transform::default(),
        MainGameCamera,
    ));

    next_state.set(souprune::app_state::AppState::Running);

    spawn_writer.write(SpawnViewRequest {
        path: VIEW_PATH.to_string(),
        mode_scope: None,
        pre_spawn_events: Vec::new(),
        bindings: Some(HashMap::from([(
            "demo".to_string(),
            DataBinding::LocalLayer,
        )])),
    });
}

fn drive_dynamic_acceptance(
    keys: Res<ButtonInput<KeyCode>>,
    mut state: Local<AcceptanceState>,
    mut roots: Query<&mut ViewRoot>,
) {
    let mut changed = false;

    if keys.just_pressed(KeyCode::Space) {
        state.compact = !state.compact;
        changed = true;
    }
    if keys.just_pressed(KeyCode::KeyH) {
        state.hidden_leaf_visible = !state.hidden_leaf_visible;
        changed = true;
    }
    if keys.just_pressed(KeyCode::KeyC) {
        state.hidden_container_visible = !state.hidden_container_visible;
        changed = true;
    }
    if keys.just_pressed(KeyCode::ArrowUp) {
        state.scroll_offset = (state.scroll_offset + SCROLL_STEP).min(SCROLL_MAX);
        changed = true;
    }
    if keys.just_pressed(KeyCode::ArrowDown) {
        state.scroll_offset = (state.scroll_offset - SCROLL_STEP).max(SCROLL_MIN);
        changed = true;
    }
    if keys.just_pressed(KeyCode::Tab) {
        state.focus_active = true;
        state.focus_index = (state.focus_index + 1) % FOCUS_SLOT_COUNT;
        changed = true;
    }
    if keys.just_pressed(KeyCode::Enter) {
        state.focus_active = true;
        state.focus_stack_depth = (state.focus_stack_depth + 1).min(FOCUS_STACK_MAX);
        changed = true;
    }
    if keys.just_pressed(KeyCode::Escape) {
        state.focus_stack_depth = (state.focus_stack_depth - 1).max(0);
        state.focus_active = state.focus_stack_depth > 0;
        changed = true;
    }
    if keys.just_pressed(KeyCode::Digit0) {
        state.reset_focus();
        changed = true;
    }

    if !changed {
        return;
    }

    for mut root in roots
        .iter_mut()
        .filter(|root| root.layout_path == VIEW_PATH)
    {
        if state.compact {
            root.override_local_value_for_debug(
                "dynamic_label",
                FactValue::String("dynamic label expanded".to_string()),
            );
            root.override_local_value_for_debug(
                "demo_items",
                FactValue::StringList(vec!["one".to_string()]),
            );
        } else {
            root.override_local_value_for_debug(
                "dynamic_label",
                FactValue::String("short".to_string()),
            );
            root.override_local_value_for_debug(
                "demo_items",
                FactValue::StringList(vec![
                    "one".to_string(),
                    "two".to_string(),
                    "three".to_string(),
                ]),
            );
        }
        root.override_local_value_for_debug(
            "stage4:hidden_leaf_visible",
            FactValue::Bool(state.hidden_leaf_visible),
        );
        root.override_local_value_for_debug(
            "stage4:hidden_container_visible",
            FactValue::Bool(state.hidden_container_visible),
        );
        root.override_local_value_for_debug(
            "stage4:scroll_offset",
            FactValue::Int(state.scroll_offset),
        );
        root.override_local_value_for_debug(
            "stage4:focus_index",
            FactValue::Int(state.focus_index),
        );
        root.override_local_value_for_debug(
            "stage4:focus_stack_depth",
            FactValue::Int(state.focus_stack_depth),
        );
        root.override_local_value_for_debug(
            "stage4:focus_active",
            FactValue::Bool(state.focus_active),
        );
        root.override_local_value_for_debug(
            "stage4:focus_label",
            FactValue::String(state.focus_label()),
        );
    }
}
