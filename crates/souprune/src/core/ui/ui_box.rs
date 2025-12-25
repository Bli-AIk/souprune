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

use super::components::{
    RonUI, UIBox, UIBoxFiller, UIBoxVisibility, UIShapeStructure, UITextTemplate,
};
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
pub(crate) fn parse_text_preserving_whitespace(text: &str) -> Text3d {
    let mut segments = Vec::new();
    let mut buffer = String::new();
    let mut chars = text.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '{' && chars.peek() == Some(&'#') {
            // Push accumulated text
            //
            // 推送累积的文本
            if !buffer.is_empty() {
                segments.push((
                    Text3dSegment::String(buffer.clone()),
                    SegmentStyle::default(),
                ));
                buffer.clear();
            }

            // Parse color tag: {#RRGGBB:content}
            //
            // 解析颜色标签：{#RRGGBB:content}
            chars.next(); // consume '#'
            let mut color_str = String::new();
            while let Some(&ch) = chars.peek() {
                if ch == ':' {
                    chars.next();
                    break;
                }
                if let Some(c) = chars.next() {
                    color_str.push(c);
                }
            }

            // Parse content until '}'
            //
            // 解析内容直到 '}'
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
            //
            // 添加着色片段
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
                //
                // 回退：视为纯文本
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
    //
    // 推送剩余文本
    if !buffer.is_empty() {
        segments.push((Text3dSegment::String(buffer), SegmentStyle::default()));
    }

    Text3d { segments }
}

type UIBoxQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static UIBox,
        &'static Transform,
        Option<&'static Children>,
    ),
    Or<(Added<UIBox>, Changed<UIBox>, Changed<Transform>)>,
>;

