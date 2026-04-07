//! # update.rs
//!
//! # update.rs 文件
//!
//! ## Module Overview
//!
//! ## 模块概述
//!
//! Keeps already-spawned RON View entities in sync with changing facts and time. It
//! updates dynamic transforms, refreshes template-driven text, and walks the view hierarchy to
//! find the right `ViewRoot` scope for each runtime element.
//!
//! 负责让已经生成的 RON View 实体持续与事实数据和时间同步。它会更新动态变换、
//! 刷新模板驱动的文本内容，并沿层级向上找到每个运行时元素应该读取的 `ViewRoot` 作用域。

use super::super::components::ViewRoot;
use super::super::components::{DynamicViewElement, TimeDependentTransform, ViewTextTemplate};
use super::super::layout::serde_types::vec2_tuple_to_static;
use super::super::sdf_view_shape::parse_text_preserving_whitespace;
use super::parsing::{
    DataPathResolvers, PlayerDataView, evaluate_float_expr, resolve_text_content,
};
use bevy::prelude::*;
use bevy_bitmap_text::TextBlock;
use bevy_fact_rule_event::LayeredFactDatabase;

/// Update time-dependent UI elements (elements with @time in expressions).
/// Runs every frame for animation effects.
///
/// 更新时间依赖的 UI 元素（表达式中包含 @time 的元素）。
/// 每帧运行以实现动画效果。
pub fn update_time_dependent_ui_elements(
    time: Res<Time>,
    layered_db: Res<LayeredFactDatabase>,
    mut query: Query<(Entity, &DynamicViewElement, &mut Transform), With<TimeDependentTransform>>,
    parent_query: Query<&ChildOf>,
    view_root_query: Query<&ViewRoot>,
) {
    for (entity, dynamic_elem, mut transform) in query.iter_mut() {
        let local_facts = find_view_root_ancestor(entity, &parent_query, &view_root_query)
            .map(|root| &root.local_facts);

        let player_data = if let Some(local) = local_facts {
            PlayerDataView::with_local_facts(&layered_db, local)
        } else {
            PlayerDataView::new(&layered_db)
        };

        update_element_transform(
            dynamic_elem,
            &mut transform,
            &player_data,
            Some(time.elapsed_secs_f64()),
        );
    }
}

/// Update fact-dependent UI elements (elements without @time).
/// Only runs when LayeredFactDatabase or ViewRoot changes.
///
/// 更新 fact 依赖的 UI 元素（不包含 @time 的元素）。
/// 仅在 LayeredFactDatabase 或 ViewRoot 变化时运行。
pub fn update_fact_dependent_ui_elements(
    layered_db: Res<LayeredFactDatabase>,
    mut query: Query<
        (Entity, &DynamicViewElement, &mut Transform),
        Without<TimeDependentTransform>,
    >,
    parent_query: Query<&ChildOf>,
    view_root_query: Query<&ViewRoot, Changed<ViewRoot>>,
    all_view_root_query: Query<&ViewRoot>,
) {
    // Check if any ViewRoot changed (local_facts modification)
    // 检查是否有任何 ViewRoot 变化（local_facts 修改）
    let any_view_root_changed = !view_root_query.is_empty();

    // Only update when fact database changes OR any ViewRoot's local_facts changed
    // 仅在 fact 数据库变化或任何 ViewRoot 的 local_facts 变化时更新
    if !layered_db.is_changed() && !any_view_root_changed {
        return;
    }

    for (entity, dynamic_elem, mut transform) in query.iter_mut() {
        let local_facts = find_view_root_ancestor(entity, &parent_query, &all_view_root_query)
            .map(|root| &root.local_facts);

        let player_data = if let Some(local) = local_facts {
            PlayerDataView::with_local_facts(&layered_db, local)
        } else {
            PlayerDataView::new(&layered_db)
        };

        // Pass None for time since these elements don't depend on it
        // 传入 None 作为时间，因为这些元素不依赖时间
        update_element_transform(dynamic_elem, &mut transform, &player_data, None);
    }
}

