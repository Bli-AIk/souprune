//! # ui_box.rs
//!
//! # ui_box.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This module handles the rendering of UI boxes using bevy_smud for SDF shapes.
//!
//! 本模块使用 bevy_smud 处理 UI 盒子的 SDF 形状渲染。
//!
//! ## Source File Overview
//!
//! ## 源文件概述
//!
//! It manages box geometry, text content, and visibility based on the current UI layer.
//!
//! 管理盒子几何形状、文本内容和基于当前 UI 层级的可见性。

use super::components::{OverworldUI, OverworldUIBox, OverworldUIBoxVisibility, UIBoxFiller};
use super::text::NeedsGlyphRefresh;
use crate::app_state::overworld::OverworldState;
use bevy::ecs::relationship::Relationship;
use bevy::prelude::*;
use bevy::sprite_render::AlphaMode2d;
use bevy_rich_text3d::{SegmentStyle, Text3d, Text3dSegment, Text3dStyling, TextAtlas};
use bevy_smud::prelude::SdfAssets;
use bevy_smud::{Frame, SmudShape};
use std::collections::VecDeque;

/// Parse text with color tags while preserving whitespace.
/// Supports `{#RRGGBB:text}` syntax for colored text.
fn parse_text_preserving_whitespace(text: &str) -> Text3d {
    let mut segments = Vec::new();
    let mut buffer = String::new();
    let mut chars = text.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '{' && chars.peek() == Some(&'#') {
            // Push accumulated text
            if !buffer.is_empty() {
                segments.push((
                    Text3dSegment::String(buffer.clone()),
                    SegmentStyle::default(),
                ));
                buffer.clear();
            }

            // Parse color tag: {#RRGGBB:content}
            chars.next(); // consume '#'
            let mut color_str = String::new();
            while let Some(&ch) = chars.peek() {
                if ch == ':' {
                    chars.next();
                    break;
                }
                color_str.push(chars.next().unwrap());
            }

            // Parse content until '}'
            let mut content = String::new();
            let mut depth = 1;
            for ch in chars.by_ref() {
                if ch == '{' {
                    depth += 1;
                    content.push(ch);
                } else if ch == '}' {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                    content.push(ch);
                } else {
                    content.push(ch);
                }
            }

            // Add colored segment
            if let Ok(color) = Srgba::hex(&color_str) {
                segments.push((
                    Text3dSegment::String(content),
                    SegmentStyle {
                        fill_color: Some(color),
                        ..Default::default()
                    },
                ));
            } else {
                // Fallback: treat as plain text
                segments.push((
                    Text3dSegment::String(format!("{{#{}:{}}}", color_str, content)),
                    SegmentStyle::default(),
                ));
            }
        } else {
            buffer.push(c);
        }
    }

    // Push remaining text
    if !buffer.is_empty() {
        segments.push((Text3dSegment::String(buffer), SegmentStyle::default()));
    }

    Text3d { segments }
}

type OverworldUIBoxQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static OverworldUIBox,
        &'static Transform,
        Option<&'static Children>,
    ),
    Or<(
        Added<OverworldUIBox>,
        Changed<OverworldUIBox>,
        Changed<Transform>,
    )>,
>;

/// Create SmudShape child entities for each UI box.
///
/// 为 UI 框创建 SmudShape 子实体。
fn spawn_ui_box_children(
    commands: &mut Commands,
    entity: Entity,
    ui_box: &OverworldUIBox,
    outer_sdf: Handle<Shader>,
    inner_sdf: Handle<Shader>,
    shaders: &mut ResMut<Assets<Shader>>,
    color_materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    info!("Spawning SmudShape children for UI box");

    let box_width = ui_box.width();
    let box_height = ui_box.height();
    let border_width = ui_box.border_width();

    let shader_source = super::shaders::load_ui_solid_fill_body();
    let solid_fill = shaders.add_fill_body(&shader_source);

    let mut filler_entity: Option<Entity> = None;

    commands.entity(entity).with_children(|parent| {
        parent
            .spawn((
                SmudShape {
                    color: Color::WHITE,
                    sdf: outer_sdf.clone(),
                    frame: Frame::Quad((box_width + border_width * 2.0) + 10.0),
                    fill: solid_fill.clone(),
                    ..default()
                },
                Transform::from_translation(Vec3::new(0.0, 0.0, 5.0)),
                Name::new("UIBoxBorder"),
            ))
            .with_children(|border_parent| {
                let filler = border_parent
                    .spawn((
                        SmudShape {
                            color: Color::BLACK,
                            sdf: inner_sdf.clone(),
                            frame: Frame::Quad(box_width.max(box_height) + 10.0),
                            fill: solid_fill.clone(),
                            ..default()
                        },
                        Transform::from_translation(Vec3::new(0.0, 0.0, 0.1)),
                        Name::new("UIBoxFiller"),
                        UIBoxFiller,
                    ))
                    .id();

                filler_entity = Some(filler);
            });
    });

    let Some(filler_entity) = filler_entity else {
        warn!("Failed to spawn UI box filler for entity {:?}", entity);
        return;
    };

    commands
        .entity(filler_entity)
        .with_children(|filler_parent| {
            for text_config in &ui_box.texts {
                info!("Spawning text for UI box: {}", text_config.content);

                let mat = color_materials.add(ColorMaterial {
                    texture: Some(TextAtlas::DEFAULT_IMAGE.clone()),
                    alpha_mode: AlphaMode2d::Blend,
                    ..Default::default()
                });

                // Manually parse color tags to preserve whitespace
                // Text3d::parse() collapses consecutive spaces, so we build segments manually
                // 手动解析颜色标签以保留空格
                // Text3d::parse() 会合并连续空格，所以我们手动构建片段
                let text3d = parse_text_preserving_whitespace(&text_config.content);

                filler_parent.spawn((
                    text_config.name.clone(),
                    text3d,
                    Text3dStyling {
                        font: text_config.font.font_name().into(),
                        size: text_config.font.default_size(),
                        world_scale: Some(text_config.world_scale),
                        color: text_config.color,
                        align: text_config.align,
                        anchor: text_config.anchor,
                        line_height: text_config.line_height,
                        ..Default::default()
                    },
                    Mesh2d::default(),
                    MeshMaterial2d(mat.clone()),
                    text_config.transform,
                    Visibility::Hidden,
                    NeedsGlyphRefresh,
                ));
            }
        });
}

