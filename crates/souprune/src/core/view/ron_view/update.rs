use super::super::components::ViewRoot;
use super::super::components::{
    DynamicViewElement, HPBarLag, HPBarSprite, HPSourceType, TimeDependentTransform,
    ViewTextTemplate,
};
use super::super::layout::serde_types::vec2_tuple_to_static;
use super::super::sdf_view_shape::parse_text_preserving_whitespace;
use super::parsing::{PlayerDataView, evaluate_float_expr, resolve_text_content};
use bevy::prelude::*;
use bevy_fact_rule_event::LayeredFactDatabase;
use bevy_rich_text3d::Text3d;

pub fn update_hp_bar_shader_params(
    time: Res<Time>,
    layered_db: Res<LayeredFactDatabase>,
    mut materials: ResMut<Assets<super::super::custom_sprite_material::CustomSpriteMaterial>>,
    mut query: Query<(
        &MeshMaterial2d<super::super::custom_sprite_material::CustomSpriteMaterial>,
        &mut HPBarLag,
        &HPBarSprite,
    )>,
) {
    // Early exit: skip if no HP bars exist
    // 提前退出：如果没有 HP 条则跳过
    if query.is_empty() {
        return;
    }

    // Check if any HP bar needs animation update (lag animation in progress)
    // 检查是否有 HP 条需要动画更新（lag 动画进行中）
    let needs_animation_update = query.iter().any(|(_, lag, _)| lag.anim_progress < 0.5);

    // Only proceed if database changed OR animation is in progress
    // 仅在数据库变化或动画进行中时继续
    if !layered_db.is_changed() && !needs_animation_update {
        return;
    }

    let player_data = PlayerDataView::new(&layered_db);

    for (material_handle, mut lag, hp_bar) in query.iter_mut() {
        // Get HP values based on source type
        // 根据来源类型获取 HP 值
        let (hp, hp_max) = get_hp_values_from_source(&hp_bar.hp_source, &player_data);
        let hp_ratio = if hp_max > 0.0 { hp / hp_max } else { 1.0 };

        // Detect significant HP drop (Damage taken)
        // 检测明显的 HP 下降（受到伤害）
        if hp_ratio < lag.last_hp_ratio - 0.001 {
            // Start the sequence immediately
            lag.delay_timer = 0.0;
            lag.start_lag_ratio = lag.lag_hp_ratio;
            lag.anim_progress = 0.0;
            trace!(
                "[HP Bar] Damage detected! Starting OutCirc animation. hp_ratio: {:.3}, last: {:.3}",
                hp_ratio, lag.last_hp_ratio
            );
        }

        lag.last_hp_ratio = hp_ratio;

        if hp_ratio > lag.lag_hp_ratio {
            // HEALED: Instant sync
            lag.lag_hp_ratio = hp_ratio;
            lag.anim_progress = 0.5;
            lag.delay_timer = 0.0;
        } else if hp_ratio < lag.lag_hp_ratio && lag.anim_progress < 0.5 {
            lag.anim_progress = (lag.anim_progress + time.delta_secs()).min(0.5);

            // OutCirc easing formula
            // t: 0.0 -> 1.0
            let t = lag.anim_progress / 0.5;
            let eased_t = (1.0 - (t - 1.0).powi(2)).sqrt();

            // Interpolate between start and current actual HP
            lag.lag_hp_ratio = lag.start_lag_ratio + (hp_ratio - lag.start_lag_ratio) * eased_t;
        }
        // Final safety sync
        if (lag.lag_hp_ratio - hp_ratio).abs() < 0.001 {
            lag.lag_hp_ratio = hp_ratio;
        }

        // Evaluate half_width from config expression - config is required
        let half_width = if let Some(ref params) = hp_bar.shader_params_expr {
            // Get the third component (half_width) from the expression tuple
            evaluate_float_expr(&params.2, &player_data, None)
        } else {
            // No config provided - use simple fallback, NOT game-specific formula
            40.0
        };

        // Evaluate alpha from config if available
        let alpha = if let Some(ref params) = hp_bar.shader_params_expr {
            evaluate_float_expr(&params.3, &player_data, None)
        } else {
            1.0
        };

        let target_params = LinearRgba::new(hp_ratio, lag.lag_hp_ratio, half_width, alpha);

        if let Some(material) = materials.get_mut(material_handle) {
            material.color_params = target_params;
        }
    }
}

/// Get HP values from the specified source.
/// 从指定来源获取 HP 值。
fn get_hp_values_from_source(hp_source: &HPSourceType, player_data: &PlayerDataView) -> (f32, f32) {
    match hp_source {
        HPSourceType::Player => {
            let hp = player_data.get_fact_int("player_hp").unwrap_or(20) as f32;
            let hp_max = player_data.get_fact_int("player_hp_max").unwrap_or(20) as f32;
            (hp, hp_max)
        }
        HPSourceType::Enemy { index } => {
            let hp = player_data
                .get_fact_int_list("enemy_hps")
                .and_then(|list| list.get(*index).copied())
                .unwrap_or(100) as f32;
            let hp_max = player_data
                .get_fact_int_list("enemy_hp_maxs")
                .and_then(|list| list.get(*index).copied())
                .unwrap_or(100) as f32;
            (hp, hp_max)
        }
        HPSourceType::Custom {
            hp_expr,
            hp_max_expr,
        } => {
            let hp =
                super::parsing::evaluate_fact_expression(hp_expr, player_data).unwrap_or(100.0);
            let hp_max =
                super::parsing::evaluate_fact_expression(hp_max_expr, player_data).unwrap_or(100.0);
            (hp, hp_max)
        }
    }
}

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
) {
    use bevy::prelude::DetectChanges;

    if !layered_db.is_changed() {
        return;
    }

    let player_data = PlayerDataView::new(&layered_db);

    info!(
        "[update_dynamic_text_system] LayeredFactDatabase changed! hp={}, hp_max={}",
        player_data.get_fact_int("player_hp").unwrap_or(0),
        player_data.get_fact_int("player_hp_max").unwrap_or(0)
    );

    for (entity, template, mut text3d, name) in text_query.iter_mut() {
        info!(
            "[update_dynamic_text_system] Updating text '{}' with template: '{}'",
            name, template.0
        );

        let new_content =
            resolve_text_content(&template.0, &mortar_strings, &player_data, &item_registry);

        info!(
            "[update_dynamic_text_system] Resolved content for '{}': '{}'",
            name, new_content
        );

        // We also need to check if there is a conditional style embedded (not fully supported by simple re-resolve yet)
        // But the original spawn logic handled conditional color.
        // For now, let's just update the content. Re-implementing conditional color here would be ideal.
        //
        // 我们还需要检查是否嵌入了条件样式（目前的简单重新解析尚未完全支持）。
        // 但原始生成逻辑处理了条件颜色。
        // 目前，我们只更新内容。在此处理想情况下重新实现条件颜色。

        // Re-parsing the text3d
        *text3d = parse_text_preserving_whitespace(&new_content);

        // CRITICAL FIX: Add NeedsGlyphRefresh to trigger text re-rendering
        // 关键修复：添加 NeedsGlyphRefresh 以触发文本重新渲染
        commands
            .entity(entity)
            .insert(super::super::text::NeedsGlyphRefresh);
        info!(
            "[update_dynamic_text_system] Added NeedsGlyphRefresh to '{}'",
            name
        );

        // Note: This simple update doesn't handle the "conditional_style" color change logic present in `spawn_ui_node`.
        // To support that, we would need to store the `conditional_style` in a component too.
        // For HP update, it is usually just text change, so this might be enough for the bug report.
    }
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