/// Shared helper to update element transform from definition.
///
/// 共享的辅助函数，用于从定义更新元素变换。
fn update_element_transform(
    dynamic_elem: &DynamicViewElement,
    transform: &mut Transform,
    player_data: &PlayerDataView,
    time: Option<f64>,
) {
    // Update sprite transform if present
    // 如果存在精灵定义则更新变换
    if let Some(sprite_def) = &dynamic_elem.sprite_def
        && let Some(t_def) = &sprite_def.transform
    {
        let new_translation = if let Some(trans) = &t_def.translation {
            Vec3::new(
                evaluate_float_expr(&trans.0, player_data, time),
                evaluate_float_expr(&trans.1, player_data, time),
                evaluate_float_expr(&trans.2, player_data, time),
            )
        } else {
            Vec3::ZERO
        };

        if let Some(scale_def) = &t_def.scale {
            let new_scale = Vec3::new(
                evaluate_float_expr(&scale_def.0, player_data, time),
                evaluate_float_expr(&scale_def.1, player_data, time),
                evaluate_float_expr(&scale_def.2, player_data, time),
            );

            // Apply pivot offset if present
            // 如果存在 pivot 则应用偏移
            if let Some(pivot) = &sprite_def.pivot {
                let (pivot_x, pivot_y) = vec2_tuple_to_static(pivot);
                let shift_x = (0.5 - pivot_x) * new_scale.x;
                let shift_y = (0.5 - pivot_y) * new_scale.y;
                let shift = transform.rotation * Vec3::new(shift_x, shift_y, 0.0);
                transform.translation = new_translation + shift;
            } else {
                transform.translation = new_translation;
            }

            transform.scale = new_scale;
        } else {
            transform.translation = new_translation;
        }

        if let Some(rot) = &t_def.rotation {
            transform.rotation =
                Quat::from_rotation_z(evaluate_float_expr(rot, player_data, time).to_radians());
        }
    }

    // Update text transform if present
    // 如果存在文本定义则更新变换
    if let Some(text_def) = &dynamic_elem.text_def {
        let new_translation = if let Some(trans) = &text_def.transform.translation {
            Vec3::new(
                evaluate_float_expr(&trans.0, player_data, time),
                evaluate_float_expr(&trans.1, player_data, time),
                evaluate_float_expr(&trans.2, player_data, time),
            )
        } else {
            Vec3::ZERO
        };
        transform.translation = new_translation;

        if let Some(scale_def) = &text_def.transform.scale {
            transform.scale = Vec3::new(
                evaluate_float_expr(&scale_def.0, player_data, time),
                evaluate_float_expr(&scale_def.1, player_data, time),
                evaluate_float_expr(&scale_def.2, player_data, time),
            );
        }
    }

    // Update view_box offset if present (fact-driven position for view_box elements)
    // 如果存在 view_box 定义则更新偏移（fact 驱动的 view_box 元素位置）
    if let Some(view_box_def) = &dynamic_elem.view_box_def {
        transform.translation = Vec3::new(
            evaluate_float_expr(&view_box_def.offset.0, player_data, time),
            evaluate_float_expr(&view_box_def.offset.1, player_data, time),
            evaluate_float_expr(&view_box_def.offset.2, player_data, time),
        );
    }
}

pub fn update_dynamic_text_system(
    mut text_query: Query<(Entity, &ViewTextTemplate, &mut TextBlock, &Name)>,
    layered_db: Res<LayeredFactDatabase>,
    mortar_strings: Res<crate::extra::mortar::MortarStringTable>,
    view_root_query: Query<(Entity, &ViewRoot)>,
    changed_view_roots: Query<Entity, Changed<ViewRoot>>,
    parent_query: Query<&ChildOf>,
    data_resolvers: Option<Res<DataPathResolvers>>,
) {
    use bevy::prelude::DetectChanges;

    let global_changed = layered_db.is_changed();
    let any_view_root_changed = !changed_view_roots.is_empty();

    if !global_changed && !any_view_root_changed {
        return;
    }

    trace!(
        "[update_dynamic_text_system] Update triggered (global_changed={}, local_changed={})",
        global_changed, any_view_root_changed,
    );

    for (entity, template, mut text_block, _name) in text_query.iter_mut() {
        let view_root_result =
            find_view_root_ancestor_entity(entity, &parent_query, &view_root_query);
        let player_data = if let Some((_, view_root)) = view_root_result {
            PlayerDataView::with_local_facts(&layered_db, &view_root.local_facts)
        } else {
            PlayerDataView::new(&layered_db)
        }
        .with_resolvers(data_resolvers.as_deref(), None);

        let new_content = resolve_text_content(&template.0, &mortar_strings, &player_data);

        // Skip if content hasn't changed.
        if text_block.full_text() == new_content {
            continue;
        }

        *text_block = parse_text_preserving_whitespace(&new_content);
    }
}

