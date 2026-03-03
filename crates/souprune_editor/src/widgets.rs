//! # Editor UI Widgets
//!
//! # 编辑器 UI 组件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This module contains reusable UI widgets for the SoupRune editor.
//!
//! 此模块包含 SoupRune 编辑器的可重用 UI 组件。

mod chapter_card;
pub mod chapter_palette;
pub mod path_picker;
pub mod property_editors;
pub mod val_editor;

pub use chapter_card::{
    chapter_color, chapter_icon, chapter_summary, chapter_type_name, get_children, has_children,
    render_chapter_card, render_chapter_card_response,
};

#[allow(unused_imports)]
pub(crate) use chapter_card::chapter_i18n_key;
