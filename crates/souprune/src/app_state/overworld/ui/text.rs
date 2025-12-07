//! # text.rs
//!
//! # text.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This module handles 3D text rendering using bevy_rich_text3d.
//!
//! 本模块使用 bevy_rich_text3d 处理 3D 文本渲染。
//!
//! ## Source File Overview
//!
//! ## 源文件概述
//!
//! It manages glyph refreshing and visibility control for UI text elements.
//!
//! 管理 UI 文本元素的字形刷新和可见性控制。

use bevy::prelude::*;
use bevy_rich_text3d::Text3d;

/// Marker component for newly spawned text that needs glyph refresh
///
/// 新生成文本的标记组件，需要刷新字形
#[derive(Component)]
pub(crate) struct NeedsGlyphRefresh;

/// After spawning, change the Text3d string in PreUpdate phase to immediately render glyphs.
///
/// 在 spawn 后，在 PreUpdate 阶段修改 Text3d 字符串以立刻渲染字形。
pub(crate) fn refresh_text_glyphs_system(
    mut commands: Commands,
    mut text_query: Query<(Entity, &mut Text3d), Added<NeedsGlyphRefresh>>,
) {
    for (entity, mut text) in text_query.iter_mut() {
        if let Some(s) = text.get_single_mut() {
            let current = s.clone();
            s.clear();
            *s = current;
        }

        commands.entity(entity).remove::<NeedsGlyphRefresh>();
        info!("Refreshed glyphs for text entity {:?}", entity);
    }
}

/// Show text once mesh is generated.
///
/// 网格生成后显示文本。
type TextMeshQuery<'w, 's> =
    Query<'w, 's, (&'static Mesh2d, &'static mut Visibility), (With<Text3d>, Changed<Mesh2d>)>;

pub(crate) fn show_text_when_ready_system(mut text_query: TextMeshQuery) {
    for (mesh, mut visibility) in text_query.iter_mut() {
        if mesh.0 != Handle::default() && *visibility == Visibility::Hidden {
            *visibility = Visibility::Inherited;
        }
    }
}
