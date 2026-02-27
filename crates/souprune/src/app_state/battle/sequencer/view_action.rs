//! # sequencer/view_action.rs
//!
//! ## Module Overview
//!
//! View control systems for the battle sequencer.
//!
//! 战斗序列管理器的视图控制系统。

use super::super::chapter_schema::{Chapter, DataBinding, FactValueMatch};
use super::context::*;
use crate::core::view::components::ViewRoot;
use bevy::prelude::*;
use bevy_fact_rule_event::{FactValue, FreAsset};
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

/// System to process SetViewFact chapters.
///
/// 处理 SetViewFact 章节的系统。
#[allow(clippy::type_complexity)]
pub fn process_set_view_fact_system(
    mut commands: Commands,
    query: Query<(Entity, &ActiveChapter), (Without<WaitTimer>, Without<ChapterFinished>)>,
    mut view_root_query: Query<&mut ViewRoot>,
) {
    for (chapter_entity, active_chapter) in query.iter() {
        if let Chapter::SetViewFact { key, value } = &active_chapter.chapter {
            // Find the active ViewRoot and set the fact
            if let Some(mut view_root) = view_root_query.iter_mut().next() {
                let fact_value = match value {
                    FactValueMatch::Bool(b) => Some(FactValue::Bool(*b)),
                    FactValueMatch::Int(i) => Some(FactValue::Int(*i)),
                    FactValueMatch::Float(f) => Some(FactValue::Float(*f)),
                    FactValueMatch::String(s) => Some(FactValue::String(s.clone())),
                    FactValueMatch::Expr(_expr) => {
                        // Read from LayeredFactDatabase for expression evaluation
                        // This is a SetViewFact, so we don't have direct access to layered_db
                        // For now, just log a warning - this should use a different approach
                        warn!("[Battle] SetViewFact: Expr not supported yet for '{}'", key);
                        None
                    }
                };

                if let Some(fv) = fact_value {
                    view_root.local_facts.set(key.as_str(), fv.clone());
                    info!("[Battle] SetViewFact: Set '{}' = {:?}", key, fv);
                }
            } else {
                warn!("[Battle] SetViewFact: No ViewRoot found!");
            }

            commands.entity(chapter_entity).insert(ChapterFinished);
        }
    }
}
