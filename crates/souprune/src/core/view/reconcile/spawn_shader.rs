use super::spawn_helpers::{SpawnContext, ViewElementSpec};
use crate::core::view::components::*;
use crate::core::view::layout::SpriteDef;
use crate::core::view::layout::serde_types::vec2_tuple_to_static;
use crate::core::view::ron_view::parsing::{RepeatContext, evaluate_float_expr_with_repeat};
use crate::core::view::ron_view::resources::RonDrivenView;
use bevy::prelude::*;

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
    use crate::core::view::ron_view::spawn_helpers::load_procedural_image_handle;

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
        if let Some(rot) = &t_def.rotation {
            transform.rotation = Quat::from_rotation_z(
                evaluate_float_expr_with_repeat(rot, &ctx.player_data, None, repeat_ctx)
                    .to_radians(),
            );
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
    // The shader path should be relative to the project root (e.g., "assets/shaders/hp_bar.wgsl")
    // because MultiSourceAssetReader already has projects/{mod_name}/ as a root.
    // 着色器路径应该相对于项目根目录（如 "assets/shaders/hp_bar.wgsl"），
    // 因为 MultiSourceAssetReader 已经将 projects/{mod_name}/ 设为根目录。
    let shader_path = if material_def.shader.starts_with("mod://") {
        // mod:// paths are expanded relative to the project root
        material_def.shader.replacen("mod://", "", 1)
    } else {
        // Direct paths like "assets/shaders/..." are used as-is
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
            view_box_def: None,
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
