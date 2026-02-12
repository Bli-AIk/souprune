use super::super::components::ViewRoot;
use super::super::components::{DynamicViewElement, TimeDependentTransform, ViewTextTemplate};
use super::super::layout::serde_types::vec2_tuple_to_static;
use super::super::sdf_view_shape::parse_text_preserving_whitespace;
use super::parsing::{PlayerDataView, evaluate_float_expr, resolve_text_content};
use bevy::prelude::*;
use bevy_fact_rule_event::LayeredFactDatabase;
use bevy_rich_text3d::Text3d;

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
}

pub fn update_dynamic_text_system(
    mut commands: Commands,
    mut text_query: Query<(Entity, &ViewTextTemplate, &mut Text3d, &Name)>,
    layered_db: Res<LayeredFactDatabase>,
    item_registry: Res<crate::core::item::ItemRegistry>,
    mortar_strings: Res<crate::extra::mortar::MortarStringTable>,
    view_root_query: Query<(Entity, &ViewRoot)>,
    changed_view_roots: Query<Entity, Changed<ViewRoot>>,
    parent_query: Query<&ChildOf>,
) {
    use bevy::prelude::DetectChanges;

    // Check if we need to update: either global DB changed or any ViewRoot's local_facts changed
    // 检查是否需要更新：全局数据库变化或任何 ViewRoot 的 local_facts 变化
    let global_changed = layered_db.is_changed();
    let any_view_root_changed = !changed_view_roots.is_empty();

    if !global_changed && !any_view_root_changed {
        return;
    }

    // Base player data for fallback
    // 用于回退的基础 player data
    let base_player_data = PlayerDataView::new(&layered_db);

    trace!(
        "[update_dynamic_text_system] Update triggered (global_changed={}, local_changed={}) hp={}, hp_max={}",
        global_changed,
        any_view_root_changed,
        base_player_data.get_fact_int("player_hp").unwrap_or(0),
        base_player_data.get_fact_int("player_hp_max").unwrap_or(0)
    );

    for (entity, template, mut text3d, name) in text_query.iter_mut() {
        // Find ViewRoot ancestor to access local facts
        // 查找 ViewRoot 祖先以访问局部事实
        let view_root_result =
            find_view_root_ancestor_entity(entity, &parent_query, &view_root_query);
        let player_data = if let Some((_, view_root)) = view_root_result {
            PlayerDataView::with_local_facts(&layered_db, &view_root.local_facts)
        } else {
            PlayerDataView::new(&layered_db)
        };

        let new_content =
            resolve_text_content(&template.0, &mortar_strings, &player_data, &item_registry);

        // Performance optimization: Skip if content hasn't changed
        // 性能优化：如果内容未变化则跳过
        // Note: Text3d doesn't directly expose content for comparison, so we always update.
        // Future improvement: Store last resolved content in a component for comparison.
        // 注意：Text3d 不直接暴露内容用于比较，所以我们总是更新。
        // 未来改进：在组件中存储上次解析的内容用于比较。

        trace!(
            "[update_dynamic_text_system] Updating text '{}': '{}'",
            name, new_content
        );

        // Re-parsing the text3d
        // 重新解析 text3d
        *text3d = parse_text_preserving_whitespace(&new_content);

        // CRITICAL FIX: Add NeedsGlyphRefresh to trigger text re-rendering
        // Use queue_handled to avoid panic if entity is despawned during the same frame
        // 关键修复：添加 NeedsGlyphRefresh 以触发文本重新渲染
        // 使用 queue_handled 避免在同一帧中实体被销毁时 panic
        commands.queue(move |world: &mut World| {
            if world.get_entity(entity).is_ok() {
                world
                    .entity_mut(entity)
                    .insert(super::super::text::NeedsGlyphRefresh);
            }
        });

        // Note: This simple update doesn't handle the "conditional_style" color change logic present in `spawn_ui_node`.
        // To support that, we would need to store the `conditional_style` in a component too.
        // For HP update, it is usually just text change, so this might be enough for the bug report.
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
pub fn update_shader_materials_system(
    time: Res<Time>,
    layered_db: Res<LayeredFactDatabase>,
    mut materials: ResMut<Assets<crate::core::view::dynamic_material::DynamicMaterial2d>>,
    mut query: Query<(
        Entity,
        &mut super::super::components::ShaderMaterial,
        &crate::core::view::dynamic_material::MeshDynamicMaterial2d,
    )>,
    parent_query: Query<&ChildOf>,
    view_root_query: Query<(Entity, &ViewRoot)>,
    changed_view_roots: Query<Entity, Changed<ViewRoot>>,
) {
    use crate::core::view::layout::Val;
    use crate::core::view::layout::view_schema::MaterialParamValue;
    use std::collections::HashMap;

    // Early exit: skip if no shader materials exist
    if query.is_empty() {
        return;
    }

    // Check if any shader material needs expression evaluation
    // 检查是否有材质需要表达式评估
    let needs_expression_eval = query.iter().any(|(_, sm, _)| {
        sm.param_defs
            .values()
            .any(|v| matches!(v, MaterialParamValue::Expr(_)))
    });

    // Check if any shader material needs animation update
    let needs_animation_update = query
        .iter()
        .any(|(_, sm, _)| sm.animation.as_ref().is_some_and(|a| a.anim_progress < 1.0));

    // Check if we need to update
    let global_changed = layered_db.is_changed();
    let any_view_root_changed = !changed_view_roots.is_empty();

    // Only proceed if database changed OR animation is in progress OR expressions need evaluation
    // 只有在数据库变化、动画进行中或需要表达式评估时才继续
    if !global_changed
        && !any_view_root_changed
        && !needs_animation_update
        && !needs_expression_eval
    {
        return;
    }

    let delta_time = time.delta_secs();

    for (entity, mut shader_mat, material_handle) in query.iter_mut() {
        // Find ViewRoot ancestor to access local facts
        let view_root_result =
            find_view_root_ancestor_entity(entity, &parent_query, &view_root_query);
        let player_data = if let Some((_, view_root)) = view_root_result {
            PlayerDataView::with_local_facts(&layered_db, &view_root.local_facts)
        } else {
            PlayerDataView::new(&layered_db)
        };

        // 1. Evaluate all parameter expressions
        // Collect into a temporary HashMap to avoid borrowing issues
        let new_values: HashMap<String, f32> = shader_mat
            .param_defs
            .iter()
            .map(|(name, param_def)| {
                let value = match param_def {
                    MaterialParamValue::Static(v) => *v,
                    MaterialParamValue::Expr(expr_str) => {
                        // Convert to Val::Expr for evaluate_float_expr
                        let val_expr: Val<f32> = Val::Expr(expr_str.clone());
                        let result = evaluate_float_expr(
                            &val_expr,
                            &player_data,
                            Some(time.elapsed_secs_f64()),
                        );
                        trace!(
                            "[ShaderMaterial Update] Entity {:?}: '{}' = Expr('{}') -> {}",
                            entity, name, expr_str, result
                        );
                        result
                    }
                };
                (name.clone(), value)
            })
            .collect();

        // Update current_values
        for (name, value) in new_values {
            shader_mat.current_values.insert(name, value);
        }

        // 2. Process lag animation if present
        // First extract animation info to avoid borrowing issues
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
            // Update animation and get lag value
            let lag_value = anim_state.update(source_value, delta_time);

            // Update target parameter with lag value
            shader_mat.current_values.insert(target_param, lag_value);
        }

        // Update debug string for inspector
        shader_mat.current_values_debug = format!("{:?}", shader_mat.current_values);

        // 3. Update DynamicMaterial2d uniforms
        if let Some(material) = materials.get_mut(&material_handle.0) {
            let new_params = shader_mat.pack_params();
            let new_extra = shader_mat.pack_extra_params();
            material.params = new_params;
            material.extra_params = new_extra;
        }
    }
}