/// Find the ViewRoot ancestor of an entity by traversing up the hierarchy.
/// Returns a tuple of (Entity, &ViewRoot) if found, None otherwise.
///
/// 通过向上遍历层级结构查找实体的 ViewRoot 祖先。
/// 如果找到则返回 (Entity, &ViewRoot) 元组，否则返回 None。
fn find_view_root_ancestor_entity<'a>(
    entity: Entity,
    parent_query: &Query<&ChildOf>,
    view_root_query: &'a Query<(Entity, &ViewRoot)>,
) -> Option<(Entity, &'a ViewRoot)> {
    let mut current = entity;

    // First check if the entity itself is a ViewRoot
    if let Ok((e, view_root)) = view_root_query.get(current) {
        return Some((e, view_root));
    }

    // Traverse up the parent hierarchy
    while let Ok(child_of) = parent_query.get(current) {
        let parent = child_of.parent();

        // Check if parent is a ViewRoot
        if let Ok((e, view_root)) = view_root_query.get(parent) {
            return Some((e, view_root));
        }

        current = parent;
    }

    None
}

/// Find the ViewRoot ancestor of an entity by traversing up the hierarchy.
/// Returns a reference to the ViewRoot if found, None otherwise.
///
/// 通过向上遍历层级结构查找实体的 ViewRoot 祖先。
/// 如果找到则返回 ViewRoot 的引用，否则返回 None。
fn find_view_root_ancestor<'a>(
    entity: Entity,
    parent_query: &Query<&ChildOf>,
    view_root_query: &'a Query<&ViewRoot>,
) -> Option<&'a ViewRoot> {
    let mut current = entity;

    // First check if the entity itself is a ViewRoot
    if let Ok(view_root) = view_root_query.get(current) {
        return Some(view_root);
    }

    // Traverse up the parent hierarchy
    while let Ok(child_of) = parent_query.get(current) {
        let parent = child_of.parent();

        // Check if parent is a ViewRoot
        if let Ok(view_root) = view_root_query.get(parent) {
            return Some(view_root);
        }

        current = parent;
    }

    None
}

