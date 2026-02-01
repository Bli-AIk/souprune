//! # lifecycle.rs
//!
//! # lifecycle.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This module handles the spawning and lifecycle of UI entities using the unified
//! InteractiveLayer system for both OW and Battle contexts.
//!
//! 本模块使用统一的 InteractiveLayer 系统处理 OW 和 Battle 场景下 UI 实体的
//! 生成和生命周期。
//!
//! ## Source File Overview
//!
//! ## 源文件概述
//!
//! It manages the root UI entity that controls the menu system, using InteractiveLayer
//! for navigation and transitions.
//!
//! 管理控制菜单系统的根 UI 实体，使用 InteractiveLayer 进行导航和转换。

use super::components::InteractiveLayer;
use super::layout::ViewLayoutAsset;
use super::ron_view::ViewLayoutHandle;
use crate::app_state::overworld::{OverworldEntity, OverworldSubState};
use crate::extra::mortar::LocaleLoaded;
use bevy::prelude::*;

/// Resource to track UI interactive state transitions.
/// Since OverworldSubState is now dynamic (string-based), we can't use OnEnter/OnExit directly.
/// This tracker monitors the `ui_interactive` flag from state config.
///
/// 用于跟踪 UI 交互状态转换的资源。
/// 由于 OverworldSubState 现在是动态的（基于字符串），我们无法直接使用 OnEnter/OnExit。
/// 此追踪器监视状态配置中的 `ui_interactive` 标志。
#[derive(Resource, Default)]
pub struct UIInteractiveStateTracker {
    /// Whether we were in a UI interactive state last frame.
    pub was_ui_interactive: bool,
    /// The layout handle for the current UI interactive state.
    pub current_layout_handle: Option<Handle<ViewLayoutAsset>>,
    /// The initial layer for the current UI interactive state.
    pub current_initial_layer: Option<String>,
}

/// Marker component for the UI root entity (formerly BackpackViewRoot).
///
/// UI 根实体的标记组件（原 BackpackViewRoot）。
#[derive(Component)]
pub struct BackpackViewRoot;

/// Spawn the UI entities using the unified InteractiveLayer system.
///
/// 使用统一的 InteractiveLayer 系统生成 UI 实体。
///
/// This system reads `interactive_layers` from the view layout RON file and spawns
/// separate entities for each layer. The initial layer (BackpackMenu) is activated.
///
/// 该系统从视图布局 RON 文件中读取 `interactive_layers` 并为每个层生成
/// 单独的实体。初始层（BackpackMenu）会被激活。
pub(crate) fn spawn_backpack_ui_system(
    mut commands: Commands,
    interactive_layer_query: Query<&InteractiveLayer>,
    locale_loaded: Option<Res<LocaleLoaded>>,
    view_layout_handle: Option<Res<ViewLayoutHandle>>,
    view_layouts: Res<Assets<ViewLayoutAsset>>,
    layered_db: Res<bevy_fact_rule_event::LayeredFactDatabase>,
) {
    use super::ron_view::parsing::PlayerDataView;
    let player_data = PlayerDataView::new(&layered_db);

    if locale_loaded.is_none() {
        return;
    }

    // Only create the UI if it does not exist yet
    //
    // 仅在 UI 尚未存在时才创建 UI
    if !interactive_layer_query.is_empty() {
        return;
    }

    // Get interactive_layers from view layout
    // 从视图布局获取 interactive_layers
    let Some(handle) = view_layout_handle.as_ref() else {
        warn!("No view layout handle found for backpack UI");
        return;
    };

    let Some(layout) = view_layouts.get(&handle.handle) else {
        debug!("View layout not yet loaded, waiting...");
        return;
    };

    let Some(interactive_layers) = &layout.interactive_layers else {
        warn!("No interactive_layers defined in view layout, cannot spawn backpack UI");
        return;
    };

    // Determine the initial layer to activate
    // 确定要激活的初始层
    let initial_layer = layout.initial_layer.as_deref().unwrap_or_else(|| {
        // Fall back to the first key in interactive_layers
        // 回退到 interactive_layers 中的第一个键
        interactive_layers
            .keys()
            .next()
            .map(|s| s.as_str())
            .unwrap_or("default")
    });

    if !interactive_layers.contains_key(initial_layer) {
        warn!(
            "Initial layer '{}' not found in interactive_layers, available layers: {:?}",
            initial_layer,
            interactive_layers.keys().collect::<Vec<_>>()
        );
        return;
    }

    // Spawn the UI root marker entity
    // 生成 UI 根标记实体
    commands.spawn((
        OverworldEntity(),
        BackpackViewRoot,
        Transform::from_translation(Vec3::ZERO),
        Name::new("Backpack Menu UI Root"),
    ));

    // Spawn InteractiveLayer entities for each defined layer
    // 为每个定义的层生成 InteractiveLayer 实体
    for (layer_id, layer_def) in interactive_layers {
        let mut layer = layer_def.build(layer_id, &player_data);

        // Activate the initial layer from configuration
        // 从配置激活初始层
        if layer_id == initial_layer {
            layer.is_active = true;
        }

        commands.spawn((
            OverworldEntity(),
            layer,
            Name::new(format!("InteractiveLayer:{}", layer_id)),
        ));

        info!("Spawned InteractiveLayer '{}' for backpack menu", layer_id);
    }

    info!(
        "Spawned backpack UI with InteractiveLayer system, initial layer: '{}'",
        initial_layer
    );
}