/// Create SmudShape child entities for each UI box.
/// Supports both classic (Border + Filler) and single layer structures.
///
/// 为 UI 框创建 SmudShape 子实体。
/// 支持经典（边框 + 填充）和单层结构。
fn spawn_ui_box_children(
    commands: &mut Commands,
    entity: Entity,
    ui_box: &UIBox,
    outer_sdf: Handle<Shader>,
    inner_sdf: Handle<Shader>,
    shaders: &mut ResMut<Assets<Shader>>,
    color_materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    let box_width = ui_box.width();
    let box_height = ui_box.height();
    let border_width = ui_box.border_width();

    // Load custom shader if specified, otherwise use default solid fill
    // 如果指定了自定义着色器则加载，否则使用默认实体填充
    let shader_source = if let Some(shader_path) = &ui_box.fill_shader {
        info!("Loading custom fill shader for UI box: {}", shader_path);
        super::shaders::load_custom_shader_body(shader_path)
    } else {
        super::shaders::load_ui_solid_fill_body()
    };
    let solid_fill = shaders.add_fill_body(&shader_source);

    match ui_box.structure {
        UIShapeStructure::Single => {
            // Single layer structure - just one SmudShape, no border
            // 单层结构 - 只有一个 SmudShape，无边框
            info!("Spawning single-layer SmudShape for UI box");

            let mut filler_entity: Option<Entity> = None;

            commands.entity(entity).with_children(|parent| {
                let filler = parent
                    .spawn((
                        SmudShape {
                            color: ui_box.fill_color,
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

            // Spawn texts as children of filler
            if let Some(filler_entity) = filler_entity {
                spawn_texts_for_filler(commands, filler_entity, ui_box, color_materials);
            }
        }
        UIShapeStructure::Classic => {
            // Classic structure - Border + Filler (original hardcoded behavior)
            // 经典结构 - 边框 + 填充（原始硬编码行为）
            info!("Spawning classic (Border + Filler) SmudShape for UI box");

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
                                    color: ui_box.fill_color,
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

            // Spawn texts as children of filler
            if let Some(filler_entity) = filler_entity {
                spawn_texts_for_filler(commands, filler_entity, ui_box, color_materials);
            } else {
                warn!("Failed to spawn UI box filler for entity {:?}", entity);
            }
        }
    }
}

/// Spawn text entities as children of the filler entity.
///
/// 将文本实体作为填充实体的子节点生成。
fn spawn_texts_for_filler(
    commands: &mut Commands,
    filler_entity: Entity,
    ui_box: &UIBox,
    color_materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    if ui_box.texts.is_empty() {
        return;
    }

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

                let mut cmd = filler_parent.spawn((
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
                    InheritedVisibility::default(),
                    ViewVisibility::default(),
                    NeedsGlyphRefresh,
                ));

                if let Some(template) = &text_config.template {
                    cmd.insert(UITextTemplate(template.clone()));
                }
            }
        });
}

/// Update UI box geometry each time layout components change.
///
/// 当布局组件变化时更新 UI 框的几何数据。
pub(crate) fn update_ui_box_system(
    mut shaders: ResMut<Assets<Shader>>,
    mut commands: Commands,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
    ui_box_query: UIBoxQuery,
    mut smud_shape_query: Query<&mut SmudShape>,
    children_query: Query<&Children>,
) {
    for (entity, ui_box, transform, children_opt) in ui_box_query.iter() {
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

        // Determine expected SmudShape count based on structure
        // 根据结构确定预期的 SmudShape 数量
        let expected_shapes = match ui_box.structure {
            UIShapeStructure::Single => 1,
            UIShapeStructure::Classic => 2,
        };

        match children_opt {
            Some(children) => {
                let mut queue: VecDeque<Entity> = VecDeque::from(children.to_vec());
                let mut smud_shape_entities: Vec<Entity> = Vec::new();

                while let Some(child) = queue.pop_front() {
                    if smud_shape_query.get(child).is_ok() {
                        smud_shape_entities.push(child);
                        if smud_shape_entities.len() >= expected_shapes {
                            break;
                        }
                    }

                    if let Ok(grandchildren) = children_query.get(child) {
                        queue.extend(grandchildren.to_vec());
                    }
                }

                if smud_shape_entities.len() >= expected_shapes {
                    trace!("Updating existing SmudShape children for UI box");

                    match ui_box.structure {
                        UIShapeStructure::Single => {
                            // Update single shape
                            if let Ok(mut shape) = smud_shape_query.get_mut(smud_shape_entities[0])
                            {
                                shape.sdf = inner_sdf.clone();
                                shape.frame = Frame::Quad(box_width.max(box_height) + 10.0);
                            }
                        }
                        UIShapeStructure::Classic => {
                            // Update outer (border) and inner (filler) shapes
                            if let Ok(mut outer_shape) =
                                smud_shape_query.get_mut(smud_shape_entities[0])
                            {
                                outer_shape.sdf = outer_sdf.clone();
                                outer_shape.frame =
                                    Frame::Quad((box_width + border_width * 2.0) + 10.0);
                            }

                            if let Ok(mut inner_shape) =
                                smud_shape_query.get_mut(smud_shape_entities[1])
                            {
                                inner_shape.sdf = inner_sdf.clone();
                                inner_shape.frame = Frame::Quad(box_width.max(box_height) + 10.0);
                            }
                        }
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

/// Toggle UI box visibility according to the active [`UILayer`] (supports both Overworld Backpack and Battle states).
///
/// 根据当前激活的 [`UILayer`] 切换 UI 框可见性（支持 Overworld 背包和 Battle 场景）。
pub(crate) fn update_ui_box_visibility_system(
    app_state: Res<State<crate::app_state::AppState>>,
    overworld_state: Option<Res<State<OverworldState>>>,
    ui_query: Query<&RonUI>,
    parent_query: Query<&ChildOf>,
    mut box_query: Query<(Entity, &UIBoxVisibility, &mut Visibility), With<UIBox>>,
) {
    // Check if we should process UI visibility (Battle or Overworld Backpack)
    // 检查是否应该处理 UI 可见性（Battle 或 Overworld 背包）
    let should_process_ui = match app_state.get() {
        crate::app_state::AppState::Battle => true,
        crate::app_state::AppState::Overworld => overworld_state
            .map(|s| s.get() == &OverworldState::Backpack)
            .unwrap_or(false),
        _ => false,
    };

    for (entity, layer_visibility, mut visibility) in box_query.iter_mut() {
        if !should_process_ui {
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

/// Toggle UI container visibility according to the active [`UILayer`] (supports both Overworld Backpack and Battle states).
/// This system handles pure container nodes that don't have a UIBox but need visibility control.
///
/// 根据当前激活的 [`UILayer`] 切换 UI 容器可见性（支持 Overworld 背包和 Battle 场景）。
/// 此系统处理没有 UIBox 但需要可见性控制的纯容器节点。
pub(crate) fn update_ui_container_visibility_system(
    app_state: Res<State<crate::app_state::AppState>>,
    overworld_state: Option<Res<State<OverworldState>>>,
    ui_query: Query<&RonUI>,
    parent_query: Query<&ChildOf>,
    mut container_query: Query<
        (
            Entity,
            &super::components::UIContainerVisibility,
            &mut Visibility,
        ),
        With<super::components::UIContainer>,
    >,
) {
    // Check if we should process UI visibility (Battle or Overworld Backpack)
    // 检查是否应该处理 UI 可见性（Battle 或 Overworld 背包）
    let should_process_ui = match app_state.get() {
        crate::app_state::AppState::Battle => true,
        crate::app_state::AppState::Overworld => overworld_state
            .map(|s| s.get() == &OverworldState::Backpack)
            .unwrap_or(false),
        _ => false,
    };

    for (entity, container_visibility, mut visibility) in container_query.iter_mut() {
        if !should_process_ui {
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

        let should_show = container_visibility.is_visible_for(overworld_ui.layer());
        if should_show {
            if *visibility != Visibility::Inherited {
                *visibility = Visibility::Inherited;
            }
        } else if *visibility != Visibility::Hidden {
            *visibility = Visibility::Hidden;
        }
    }
}
