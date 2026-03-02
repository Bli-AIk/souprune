//! 数据操作模块 — 序列加载、保存、CRUD 操作、撤销/重做。

mod sequence_ops;
#[allow(dead_code)]
mod undo_actions;

pub use sequence_ops::{
    auto_save_system, load_sequence_from_file, save_sequence_to_file, EditorSequence,
};

#[allow(unused_imports)]
pub use sequence_ops::SequenceFileEvent;

#[allow(unused_imports)]
pub use undo_actions::{
    BatchChapterAction, InsertChapterAction, ModifyChapterAction, MoveChapterAction,
    RemoveChapterAction,
};
