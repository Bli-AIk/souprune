//! # spawn_helpers.rs
//!
//! # 生成辅助函数模块
//!
//! Reusable helper functions for spawning view elements.
//! These are extracted from spawn.rs for use by the reconciliation system.
//!
//! 用于生成视图元素的可复用辅助函数。
//! 这些函数从 spawn.rs 中提取，供协调系统使用。

use crate::core::view::components::*;
use crate::core::view::layout::serde_types::{color_tuple_to_static, vec2_tuple_to_static};
use crate::core::view::layout::*;
use crate::core::view::ron_view::parsing::{
    PlayerDataView, RepeatContext, evaluate_float_expr, evaluate_float_expr_with_repeat,
    evaluate_visible_when, resolve_text_content,
};
use crate::core::view::ron_view::resources::RonDrivenView;
use crate::core::view::sdf_view_shape::parse_text_preserving_whitespace;
use bevy::prelude::*;
use bevy_rich_text3d::Text3dStyling;

/// Context containing all resources needed for spawning view elements.
/// This reduces the number of parameters needed by helper functions.
///
/// 包含生成视图元素所需所有资源的上下文。
/// 这减少了辅助函数所需的参数数量。
pub struct SpawnContext<'a> {
    pub asset_server: &'a AssetServer,
    pub mortar_strings: &'a crate::extra::mortar::MortarStringTable,
    pub player_data: PlayerDataView<'a>,
    pub item_registry: &'a crate::core::item::ItemRegistry,
    pub camera_transform: &'a Transform,
    pub namespace: &'a str,
}

impl<'a> SpawnContext<'a> {
    pub fn new(
        asset_server: &'a AssetServer,
        mortar_strings: &'a crate::extra::mortar::MortarStringTable,
        player_data: PlayerDataView<'a>,
        item_registry: &'a crate::core::item::ItemRegistry,
        camera_transform: &'a Transform,
        namespace: &'a str,
    ) -> Self {
        Self {
            asset_server,
            mortar_strings,
            player_data,
            item_registry,
            camera_transform,
            namespace,
        }
    }
}

/// Configuration for spawning a view element.
/// Contains all the resolved values needed to create the entity.
///
/// 生成视图元素的配置。
/// 包含创建实体所需的所有已解析值。
pub struct ViewElementSpec {
    pub full_name: String,
    pub local_name: String,
    pub namespace: String,
    pub tags: Vec<String>,
    pub transform: Transform,
    pub visibility: Visibility,
    pub visible_when_expr: Option<String>,
}

