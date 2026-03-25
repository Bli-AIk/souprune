//! Executes flow-level chapter actions that complete immediately after dispatch.
//!
//! 执行那些分发后即可立刻完成的流程级章节动作。
//!
//! Handles the chapter variants that are essentially imperative
//! dispatch steps in the sequencer: custom actions and battle-box split/merge
//! commands. They do not own long-lived state, so they live in the flow action
//! executor rather than in a separate runtime module.
//!
//! 处理 sequencer 里那类本质上只是命令分发步骤的章节：自定义动作，
//! 以及战斗框的拆分/合并命令。它们不会持有长期状态，因此适合放在流程动作执行器，
//! 而不是再拆出一个独立运行时模块。

use bevy::ecs::message::MessageWriter;
use bevy::prelude::*;
use bevy_fact_rule_event::LayeredFactDatabase;

use super::super::chapter_schema::Chapter;
use super::super::context::{ActiveChapter, ChapterFinished};

/// System that handles Custom chapters by dispatching via ActionHandlerRegistry
/// when a handler is registered, or emitting FreCustomActionEvent as fallback.
/// The chapter completes immediately after dispatching the event.
///
/// 处理 Custom 章节：优先通过 ActionHandlerRegistry 分发，
/// 未注册的处理器则发出 FreCustomActionEvent。章节在分发后立即完成。
pub fn process_custom_chapter_system(
    mut commands: Commands,
    query: Query<(Entity, &ActiveChapter), Without<ChapterFinished>>,
    handler_registry: Res<crate::core::game_action::GameActionHandlerRegistry>,
    fact_db: Res<LayeredFactDatabase>,
    custom_action_writer: Option<MessageWriter<crate::core::fre_bridge::FreCustomActionEvent>>,
) {
    let mut custom_action_writer = custom_action_writer;
    for (entity, active_chapter) in query.iter() {
        if let Chapter::Custom {
            action_type,
            params,
        } = &active_chapter.chapter
        {
            info!(
                "Custom Chapter: dispatching action '{}' with params {:?}",
                action_type, params
            );

            let action = crate::core::game_action::GameActionDef::Custom {
                action_type: action_type.clone(),
                params: params.clone(),
            };

            if handler_registry.has_handler(action_type) {
                handler_registry.execute(&action, &fact_db, &mut commands);
            } else if let Some(ref mut writer) = custom_action_writer {
                writer.write(crate::core::fre_bridge::FreCustomActionEvent {
                    action_type: action_type.clone(),
                    params: params.clone(),
                });
            }

            commands.entity(entity).insert(ChapterFinished);
        }
    }
}

/// System that handles SplitBattleBox / MergeBattleBoxes chapters.
/// Sends the corresponding messages and completes immediately.
///
/// 处理 SplitBattleBox / MergeBattleBoxes 章节。
/// 发送对应的消息并立即完成。
pub fn process_battle_box_chapter_system(
    mut commands: Commands,
    query: Query<(Entity, &ActiveChapter), Without<ChapterFinished>>,
    mut split_writer: MessageWriter<crate::core::battle_box::SplitBattleBox>,
    mut merge_writer: MessageWriter<crate::core::battle_box::MergeBattleBoxes>,
) {
    for (entity, active_chapter) in query.iter() {
        match &active_chapter.chapter {
            Chapter::SplitBattleBox {
                source,
                result,
                axis,
                position,
                gap,
                gap_policy,
                duration,
                easing,
            } => {
                info!(
                    "SplitBattleBox Chapter: '{}' → '{}' + '{}' (axis={:?}, gap_policy={:?}, duration={}, easing={:?})",
                    source, result.0, result.1, axis, gap_policy, duration, easing
                );
                split_writer.write(crate::core::battle_box::SplitBattleBox {
                    source_box: source.clone(),
                    result_boxes: result.clone(),
                    split_axis: *axis,
                    split_position: *position,
                    gap: *gap,
                    gap_policy: *gap_policy,
                    duration: *duration,
                    easing: *easing,
                });
                commands.entity(entity).insert(ChapterFinished);
            }
            Chapter::MergeBattleBoxes {
                sources,
                result,
                gap_policy,
                duration,
                easing,
            } => {
                info!(
                    "MergeBattleBoxes Chapter: '{}' + '{}' → '{}' (gap_policy={:?}, duration={}, easing={:?})",
                    sources.0, sources.1, result, gap_policy, duration, easing
                );
                merge_writer.write(crate::core::battle_box::MergeBattleBoxes {
                    source_boxes: sources.clone(),
                    result_box: result.clone(),
                    gap_policy: *gap_policy,
                    duration: *duration,
                    easing: *easing,
                });
                commands.entity(entity).insert(ChapterFinished);
            }
            _ => {}
        }
    }
}
