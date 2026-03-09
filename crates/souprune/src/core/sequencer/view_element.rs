//! # sequencer/view_element.rs
//!
//! ## Module Overview
//!
//! ModifyViewElement systems for the battle sequencer.
//!
//! 战斗序列管理器的 ModifyViewElement 系统。

use super::chapter_schema::Chapter;
use super::context::*;
use crate::core::view::components::ViewBox;
use bevy::prelude::*;
use bevy_fact_rule_event::LayeredFactDatabase;

/// System to process ModifyViewElement chapters.
///
/// 处理 ModifyViewElement 章节的系统。
pub fn process_modify_view_element_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    active_chapters: Query<
        (Entity, &ActiveChapter),
        (Without<WaitTimer>, Without<ChapterFinished>),
    >,
    view_elements: Query<(Entity, &crate::core::view::components::ViewElement)>,
    mut transforms: Query<&mut Transform>,
    mut sprites: Query<&mut Sprite>,
    mut visibilities: Query<&mut Visibility>,
    mut histories: Query<&mut crate::core::view::ViewElementHistory>,
    mut ui_boxes: Query<&mut ViewBox>,
    layered_db: Res<LayeredFactDatabase>,
) {
    use crate::core::view::ron_view::parsing::PlayerDataView;
    let player_data = PlayerDataView::new(&layered_db);

    for (chapter_entity, active_chapter) in active_chapters.iter() {
        let Chapter::ModifyViewElement {
            selector,
            modification,
        } = &active_chapter.chapter
        else {
            continue;
        };

        info!(
            "[ModifyViewElement] Processing: selector={:?}, modification={:?}",
            selector, modification
        );

        // Resolve the selector to get target entities
        // 解析选择器以获取目标实体
        let target_entities = match selector {
            super::chapter_schema::ElementSelector::FullName(full_name) => {
                if let Some(entity) =
                    crate::core::view::find_element_by_full_name(&view_elements, full_name)
                {
                    info!(
                        "[ModifyViewElement] Found element: {:?} (full_name={})",
                        entity, full_name
                    );
                    vec![entity]
                } else {
                    warn!(
                        "[ModifyViewElement] Element not found (full name): {}",
                        full_name
                    );
                    vec![]
                }
            }
            super::chapter_schema::ElementSelector::LocalName(local_name) => {
                // For simplicity, search in all namespaces
                // 为简单起见，在所有命名空间中搜索
                view_elements
                    .iter()
                    .filter(|(_, elem)| elem.local_name == *local_name)
                    .map(|(entity, _)| entity)
                    .collect()
            }
            super::chapter_schema::ElementSelector::Tag(tag) => {
                crate::core::view::find_elements_by_tag(&view_elements, tag)
            }
        };

        // Apply the modification to all target entities
        // 对所有目标实体应用修改
        for entity in target_entities {
            info!(
                "[ModifyViewElement] Applying modification to entity {:?}",
                entity
            );

            apply_modification(
                &mut commands,
                &asset_server,
                entity,
                modification,
                &mut transforms,
                &mut sprites,
                &mut visibilities,
                &mut histories,
                &mut ui_boxes,
                &player_data,
            );
        }

        // Mark chapter as finished
        // 标记章节为完成
        commands.entity(chapter_entity).insert(ChapterFinished);
    }
}

