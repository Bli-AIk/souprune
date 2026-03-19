use super::chapter_schema::{Chapter, LogLevel};
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
