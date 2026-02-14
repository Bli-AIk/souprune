//! # lifecycle.rs
//!
//! # lifecycle.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This module handles the spawning and lifecycle of UI entities.
//!
//! 本模块处理 UI 实体的生成和生命周期。

// TODO: Refactor View spawning mechanism / 重构 View 生成机制
//
// Currently there are two ways to spawn Views:
// 1. **State-driven**: `backpack_state_transition_system` uses `ui_interactive` flag
//    from `states.ron` to automatically spawn/despawn Views when state changes
// 2. **FRE action-driven**: `SpawnViewRequest` allows FRE rules to spawn Views dynamically
//
// This causes confusion about which method to use:
// - Backpack uses method 1 (state-driven)
// - Dialogue uses method 2 (FRE action-driven)
//
// A unified approach should be considered:
// - Either extend state config to support all View spawn scenarios
// - Or remove state-driven spawning and use FRE actions consistently
//
// 目前有两种生成 View 的方式：
// 1. **状态驱动**：`backpack_state_transition_system` 使用 `states.ron` 中的
//    `ui_interactive` 标志在状态变化时自动生成/销毁 View
// 2. **FRE action 驱动**：`SpawnViewRequest` 允许 FRE 规则动态生成 View
//
// 这导致使用哪种方法的困惑：
// - 背包使用方式 1（状态驱动）
// - 对话使用方式 2（FRE action 驱动）
//
// 应考虑统一的方法：
// - 扩展状态配置以支持所有 View 生成场景
// - 或移除状态驱动生成，统一使用 FRE action

use super::layout::ViewLayoutAsset;
use super::ron_view::ViewLayoutHandle;
use crate::app_state::overworld::{OverworldEntity, OverworldSubState};
use crate::core::audio;
use crate::extra::mortar::LocaleLoaded;
use bevy::prelude::*;
use bevy_fact_rule_event::LayeredRuleRegistry;

use super::components::ViewRoot;

/// Resource to track state transitions for playing sounds.
/// This monitors state changes and plays configured on_enter/on_exit sounds.
///
/// 用于跟踪状态转换以播放声音的资源。
/// 监视状态变化并播放配置的进入/退出声音。
#[derive(Resource, Default)]
pub struct StateTransitionTracker {
    /// The previous state name.
    pub previous_state: Option<String>,
}

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
}

/// Marker component for the UI root entity (formerly BackpackViewRoot).
///
/// UI 根实体的标记组件（原 BackpackViewRoot）。
#[derive(Component)]
pub struct BackpackViewRoot;

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
    locale_loaded: Option<Res<LocaleLoaded>>,
    view_layouts: Res<Assets<ViewLayoutAsset>>,
    root_query: Query<Entity, With<BackpackViewRoot>>,
) {
    let Some(state_config) = state_config else {
        return;
    };

    // Check if current state has ui_interactive enabled
    let state_name = overworld_state.name();
    let is_ui_interactive = state_config.is_ui_interactive(state_name);

    // Log state every frame for debugging
    trace!(
        "[lifecycle] state='{}', is_ui_interactive={}, was_ui_interactive={}, has_handle={}, root_count={}",
        state_name,
        is_ui_interactive,
        tracker.was_ui_interactive,
        tracker.current_layout_handle.is_some(),
        root_query.iter().count()
    );

    // Detect entering UI interactive state
    if is_ui_interactive && !tracker.was_ui_interactive {
        info!(
            "[lifecycle] Entering UI interactive state '{}' - loading view layout",
            state_name
        );

        // Get view layout path from state config (clone to avoid lifetime issues)
        if let Some(view_layout_path) = state_config
            .get_view_layout(state_name)
            .map(|s| s.to_string())
        {
            let handle = asset_server.load::<ViewLayoutAsset>(&view_layout_path);

            info!("[lifecycle] Loading view layout: '{}'", view_layout_path);
            tracker.current_layout_handle = Some(handle);
        } else {
            warn!(
                "[lifecycle] UI interactive state '{}' has no view_layout configured",
                state_name
            );
        }
    }

    // Try to spawn UI if we have a pending layout handle
    if is_ui_interactive
        && tracker.current_layout_handle.is_some()
        && let Some(ref handle) = tracker.current_layout_handle
        && view_layouts.get(handle).is_some()
    {
        info!(
            "[lifecycle] View layout loaded, spawning UI for state '{}'",
            state_name
        );

        // Get the view_layout path for the ViewLayoutHandle resource
        // 获取 ViewLayoutHandle 资源的 view_layout 路径
        let view_layout_path = state_config
            .get_view_layout(state_name)
            .map(|s| s.to_string())
            .unwrap_or_default();

        // Insert ViewLayoutHandle resource so spawn_ron_view_system can find it
        // 插入 ViewLayoutHandle 资源以便 spawn_ron_view_system 可以找到它
        info!(
            "[lifecycle] Inserting ViewLayoutHandle resource: '{}'",
            view_layout_path
        );
        commands.insert_resource(ViewLayoutHandle {
            handle: handle.clone(),
            last_modified: None,
            path: view_layout_path,
        });

        spawn_ui_root(&mut commands, locale_loaded.as_deref());
        // Clear handle after spawning to prevent repeated spawning
        // 生成后清除 handle 以防止重复生成
        info!("[lifecycle] Clearing tracker.current_layout_handle after spawn");
        tracker.current_layout_handle = None;
    }

    // Detect exiting UI interactive state
    if !is_ui_interactive && tracker.was_ui_interactive {
        info!("[lifecycle] Exiting UI interactive state - despawning UI");
        despawn_ui(&mut commands, &root_query);
        // Remove ViewLayoutHandle resource when exiting UI state
        // 退出 UI 状态时移除 ViewLayoutHandle 资源
        info!("[lifecycle] Removing ViewLayoutHandle resource");
        commands.remove_resource::<ViewLayoutHandle>();
        tracker.current_layout_handle = None;
    }

    tracker.was_ui_interactive = is_ui_interactive;
}

