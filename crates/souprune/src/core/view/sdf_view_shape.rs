//! # sdf_view_shape.rs
//!
//! # sdf_view_shape.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! This module handles the rendering of UI shapes using bevy_alight_motion's SdfMaterial.
//!
//! 本模块使用 bevy_alight_motion 的 SdfMaterial 处理 UI 形状的 SDF 渲染。
//!
//! ## Source File Overview
//!
//! ## 源文件概述
//!
//! It manages shape geometry, text content, and visibility based on the current UI layer.
//! Structures are loaded from external RON files for maximum flexibility.
//!
//! 管理形状几何、文本内容和基于当前 UI 层级的可见性。
//! 结构从外部 RON 文件加载以获得最大灵活性。

use super::components::{RonUI, UIBox, UIBoxFiller, UIBoxVisibility, UITextTemplate};
use super::layout::serde_types::color_tuple_to_static;
use super::layout::{SdfColorSource, SdfLayerDef, SdfShapeKind, SdfStructureAsset};
use super::sdf_shape::ViewSdfShape;
use super::text::NeedsGlyphRefresh;
use crate::app_state::overworld::OverworldState;
use bevy::ecs::relationship::Relationship;
use bevy::prelude::*;
use bevy::sprite_render::AlphaMode2d;
use bevy_alight_motion::sdf_material::SdfMaterial;
use bevy_rich_text3d::{SegmentStyle, Text3d, Text3dSegment, Text3dStyling, TextAtlas};
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

