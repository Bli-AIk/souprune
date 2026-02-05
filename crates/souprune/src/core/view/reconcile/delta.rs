//! # delta.rs
//!
//! # 视图差异模块
//!
//! Defines ViewDelta enum and apply_deltas function.
//!
//! 定义 ViewDelta 枚举和 apply_deltas 函数。

use super::tree::{DesiredElement, DesiredHpBar, DesiredSprite, DesiredText};
use bevy::prelude::*;

/// Represents a single change operation to be applied to the ECS world.
/// 表示要应用到 ECS 世界的单个更改操作。
#[derive(Debug)]
pub enum ViewDelta {
    /// Spawn a new element with all its components.
    /// 生成带有所有组件的新元素。
    Spawn {
        /// Parent entity to attach to (None for root)
        /// 要附加的父实体（根为 None）
        parent: Option<Entity>,

        /// Full element specification
        /// 完整的元素规格
        spec: DesiredElement,
    },

    /// Despawn an existing element.
    /// 销毁现有元素。
    Despawn {
        /// Entity to despawn
        /// 要销毁的实体
        entity: Entity,
    },

    /// Update transform of an existing element.
    /// 更新现有元素的变换。
    UpdateTransform {
        entity: Entity,
        new_value: Transform,
    },

    /// Update visibility of an existing element.
    /// 更新现有元素的可见性。
    UpdateVisibility {
        entity: Entity,
        new_value: Visibility,
    },

    /// Update sprite properties of an existing element.
    /// 更新现有元素的精灵属性。
    UpdateSprite {
        entity: Entity,
        new_value: DesiredSprite,
    },

    /// Update text content of an existing element.
    /// 更新现有元素的文本内容。
    UpdateText {
        entity: Entity,
        text_index: usize,
        new_value: DesiredText,
    },

    /// Update HP bar parameters of an existing element.
    /// 更新现有元素的 HP 条参数。
    UpdateHpBar {
        entity: Entity,
        new_value: DesiredHpBar,
    },

    /// Update the visible_when expression component.
    /// 更新 visible_when 表达式组件。
    UpdateVisibleWhen {
        entity: Entity,
        new_expression: String,
    },

    /// Update camera offset of an existing element.
    /// 更新现有元素的相机偏移。
    UpdateCameraOffset { entity: Entity, new_offset: Vec3 },
}

/// Apply a list of deltas to the ECS world.
/// This is the only function that mutates ECS state.
///
/// 将差异列表应用到 ECS 世界。
/// 这是唯一修改 ECS 状态的函数。
pub fn apply_deltas(commands: &mut Commands, deltas: &[ViewDelta]) {
    for delta in deltas {
        match delta {
            ViewDelta::Spawn { parent, spec } => {
                apply_spawn_delta(commands, *parent, spec);
            }
            ViewDelta::Despawn { entity } => {
                commands.entity(*entity).despawn();
            }
            ViewDelta::UpdateTransform { entity, new_value } => {
                commands.entity(*entity).insert(*new_value);
            }
            ViewDelta::UpdateVisibility { entity, new_value } => {
                commands.entity(*entity).insert(*new_value);
            }
            ViewDelta::UpdateSprite { entity, new_value } => {
                // Queue update to preserve existing sprite properties like texture
                // 队列更新以保留现有的 sprite 属性（如纹理）
                let color = new_value.color;
                let flip_x = new_value.flip_x;
                let flip_y = new_value.flip_y;
                let anchor = new_value.anchor;
                let entity_id = *entity;

                commands.queue(move |world: &mut World| {
                    if let Ok(mut entity_mut) = world.get_entity_mut(entity_id) {
                        // Update sprite properties individually to preserve texture
                        // 单独更新 sprite 属性以保留纹理
                        if let Some(mut sprite) = entity_mut.get_mut::<Sprite>() {
                            sprite.color = color;
                            sprite.flip_x = flip_x;
                            sprite.flip_y = flip_y;
                        }
                        // Insert or update anchor
                        entity_mut.insert(anchor);
                    }
                });
            }
            ViewDelta::UpdateText {
                entity,
                text_index: _,
                new_value,
            } => {
                // Update text content
                // Note: This is simplified; actual implementation needs to handle Text2d
                commands
                    .entity(*entity)
                    .insert(Text2d::new(&new_value.content));
            }
            ViewDelta::UpdateHpBar {
                entity,
                new_value: _,
            } => {
                // HP bar updates are handled by dedicated systems
                // This delta is mainly for tracking that an update is needed
                // HP 条更新由专门的系统处理
                // 此差异主要用于跟踪需要更新
                let _ = entity;
            }
            ViewDelta::UpdateVisibleWhen {
                entity,
                new_expression,
            } => {
                commands
                    .entity(*entity)
                    .insert(crate::core::view::components::VisibleWhen {
                        expression: new_expression.clone(),
                    });
            }
            ViewDelta::UpdateCameraOffset { entity, new_offset } => {
                // Queue update to preserve existing CameraAnchored
                // 队列更新以保留现有的 CameraAnchored
                let offset = *new_offset;
                let entity_id = *entity;

                commands.queue(move |world: &mut World| {
                    if let Ok(mut entity_mut) = world.get_entity_mut(entity_id)
                        && let Some(mut camera_anchored) =
                            entity_mut.get_mut::<crate::core::view::components::CameraAnchored>()
                    {
                        camera_anchored.offset = offset;
                    }
                });
            }
        }
    }
}