/// Spawn a standalone sprite entity.
/// Returns the spawned entity ID.
///
/// 生成独立精灵实体。
/// 返回生成的实体 ID。
pub fn spawn_sprite_entity(
    commands: &mut Commands,
    parent: Option<Entity>,
    ctx: &SpawnContext,
    spec: &ViewElementSpec,
    sprite_def: &SpriteDef,
    repeat_ctx: Option<&RepeatContext>,
) -> Entity {
    use crate::config::load_config;
    use crate::core::visual::{get_asset_path, resolve_visual_path};

    let config = load_config();
    let visual_path = sprite_def.visual.path().to_owned();

    // Build transform from sprite_def
    let mut transform = spec.transform;
    if let Some(t_def) = &sprite_def.transform {
        if let Some(trans) = &t_def.translation {
            transform.translation = Vec3::new(
                evaluate_float_expr_with_repeat(&trans.0, &ctx.player_data, None, repeat_ctx),
                evaluate_float_expr_with_repeat(&trans.1, &ctx.player_data, None, repeat_ctx),
                evaluate_float_expr_with_repeat(&trans.2, &ctx.player_data, None, repeat_ctx),
            );
        }
        if let Some(scale) = &t_def.scale {
            transform.scale = Vec3::new(
                evaluate_float_expr_with_repeat(&scale.0, &ctx.player_data, None, repeat_ctx),
                evaluate_float_expr_with_repeat(&scale.1, &ctx.player_data, None, repeat_ctx),
                evaluate_float_expr_with_repeat(&scale.2, &ctx.player_data, None, repeat_ctx),
            );
        }
        if let Some(rot) = t_def.rotation {
            transform.rotation = Quat::from_rotation_z(rot.to_radians());
        }
    }

    // Handle pivot adjustment
    if let Some(pivot) = &sprite_def.pivot {
        let (pivot_x, pivot_y) = vec2_tuple_to_static(pivot);
        let shift_x = (0.5 - pivot_x) * transform.scale.x;
        let shift_y = (0.5 - pivot_y) * transform.scale.y;
        let shift = transform.rotation * Vec3::new(shift_x, shift_y, 0.0);
        transform.translation += shift;
    }

    // Load texture
    let texture_handle: Handle<Image> = if visual_path.starts_with("procedural://") {
        Handle::default()
    } else if let Some(resolved) = resolve_visual_path(&visual_path, &config.project.mod_name) {
        let asset_path = get_asset_path(&resolved, &config.project.mod_name);
        ctx.asset_server.load(&asset_path)
    } else {
        ctx.asset_server.load(&visual_path)
    };

    // Build sprite component
    let (r, g, b, a) = if let Some(color) = &sprite_def.color {
        color_tuple_to_static(color)
    } else {
        (1.0, 1.0, 1.0, 1.0)
    };

    let flip_x = sprite_def.flip_x;
    let flip_y = sprite_def.flip_y;

    // Create view element component
    let view_element = ViewElement::new(
        spec.namespace.clone(),
        spec.local_name.clone(),
        spec.tags.clone(),
    );

    // Spawn the entity
    let mut entity_commands = commands.spawn((
        Sprite {
            image: texture_handle,
            color: Color::srgba(r, g, b, a),
            flip_x,
            flip_y,
            ..default()
        },
        transform,
        GlobalTransform::default(),
        spec.visibility,
        InheritedVisibility::default(),
        ViewVisibility::default(),
        Name::new(spec.local_name.clone()),
        view_element,
        RonDrivenView,
    ));

    // Add VisibleWhen if present
    if let Some(ref expr) = spec.visible_when_expr {
        entity_commands.insert(VisibleWhen {
            expression: expr.clone(),
        });
    }

    let entity_id = entity_commands.id();

    // Check if transform has dynamic expressions, add DynamicViewElement if so.
    // This ensures fact-dependent transforms are re-evaluated when facts change.
    let mut has_dynamic = false;
    let mut has_time_dependency = false;
    if let Some(t) = &sprite_def.transform {
        if let Some(trans) = &t.translation {
            if trans.0.is_dynamic() || trans.1.is_dynamic() || trans.2.is_dynamic() {
                has_dynamic = true;
            }
            if crate::core::view::ron_view::parsing::vec3_tuple_depends_on_time(trans) {
                has_time_dependency = true;
            }
        }
        if let Some(s) = &t.scale {
            if s.0.is_dynamic() || s.1.is_dynamic() || s.2.is_dynamic() {
                has_dynamic = true;
            }
            if crate::core::view::ron_view::parsing::vec3_tuple_depends_on_time(s) {
                has_time_dependency = true;
            }
        }
    }

    if has_dynamic {
        let processed_sprite_def = if let Some(rctx) = repeat_ctx {
            crate::core::view::ron_view::parsing::preprocess_sprite_def_for_repeat(sprite_def, rctx)
        } else {
            sprite_def.clone()
        };

        commands.entity(entity_id).insert(DynamicViewElement {
            sprite_def: Some(processed_sprite_def),
            text_def: None,
        });

        if has_time_dependency {
            commands.entity(entity_id).insert(TimeDependentTransform);
        }
    }

    // Set parent if provided
    if let Some(parent_entity) = parent {
        commands.entity(entity_id).insert(ChildOf(parent_entity));
    }

    info!(
        "[spawn_helpers] Spawned sprite '{}' (Entity {:?})",
        spec.full_name, entity_id
    );

    entity_id
}