/// Update shader material parameters for DynamicMaterial2d entities.
/// Evaluates parameter expressions and updates material uniforms.
///
/// 更新 DynamicMaterial2d 实体的着色器材质参数。
/// 评估参数表达式并更新材质 uniform。
///
/// ## Performance Optimization / 性能优化
///
/// This system uses a two-phase approach:
/// 1. **Animation phase**: Always process entities with active animations (anim_progress < 1.0)
/// 2. **Expression phase**: Only evaluate expressions when fact database changes
///
/// This avoids evaluating expressions every frame when data hasn't changed.
///
/// 该系统使用两阶段方法：
/// 1. **动画阶段**：始终处理有活跃动画的实体（anim_progress < 1.0）
/// 2. **表达式阶段**：仅在 fact 数据库变化时评估表达式
pub fn update_shader_materials_system(
    time: Res<Time>,
    layered_db: Res<LayeredFactDatabase>,
    mut materials: ResMut<Assets<crate::core::view::dynamic_material::DynamicMaterial2d>>,
    mut query: Query<(
        Entity,
        &mut super::super::components::ShaderMaterial,
        &crate::core::view::dynamic_material::MeshDynamicMaterial2d,
    )>,
    added_materials: Query<
        Entity,
        Added<crate::core::view::dynamic_material::MeshDynamicMaterial2d>,
    >,
    parent_query: Query<&ChildOf>,
    view_root_query: Query<(Entity, &ViewRoot)>,
    changed_view_roots: Query<Entity, Changed<ViewRoot>>,
) {
    use crate::core::view::layout::Value;
    use crate::core::view::layout::view_schema::MaterialParamValue;

    // Early exit: skip if no shader materials exist
    if query.is_empty() {
        return;
    }

    let delta_time = time.delta_secs();
    let elapsed_secs = time.elapsed_secs_f64();

    // Check if fact data changed (triggers expression re-evaluation)
    // Also force evaluation for newly added materials to ensure correct initial values
    // 检查 fact 数据是否变化（触发表达式重新评估）
    // 同时为新添加的材质强制评估以确保初始值正确
    let facts_changed = layered_db.is_changed() || !changed_view_roots.is_empty();
    let has_new_materials = !added_materials.is_empty();

    for (entity, mut shader_mat, material_handle) in query.iter_mut() {
        // Determine what needs updating for this entity
        let has_active_animation = shader_mat
            .animation
            .as_ref()
            .is_some_and(|a| a.anim_progress < 1.0);

        // Check if this entity has any animation config (even if not currently active)
        // This is needed to detect source value changes and restart animations
        // 检查此实体是否有动画配置（即使当前不活跃）
        // 这是为了检测源值变化并重启动画
        let has_animation_config = shader_mat.animation.is_some();

        // Check if this is a newly added material that needs initial expression evaluation
        // 检查是否是新添加的材质，需要初始表达式评估
        let is_newly_added = has_new_materials && added_materials.contains(entity);

        // Skip this entity if:
        // - No active animation AND
        // - No fact changes AND
        // - Not newly added AND
        // - No animation config (nothing could trigger animation restart)
        // 跳过条件：无活跃动画 且 无 fact 变化 且 非新添加 且 无动画配置
        if !has_active_animation && !facts_changed && !is_newly_added {
            continue;
        }

        let mut material_changed = false;

        // Phase 1: Evaluate expressions if facts changed OR material is newly added
        // 阶段 1：当 facts 变化或材质新添加时评估表达式
        let expr_updates: Vec<(String, f32)> = if facts_changed || is_newly_added {
            // Find ViewRoot ancestor to access local facts (lazy evaluation)
            let view_root_result =
                find_view_root_ancestor_entity(entity, &parent_query, &view_root_query);
            let player_data = if let Some((_, view_root)) = view_root_result {
                PlayerDataView::with_local_facts(&layered_db, &view_root.local_facts)
            } else {
                PlayerDataView::new(&layered_db)
            };

            // Evaluate parameter expressions and collect updates
            // 评估参数表达式并收集更新
            shader_mat
                .param_defs
                .iter()
                .map(|(name, param_def)| {
                    let new_value = match param_def {
                        MaterialParamValue::Static(v) => *v,
                        MaterialParamValue::Expr(expr_str) => evaluate_float_expr(
                            &Value::Expr(expr_str.clone()),
                            &player_data,
                            Some(elapsed_secs),
                        ),
                    };
                    (name.clone(), new_value)
                })
                .collect()
        } else {
            Vec::new()
        };

        // Apply updates with change detection
        // 应用更新并检测变化
        for (name, new_value) in expr_updates {
            let should_update = shader_mat
                .current_values
                .get(&name)
                .map(|old| (*old - new_value).abs() > f32::EPSILON)
                .unwrap_or(true);

            if should_update {
                shader_mat.current_values.insert(name, new_value);
                material_changed = true;
            }
        }

        // Phase 2: Process lag animation if present
        // 阶段 2：处理延迟动画（如果存在）
        //
        // Run animation update when:
        // - Animation is currently active (anim_progress < 1.0), OR
        // - Facts changed and animation config exists (to detect source value changes)
        //
        // 运行动画更新的条件：
        // - 动画当前活跃（anim_progress < 1.0），或
        // - Facts 变化且存在动画配置（检测源值变化）
        if has_active_animation || (facts_changed && has_animation_config) {
            // Extract animation info to avoid borrowing issues
            let anim_update = shader_mat.animation.as_ref().map(|anim_state| {
                let source_value = shader_mat
                    .current_values
                    .get(&anim_state.source_param)
                    .copied()
                    .unwrap_or(1.0);
                (anim_state.target_param.clone(), source_value)
            });

            if let Some((target_param, source_value)) = anim_update
                && let Some(ref mut anim_state) = shader_mat.animation
            {
                let lag_value = anim_state.update(source_value, delta_time);
                shader_mat.current_values.insert(target_param, lag_value);
                material_changed = true;
            }
        }

        // Phase 3: Update DynamicMaterial2d uniforms only if values changed
        // 阶段 3：仅在值变化时更新材质 uniform
        if material_changed {
            if let Some(material) = materials.get_mut(&material_handle.0) {
                material.params = shader_mat.pack_params();
                material.extra_params = shader_mat.pack_extra_params();
            }

            // Update debug string only when values change (not every frame)
            // 仅在值变化时更新调试字符串（而非每帧）
            shader_mat.current_values_debug = format!("{:?}", shader_mat.current_values);
        }
    }
}
