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
//!
//! ## Source File Overview
//!
//! ## 源文件概述
//!
//! It manages the root UI entity that controls the menu system.
//!
//! 管理控制菜单系统的根 UI 实体。

use super::components::{RonUI, UILayer, UILayerNavigationConfig};
use crate::app_state::overworld::OverworldEntity;
use crate::extra::mortar::LocaleLoaded;
use bevy::prelude::*;

/// Spawn the root UI entity that drives the backpack menu.
///
/// 生成 Undertale 风背包菜单的根 UI 实体。
pub(crate) fn spawn_backpack_ui_system(
    mut commands: Commands,
    overworld_ui_query: Query<&RonUI>,
    locale_loaded: Option<Res<LocaleLoaded>>,
    navigation_config: Res<UILayerNavigationConfig>,
) {
    if locale_loaded.is_none() {
        return;
    }

    // Only create the UI if it does not exist yet and we are in the menu state.
    //
    // 仅在处于菜单状态且 UI 尚未存在时才创建 UI。
    if !overworld_ui_query.is_empty() {
        return;
    }

    // Get max_index from navigation config, or fallback to hardcoded value
    //
    // 从导航配置中获取 max_index，或回退到硬编码值
    let player_data = crate::core::data::PlayerData::default();
    let max_index = navigation_config
        .get(&UILayer::BACKPACK_MENU)
        .and_then(|rule| {
            rule.max_index()
                .as_ref()
                .map(|bound| super::ron_ui_system::evaluate_index_bound(bound, &player_data))
        })
        .unwrap_or_else(|| UILayer::BACKPACK_MENU_OPTIONS.len());

    commands.spawn((
        OverworldEntity(),
        RonUI::new(UILayer::BACKPACK_MENU, max_index),
        // Add a Transform so the UI entity can be positioned.
        //
        // 添加 Transform 组件以便控制 UI 实体的位置。
        Transform::from_translation(Vec3::ZERO),
        Name::new("Backpack Menu UI"),
    ));

    info!(
        "Spawned backpack UI in Menu state with max_index {}",
        max_index
    );
}

/// Despawn the root UI entity.
///
/// 销毁根 UI 实体。
pub(crate) fn despawn_backpack_ui_system(
    mut commands: Commands,
    overworld_ui_query: Query<Entity, With<RonUI>>,
) {
    for entity in &overworld_ui_query {
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