/// Spawn a text entity with Text3d component.
/// Returns the spawned entity ID.
///
/// 生成带有 Text3d 组件的文本实体。
/// 返回生成的实体 ID。
pub fn spawn_text_entity(
    commands: &mut Commands,
    parent: Option<Entity>,
    ctx: &SpawnContext,
    text_def: &TextDef,
    repeat_ctx: Option<&RepeatContext>,
) -> Entity {
    let raw_content = text_def.content.as_deref().unwrap_or("");
    let content = resolve_text_content(
        raw_content,
        ctx.mortar_strings,
        &ctx.player_data,
        ctx.item_registry,
    );

    let text3d = parse_text_preserving_whitespace(&content);

    // Build transform
    let mut transform = Transform::default();
    if let Some(trans) = &text_def.transform.translation {
        transform.translation = Vec3::new(
            evaluate_float_expr_with_repeat(&trans.0, &ctx.player_data, None, repeat_ctx),
            evaluate_float_expr_with_repeat(&trans.1, &ctx.player_data, None, repeat_ctx),
            evaluate_float_expr_with_repeat(&trans.2, &ctx.player_data, None, repeat_ctx),
        );
    }
    if let Some(scale) = &text_def.transform.scale {
        transform.scale = Vec3::new(
            evaluate_float_expr_with_repeat(&scale.0, &ctx.player_data, None, repeat_ctx),
            evaluate_float_expr_with_repeat(&scale.1, &ctx.player_data, None, repeat_ctx),
            evaluate_float_expr_with_repeat(&scale.2, &ctx.player_data, None, repeat_ctx),
        );
    }
    if let Some(rot) = text_def.transform.rotation {
        transform.rotation = Quat::from_rotation_z(rot.to_radians());
    }

    // Calculate world position
    let text_world_transform =
        Transform::from_translation(ctx.camera_transform.translation + transform.translation)
            .with_rotation(transform.rotation)
            .with_scale(transform.scale);

    // Parse color
    let (r, g, b, a) = color_tuple_to_static(&text_def.color);
    let color = Srgba::new(r, g, b, a);

    // Parse world_scale
    let (ws_x, ws_y) = vec2_tuple_to_static(&text_def.world_scale);
    let world_scale = Vec2::new(ws_x, ws_y);

    // Spawn entity
    let view_font: crate::core::view::components::ViewFont = text_def.font.clone().into();
    let mut entity_commands = commands.spawn((
        Name::new(text_def.id.clone()),
        text3d,
        Text3dStyling {
            font: view_font.font_name().into(),
            size: view_font.default_size(),
            world_scale: Some(world_scale),
            color,
            line_height: text_def.line_height.unwrap_or(1.0),
            ..default()
        },
        Mesh2d::default(),
        crate::core::view::text::NeedsTextMaterial,
        text_world_transform,
        Visibility::Hidden, // Will be shown when material is ready
        InheritedVisibility::default(),
        ViewVisibility::default(),
        crate::core::view::text::NeedsGlyphRefresh,
        RonDrivenView,
    ));

    // Add template for dynamic updates
    entity_commands.insert(ViewTextTemplate(raw_content.to_string()));

    // Add VisibleWhen if present
    if let Some(ref visible_when_expr) = text_def.visible_when {
        let expr = visible_when_expr.trim();
        if !expr.is_empty() {
            let is_visible = evaluate_visible_when(expr, &ctx.player_data);
            entity_commands.insert(VisibleWhen {
                expression: expr.to_string(),
            });
            if is_visible {
                entity_commands.insert(Visibility::Inherited);
            }
        }
    }

    let entity_id = entity_commands.id();

    // Set parent if provided
    if let Some(parent_entity) = parent {
        commands.entity(entity_id).insert(ChildOf(parent_entity));
    }

    info!(
        "[spawn_helpers] Spawned text '{}' (Entity {:?})",
        text_def.id, entity_id
    );

    entity_id
}

