//! # 编辑器撤销/重做操作
//!
//! 定义序列编辑操作的 UndoAction 实现。

use bevy::prelude::*;
use bevy_workbench::prelude::*;
use souprune::core::sequencer::chapter_schema::Chapter;

use crate::panels::sequence_timeline::EditorSequenceState;

/// 插入章节操作。
pub struct InsertChapterAction {
    pub index: usize,
    pub chapter: Chapter,
}

impl UndoAction for InsertChapterAction {
    fn undo(&self, world: &mut World) {
        let mut state = world.resource_mut::<EditorSequenceState>();
        if let Some(seq) = &mut state.current
            && self.index < seq.chapters.len()
        {
            seq.chapters.remove(self.index);
            seq.dirty = true;
        }
    }

    fn redo(&self, world: &mut World) {
        let mut state = world.resource_mut::<EditorSequenceState>();
        if let Some(seq) = &mut state.current {
            let pos = self.index.min(seq.chapters.len());
            seq.chapters.insert(pos, self.chapter.clone());
            seq.dirty = true;
        }
    }

    fn description(&self) -> &str {
        "插入章节"
    }
}

/// 删除章节操作。
pub struct RemoveChapterAction {
    pub index: usize,
    pub chapter: Chapter,
}

impl UndoAction for RemoveChapterAction {
    fn undo(&self, world: &mut World) {
        let mut state = world.resource_mut::<EditorSequenceState>();
        if let Some(seq) = &mut state.current {
            let pos = self.index.min(seq.chapters.len());
            seq.chapters.insert(pos, self.chapter.clone());
            seq.dirty = true;
        }
    }

    fn redo(&self, world: &mut World) {
        let mut state = world.resource_mut::<EditorSequenceState>();
        if let Some(seq) = &mut state.current
            && self.index < seq.chapters.len()
        {
            seq.chapters.remove(self.index);
            seq.dirty = true;
        }
    }

    fn description(&self) -> &str {
        "删除章节"
    }
}

/// 移动章节操作。
pub struct MoveChapterAction {
    pub from: usize,
    pub to: usize,
}

impl UndoAction for MoveChapterAction {
    fn undo(&self, world: &mut World) {
        let mut state = world.resource_mut::<EditorSequenceState>();
        if let Some(seq) = &mut state.current
            && self.to < seq.chapters.len()
        {
            let chapter = seq.chapters.remove(self.to);
            let pos = self.from.min(seq.chapters.len());
            seq.chapters.insert(pos, chapter);
            seq.dirty = true;
        }
    }

    fn redo(&self, world: &mut World) {
        let mut state = world.resource_mut::<EditorSequenceState>();
        if let Some(seq) = &mut state.current
            && self.from < seq.chapters.len()
        {
            let chapter = seq.chapters.remove(self.from);
            let pos = self.to.min(seq.chapters.len());
            seq.chapters.insert(pos, chapter);
            seq.dirty = true;
        }
    }

    fn description(&self) -> &str {
        "移动章节"
    }
}

/// 修改章节属性操作。
pub struct ModifyChapterAction {
    pub index: usize,
    pub old_chapter: Chapter,
    pub new_chapter: Chapter,
}

impl UndoAction for ModifyChapterAction {
    fn undo(&self, world: &mut World) {
        let mut state = world.resource_mut::<EditorSequenceState>();
        if let Some(seq) = &mut state.current
            && let Some(ch) = seq.chapters.get_mut(self.index)
        {
            *ch = self.old_chapter.clone();
            seq.dirty = true;
        }
    }

    fn redo(&self, world: &mut World) {
        let mut state = world.resource_mut::<EditorSequenceState>();
        if let Some(seq) = &mut state.current
            && let Some(ch) = seq.chapters.get_mut(self.index)
        {
            *ch = self.new_chapter.clone();
            seq.dirty = true;
        }
    }

    fn description(&self) -> &str {
        "修改章节属性"
    }
}

/// 批量操作（多个操作一起撤销/重做）。
pub struct BatchChapterAction {
    pub actions: Vec<Box<dyn UndoAction>>,
    pub desc: String,
}

impl UndoAction for BatchChapterAction {
    fn undo(&self, world: &mut World) {
        for action in self.actions.iter().rev() {
            action.undo(world);
        }
    }

    fn redo(&self, world: &mut World) {
        for action in &self.actions {
            action.redo(world);
        }
    }

    fn description(&self) -> &str {
        &self.desc
    }
}