/// Apply a modification to a single entity.
///
/// 对单个实体应用修改。
fn apply_modification(
    commands: &mut Commands,
    asset_server: &AssetServer,
    entity: Entity,
    modification: &super::chapter_schema::ElementModification,
    transforms: &mut Query<&mut Transform>,
    sprites: &mut Query<&mut Sprite>,
    visibilities: &mut Query<&mut Visibility>,
    histories: &mut Query<&mut crate::core::view::ViewElementHistory>,
    ui_boxes: &mut Query<&mut ViewBox>,
    player_data: &crate::core::view::ron_view::parsing::PlayerDataView<'_>,
) {
    use crate::core::view::ron_view::parsing::resolve_val_f32;

    match modification {
        super::chapter_schema::ElementModification::SetTexture(path) => {
            if let Ok(mut sprite) = sprites.get_mut(entity) {
                let texture_path = if path.starts_with("assets/textures/") {
                    path.clone()
                } else {
                    format!("assets/textures/{}", path)
                };
                sprite.image = asset_server.load(&texture_path);
                info!("Set texture for entity {:?}: {}", entity, texture_path);
            }
        }
        super::chapter_schema::ElementModification::SetPosition(x, y, z) => {
            if let Ok(mut transform) = transforms.get_mut(entity) {
                let final_x = resolve_val_f32(x, Some(transform.translation.x), player_data, None);
                let final_y = resolve_val_f32(y, Some(transform.translation.y), player_data, None);
                let final_z = resolve_val_f32(z, Some(transform.translation.z), player_data, None);

                // Ensure history exists or create it
                // 确保历史存在或创建它
                let history_exists = histories.get_mut(entity).is_ok();
                if !history_exists {
                    let original_state = crate::core::view::ElementState::capture(
                        Some(&*transform),
                        sprites.get(entity).ok(),
                        visibilities.get(entity).ok(),
                    );
                    commands
                        .entity(entity)
                        .insert(crate::core::view::ViewElementHistory::new(original_state));
                }

                // Apply modification
                // 应用修改
                transform.translation = Vec3::new(final_x, final_y, final_z);
                info!(
                    "Set position for entity {:?}: ({}, {}, {})",
                    entity, final_x, final_y, final_z
                );

                // Push NEW state to history AFTER modification
                // 在修改后将新状态推送到历史
                if let Ok(mut history) = histories.get_mut(entity) {
                    let new_state = crate::core::view::ElementState::capture(
                        Some(&*transform),
                        sprites.get(entity).ok(),
                        visibilities.get(entity).ok(),
                    );
                    history.push(new_state);
                }
            }
        }
        super::chapter_schema::ElementModification::SetScale(x, y, z) => {
            if let Ok(mut transform) = transforms.get_mut(entity) {
                let final_x = resolve_val_f32(x, Some(transform.scale.x), player_data, None);
                let final_y = resolve_val_f32(y, Some(transform.scale.y), player_data, None);
                let final_z = resolve_val_f32(z, Some(transform.scale.z), player_data, None);

                transform.scale = Vec3::new(final_x, final_y, final_z);
                info!(
                    "Set scale for entity {:?}: ({}, {}, {})",
                    entity, final_x, final_y, final_z
                );
            }
        }
        super::chapter_schema::ElementModification::SetColor(r, g, b, a) => {
            if let Ok(mut sprite) = sprites.get_mut(entity) {
                let color = sprite.color;

                let final_r = resolve_val_f32(r, Some(color.to_srgba().red), player_data, None);
                let final_g = resolve_val_f32(g, Some(color.to_srgba().green), player_data, None);
                let final_b = resolve_val_f32(b, Some(color.to_srgba().blue), player_data, None);
                let final_a = resolve_val_f32(a, Some(color.to_srgba().alpha), player_data, None);

                sprite.color = Color::srgba(final_r, final_g, final_b, final_a);
                info!(
                    "Set color for entity {:?}: ({}, {}, {}, {})",
                    entity, final_r, final_g, final_b, final_a
                );
            }
        }
        super::chapter_schema::ElementModification::SetVisibility(visible) => {
            if let Ok(mut visibility) = visibilities.get_mut(entity) {
                use crate::core::view::ron_view::parsing::resolve_val_bool;

                let is_visible = resolve_val_bool(visible, player_data);
                *visibility = if is_visible {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
                info!("Set visibility for entity {:?}: {}", entity, is_visible);
            }
        }
        super::chapter_schema::ElementModification::SetBoxSize(width, height) => {
            if let Ok(mut ui_box) = ui_boxes.get_mut(entity) {
                let new_width = resolve_val_f32(width, None, player_data, None);
                let new_height = resolve_val_f32(height, None, player_data, None);
                ui_box.width = new_width;
                ui_box.height = new_height;
                info!(
                    "Set box size for entity {:?}: {}x{}",
                    entity, new_width, new_height
                );
            }
        }
        super::chapter_schema::ElementModification::Undo => {
            if let Ok(mut history) = histories.get_mut(entity)
                && let Some(previous_state) = history.undo()
            {
                // Apply previous state
                // 应用之前的状态
                if let Some((trans, rot, scale)) = previous_state.transform
                    && let Ok(mut transform) = transforms.get_mut(entity)
                {
                    transform.translation = trans;
                    transform.rotation = rot;
                    transform.scale = scale;
                }
                if let Some(color) = previous_state.color
                    && let Ok(mut sprite) = sprites.get_mut(entity)
                {
                    sprite.color = color;
                }
                if let Some(vis) = previous_state.visibility
                    && let Ok(mut visibility) = visibilities.get_mut(entity)
                {
                    *visibility = vis;
                }
            }
        }
        super::chapter_schema::ElementModification::Redo => {
            if let Ok(mut history) = histories.get_mut(entity)
                && let Some(next_state) = history.redo()
            {
                // Apply next state
                // 应用下一个状态
                if let Some((trans, rot, scale)) = next_state.transform
                    && let Ok(mut transform) = transforms.get_mut(entity)
                {
                    transform.translation = trans;
                    transform.rotation = rot;
                    transform.scale = scale;
                }
                if let Some(color) = next_state.color
                    && let Ok(mut sprite) = sprites.get_mut(entity)
                {
                    sprite.color = color;
                }
                if let Some(vis) = next_state.visibility
                    && let Ok(mut visibility) = visibilities.get_mut(entity)
                {
                    *visibility = vis;
                }
            }
        }
        super::chapter_schema::ElementModification::Reset => {
            if let Ok(mut history) = histories.get_mut(entity) {
                let original_state = history.reset();
                // Apply original state
                // 应用原始状态
                if let Some((trans, rot, scale)) = original_state.transform
                    && let Ok(mut transform) = transforms.get_mut(entity)
                {
                    transform.translation = trans;
                    transform.rotation = rot;
                    transform.scale = scale;
                }
                if let Some(color) = original_state.color
                    && let Ok(mut sprite) = sprites.get_mut(entity)
                {
                    sprite.color = color;
                }
                if let Some(vis) = original_state.visibility
                    && let Ok(mut visibility) = visibilities.get_mut(entity)
                {
                    *visibility = vis;
                }
            }
        }
    }
}