/// Spawn a shader material entity with DynamicMaterial2d.
/// Returns the spawned entity ID.
///
/// 生成带有 DynamicMaterial2d 的着色器材质实体。
/// 返回生成的实体 ID。
pub fn spawn_shader_material_entity(
    commands: &mut Commands,
    parent: Option<Entity>,
    ctx: &SpawnContext,
    spec: &ViewElementSpec,
    sprite_def: &SpriteDef,
    material_def: &crate::core::view::layout::view_schema::MaterialDef,
    repeat_ctx: Option<&RepeatContext>,
) -> Entity {
    use crate::core::view::components::ShaderMaterial;
    use crate::core::view::ron_view::spawn::load_procedural_image_handle;

    let visual_path = sprite_def.visual.path().to_owned();

    // Build transform
    let mut transform = spec.transform;
    if let Some(t_def) = &sprite_def.transform {
        if let Some(trans) = &t_def.translation {
            transform.translation = Vec3::new(
                evaluate_float_expr_with_repeat(&trans.0, &ctx.player_data, None, repeat_ctx),
                evaluate_float_expr_with_repeat(&trans.1, &ctx.player_data, None, repeat_ctx),
                evaluate_float_expr_with_repeat(&trans.2, &ctx.player_data, None, repeat_ctx),
            );
        }
        if let Some(scale) = &t_def.scale {
            transform.scale = Vec3::new(
                evaluate_float_expr_with_repeat(&scale.0, &ctx.player_data, None, repeat_ctx),
                evaluate_float_expr_with_repeat(&scale.1, &ctx.player_data, None, repeat_ctx),
                evaluate_float_expr_with_repeat(&scale.2, &ctx.player_data, None, repeat_ctx),
            );
        }
        if let Some(rot) = t_def.rotation {
            transform.rotation = Quat::from_rotation_z(rot.to_radians());
        }
    }

    // Handle pivot adjustment
    if let Some(pivot) = &sprite_def.pivot {
        let (pivot_x, pivot_y) = vec2_tuple_to_static(pivot);
        let shift_x = (0.5 - pivot_x) * transform.scale.x;
        let shift_y = (0.5 - pivot_y) * transform.scale.y;
        let shift = transform.rotation * Vec3::new(shift_x, shift_y, 0.0);
        transform.translation += shift;
    }

    // Load shader
    // The shader path should be relative to the project root (e.g., "shared/shaders/hp_bar.wgsl")
    // because MultiSourceAssetReader already has projects/{mod_name}/ as a root.
    // 着色器路径应该相对于项目根目录（如 "shared/shaders/hp_bar.wgsl"），
    // 因为 MultiSourceAssetReader 已经将 projects/{mod_name}/ 设为根目录。
    let shader_path = if material_def.shader.starts_with("mod://") {
        // mod:// paths are expanded relative to the project root
        material_def.shader.replacen("mod://", "", 1)
    } else {
        // Direct paths like "shared/shaders/..." are used as-is
        material_def.shader.clone()
    };
    let shader_handle = ctx.asset_server.load(&shader_path);

    // Load texture
    let texture_handle: Handle<Image> = if visual_path.starts_with("procedural://") {
        load_procedural_image_handle(&visual_path, ctx.asset_server)
    } else {
        ctx.asset_server.load(&visual_path)
    };

    // Create ShaderMaterial component
    let shader_material = ShaderMaterial::from_def(shader_handle.clone(), material_def);

    // Create view element component
    let view_element = ViewElement::new(
        spec.namespace.clone(),
        spec.local_name.clone(),
        spec.tags.clone(),
    );

    // Spawn entity with marker (material will be applied by setup system)
    // The actual MeshDynamicMaterial2d and Mesh2d will be added by the setup system
    // because we need to create the DynamicMaterial2d asset first
    let mut entity_commands = commands.spawn((
        transform,
        GlobalTransform::default(),
        spec.visibility,
        InheritedVisibility::default(),
        ViewVisibility::default(),
        Name::new(spec.local_name.clone()),
        view_element,
        RonDrivenView,
        shader_material,
        // Store texture handle for setup system
        ShaderMaterialPendingSetup {
            texture: texture_handle,
        },
    ));

    // Add VisibleWhen if present
    if let Some(ref expr) = spec.visible_when_expr {
        entity_commands.insert(VisibleWhen {
            expression: expr.clone(),
        });
    }

    // Check if transform has dynamic expressions and add DynamicViewElement if needed.
    // This ensures shader material elements (like HealthBar) have their transforms updated
    // when facts change, fixing position offset bugs.
    //
    // 检查 transform 是否有动态表达式，如果有则添加 DynamicViewElement。
    // 这确保着色器材质元素（如 HealthBar）在 facts 变化时更新 transform，修复位置偏移 bug。
    let mut has_dynamic = false;
    let mut has_time_dependency = false;
    if let Some(t) = &sprite_def.transform {
        if let Some(trans) = &t.translation {
            if trans.0.is_dynamic() || trans.1.is_dynamic() || trans.2.is_dynamic() {
                has_dynamic = true;
            }
            if crate::core::view::ron_view::parsing::vec3_tuple_depends_on_time(trans) {
                has_time_dependency = true;
            }
        }
        if let Some(s) = &t.scale {
            if s.0.is_dynamic() || s.1.is_dynamic() || s.2.is_dynamic() {
                has_dynamic = true;
            }
            if crate::core::view::ron_view::parsing::vec3_tuple_depends_on_time(s) {
                has_time_dependency = true;
            }
        }
    }

    if has_dynamic {
        // Preprocess sprite_def to resolve repeat variables if repeat context exists
        // 如果存在 repeat 上下文，预处理 sprite_def 以解析 repeat 变量
        let processed_sprite_def = if let Some(ctx) = repeat_ctx {
            crate::core::view::ron_view::parsing::preprocess_sprite_def_for_repeat(sprite_def, ctx)
        } else {
            sprite_def.clone()
        };

        entity_commands.insert(DynamicViewElement {
            sprite_def: Some(processed_sprite_def),
            text_def: None,
        });

        // Add TimeDependentTransform marker if expression uses @time
        // 如果表达式使用 @time 则添加 TimeDependentTransform 标记
        if has_time_dependency {
            entity_commands.insert(TimeDependentTransform);
        }
    }

    let entity_id = entity_commands.id();

    // Set parent if provided
    if let Some(parent_entity) = parent {
        commands.entity(entity_id).insert(ChildOf(parent_entity));
    }

    info!(
        "[spawn_helpers] Spawned shader material '{}' with shader '{}' (Entity {:?}, dynamic={})",
        spec.full_name, shader_path, entity_id, has_dynamic
    );

    entity_id
}

