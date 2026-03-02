//! # lifecycle.rs
//!
//! # lifecycle.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This module handles the spawning and lifecycle of UI entities.
//! All View spawning is unified through SpawnViewRequest/DespawnViewRequest messages.
//!
//! 本模块处理 UI 实体的生成和生命周期。
//! 所有 View 生成均通过 SpawnViewRequest/DespawnViewRequest 消息统一处理。

use crate::app_state::SequenceSubState;
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
/// Since SequenceSubState is dynamic (string-based), we can't use OnEnter/OnExit directly.
/// This tracker monitors the `ui_interactive` flag from state config.
///
/// 用于跟踪 UI 交互状态转换的资源。
/// 由于 SequenceSubState 是动态的（基于字符串），我们无法直接使用 OnEnter/OnExit。
/// 此追踪器监视状态配置中的 `ui_interactive` 标志。
#[derive(Resource, Default)]
pub struct UIInteractiveStateTracker {
    /// Whether we were in a UI interactive state last frame.
    pub was_view_interactive: bool,
    /// The view layout path currently active (for despawning).
    pub current_view_path: Option<String>,
}

/// System to detect UI interactive state transitions and emit SpawnViewRequest/DespawnViewRequest.
/// Checks the `ui_interactive` flag from state config.
pub(crate) fn backpack_state_transition_system(
    sub_state: Res<State<SequenceSubState>>,
    state_config: Option<Res<crate::core::state_config::LoadedStateConfig>>,
    mut tracker: ResMut<UIInteractiveStateTracker>,
    locale_loaded: Option<Res<LocaleLoaded>>,
    mut spawn_writer: MessageWriter<super::SpawnViewRequest>,
    mut despawn_writer: MessageWriter<super::DespawnViewRequest>,
) {
    let Some(state_config) = state_config else {
        return;
    };

    let state_name = sub_state.name();
    let is_view_interactive = state_config.is_view_interactive(state_name);

    trace!(
        "[lifecycle] state='{}', is_view_interactive={}, was_view_interactive={}",
        state_name,
        is_view_interactive,
        tracker.was_view_interactive,
    );

    // Detect entering UI interactive state
    if is_view_interactive && !tracker.was_view_interactive {
        // Only spawn if locale is loaded
        if locale_loaded.is_none() {
            return;
        }

        if let Some(view_layout_path) = state_config
            .get_view_layout(state_name)
            .map(|s| s.to_string())
        {
            info!(
                "[lifecycle] Entering UI interactive state '{}' - emitting SpawnViewRequest: '{}'",
                state_name, view_layout_path
            );

            spawn_writer.write(super::SpawnViewRequest {
                path: view_layout_path.clone(),
                mode_scope: Some("overworld".to_string()),
                bindings: None,
            });

            tracker.current_view_path = Some(view_layout_path);
        } else {
            warn!(
                "[lifecycle] UI interactive state '{}' has no view_layout configured",
                state_name
            );
        }
    }

    // Detect exiting UI interactive state
    if !is_view_interactive && tracker.was_view_interactive {
        info!("[lifecycle] Exiting UI interactive state - emitting DespawnViewRequest");

        despawn_writer.write(super::DespawnViewRequest {
            path: tracker.current_view_path.take(),
        });
    }

    tracker.was_view_interactive = is_view_interactive;
}

/// System to play state transition sounds based on state config.
/// This handles on_enter_sound and on_exit_sound for all state transitions.
///
/// 根据状态配置播放状态转换音效的系统。
/// 处理所有状态转换的 on_enter_sound 和 on_exit_sound。
pub(crate) fn state_transition_sound_system(
    audio: Res<bevy_kira_audio::Audio>,
    asset_server: Res<AssetServer>,
    sub_state: Res<State<SequenceSubState>>,
    state_config: Option<Res<crate::core::state_config::LoadedStateConfig>>,
    mut tracker: ResMut<StateTransitionTracker>,
) {
    let Some(state_config) = state_config else {
        return;
    };

    let current_state = sub_state.name();

    // Detect state change
    if tracker.previous_state.as_deref() != Some(current_state) {
        // Play exit sound for previous state
        if let Some(ref prev_state) = tracker.previous_state
            && let Some(state_def) = state_config.get(prev_state)
            && let Some(ref sound_path) = state_def.on_exit_sound
        {
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
            audio::play_sound_full_path(&audio, &asset_server, sound_path);
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
                        info!(
                            "[lifecycle] Registering View rule '{}' with {} outputs: {:?}",
                            rule_id,
                            rule.outputs.len(),
                            rule.outputs
                        );
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

            // Set view_rules_loaded fact in View's local_facts
            // This allows AwaitFact to wait for FRE rules to be ready
            // 在 View 的 local_facts 中设置 view_rules_loaded fact
            // 这允许 AwaitFact 等待 FRE 规则准备就绪
            view_root.local_facts.set(
                "view_rules_loaded",
                bevy_fact_rule_event::FactValue::Bool(true),
            );

            info!(
                "[lifecycle] All pending FRE files loaded for entity {:?}, removing PendingViewRules, set view_rules_loaded=true",
                entity
            );
        }
    }
}