/// Despawn the root UI entity and all associated InteractiveLayers.
///
/// 销毁根 UI 实体及所有关联的 InteractiveLayer。
pub(crate) fn despawn_backpack_ui_system(
    mut commands: Commands,
    root_query: Query<Entity, With<BackpackViewRoot>>,
    interactive_layer_query: Query<Entity, With<InteractiveLayer>>,
) {
    // Despawn InteractiveLayers
    // 销毁 InteractiveLayers
    for entity in &interactive_layer_query {
        commands.entity(entity).despawn();
        debug!("Despawned InteractiveLayer entity {:?}", entity);
    }

    // Despawn BackpackViewRoot and its children
    // 销毁 BackpackViewRoot 及其子实体
    for entity in &root_query {
        let root = entity;
        commands.queue(move |world: &mut World| {
            let mut stack = vec![root];
            while let Some(entity) = stack.pop() {
                if let Ok(entity_ref) = world.get_entity(entity)
                    && let Some(children) = entity_ref.get::<Children>()
                {
                    for child in children.iter() {
                        stack.push(child);
                    }
                }

                let _ = world.despawn(entity);
            }
        });
        info!("Despawned backpack UI");
    }
}

/// System to detect UI interactive state transitions and trigger spawn/despawn.
/// This replaces OnEnter/OnExit for the dynamic OverworldSubState.
/// It checks the `ui_interactive` flag from state config instead of hardcoded state names.
///
/// 检测 UI 交互状态转换并触发生成/销毁的系统。
/// 这替代了动态 OverworldSubState 的 OnEnter/OnExit。
/// 它检查状态配置中的 `ui_interactive` 标志而不是硬编码的状态名称。
pub(crate) fn backpack_state_transition_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    overworld_state: Res<State<OverworldSubState>>,
    state_config: Option<Res<crate::core::state_config::LoadedStateConfig>>,
    mut tracker: ResMut<UIInteractiveStateTracker>,
    interactive_layer_query: Query<&InteractiveLayer>,
    locale_loaded: Option<Res<LocaleLoaded>>,
    view_layouts: Res<Assets<ViewLayoutAsset>>,
    layered_db: Res<bevy_fact_rule_event::LayeredFactDatabase>,
    root_query: Query<Entity, With<BackpackViewRoot>>,
    interactive_layer_entities: Query<Entity, With<InteractiveLayer>>,
) {
    let Some(state_config) = state_config else {
        return;
    };

    // Check if current state has ui_interactive enabled
    let state_name = overworld_state.name();
    let is_ui_interactive = state_config.is_ui_interactive(state_name);

    // Detect entering UI interactive state
    if is_ui_interactive && !tracker.was_ui_interactive {
        info!(
            "Entering UI interactive state '{}' - loading view layout",
            state_name
        );

        // Get view layout path from state config (clone to avoid lifetime issues)
        if let Some(view_layout_path) = state_config
            .get_view_layout(state_name)
            .map(|s| s.to_string())
        {
            let handle = asset_server.load::<ViewLayoutAsset>(&view_layout_path);
            let initial_layer = state_config
                .get(state_name)
                .and_then(|s| s.initial_layer.clone());

            info!(
                "Loading view layout: '{}', initial_layer: {:?}",
                view_layout_path, initial_layer
            );
            tracker.current_layout_handle = Some(handle);
            tracker.current_initial_layer = initial_layer;
        } else {
            warn!(
                "UI interactive state '{}' has no view_layout configured",
                state_name
            );
        }
    }

    // Try to spawn UI if we have a pending layout handle
    if is_ui_interactive
        && tracker.current_layout_handle.is_some()
        && let Some(ref handle) = tracker.current_layout_handle
        && let Some(layout) = view_layouts.get(handle)
    {
        debug!("View layout loaded, spawning UI for state '{}'", state_name);
        spawn_backpack_ui_with_layout(
            &mut commands,
            &interactive_layer_query,
            locale_loaded.as_deref(),
            layout,
            tracker.current_initial_layer.as_deref(),
            &layered_db,
        );
        // Clear handle after spawning (we've used it)
        // Keep initial_layer for reference
    }

    // Detect exiting UI interactive state
    if !is_ui_interactive && tracker.was_ui_interactive {
        info!("Exiting UI interactive state - despawning UI");
        despawn_backpack_ui_internal(&mut commands, &root_query, &interactive_layer_entities);
        tracker.current_layout_handle = None;
        tracker.current_initial_layer = None;
    }

    tracker.was_ui_interactive = is_ui_interactive;
}

