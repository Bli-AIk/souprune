//! # sequencer/ui.rs
//!
//! ## Module Overview
//!
//! UI control systems for the battle sequencer.
//!
//! 战斗序列管理器的 UI 控制系统。

use super::super::chapter_schema::Chapter;
use super::context::*;
use bevy::prelude::*;

/// System to process UI actions.
///
/// 处理 UI 动作的系统。
#[allow(clippy::type_complexity)]
pub fn process_ui_action_system(
    mut commands: Commands,
    query: Query<(Entity, &ActiveChapter), (Without<WaitTimer>, Without<ChapterFinished>)>,
    asset_server: Res<AssetServer>,
) {
    for (entity, active_chapter) in query.iter() {
        if let Chapter::SetUI(action) = &active_chapter.chapter {
            match action {
                super::super::chapter_schema::UIAction::LoadLayout(path) => {
                    let handle = asset_server.load(path);
                    commands.insert_resource(crate::core::view::UILayoutHandle {
                        handle,
                        last_modified: None,
                        path: path.clone(),
                    });
                    commands.spawn((
                        crate::app_state::battle::BattleUIRoot,
                        Transform::default(),
                        GlobalTransform::default(),
                        Visibility::default(),
                        InheritedVisibility::default(),
                        ViewVisibility::default(),
                        crate::app_state::battle::BattleEntity,
                        Name::new("BattleUI Root"),
                    ));
                    commands.init_resource::<crate::core::view::UILayoutWatcher>();
                }
                _ => {
                    warn!("UI action {:?} not fully implemented yet", action);
                }
            }
            commands.entity(entity).insert(ChapterFinished);
        } else if let Chapter::SpawnView { view_layout } = &active_chapter.chapter {
            info!("[Battle] Loading view layout for battle: {}", view_layout);
            let handle = asset_server.load(view_layout);
            commands.insert_resource(crate::core::view::UILayoutHandle {
                handle,
                last_modified: None,
                path: view_layout.clone(),
            });
            commands.spawn((
                crate::app_state::battle::BattleUIRoot,
                Transform::default(),
                GlobalTransform::default(),
                Visibility::default(),
                InheritedVisibility::default(),
                ViewVisibility::default(),
                crate::app_state::battle::BattleEntity,
                Name::new("BattleUI Root"),
            ));
            commands.entity(entity).insert(ChapterFinished);
        }
    }
}
