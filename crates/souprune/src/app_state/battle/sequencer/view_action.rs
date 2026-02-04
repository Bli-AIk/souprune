//! # sequencer/view_action.rs
//!
//! ## Module Overview
//!
//! View control systems for the battle sequencer.
//!
//! 战斗序列管理器的视图控制系统。

use super::super::chapter_schema::{Chapter, DataBinding};
use super::context::*;
use bevy::prelude::*;
use bevy_fact_rule_event::FreAsset;
use std::collections::HashMap;

/// Resource to store view bindings for the current view spawn.
/// Used to pass bindings from SpawnView chapter to the view spawn system.
///
/// 存储当前视图生成的绑定的资源。
/// 用于将 SpawnView 章节的绑定传递给视图生成系统。
#[derive(Resource, Default)]
pub struct PendingViewBindings {
    pub bindings: HashMap<String, DataBinding>,
    /// Handles to FRE assets being loaded for bindings.
    /// 正在为绑定加载的 FRE 资产句柄。
    #[allow(dead_code)]
    pub fre_handles: Vec<Handle<FreAsset>>,
}

/// System to process view actions.
///
/// 处理视图动作的系统。
#[allow(clippy::type_complexity)]
pub fn process_view_action_system(
    mut commands: Commands,
    query: Query<(Entity, &ActiveChapter), (Without<WaitTimer>, Without<ChapterFinished>)>,
    asset_server: Res<AssetServer>,
) {
    for (entity, active_chapter) in query.iter() {
        if let Chapter::SetUI(action) = &active_chapter.chapter {
            match action {
                super::super::chapter_schema::UIAction::LoadLayout(path) => {
                    let handle = asset_server.load(path);
                    commands.insert_resource(crate::core::view::ViewLayoutHandle {
                        handle,
                        last_modified: None,
                        path: path.clone(),
                    });
                    commands.spawn((
                        crate::app_state::battle::BattleViewRoot,
                        crate::core::view::components::ActiveView, // For FRE input handling
                        Transform::default(),
                        GlobalTransform::default(),
                        Visibility::default(),
                        InheritedVisibility::default(),
                        ViewVisibility::default(),
                        crate::app_state::battle::BattleEntity,
                        Name::new("BattleView Root"),
                    ));
                }
                _ => {
                    warn!("View action {:?} not fully implemented yet", action);
                }
            }
            commands.entity(entity).insert(ChapterFinished);
        } else if let Chapter::SpawnView {
            view_layout,
            bindings,
        } = &active_chapter.chapter
        {
            info!("[Battle] Loading view layout for battle: {}", view_layout);
            let handle = asset_server.load(view_layout);
            commands.insert_resource(crate::core::view::ViewLayoutHandle {
                handle,
                last_modified: None,
                path: view_layout.clone(),
            });

            // Pre-load FRE files referenced in bindings
            // 预加载绑定中引用的 FRE 文件
            let mut fre_handles = Vec::new();
            for binding in bindings.values() {
                match binding {
                    DataBinding::File(path) => {
                        let handle: Handle<FreAsset> = asset_server.load(path.clone());
                        info!("[Battle] Pre-loading FRE file for binding: {}", path);
                        fre_handles.push(handle);
                    }
                    DataBinding::Files(paths) => {
                        for path in paths {
                            let handle: Handle<FreAsset> = asset_server.load(path.clone());
                            info!("[Battle] Pre-loading FRE file for binding: {}", path);
                            fre_handles.push(handle);
                        }
                    }
                    _ => {}
                }
            }

            // Store bindings for the view spawn system
            // 存储绑定供视图生成系统使用
            if !bindings.is_empty() {
                commands.insert_resource(PendingViewBindings {
                    bindings: bindings.clone(),
                    fre_handles,
                });
                info!("[Battle] SpawnView with {} bindings", bindings.len());
            }

            commands.spawn((
                crate::app_state::battle::BattleViewRoot,
                crate::core::view::components::ActiveView, // For FRE input handling
                Transform::default(),
                GlobalTransform::default(),
                Visibility::default(),
                InheritedVisibility::default(),
                ViewVisibility::default(),
                crate::app_state::battle::BattleEntity,
                Name::new("BattleView Root"),
            ));
            commands.entity(entity).insert(ChapterFinished);
        }
    }
}