/// Spawn the UI root entity.
/// The actual view elements will be spawned by spawn_ron_view_system.
///
/// 生成 UI 根实体。
/// 实际的视图元素将由 spawn_ron_view_system 生成。
fn spawn_ui_root(commands: &mut Commands, locale_loaded: Option<&LocaleLoaded>) {
    use crate::core::view::components::ActiveView;

    if locale_loaded.is_none() {
        return;
    }

    commands.spawn((
        OverworldEntity(),
        BackpackViewRoot,
        ActiveView, // Mark as active for FRE system input handling
        Transform::from_translation(Vec3::ZERO),
        Visibility::default(),
        InheritedVisibility::default(),
        ViewVisibility::default(),
        Name::new("UI Interactive Root"),
    ));

    info!(
        "Spawned UI root with ActiveView marker (elements will be spawned by spawn_ron_view_system)"
    );
}

/// Despawn the UI root entity and its children.
///
/// 销毁 UI 根实体及其子实体。
///
/// In Bevy 0.18+, despawn() automatically handles child entities,
/// so we just need to despawn the root entity.
///
/// 在 Bevy 0.18+ 中，despawn() 自动处理子实体，
/// 所以我们只需要销毁根实体。
fn despawn_ui(commands: &mut Commands, root_query: &Query<Entity, With<BackpackViewRoot>>) {
    for entity in root_query.iter() {
        commands.entity(entity).despawn();
        info!("Despawned UI");
    }
}

/// System to play state transition sounds based on state config.
/// This handles on_enter_sound and on_exit_sound for all state transitions.
///
/// 根据状态配置播放状态转换音效的系统。
/// 处理所有状态转换的 on_enter_sound 和 on_exit_sound。
#[cfg(all(feature = "bevy_kira_audio", not(feature = "firewheel")))]
pub(crate) fn state_transition_sound_system(
    audio: Res<bevy_kira_audio::Audio>,
    asset_server: Res<AssetServer>,
    overworld_state: Res<State<OverworldSubState>>,
    state_config: Option<Res<crate::core::state_config::LoadedStateConfig>>,
    mut tracker: ResMut<StateTransitionTracker>,
) {
    let Some(state_config) = state_config else {
        return;
    };

    let current_state = overworld_state.name();

    // Detect state change
    if tracker.previous_state.as_deref() != Some(current_state) {
        // Play exit sound for previous state
        if let Some(ref prev_state) = tracker.previous_state
            && let Some(state_def) = state_config.get(prev_state)
            && let Some(ref sound_path) = state_def.on_exit_sound
        {
            // Use full path function since config contains complete asset path
            // 使用完整路径函数，因为配置包含完整的资源路径
            audio::play_sound_full_path(&audio, &asset_server, sound_path);
            debug!(
                "Playing on_exit_sound for state '{}': {}",
                prev_state, sound_path
            );
        }

        // Play enter sound for current state
        if let Some(state_def) = state_config.get(current_state)
            && let Some(ref sound_path) = state_def.on_enter_sound
        {
            // Use full path function since config contains complete asset path
            // 使用完整路径函数，因为配置包含完整的资源路径
            audio::play_sound_full_path(&audio, &asset_server, sound_path);
            debug!(
                "Playing on_enter_sound for state '{}': {}",
                current_state, sound_path
            );
        }

        tracker.previous_state = Some(current_state.to_string());
    }
}

/// System to play state transition sounds based on state config (firewheel backend).
#[cfg(feature = "firewheel")]
pub(crate) fn state_transition_sound_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    overworld_state: Res<State<OverworldSubState>>,
    state_config: Option<Res<crate::core::state_config::LoadedStateConfig>>,
    mut tracker: ResMut<StateTransitionTracker>,
) {
    let Some(state_config) = state_config else {
        return;
    };

    let current_state = overworld_state.name();

    // Detect state change
    if tracker.previous_state.as_deref() != Some(current_state) {
        // Play exit sound for previous state
        if let Some(ref prev_state) = tracker.previous_state
            && let Some(state_def) = state_config.get(prev_state)
            && let Some(ref sound_path) = state_def.on_exit_sound
        {
            // Use full path function since config contains complete asset path
            // 使用完整路径函数，因为配置包含完整的资源路径
            audio::play_sound_full_path(&mut commands, &asset_server, sound_path);
            debug!(
                "Playing on_exit_sound for state '{}': {}",
                prev_state, sound_path
            );
        }

        // Play enter sound for current state
        if let Some(state_def) = state_config.get(current_state)
            && let Some(ref sound_path) = state_def.on_enter_sound
        {
            // Use full path function since config contains complete asset path
            // 使用完整路径函数，因为配置包含完整的资源路径
            audio::play_sound_full_path(&mut commands, &asset_server, sound_path);
            debug!(
                "Playing on_enter_sound for state '{}': {}",
                current_state, sound_path
            );
        }

        tracker.previous_state = Some(current_state.to_string());
    }
}

