//! 数据操作模块 — 序列加载、保存、CRUD 操作、撤销/重做。

mod sequence_ops;
mod undo_actions;

pub use sequence_ops::{
    EditorSequence, auto_save_system, load_sequence_from_file, save_sequence_to_file,
};

#[allow(unused_imports)]
pub use sequence_ops::SequenceFileEvent;

pub use undo_actions::{
    InsertChapterAction, ModifyChapterAction, MoveChapterAction, RemoveChapterAction,
};
