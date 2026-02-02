use super::super::components::ViewRoot;
use super::super::components::{DynamicViewElement, HPBarLag, HPBarSprite, ViewTextTemplate};
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
    )>,
) {
    let player_data = PlayerDataView::new(&layered_db);
    let hp_ratio = player_data.hp() as f32 / player_data.hp_max() as f32;

    for (material_handle, mut lag) in query.iter_mut() {
        // Detect significant HP drop (Damage taken)
        if hp_ratio < lag.last_hp_ratio {
            // Start the sequence immediately
            lag.delay_timer = 0.0;
            lag.start_lag_ratio = lag.lag_hp_ratio;
            lag.anim_progress = 0.0;
            info!("[HP Bar] Damage detected! Starting OutCirc animation immediately.");
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

        let half_width = 40.0 + (player_data.hp_max() as f32 - 20.0) * 95.0 / 79.0 / 2.0; // Dynamic based on hp_max
        let target_params = LinearRgba::new(hp_ratio, lag.lag_hp_ratio, half_width, 1.0);

        if let Some(material) = materials.get_mut(material_handle) {
            material.color_params = target_params;
        }
    }
}

pub fn update_dynamic_ui_elements(
    time: Res<Time>,
    layered_db: Res<LayeredFactDatabase>,
    mut query: Query<(
        Entity,
        &DynamicViewElement,
        &mut Transform,
        Option<&mut HPBarSprite>,
    )>,
    parent_query: Query<&ChildOf>,
    view_root_query: Query<&ViewRoot>,
    mut frame_count: Local<usize>,
) {
    *frame_count += 1;
    if !query.is_empty() && (*frame_count).is_multiple_of(60) {
        debug!(
            "update_dynamic_ui_elements: processing {} entities. Time: {}",
            query.iter().len(),
            time.elapsed_secs_f64()
        );
    }

    for (entity, dynamic_elem, mut transform, hp_bar) in query.iter_mut() {
        // Find ViewRoot ancestor to get local_facts
        let local_facts = find_view_root_ancestor(entity, &parent_query, &view_root_query)
            .map(|root| &root.local_facts);

        // Create PlayerDataView with local facts if available
        let player_data = if let Some(local) = local_facts {
            PlayerDataView::with_local_facts(&layered_db, local)
        } else {
            PlayerDataView::new(&layered_db)
        };

        // Update sprite transform and shader params if present
        if let Some(sprite_def) = &dynamic_elem.sprite_def {
            if let Some(t_def) = &sprite_def.transform {
                let new_translation = if let Some(trans) = &t_def.translation {
                    Vec3::new(
                        evaluate_float_expr(&trans.0, &player_data, Some(time.elapsed_secs_f64())),
                        evaluate_float_expr(&trans.1, &player_data, Some(time.elapsed_secs_f64())),
                        evaluate_float_expr(&trans.2, &player_data, Some(time.elapsed_secs_f64())),
                    )
                } else {
                    Vec3::ZERO
                };

                if let Some(scale_def) = &t_def.scale {
                    let new_scale = Vec3::new(
                        evaluate_float_expr(
                            &scale_def.0,
                            &player_data,
                            Some(time.elapsed_secs_f64()),
                        ),
                        evaluate_float_expr(
                            &scale_def.1,
                            &player_data,
                            Some(time.elapsed_secs_f64()),
                        ),
                        evaluate_float_expr(
                            &scale_def.2,
                            &player_data,
                            Some(time.elapsed_secs_f64()),
                        ),
                    );

                    // Apply pivot offset if present
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

            // Update HP bar shader params if present
            if let (Some(mut hp_bar_sprite), Some(shader_params)) =
                (hp_bar, &sprite_def.shader_params)
            {
                hp_bar_sprite.shader_params = Color::srgba(
                    evaluate_float_expr(
                        &shader_params.0,
                        &player_data,
                        Some(time.elapsed_secs_f64()),
                    ),
                    evaluate_float_expr(
                        &shader_params.1,
                        &player_data,
                        Some(time.elapsed_secs_f64()),
                    ),
                    evaluate_float_expr(
                        &shader_params.2,
                        &player_data,
                        Some(time.elapsed_secs_f64()),
                    ),
                    evaluate_float_expr(
                        &shader_params.3,
                        &player_data,
                        Some(time.elapsed_secs_f64()),
                    ),
                );
            }
        }

        // Update text transform if present
        if let Some(text_def) = &dynamic_elem.text_def {
            let new_translation = if let Some(trans) = &text_def.transform.translation {
                Vec3::new(
                    evaluate_float_expr(&trans.0, &player_data, Some(time.elapsed_secs_f64())),
                    evaluate_float_expr(&trans.1, &player_data, Some(time.elapsed_secs_f64())),
                    evaluate_float_expr(&trans.2, &player_data, Some(time.elapsed_secs_f64())),
                )
            } else {
                Vec3::ZERO
            };
            transform.translation = new_translation;

            if let Some(scale_def) = &text_def.transform.scale {
                transform.scale = Vec3::new(
                    evaluate_float_expr(&scale_def.0, &player_data, Some(time.elapsed_secs_f64())),
                    evaluate_float_expr(&scale_def.1, &player_data, Some(time.elapsed_secs_f64())),
                    evaluate_float_expr(&scale_def.2, &player_data, Some(time.elapsed_secs_f64())),
                );
            }
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
        player_data.hp(),
        player_data.hp_max()
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