/// Marker component for shader materials pending setup.
/// Contains the texture handle to be used when creating DynamicMaterial2d.
///
/// 等待设置的着色器材质标记组件。
/// 包含创建 DynamicMaterial2d 时要使用的纹理句柄。
#[derive(Component)]
pub struct ShaderMaterialPendingSetup {
    pub texture: Handle<Image>,
}

/// Spawn a ViewBox entity with SDF shape.
/// Returns the spawned entity ID.
///
/// 生成带有 SDF 形状的 ViewBox 实体。
/// 返回生成的实体 ID。
pub fn spawn_viewbox_entity(
    commands: &mut Commands,
    parent: Option<Entity>,
    _ctx: &SpawnContext,
    spec: &ViewElementSpec,
    view_box: &ViewBoxLogicDef,
    texts: Vec<ViewTextConfig>,
) -> Entity {
    use crate::core::view::layout::serde_types::serializable_vec3_to_static;

    let offset = serializable_vec3_to_static(&view_box.offset);

    // Convert fill color
    let fill_color = view_box
        .fill_color
        .as_ref()
        .map(|c| {
            let (r, g, b, a) = color_tuple_to_static(c);
            Color::srgba(r, g, b, a)
        })
        .unwrap_or(Color::BLACK);

    // Create ViewBox
    let view_box_def = view_box;
    let view_box_component = ViewBox::new_full(
        view_box_def.width,
        view_box_def.height,
        view_box_def.border_width,
        texts,
        view_box_def.fill_shader.clone(),
        view_box_def.structure_file.clone(),
        fill_color,
    );

    // Create view element component
    let view_element = ViewElement::new(
        spec.namespace.clone(),
        spec.local_name.clone(),
        spec.tags.clone(),
    );

    let mut entity_commands = commands.spawn((
        view_box_component,
        Transform::from_translation(offset),
        GlobalTransform::default(),
        spec.visibility,
        InheritedVisibility::default(),
        ViewVisibility::default(),
        Name::new(spec.local_name.clone()),
        view_element,
        RonDrivenView,
    ));

    // Add VisibleWhen if present
    if let Some(ref expr) = spec.visible_when_expr {
        entity_commands.insert(VisibleWhen {
            expression: expr.clone(),
        });
    }

    let entity_id = entity_commands.id();

    // Set parent if provided
    if let Some(parent_entity) = parent {
        commands.entity(entity_id).insert(ChildOf(parent_entity));
    }

    info!(
        "[spawn_helpers] Spawned ViewBox '{}' (Entity {:?})",
        spec.full_name, entity_id
    );

    entity_id
}