/// Apply a spawn delta by creating a new entity with all components.
/// 通过创建带有所有组件的新实体来应用生成差异。
fn apply_spawn_delta(commands: &mut Commands, parent: Option<Entity>, spec: &DesiredElement) {
    // Extract namespace from full_name
    let (namespace, local_name) = if let Some(idx) = spec.key.full_name.rfind("::") {
        (
            spec.key.full_name[..idx].to_string(),
            spec.key.full_name[idx + 2..].to_string(),
        )
    } else {
        (String::new(), spec.key.full_name.clone())
    };

    // Create base entity bundle
    let mut entity_commands = commands.spawn((
        spec.transform,
        spec.visibility,
        crate::core::view::components::ViewElement {
            full_name: spec.key.full_name.clone(),
            local_name,
            namespace,
            tags: spec.tags.clone(),
        },
    ));

    // Add VisibleWhen component if expression exists
    if let Some(ref expr) = spec.visible_when_expr {
        entity_commands.insert(crate::core::view::components::VisibleWhen {
            expression: expr.clone(),
        });
    }

    // Add tags
    // Tags are stored in ViewElement, already added above

    // Add sprite if present
    if let Some(ref sprite_spec) = spec.sprite {
        entity_commands.insert(Sprite {
            color: sprite_spec.color,
            flip_x: sprite_spec.flip_x,
            flip_y: sprite_spec.flip_y,
            ..default()
        });
        // Add anchor as separate component
        entity_commands.insert(sprite_spec.anchor);
        // Note: Texture loading would need AssetServer which isn't available here
        // This is handled by a separate setup system
    }

    // Add parent relationship
    if let Some(parent_entity) = parent {
        entity_commands.insert(ChildOf(parent_entity));
    }

    // Recursively spawn children
    // Note: In the actual implementation, we'd need to get the spawned entity ID
    // and pass it as parent to children. This requires a different approach using
    // commands.spawn().id() or deferred commands.

    // For now, child spawning would be handled in a follow-up system
    // or by restructuring to use nested spawning patterns
}

/// Statistics about applied deltas for debugging.
/// 应用差异的统计信息，用于调试。
#[derive(Debug, Default)]
pub struct DeltaStats {
    pub spawns: usize,
    pub despawns: usize,
    pub transform_updates: usize,
    pub visibility_updates: usize,
    pub sprite_updates: usize,
    pub text_updates: usize,
    pub hp_bar_updates: usize,
    pub visible_when_updates: usize,
    pub camera_offset_updates: usize,
}

impl DeltaStats {
    /// Count deltas by type.
    /// 按类型统计差异。
    pub fn from_deltas(deltas: &[ViewDelta]) -> Self {
        let mut stats = Self::default();
        for delta in deltas {
            match delta {
                ViewDelta::Spawn { .. } => stats.spawns += 1,
                ViewDelta::Despawn { .. } => stats.despawns += 1,
                ViewDelta::UpdateTransform { .. } => stats.transform_updates += 1,
                ViewDelta::UpdateVisibility { .. } => stats.visibility_updates += 1,
                ViewDelta::UpdateSprite { .. } => stats.sprite_updates += 1,
                ViewDelta::UpdateText { .. } => stats.text_updates += 1,
                ViewDelta::UpdateHpBar { .. } => stats.hp_bar_updates += 1,
                ViewDelta::UpdateVisibleWhen { .. } => stats.visible_when_updates += 1,
                ViewDelta::UpdateCameraOffset { .. } => stats.camera_offset_updates += 1,
            }
        }
        stats
    }

    /// Check if any deltas were generated.
    /// 检查是否生成了任何差异。
    pub fn has_changes(&self) -> bool {
        self.spawns > 0
            || self.despawns > 0
            || self.transform_updates > 0
            || self.visibility_updates > 0
            || self.sprite_updates > 0
            || self.text_updates > 0
            || self.hp_bar_updates > 0
            || self.visible_when_updates > 0
            || self.camera_offset_updates > 0
    }
}
