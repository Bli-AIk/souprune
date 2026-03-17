//! # text.rs
//!
//! ## 模块概述
//!
//! 处理 bitmap text 渲染的辅助系统。
//! bevy_bitmap_text 内部已处理布局和子实体同步，
//! 此模块仅提供 souprune 特定的初始化逻辑。

use bevy::prelude::*;
use bevy_bitmap_text::TextBlockLayout;

/// Show text once layout is computed.
/// Skips entities that have VisibleWhen component (managed by visible_when system).
///
/// 布局计算完成后显示文本。
/// 跳过具有 VisibleWhen 组件的实体（由 visible_when 系统管理）。
pub fn show_text_when_ready_system(
    mut query: Query<
        (&TextBlockLayout, &mut Visibility),
        (
            Changed<TextBlockLayout>,
            Without<super::components::VisibleWhen>,
        ),
    >,
) {
    for (layout, mut visibility) in query.iter_mut() {
        if !layout.glyphs.is_empty() && *visibility == Visibility::Hidden {
            *visibility = Visibility::Inherited;
        }
    }
}
