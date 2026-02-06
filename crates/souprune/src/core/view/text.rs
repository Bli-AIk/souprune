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
//! It manages glyph refreshing for UI text elements.
//!
//! 管理 UI 文本元素的字形刷新。

use bevy::prelude::*;
use bevy::sprite_render::AlphaMode2d;
use bevy_rich_text3d::{Text3d, TextAtlas};

/// Marker component for newly spawned text that needs glyph refresh
///
/// 新生成文本的标记组件，需要刷新字形
#[derive(Component)]
pub(crate) struct NeedsGlyphRefresh;

/// Marker component for text entities that need material assignment.
///
/// 需要分配材质的文本实体的标记组件。
#[derive(Component)]
pub(crate) struct NeedsTextMaterial;

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
        debug!("Refreshed glyphs for text entity {:?}", entity);
    }
}

/// Assign proper ColorMaterial to text entities that need it.
/// This fixes the purple box issue for container texts.
///
/// 为需要的文本实体分配正确的 ColorMaterial。
/// 这修复了容器文本的紫色方块问题。
pub(crate) fn assign_text_material_system(
    mut commands: Commands,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
    text_query: Query<Entity, (With<Text3d>, With<NeedsTextMaterial>)>,
) {
    for entity in text_query.iter() {
        let mat = color_materials.add(ColorMaterial {
            texture: Some(TextAtlas::DEFAULT_IMAGE.clone()),
            alpha_mode: AlphaMode2d::Blend,
            ..Default::default()
        });

        commands
            .entity(entity)
            .insert(MeshMaterial2d(mat))
            .remove::<NeedsTextMaterial>();

        debug!("Assigned text material to entity {:?}", entity);
    }
}

/// Show text once mesh is generated.
/// Skips entities that have VisibleWhen component (they're managed by visible_when system).
///
/// 网格生成后显示文本。
/// 跳过具有 VisibleWhen 组件的实体（它们由 visible_when 系统管理）。
type TextMeshQuery<'w, 's> = Query<
    'w,
    's,
    (&'static Mesh2d, &'static mut Visibility),
    (
        With<Text3d>,
        Changed<Mesh2d>,
        Without<super::components::VisibleWhen>,
    ),
>;

pub(crate) fn show_text_when_ready_system(mut text_query: TextMeshQuery) {
    for (mesh, mut visibility) in text_query.iter_mut() {
        if mesh.0 != Handle::default() && *visibility == Visibility::Hidden {
            *visibility = Visibility::Inherited;
        }
    }
}
