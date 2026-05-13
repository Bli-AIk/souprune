//! Chapter handlers for battle box split/merge operations.
//!
//! 战斗框分割/合并操作的章节处理器。

use bevy::ecs::message::MessageWriter;
use bevy::prelude::*;

use crate::core::battle_box::{MergeBattleBoxes, SplitBattleBox};
use crate::core::sequencer::chapter_schema::Chapter;
use crate::core::sequencer::{ActiveChapter, ChapterFinished};

/// System that handles SplitBattleBox / MergeBattleBoxes chapters.
/// Sends the corresponding messages and completes immediately.
///
/// 处理 SplitBattleBox / MergeBattleBoxes 章节。
/// 发送对应的消息并立即完成。
pub fn process_battle_box_chapter_system(
    mut commands: Commands,
    query: Query<(Entity, &ActiveChapter), Without<ChapterFinished>>,
    mut split_writer: MessageWriter<SplitBattleBox>,
    mut merge_writer: MessageWriter<MergeBattleBoxes>,
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
                split_writer.write(SplitBattleBox {
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
                merge_writer.write(MergeBattleBoxes {
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
