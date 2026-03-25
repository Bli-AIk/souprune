//! Executes diagnostic `Log` chapters that print messages and immediately finish.
//!
//! 执行用于诊断的 `Log` 章节：输出消息，然后立即结束。
//!
//! Keeps a narrow scope because the chapter type itself is small. It keeps
//! the logging chapter behavior out of the main flow systems while still giving
//! sequence authors a dedicated chapter for tracing and debugging sequence execution.
//!
//! 对应的章节类型本身就很小，所以这里只保留很窄的职责。它把日志章节的行为从主流程
//! 系统里剥离出来，同时为序列作者保留了一个专门用于跟踪和调试执行过程的章节类型。

use super::chapter_schema::Chapter;
use super::context::{ActiveChapter, ChapterFinished};
use bevy::prelude::*;

pub fn process_log_chapter_system(
    mut commands: Commands,
    query: Query<(Entity, &ActiveChapter), Without<ChapterFinished>>,
) {
    for (entity, active) in query.iter() {
        if let Chapter::Log { text, level } = &active.chapter {
            level.action(text);
            commands.entity(entity).insert(ChapterFinished);
        }
    }
}