/// Update UI box geometry each time layout components change.
///
/// 当布局组件变化时更新 UI 框的几何数据。
pub(crate) fn update_overworld_ui_box_system(
    mut shaders: ResMut<Assets<Shader>>,
    mut commands: Commands,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
    overworld_ui_box_query: OverworldUIBoxQuery,
    mut smud_shape_query: Query<&mut SmudShape>,
    children_query: Query<&Children>,
) {
    for (entity, ui_box, transform, children_opt) in overworld_ui_box_query.iter() {
        let box_width = ui_box.width();
        let box_height = ui_box.height();
        let border_width = ui_box.border_width();

        let outer_sdf = shaders.add_sdf_expr(format!(
            "smud::sd_box(p, vec2<f32>({}, {}))",
            (box_width + border_width * 2.0) / 2.0,
            (box_height + border_width * 2.0) / 2.0
        ));

        let inner_sdf = shaders.add_sdf_expr(format!(
            "smud::sd_box(p, vec2<f32>({}, {}))",
            box_width / 2.0,
            box_height / 2.0
        ));

        match children_opt {
            Some(children) => {
                let mut queue: VecDeque<Entity> = VecDeque::from(children.to_vec());
                let mut smud_shape_entities: Vec<Entity> = Vec::new();

                while let Some(child) = queue.pop_front() {
                    if smud_shape_query.get(child).is_ok() {
                        smud_shape_entities.push(child);
                        if smud_shape_entities.len() >= 2 {
                            break;
                        }
                    }

                    if let Ok(grandchildren) = children_query.get(child) {
                        queue.extend(grandchildren.to_vec());
                    }
                }

                if smud_shape_entities.len() >= 2 {
                    info!("Updating existing SmudShape children for UI box");

                    if let Ok(mut outer_shape) = smud_shape_query.get_mut(smud_shape_entities[0]) {
                        outer_shape.sdf = outer_sdf.clone();
                        outer_shape.frame = Frame::Quad((box_width + border_width * 2.0) + 10.0);
                    }

                    if let Ok(mut inner_shape) = smud_shape_query.get_mut(smud_shape_entities[1]) {
                        inner_shape.sdf = inner_sdf.clone();
                        inner_shape.frame = Frame::Quad(box_width.max(box_height) + 10.0);
                    }
                } else {
                    info!(
                        "Adding SmudShape children to existing UI box at position: {:?}",
                        transform.translation
                    );

                    spawn_ui_box_children(
                        &mut commands,
                        entity,
                        ui_box,
                        outer_sdf,
                        inner_sdf,
                        &mut shaders,
                        &mut color_materials,
                    );
                }
            }
            None => {
                info!(
                    "Creating new SmudShape children for UI box at position: {:?}",
                    transform.translation
                );

                spawn_ui_box_children(
                    &mut commands,
                    entity,
                    ui_box,
                    outer_sdf,
                    inner_sdf,
                    &mut shaders,
                    &mut color_materials,
                );
            }
        }
    }
}

/// Toggle UI box visibility according to the active [`UILayer`].
///
/// 根据当前激活的 [`UILayer`] 切换 UI 框可见性。
pub(crate) fn update_overworld_ui_box_visibility_system(
    overworld_state: Res<State<OverworldState>>,
    ui_query: Query<&OverworldUI>,
    parent_query: Query<&ChildOf>,
    mut box_query: Query<
        (Entity, &OverworldUIBoxVisibility, &mut Visibility),
        With<OverworldUIBox>,
    >,
) {
    let in_backpack = overworld_state.get() == &OverworldState::Backpack;

    for (entity, layer_visibility, mut visibility) in box_query.iter_mut() {
        if !in_backpack {
            if *visibility != Visibility::Hidden {
                *visibility = Visibility::Hidden;
            }
            continue;
        }

        let Ok(parent) = parent_query.get(entity) else {
            if *visibility != Visibility::Hidden {
                *visibility = Visibility::Hidden;
            }
            continue;
        };

        let Ok(overworld_ui) = ui_query.get(parent.get()) else {
            if *visibility != Visibility::Hidden {
                *visibility = Visibility::Hidden;
            }
            continue;
        };

        let should_show = layer_visibility.is_visible_for(overworld_ui.layer());
        if should_show {
            if *visibility != Visibility::Inherited {
                *visibility = Visibility::Inherited;
            }
        } else if *visibility != Visibility::Hidden {
            *visibility = Visibility::Hidden;
        }
    }
}