/// Spawn UI using a loaded view layout asset.
/// Used by the state transition system after the layout is loaded.
fn spawn_backpack_ui_with_layout(
    commands: &mut Commands,
    interactive_layer_query: &Query<&InteractiveLayer>,
    locale_loaded: Option<&LocaleLoaded>,
    layout: &ViewLayoutAsset,
    override_initial_layer: Option<&str>,
    layered_db: &bevy_fact_rule_event::LayeredFactDatabase,
) {
    use super::ron_view::parsing::PlayerDataView;
    let player_data = PlayerDataView::new(layered_db);

    if locale_loaded.is_none() {
        return;
    }

    // Only create the UI if it does not exist yet
    if !interactive_layer_query.is_empty() {
        return;
    }

    let Some(interactive_layers) = layout.interactive_layers.as_ref() else {
        warn!("No interactive_layers defined in view layout, skipping UI spawn");
        return;
    };

    // Use override initial layer from state config, fall back to layout's initial_layer
    let initial_layer = override_initial_layer
        .map(|s| s.to_string())
        .or_else(|| layout.initial_layer.clone());

    let Some(ref initial_layer_name) = initial_layer else {
        warn!("No initial_layer configured in state config or view layout, skipping UI spawn");
        return;
    };

    if !interactive_layers.contains_key(initial_layer_name) {
        warn!(
            "initial_layer '{}' not found in interactive_layers. Available: {:?}",
            initial_layer_name,
            interactive_layers.keys().collect::<Vec<_>>()
        );
        return;
    }

    commands.spawn((
        OverworldEntity(),
        BackpackViewRoot,
        Transform::from_translation(Vec3::ZERO),
        Name::new("UI Interactive Root"),
    ));

    for (layer_id, layer_def) in interactive_layers {
        let mut layer = layer_def.build(layer_id, &player_data);

        if layer_id == initial_layer_name {
            layer.is_active = true;
        }

        commands.spawn((
            OverworldEntity(),
            layer,
            Name::new(format!("InteractiveLayer:{}", layer_id)),
        ));

        info!(
            "Spawned InteractiveLayer '{}' for UI interactive state",
            layer_id
        );
    }

    info!(
        "Spawned UI with InteractiveLayer system, initial layer: '{}'",
        initial_layer_name
    );
}