/// System to clean up View-layer rules when ViewRoot entities are despawned.
/// Uses RemovedComponents to detect when ViewRoot entities are removed.
///
/// 当 ViewRoot 实体被销毁时清理 View 层规则的系统。
/// 使用 RemovedComponents 来检测 ViewRoot 实体何时被移除。
pub(crate) fn cleanup_view_rules_system(
    mut removed_views: RemovedComponents<ViewRoot>,
    mut rule_registry: ResMut<LayeredRuleRegistry>,
) {
    for entity in removed_views.read() {
        // Clear View-layer rules for this entity
        // 清除此实体的 View 层规则
        rule_registry.clear_view(entity);
        info!(
            "[lifecycle] Cleared View-layer rules for despawned ViewRoot entity {:?}",
            entity
        );
    }
}

/// System to process pending View rules that were waiting for FRE assets to load.
/// Checks PendingViewRules components and registers rules when assets become available.
///
/// 处理等待 FRE 资产加载的待处理 View 规则的系统。
/// 检查 PendingViewRules 组件并在资产可用时注册规则。
pub(crate) fn process_pending_view_rules_system(
    mut commands: Commands,
    fre_assets: Res<Assets<bevy_fact_rule_event::FreAsset>>,
    mut query: Query<(
        Entity,
        &mut super::components::PendingViewRules,
        &mut ViewRoot,
    )>,
    mut rule_registry: ResMut<LayeredRuleRegistry>,
    mut action_defs: ResMut<crate::app_state::overworld::trigger::RuleActionDefs>,
    mortar_strings: Res<crate::extra::mortar::MortarStringTable>,
) {
    use super::ron_view::spawn::load_fre_into_view_root;
    use bevy_fact_rule_event::RuleScope;

    for (entity, mut pending, mut view_root) in query.iter_mut() {
        // Collect handles that are now loaded
        // 收集现在已加载的句柄
        let mut loaded_paths = Vec::new();
        let mut still_pending = Vec::new();

        for (path, handle) in pending.pending_handles.drain(..) {
            if let Some(fre_asset) = fre_assets.get(&handle) {
                // Asset loaded! Register rules
                // 资产已加载！注册规则
                load_fre_into_view_root(&mut view_root, fre_asset, &mortar_strings);

                let rule_defs = fre_asset.get_rule_defs();
                let scope = fre_asset.scope();

                for (idx, rule_def) in rule_defs.iter().enumerate() {
                    // For FRE files loaded via View's requires, treat Local as View
                    // 对于通过 View 的 requires 加载的文件，将 Local 视为 View
                    let effective_scope = if scope == RuleScope::Local {
                        RuleScope::View
                    } else {
                        scope
                    };

                    let rule = rule_def.to_rule_with_index(idx, effective_scope);
                    let rule_id = rule_def.generate_id(idx);

                    // Store actions for this rule
                    // 存储此规则的 actions
                    if !rule_def.actions.is_empty() {
                        action_defs
                            .actions_by_rule
                            .insert(rule_id.clone(), rule_def.actions.clone());
                    }

                    if effective_scope == RuleScope::View {
                        rule_registry.register_view_rule(entity, rule);
                        info!(
                            "[lifecycle] Registered pending View rule '{}' for entity {:?} from '{}'",
                            rule_id, entity, path
                        );
                    } else {
                        rule_registry.register(rule);
                    }
                }

                loaded_paths.push((path, rule_defs.len()));
            } else {
                // Still not loaded, keep in pending (with handle to keep loading alive)
                // 仍未加载，保留在待处理列表中（保留句柄以保持加载请求）
                still_pending.push((path, handle));
            }
        }

        // Log what we loaded
        // 记录我们加载的内容
        for (path, count) in loaded_paths {
            info!(
                "[lifecycle] Processed pending FRE '{}': {} rules for entity {:?}",
                path, count, entity
            );
        }

        // Update pending handles
        // 更新待处理句柄
        pending.pending_handles = still_pending;

        // Remove component if no more pending handles
        // 如果没有更多待处理句柄，移除组件
        if pending.pending_handles.is_empty() {
            commands
                .entity(entity)
                .remove::<super::components::PendingViewRules>();
            info!(
                "[lifecycle] All pending FRE files loaded for entity {:?}, removing PendingViewRules",
                entity
            );
        }
    }
}
