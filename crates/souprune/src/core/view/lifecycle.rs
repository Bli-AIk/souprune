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
//! for navigation and transitions instead of the legacy RonUI component.
//!
//! 管理控制菜单系统的根 UI 实体，使用 InteractiveLayer 进行导航和转换，
//! 取代旧的 RonUI 组件。

use super::components::InteractiveLayer;
use super::layout::ViewLayoutAsset;
use super::ron_view::UILayoutHandle;
use crate::app_state::overworld::OverworldEntity;
use crate::extra::mortar::LocaleLoaded;
use bevy::prelude::*;

/// Marker component for the backpack UI root entity.
///
/// 背包 UI 根实体的标记组件。
#[derive(Component)]
pub struct BackpackUIRoot;

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
    view_layout_handle: Option<Res<UILayoutHandle>>,
    view_layouts: Res<Assets<ViewLayoutAsset>>,
) {
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

    if !interactive_layers.contains_key("BackpackMenu") {
        warn!("BackpackMenu layer not found in interactive_layers");
        return;
    }

    // Spawn the UI root marker entity
    // 生成 UI 根标记实体
    commands.spawn((
        OverworldEntity(),
        BackpackUIRoot,
        Transform::from_translation(Vec3::ZERO),
        Name::new("Backpack Menu UI Root"),
    ));

    // Spawn InteractiveLayer entities for each defined layer
    // 为每个定义的层生成 InteractiveLayer 实体
    for (layer_id, layer_def) in interactive_layers {
        let mut layer = layer_def.build(layer_id);

        // Activate the initial layer (BackpackMenu)
        // 激活初始层（BackpackMenu）
        if layer_id == "BackpackMenu" {
            layer.is_active = true;
        }

        commands.spawn((
            OverworldEntity(),
            layer,
            Name::new(format!("InteractiveLayer:{}", layer_id)),
        ));

        info!("Spawned InteractiveLayer '{}' for backpack menu", layer_id);
    }

    info!("Spawned backpack UI with InteractiveLayer system");
}

/// Despawn the root UI entity and all associated InteractiveLayers.
///
/// 销毁根 UI 实体及所有关联的 InteractiveLayer。
pub(crate) fn despawn_backpack_ui_system(
    mut commands: Commands,
    root_query: Query<Entity, With<BackpackUIRoot>>,
    interactive_layer_query: Query<Entity, With<InteractiveLayer>>,
) {
    // Despawn InteractiveLayers
    // 销毁 InteractiveLayers
    for entity in &interactive_layer_query {
        commands.entity(entity).despawn();
        debug!("Despawned InteractiveLayer entity {:?}", entity);
    }

    // Despawn BackpackUIRoot and its children
    // 销毁 BackpackUIRoot 及其子实体
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