/// Internal helper function for spawning backpack UI (legacy, uses ViewLayoutHandle).
#[allow(dead_code)]
fn spawn_backpack_ui_internal(
    commands: &mut Commands,
    interactive_layer_query: &Query<&InteractiveLayer>,
    locale_loaded: Option<&LocaleLoaded>,
    view_layout_handle: Option<&ViewLayoutHandle>,
    view_layouts: &Assets<ViewLayoutAsset>,
    layered_db: &bevy_fact_rule_event::LayeredFactDatabase,
) {
    use super::ron_view::parsing::PlayerDataView;
    let player_data = PlayerDataView::new(layered_db);

    if locale_loaded.is_none() {
        return;
    }

    if !interactive_layer_query.is_empty() {
        return;
    }

    let Some(layout_handle) = view_layout_handle else {
        return;
    };

    let Some(layout) = view_layouts.get(&layout_handle.handle) else {
        return;
    };

    let Some(interactive_layers) = layout.interactive_layers.as_ref() else {
        warn!("No interactive_layers defined in view layout, skipping backpack UI spawn");
        return;
    };
    let Some(initial_layer) = layout.initial_layer.as_ref() else {
        warn!("No initial_layer defined in view layout, skipping backpack UI spawn");
        return;
    };

    if !interactive_layers.contains_key(initial_layer) {
        warn!(
            "initial_layer '{}' not found in interactive_layers. Available: {:?}",
            initial_layer,
            interactive_layers.keys().collect::<Vec<_>>()
        );
        return;
    }

    commands.spawn((
        OverworldEntity(),
        BackpackViewRoot,
        Transform::from_translation(Vec3::ZERO),
        Name::new("Backpack Menu UI Root"),
    ));

    for (layer_id, layer_def) in interactive_layers {
        let mut layer = layer_def.build(layer_id, &player_data);

        if layer_id == initial_layer {
            layer.is_active = true;
        }

        commands.spawn((
            OverworldEntity(),
            layer,
            Name::new(format!("InteractiveLayer:{}", layer_id)),
        ));

        info!("Spawned InteractiveLayer '{}' for backpack menu", layer_id);
    }

    info!(
        "Spawned backpack UI with InteractiveLayer system, initial layer: '{}'",
        initial_layer
    );
}

/// Internal helper function for despawning backpack UI.
fn despawn_backpack_ui_internal(
    commands: &mut Commands,
    root_query: &Query<Entity, With<BackpackViewRoot>>,
    interactive_layer_query: &Query<Entity, With<InteractiveLayer>>,
) {
    for entity in interactive_layer_query.iter() {
        commands.entity(entity).despawn();
        debug!("Despawned InteractiveLayer entity {:?}", entity);
    }

    for entity in root_query.iter() {
        let root = entity;
        commands.queue(move |world: &mut World| {
            let mut stack = vec![root];
            while let Some(entity) = stack.pop() {
                if let Ok(entity_ref) = world.get_entity(entity)
                    && let Some(children) = entity_ref.get::<Children>()
                {
                    for child in children.iter() {
                        stack.push(child);
                    }
                }

                let _ = world.despawn(entity);
            }
        });
        info!("Despawned backpack UI");
    }
}
