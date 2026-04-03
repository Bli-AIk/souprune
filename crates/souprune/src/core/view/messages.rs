//! Defines the message boundary used to spawn and despawn RON-driven views.
//!
//! 定义生成与销毁 RON 驱动 View 时使用的消息边界。
//!
//! Turns view creation into an explicit message flow instead of
//! requiring callers to know the low-level entity bundle and pending-binding
//! setup. It also centralizes the teardown path so views can be removed by path
//! or in bulk without every caller reimplementing the same query logic.
//!
//! 把 View 的创建变成显式消息流，调用方不需要知道底层实体 bundle、
//! 绑定数据和待加载资源应该怎么拼装。它也把销毁路径集中到一起，让系统可以按
//! 路径或批量移除 View，而不用每个调用点都重写一遍查询逻辑。

use bevy::prelude::*;

use super::layout::ViewLayoutAsset;
use super::{RonDrivenView, components, ron_view};
use crate::core::mode::ModeScoped;

#[derive(Message, Debug, Clone)]
pub struct SpawnViewRequest {
    pub path: String,
    pub mode_scope: Option<String>,
    pub bindings: Option<
        std::collections::HashMap<String, crate::core::sequencer::chapter_schema::DataBinding>,
    >,
}

#[derive(Message, Debug, Clone)]
pub struct DespawnViewRequest {
    pub path: Option<String>,
}

fn collect_fre_handles(
    bindings: &std::collections::HashMap<
        String,
        crate::core::sequencer::chapter_schema::DataBinding,
    >,
    asset_server: &AssetServer,
) -> Vec<Handle<crate::core::game_action::GameFreAsset>> {
    use crate::core::sequencer::chapter_schema::DataBinding;

    let mut fre_handles = Vec::new();
    for binding in bindings.values() {
        let paths: Vec<&String> = match binding {
            DataBinding::File(path) => vec![path],
            DataBinding::Files(paths) => paths.iter().collect(),
            _ => continue,
        };
        for path in paths {
            let handle = asset_server.load(path.clone());
            info!(
                "[SpawnViewRequest] Pre-loading FRE file for binding: {}",
                path
            );
            fre_handles.push(handle);
        }
    }
    fre_handles
}

pub(super) fn handle_spawn_view_request_system(
    mut events: MessageReader<SpawnViewRequest>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    for request in events.read() {
        info!("Spawning view from request: {}", request.path);

        let handle: Handle<ViewLayoutAsset> = asset_server.load(&request.path);

        // TODO: 未来扩展：如果需要多个 同时交互 的 View，可以引入 `PrimaryView` / `SecondaryView` 概念
        let mut entity_commands = commands.spawn((
            ron_view::HotReloadableViewRoot {
                layout_path: request.path.clone(),
                layout_handle: handle.clone(),
            },
            components::ViewRoot::new(request.path.clone()),
            RonDrivenView,
            Name::new(format!("SpawnedView:{}", request.path)),
            Transform::default(),
            GlobalTransform::default(),
            Visibility::Visible,
            InheritedVisibility::default(),
        ));

        if let Some(ref scope) = request.mode_scope {
            entity_commands.insert(ModeScoped(scope.clone()));
        }

        if let Some(ref bindings) = request.bindings {
            // 只有带 bindings 的 View 才标记为 ActiveView（接收 FRE 规则和交互）
            entity_commands.insert(components::ActiveView);
            let fre_handles = collect_fre_handles(bindings, &asset_server);
            entity_commands.insert(components::PendingViewData {
                bindings: bindings.clone(),
                fre_handles,
            });
        }
    }
}

pub(super) fn handle_despawn_view_request_system(
    mut events: MessageReader<DespawnViewRequest>,
    mut commands: Commands,
    query: Query<(Entity, &components::ViewRoot), With<RonDrivenView>>,
) {
    for request in events.read() {
        let matching: Vec<_> = match request.path {
            Some(ref path) => query
                .iter()
                .filter(|(_, vr)| vr.layout_path == *path)
                .collect(),
            None => query.iter().collect(),
        };

        for &(entity, view_root) in &matching {
            info!(
                "Despawning view: {} (entity {:?})",
                view_root.layout_path, entity
            );
            commands.entity(entity).despawn();
        }

        let despawned_count = matching.len();
        if despawned_count == 0 {
            info!(
                "DespawnViewRequest: no views to despawn (path: {:?})",
                request.path
            );
        } else {
            info!(
                "DespawnViewRequest: despawned {} views (path: {:?})",
                despawned_count, request.path
            );
        }
    }
}
