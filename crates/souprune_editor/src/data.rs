//! # Data Operations
//!
//! # 数据操作模块
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This module handles data operations for the editor: sequence loading/saving, CRUD operations, and undo/redo.
//!
//! 此模块处理编辑器的数据操作：序列加载/保存、CRUD 操作和撤销/重做。

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
