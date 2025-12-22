//! # sequencer.rs
//!
//! # sequencer.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Sequencer is the linear sequence manager for the battle system.
//! It is responsible for managing and executing Chapters in the battle,
//! ensuring they proceed in order.
//!
//! Sequencer 是战斗系统的线性序列管理器。
//! 它负责管理和执行战斗中的章节（Chapter），确保它们按顺序进行。

pub(crate) struct SequencerPlugin;

impl Plugin for SequencerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, advance_battle_flow_system);
    }
}

use super::chapter::Chapter;
use bevy::prelude::*;

/// [Resource] includes the queue of Chapters that have not yet occurred
///
/// [Resource] 存放还没发生的章节队列
#[derive(Resource, Default)]
struct BattleQueue {
    chapters: Vec<Chapter>,
}

#[derive(Component)]
struct ActiveChapter(Chapter);

#[derive(Event)]
struct ChapterFinishedEvent;

/// Advance the battle flow system.
/// When there is no active chapter running,
/// it takes the next chapter from the BattleQueue and executes it.
///
/// 调度器系统，负责推进战斗流程。
/// 在没有章节运行时，从 BattleQueue 中取出下一个章节并执行它。
fn advance_battle_flow_system(
    mut commands: Commands,
    mut queue: ResMut<BattleQueue>,
    active_query: Query<Entity, With<ActiveChapter>>,
) {
    // If there is already an active chapter, do nothing
    //
    // 如果已经有一个激活的章节，那就啥也不做
    if !active_query.is_empty() {
        return;
    }

    // If there are no chapters left in the queue, do nothing
    //
    // 如果队列中没有剩余章节，依旧啥也不做
    if queue.chapters.is_empty() {
        return;
    }
    let next_chapter = queue.chapters.remove(0);

    // TODO 给这个新 Entity 挂上对应的逻辑组件（比如 Timer，HealthWatcher，DialogueListener）...

    commands.spawn(ActiveChapter(next_chapter));
}

//TODO: 执行器系统