/// Build ViewTextConfig from TextDef (extracted from spawn.rs).
///
/// 从 TextDef 构建 ViewTextConfig（从 spawn.rs 提取）。
pub fn build_text_config(text_def: &TextDef, ctx: &SpawnContext) -> ViewTextConfig {
    let raw_content = text_def.content.as_deref().unwrap_or("");
    let content = resolve_text_content(
        raw_content,
        ctx.mortar_strings,
        &ctx.player_data,
        ctx.item_registry,
    );

    let (r, g, b, a) = color_tuple_to_static(&text_def.color);
    let color = Srgba::new(r, g, b, a);

    let (ws_x, ws_y) = vec2_tuple_to_static(&text_def.world_scale);

    // Build transform
    let mut transform = Transform::default();
    if let Some(trans) = &text_def.transform.translation {
        transform.translation = Vec3::new(
            evaluate_float_expr(&trans.0, &ctx.player_data, None),
            evaluate_float_expr(&trans.1, &ctx.player_data, None),
            evaluate_float_expr(&trans.2, &ctx.player_data, None),
        );
    }
    if let Some(scale) = &text_def.transform.scale {
        transform.scale = Vec3::new(
            evaluate_float_expr(&scale.0, &ctx.player_data, None),
            evaluate_float_expr(&scale.1, &ctx.player_data, None),
            evaluate_float_expr(&scale.2, &ctx.player_data, None),
        );
    }
    if let Some(rot) = text_def.transform.rotation {
        transform.rotation = Quat::from_rotation_z(rot.to_radians());
    }

    ViewTextConfig {
        name: Name::new(text_def.id.clone()),
        content,
        template: Some(raw_content.to_string()),
        font: text_def.font.clone().into(),
        world_scale: Vec2::new(ws_x, ws_y),
        color,
        transform,
        line_height: text_def.line_height.unwrap_or(1.0),
        visible_when: text_def.visible_when.clone(),
        ..Default::default()
    }
}