/// Create SDF shape child entities for each UI box.
/// By default, generates a single SDF shape. If structure_file is specified, loads complex structure from file.
///
/// 为 UI 框创建 SDF 形状子实体。
/// 默认生成单个 SDF 形状。如果指定了 structure_file，则从文件加载复杂结构。
fn spawn_ui_box_children(
    commands: &mut Commands,
    entity: Entity,
    ui_box: &UIBox,
    meshes: &mut ResMut<Assets<Mesh>>,
    sdf_materials: &mut ResMut<Assets<SdfMaterial>>,
    color_materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    let box_width = ui_box.width();
    let box_height = ui_box.height();

    // Check if we should load a complex structure from file
    // 检查是否应该从文件加载复杂结构
    if let Some(structure_file) = &ui_box.structure_file {
        // Load structure from file
        // 从文件加载结构
        info!("Loading SDF structure from file: {}", structure_file);
        spawn_structure_from_file(
            commands,
            entity,
            ui_box,
            structure_file,
            meshes,
            sdf_materials,
            color_materials,
        );
    } else {
        // Default: single layer SDF shape
        // 默认：单层 SDF 形状
        info!(
            "[SdfShape] Spawning single-layer SDF shape, dimensions: {}x{}",
            box_width, box_height
        );

        let shape = ViewSdfShape::new(box_width, box_height, ui_box.fill_color);
        let mesh = meshes.add(shape.create_mesh());
        let material = sdf_materials.add(shape.to_material());

        let mut filler_entity: Option<Entity> = None;

        commands.entity(entity).with_children(|parent| {
            let filler = parent
                .spawn((
                    shape,
                    Mesh2d(mesh),
                    MeshMaterial2d(material),
                    Transform::from_translation(Vec3::new(0.0, 0.0, 0.1)),
                    GlobalTransform::default(),
                    Visibility::default(),
                    InheritedVisibility::default(),
                    ViewVisibility::default(),
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
}

/// Load and spawn SDF structure from external RON file.
///
/// 从外部 RON 文件加载并生成 SDF 结构。
#[allow(clippy::too_many_arguments)]
fn spawn_structure_from_file(
    commands: &mut Commands,
    entity: Entity,
    ui_box: &UIBox,
    structure_file: &str,
    meshes: &mut ResMut<Assets<Mesh>>,
    sdf_materials: &mut ResMut<Assets<SdfMaterial>>,
    color_materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    // Load structure definition from RON file
    // 从 RON 文件加载结构定义
    let structure = match load_sdf_structure(structure_file) {
        Some(s) => s,
        None => {
            warn!(
                "Failed to load structure from '{}', falling back to single layer",
                structure_file
            );
            spawn_single_layer_fallback(
                commands,
                entity,
                ui_box,
                meshes,
                sdf_materials,
                color_materials,
            );
            return;
        }
    };

    info!(
        "Spawning SDF structure from '{}' with {} layers",
        structure_file, structure.layer_count
    );

    // Spawn the structure recursively based on the RON definition
    // 基于 RON 定义递归生成结构
    let filler_entity = spawn_layer_recursive(
        commands,
        entity,
        &structure.root,
        ui_box,
        meshes,
        sdf_materials,
    );

    // Spawn texts as children of filler (if found)
    // 将文本作为 filler 的子节点生成（如果找到）
    if let Some(filler_entity) = filler_entity {
        spawn_texts_for_filler(commands, filler_entity, ui_box, color_materials);
    } else {
        warn!(
            "No filler layer found in structure '{}' for entity {:?}",
            structure_file, entity
        );
    }
}

/// Load SdfStructureAsset from a RON file synchronously.
///
/// 从 RON 文件同步加载 SdfStructureAsset。
fn load_sdf_structure(structure_file: &str) -> Option<SdfStructureAsset> {
    let config = crate::config::load_config();
    let full_path = format!("projects/{}/{}", config.project.mod_name, structure_file);

    let content = std::fs::read_to_string(&full_path)
        .map_err(|e| {
            warn!("Failed to read structure file '{}': {}", full_path, e);
            e
        })
        .ok()?;

    ron::de::from_str::<SdfStructureAsset>(&content)
        .map_err(|e| {
            warn!("Failed to parse structure file '{}': {}", full_path, e);
            e
        })
        .ok()
}

/// Recursively spawn SDF layers based on the structure definition.
/// Returns the entity ID of the filler layer (if any).
///
/// 基于结构定义递归生成 SDF 层。
/// 返回 filler 层的实体 ID（如果有）。
fn spawn_layer_recursive(
    commands: &mut Commands,
    parent: Entity,
    layer_def: &SdfLayerDef,
    ui_box: &UIBox,
    meshes: &mut ResMut<Assets<Mesh>>,
    sdf_materials: &mut ResMut<Assets<SdfMaterial>>,
) -> Option<Entity> {
    let box_width = ui_box.width();
    let box_height = ui_box.height();
    let border_width = ui_box.border_width();

    // Determine shape dimensions based on layer definition
    // 根据层定义确定形状尺寸
    let (shape_width, shape_height) = match layer_def.sdf_type {
        SdfShapeKind::Outer => (
            box_width + border_width * 2.0,
            box_height + border_width * 2.0,
        ),
        SdfShapeKind::Inner => (box_width, box_height),
    };

    // Determine color based on layer definition
    // 根据层定义确定颜色
    let color = match &layer_def.color_source {
        SdfColorSource::FillColor => ui_box.fill_color,
        SdfColorSource::White => Color::WHITE,
        SdfColorSource::Custom(c) => {
            let (r, g, b, a) = color_tuple_to_static(c);
            Color::srgba(r, g, b, a)
        }
    };

    let shape = ViewSdfShape::new(shape_width, shape_height, color);
    let mesh = meshes.add(shape.create_mesh());
    let material = sdf_materials.add(shape.to_material());

    let mut spawned_entity: Option<Entity> = None;
    let mut filler_entity: Option<Entity> = None;

    commands.entity(parent).with_children(|parent_builder| {
        let mut entity_cmd = parent_builder.spawn((
            shape,
            Mesh2d(mesh),
            MeshMaterial2d(material),
            Transform::from_translation(Vec3::new(0.0, 0.0, layer_def.z_offset)),
            GlobalTransform::default(),
            Visibility::default(),
            InheritedVisibility::default(),
            ViewVisibility::default(),
            Name::new(layer_def.name.clone()),
        ));

        // Add UIBoxFiller marker if this is the filler layer
        // 如果这是 filler 层，添加 UIBoxFiller 标记
        if layer_def.is_filler {
            entity_cmd.insert(UIBoxFiller);
        }

        spawned_entity = Some(entity_cmd.id());

        // Track filler entity
        if layer_def.is_filler {
            filler_entity = spawned_entity;
        }
    });

    // Recursively spawn children
    // 递归生成子节点
    if let Some(spawned) = spawned_entity {
        for child_def in &layer_def.children {
            if let Some(child_filler) =
                spawn_layer_recursive(commands, spawned, child_def, ui_box, meshes, sdf_materials)
            {
                // Propagate filler entity from children
                // 从子节点传播 filler 实体
                if filler_entity.is_none() {
                    filler_entity = Some(child_filler);
                }
            }
        }
    }

    filler_entity
}

/// Spawn a single layer fallback structure when structure file loading fails.
///
/// 当结构文件加载失败时生成单层回退结构。
fn spawn_single_layer_fallback(
    commands: &mut Commands,
    entity: Entity,
    ui_box: &UIBox,
    meshes: &mut ResMut<Assets<Mesh>>,
    sdf_materials: &mut ResMut<Assets<SdfMaterial>>,
    color_materials: &mut ResMut<Assets<ColorMaterial>>,
) {
    let box_width = ui_box.width();
    let box_height = ui_box.height();

    let shape = ViewSdfShape::new(box_width, box_height, ui_box.fill_color);
    let mesh = meshes.add(shape.create_mesh());
    let material = sdf_materials.add(shape.to_material());

    let mut filler_entity: Option<Entity> = None;
    commands.entity(entity).with_children(|parent| {
        let filler = parent
            .spawn((
                shape,
                Mesh2d(mesh),
                MeshMaterial2d(material),
                Transform::from_translation(Vec3::new(0.0, 0.0, 0.1)),
                GlobalTransform::default(),
                Visibility::default(),
                InheritedVisibility::default(),
                ViewVisibility::default(),
                Name::new("UIBoxFiller"),
                UIBoxFiller,
            ))
            .id();
        filler_entity = Some(filler);
    });

    if let Some(filler_entity) = filler_entity {
        spawn_texts_for_filler(commands, filler_entity, ui_box, color_materials);
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

/// Update SDF-based UI geometry each time layout components change.
///
/// 当布局组件变化时更新基于 SDF 的 UI 几何数据。
pub(crate) fn update_sdf_view_shape_system(
    mut meshes: ResMut<Assets<Mesh>>,
    mut sdf_materials: ResMut<Assets<SdfMaterial>>,
    mut commands: Commands,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
    ui_box_query: UIBoxQuery,
    mut sdf_shape_query: Query<(&mut ViewSdfShape, &MeshMaterial2d<SdfMaterial>)>,
    children_query: Query<&Children>,
) {
    for (entity, ui_box, transform, children_opt) in ui_box_query.iter() {
        let box_width = ui_box.width();
        let box_height = ui_box.height();
        let border_width = ui_box.border_width();

        // Determine expected SDF shape count based on structure_file
        // 根据 structure_file 确定预期的 SDF 形状数量
        let expected_shapes = if let Some(structure_file) = &ui_box.structure_file {
            // Load layer_count from the structure file
            // 从结构文件加载 layer_count
            load_sdf_structure(structure_file)
                .map(|s| s.layer_count)
                .unwrap_or(1) // Fallback to single layer if loading fails
        } else {
            1 // Single layer (default)
        };

        match children_opt {
            Some(children) => {
                let mut queue: VecDeque<Entity> = VecDeque::from(children.to_vec());
                let mut sdf_shape_entities: Vec<Entity> = Vec::new();

                while let Some(child) = queue.pop_front() {
                    if sdf_shape_query.get(child).is_ok() {
                        sdf_shape_entities.push(child);
                        if sdf_shape_entities.len() >= expected_shapes {
                            break;
                        }
                    }

                    if let Ok(grandchildren) = children_query.get(child) {
                        queue.extend(grandchildren.to_vec());
                    }
                }

                if sdf_shape_entities.len() >= expected_shapes {
                    trace!("Updating existing SDF shape children for UI box");

                    if expected_shapes == 1 {
                        // Update single shape
                        if let Ok((mut shape, mat_handle)) =
                            sdf_shape_query.get_mut(sdf_shape_entities[0])
                        {
                            shape.half_width = box_width / 2.0;
                            shape.half_height = box_height / 2.0;
                            // Update material
                            if let Some(material) = sdf_materials.get_mut(&mat_handle.0) {
                                *material = shape.to_material();
                            }
                        }
                    } else {
                        // Update outer (border) and inner (filler) shapes
                        if let Ok((mut outer_shape, mat_handle)) =
                            sdf_shape_query.get_mut(sdf_shape_entities[0])
                        {
                            outer_shape.half_width = (box_width + border_width * 2.0) / 2.0;
                            outer_shape.half_height = (box_height + border_width * 2.0) / 2.0;
                            if let Some(material) = sdf_materials.get_mut(&mat_handle.0) {
                                *material = outer_shape.to_material();
                            }
                        }

                        if let Ok((mut inner_shape, mat_handle)) =
                            sdf_shape_query.get_mut(sdf_shape_entities[1])
                        {
                            inner_shape.half_width = box_width / 2.0;
                            inner_shape.half_height = box_height / 2.0;
                            if let Some(material) = sdf_materials.get_mut(&mat_handle.0) {
                                *material = inner_shape.to_material();
                            }
                        }
                    }
                } else {
                    trace!(
                        "Adding SDF shape children to existing UI box at position: {:?}",
                        transform.translation
                    );

                    spawn_ui_box_children(
                        &mut commands,
                        entity,
                        ui_box,
                        &mut meshes,
                        &mut sdf_materials,
                        &mut color_materials,
                    );
                }
            }
            None => {
                trace!(
                    "Creating new SDF shape children for UI box at position: {:?}",
                    transform.translation
                );

                spawn_ui_box_children(
                    &mut commands,
                    entity,
                    ui_box,
                    &mut meshes,
                    &mut sdf_materials,
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
    let is_battle = matches!(app_state.get(), crate::app_state::AppState::Battle);
    let is_backpack = matches!(app_state.get(), crate::app_state::AppState::Overworld)
        && overworld_state
            .map(|s| s.get() == &OverworldState::Backpack)
            .unwrap_or(false);

    let should_process_ui = is_battle || is_backpack;

    for (entity, layer_visibility, mut visibility) in box_query.iter_mut() {
        if !should_process_ui {
            if *visibility != Visibility::Hidden {
                *visibility = Visibility::Hidden;
            }
            continue;
        }

        // For Battle state, use visibility rule directly without requiring RonUI parent
        // 对于 Battle 状态，直接使用可见性规则，不需要 RonUI 父节点
        if is_battle {
            // In Battle mode, check if the visibility rule allows showing
            // (typically Always or specific battle layers)
            // 在 Battle 模式下，检查可见性规则是否允许显示
            // （通常是 Always 或特定的战斗层）
            let should_show = matches!(
                layer_visibility.rule(),
                super::components::UILayerVisibilityRule::Always
            ) || layer_visibility
                .is_visible_for(&super::components::UILayer::new("Battle"));

            if should_show {
                if *visibility != Visibility::Inherited {
                    *visibility = Visibility::Inherited;
                }
            } else if *visibility != Visibility::Hidden {
                *visibility = Visibility::Hidden;
            }
            continue;
        }

        // For Backpack state, use the original layer-based visibility logic
        // 对于 Backpack 状态，使用原始的基于层的可见性逻辑
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
    let is_battle = matches!(app_state.get(), crate::app_state::AppState::Battle);
    let is_backpack = matches!(app_state.get(), crate::app_state::AppState::Overworld)
        && overworld_state
            .map(|s| s.get() == &OverworldState::Backpack)
            .unwrap_or(false);

    let should_process_ui = is_battle || is_backpack;

    for (entity, container_visibility, mut visibility) in container_query.iter_mut() {
        if !should_process_ui {
            if *visibility != Visibility::Hidden {
                *visibility = Visibility::Hidden;
            }
            continue;
        }

        // For Battle state, use visibility rule directly without requiring RonUI parent
        // 对于 Battle 状态，直接使用可见性规则，不需要 RonUI 父节点
        if is_battle {
            let should_show = matches!(
                container_visibility.rule(),
                super::components::UILayerVisibilityRule::Always
            ) || container_visibility
                .is_visible_for(&super::components::UILayer::new("Battle"));

            if should_show {
                if *visibility != Visibility::Inherited {
                    *visibility = Visibility::Inherited;
                }
            } else if *visibility != Visibility::Hidden {
                *visibility = Visibility::Hidden;
            }
            continue;
        }

        // For Backpack state, use the original layer-based visibility logic
        // 对于 Backpack 状态，使用原始的基于层的可见性逻辑
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
