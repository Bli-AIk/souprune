//! # action_handlers.rs
//!
//! # action_handlers.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Registers the overworld-specific custom FRE actions and applies their deferred side
//! effects. It is the place where generic FRE action definitions are translated into Souprune's
//! mode changes, view requests, chase transitions, and danmaku playback.
//!
//! 负责注册大地图专用的自定义 FRE 动作，并落地它们延迟执行的副作用。它把通用的
//! FRE 动作定义翻译成 Souprune 里的模式切换、View 请求、追逐状态切换和弹幕播放。

use super::*;
use crate::core::danmaku::PlayPerformanceEvent;
use crate::core::game_action::{GameActionDef, GameActionHandlerRegistry};

/// Resource to store pending danmaku play requests from FRE actions.
#[derive(Resource, Default)]
pub struct PendingDanmakuActions {
    pub requests: Vec<String>,
}

/// Resource to store pending view spawn/despawn requests from FRE action handlers.
#[derive(Resource, Default)]
pub struct PendingViewActions {
    pub spawn_requests: Vec<crate::core::view::SpawnViewRequest>,
    pub despawn_requests: Vec<crate::core::view::DespawnViewRequest>,
}

/// System to setup overworld-specific action handlers in ActionHandlerRegistry.
pub fn setup_action_handlers_system(world: &mut World) {
    world.init_resource::<PendingDanmakuActions>();
    world.init_resource::<PendingViewActions>();

    let mut handler_registry = world.resource_mut::<GameActionHandlerRegistry>();

    handler_registry.register("SetMode", |action, _db, commands| {
        let GameActionDef::Custom { params, .. } = action else {
            return;
        };
        let Some(mode) = params.get("mode").cloned() else {
            warn!("FRE: SetMode action missing 'mode' param");
            return;
        };
        info!("FRE: Setting mode to '{}' via registered handler", mode);
        commands.queue(move |world: &mut World| {
            world.resource_mut::<crate::app_state::SequenceMode>().0 = Some(mode);
        });
    });

    handler_registry.register("SetSubState", |action, _db, commands| {
        let GameActionDef::Custom { params, .. } = action else {
            return;
        };
        let Some(state) = params.get("state").cloned() else {
            warn!("FRE: SetSubState action missing 'state' param");
            return;
        };
        info!(
            "FRE: Setting sub-state to '{}' via registered handler",
            state
        );
        commands.queue(move |world: &mut World| {
            world
                .resource_mut::<NextState<crate::app_state::SequenceSubState>>()
                .set(crate::app_state::SequenceSubState::new(&state));
        });
    });

    handler_registry.register("EnterChaseState", |_action, _db, commands| {
        commands.queue(move |world: &mut World| {
            let chase_enabled = world.resource::<super::super::chase::ChaseEnabled>().0;
            if !chase_enabled {
                warn!("FRE: EnterChaseState action ignored - chase not enabled");
                return;
            }
            let chase_state_name = world
                .resource::<super::super::chase::ChaseStateName>()
                .0
                .clone();
            let Some(state_name) = chase_state_name else {
                warn!("FRE: EnterChaseState action ignored - no chase state name configured");
                return;
            };
            info!(
                "FRE: Entering chase state '{}' via registered handler",
                state_name
            );
            world
                .resource_mut::<NextState<crate::app_state::SequenceSubState>>()
                .set(crate::app_state::SequenceSubState::new(&state_name));
        });
    });

    handler_registry.register("ExitChaseState", |_action, _db, commands| {
        commands.queue(move |world: &mut World| {
            let chase_enabled = world.resource::<super::super::chase::ChaseEnabled>().0;
            if !chase_enabled {
                warn!("FRE: ExitChaseState action ignored - chase not enabled");
                return;
            }
            info!("FRE: Exiting chase state via registered handler");
            world
                .resource_mut::<NextState<crate::app_state::SequenceSubState>>()
                .set(crate::app_state::SequenceSubState::default());
        });
    });

    handler_registry.register("SpawnView", |action, _db, commands| {
        let GameActionDef::Custom { params, .. } = action else {
            return;
        };
        let Some(path) = params.get("path").cloned() else {
            warn!("FRE: SpawnView action missing 'path' param");
            return;
        };
        info!("FRE: Spawning view '{}' via registered handler", path);
        commands.queue(move |world: &mut World| {
            world
                .resource_mut::<PendingViewActions>()
                .spawn_requests
                .push(crate::core::view::SpawnViewRequest {
                    path,
                    mode_scope: None,
                    bindings: None,
                });
        });
    });

    handler_registry.register("DespawnView", |action, _db, commands| {
        if let GameActionDef::Custom { params, .. } = action {
            let path = params.get("path").cloned();
            info!(
                "FRE: Despawning view(s) via registered handler (path: {:?})",
                path
            );
            commands.queue(move |world: &mut World| {
                world
                    .resource_mut::<PendingViewActions>()
                    .despawn_requests
                    .push(crate::core::view::DespawnViewRequest { path });
            });
        }
    });

    handler_registry.register("PlayDanmaku", |action, _db, commands| {
        let GameActionDef::Custom { params, .. } = action else {
            return;
        };
        let Some(path) = params.get("path").cloned() else {
            warn!("FRE: PlayDanmaku action missing 'path' param");
            return;
        };
        info!("FRE: PlayDanmaku '{}' via registered handler", path);
        commands.queue(move |world: &mut World| {
            world
                .resource_mut::<PendingDanmakuActions>()
                .requests
                .push(path);
        });
    });

    handler_registry.register("SetPlayerHP", |action, _db, _commands| {
        if let GameActionDef::Custom { params, .. } = action
            && let Some(value_str) = params.get("value")
            && let Ok(hp) = value_str.parse::<usize>()
        {
            info!("FRE Action: SetPlayerHP requested with value {}", hp);
        }
    });

    info!("FRE: Overworld action handlers registered (8 handlers)");
}

/// System that handles unregistered Custom FRE actions.
pub fn handle_overworld_custom_actions_system(
    mut events: MessageReader<crate::core::fre_bridge::FreCustomActionEvent>,
) {
    for event in events.read() {
        debug!(
            "FRE: Unhandled custom action '{}' with params {:?}",
            event.action_type, event.params
        );
    }
}

/// System to apply pending view actions from registered FRE action handlers.
pub fn apply_pending_view_actions_system(
    mut pending: ResMut<PendingViewActions>,
    mut spawn_writer: MessageWriter<crate::core::view::SpawnViewRequest>,
    mut despawn_writer: MessageWriter<crate::core::view::DespawnViewRequest>,
) {
    for request in pending.spawn_requests.drain(..) {
        spawn_writer.write(request);
    }
    for request in pending.despawn_requests.drain(..) {
        despawn_writer.write(request);
    }
}

/// System to play danmaku from pending FRE PlayDanmaku actions.
pub fn play_danmaku_from_actions_system(
    mut performance_writer: MessageWriter<PlayPerformanceEvent>,
    player_query: Query<&Transform, With<PlayerControlled>>,
    mut pending: ResMut<PendingDanmakuActions>,
) {
    if pending.requests.is_empty() {
        return;
    }

    let spawn_pos = player_query
        .iter()
        .next()
        .map(|t| t.translation.truncate())
        .unwrap_or(Vec2::ZERO);

    for path in pending.requests.drain(..) {
        info!(
            "FRE: Playing danmaku performance: {} at {:?}",
            path, spawn_pos
        );
        performance_writer.write(PlayPerformanceEvent::new(&path).at_position(spawn_pos));
    }
}
