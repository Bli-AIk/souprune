//! # delta.rs
//!
//! # 视图差异模块
//!
//! Defines ViewDelta enum and apply_deltas function.
//!
//! 定义 ViewDelta 枚举和 apply_deltas 函数。

use super::tree::{DesiredElement, DesiredHealthBar, DesiredMaterial, DesiredSprite, DesiredText};
use crate::core::view::layout::ViewLayoutRect;
use bevy::prelude::*;
use bevy::sprite::Anchor;

/// Represents a single change operation to be applied to the ECS world.
/// 表示要应用到 ECS 世界的单个更改操作。
#[derive(Debug)]
#[expect(clippy::large_enum_variant)] // reason: Spawn variant intentionally holds full spec inline
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

    /// Update stored layout rectangle of an existing element.
    /// 更新现有元素存储的布局矩形。
    UpdateLayout {
        entity: Entity,
        new_value: ViewLayoutRect,
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
    UpdateHealthBar {
        entity: Entity,
        new_value: DesiredHealthBar,
    },

    /// Update the visible_when expression component.
    /// 更新 visible_when 表达式组件。
    UpdateVisibleWhen {
        entity: Entity,
        new_expression: String,
    },

    /// Update material definition of an existing element.
    /// 更新现有元素的材质定义。
    UpdateMaterial {
        entity: Entity,
        new_value: DesiredMaterial,
    },
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
                commands.entity(*entity).try_insert(*new_value);
            }
            ViewDelta::UpdateLayout { entity, new_value } => {
                commands.entity(*entity).try_insert(*new_value);
            }
            ViewDelta::UpdateVisibility { entity, new_value } => {
                commands.entity(*entity).try_insert(*new_value);
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
                    queue_update_sprite(world, entity_id, color, flip_x, flip_y, anchor);
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
                    .try_insert(Text2d::new(&new_value.content));
            }
            ViewDelta::UpdateHealthBar {
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
                    .try_insert(crate::core::view::components::VisibleWhen {
                        expression: new_expression.clone(),
                    });
            }
            ViewDelta::UpdateMaterial {
                entity,
                new_value: _,
            } => {
                // Material updates are handled by dedicated systems
                // This delta is mainly for tracking that an update is needed
                // 材质更新由专门的系统处理
                // 此差异主要用于跟踪需要更新
                let _ = entity;
            }
        }
    }
}

/// Queued world command: update sprite properties on an entity.
fn queue_update_sprite(
    world: &mut World,
    entity_id: Entity,
    color: Color,
    flip_x: bool,
    flip_y: bool,
    anchor: Anchor,
) {
    let Ok(mut entity_mut) = world.get_entity_mut(entity_id) else {
        return;
    };
    if let Some(mut sprite) = entity_mut.get_mut::<Sprite>() {
        sprite.color = color;
        sprite.flip_x = flip_x;
        sprite.flip_y = flip_y;
    }
    entity_mut.insert(anchor);
}

/// Apply a spawn delta by creating a new entity with all components.
/// 通过创建带有所有组件的新实体来应用生成差异。
fn apply_spawn_delta(
    commands: &mut Commands,
    parent: Option<Entity>,
    spec: &DesiredElement,
) -> Entity {
    // Extract namespace from full_name
    let (namespace, local_name) = if let Some(idx) = spec.key.full_name.rfind("::") {
        (
            spec.key.full_name[..idx].to_string(),
            spec.key.full_name[idx + 2..].to_string(),
        )
    } else {
        (String::new(), spec.key.full_name.clone())
    };

    let entity_id = {
        let mut entity_commands = commands.spawn((
            spec.transform,
            GlobalTransform::default(),
            spec.visibility,
            InheritedVisibility::default(),
            ViewVisibility::default(),
            crate::core::view::ron_view::resources::RonDrivenView,
            crate::core::view::components::ViewElement {
                full_name: spec.key.full_name.clone(),
                local_name,
                namespace,
                tags: spec.tags.clone(),
            },
        ));

        if spec.sprite.is_none() && (!spec.texts.is_empty() || !spec.children.is_empty()) {
            entity_commands.insert(crate::core::view::components::ViewContainer);
        }

        if let Some(layout_rect) = spec.layout_rect {
            entity_commands.insert(layout_rect);
        }

        if let Some(ref expr) = spec.visible_when_expr {
            entity_commands.insert(crate::core::view::components::VisibleWhen {
                expression: expr.clone(),
            });
        }

        if let Some(ref sprite_spec) = spec.sprite {
            entity_commands.insert(Sprite {
                color: sprite_spec.color,
                flip_x: sprite_spec.flip_x,
                flip_y: sprite_spec.flip_y,
                ..default()
            });
            entity_commands.insert(sprite_spec.anchor);
        }

        if !spec.texts.is_empty() {
            let content = spec
                .texts
                .iter()
                .map(|text| text.content.as_str())
                .collect::<Vec<_>>()
                .join("\n");
            entity_commands.insert(Text2d::new(content));
        }

        if let Some(parent_entity) = parent {
            entity_commands.insert(ChildOf(parent_entity));
        }

        entity_commands.id()
    };

    for child in &spec.children {
        apply_spawn_delta(commands, Some(entity_id), child);
    }

    entity_id
}

/// Statistics about applied deltas for debugging.
/// 应用差异的统计信息，用于调试。
#[derive(Debug, Default)]
pub struct DeltaStats {
    pub spawns: usize,
    pub despawns: usize,
    pub transform_updates: usize,
    pub layout_updates: usize,
    pub visibility_updates: usize,
    pub sprite_updates: usize,
    pub text_updates: usize,
    pub health_bar_updates: usize,
    pub visible_when_updates: usize,
    pub material_updates: usize,
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
                ViewDelta::UpdateLayout { .. } => stats.layout_updates += 1,
                ViewDelta::UpdateVisibility { .. } => stats.visibility_updates += 1,
                ViewDelta::UpdateSprite { .. } => stats.sprite_updates += 1,
                ViewDelta::UpdateText { .. } => stats.text_updates += 1,
                ViewDelta::UpdateHealthBar { .. } => stats.health_bar_updates += 1,
                ViewDelta::UpdateVisibleWhen { .. } => stats.visible_when_updates += 1,
                ViewDelta::UpdateMaterial { .. } => stats.material_updates += 1,
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
            || self.layout_updates > 0
            || self.visibility_updates > 0
            || self.sprite_updates > 0
            || self.text_updates > 0
            || self.health_bar_updates > 0
            || self.visible_when_updates > 0
            || self.material_updates > 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Resource)]
    struct StaleEntity(Entity);

    fn update_stale_transform(mut commands: Commands, stale: Res<StaleEntity>) {
        apply_deltas(
            &mut commands,
            &[ViewDelta::UpdateTransform {
                entity: stale.0,
                new_value: Transform::from_xyz(1.0, 2.0, 3.0),
            }],
        );
    }

    #[test]
    fn transform_update_ignores_already_despawned_entity() {
        let mut app = App::new();
        let entity = app.world_mut().spawn(Transform::default()).id();
        app.world_mut().despawn(entity);
        app.insert_resource(StaleEntity(entity));
        app.add_systems(Update, update_stale_transform);

        app.update();
    }

    #[derive(Resource)]
    struct LayoutEntity(Entity);

    fn update_layout_rect(mut commands: Commands, target: Res<LayoutEntity>) {
        apply_deltas(
            &mut commands,
            &[ViewDelta::UpdateLayout {
                entity: target.0,
                new_value: ViewLayoutRect {
                    x: 12.0,
                    y: 8.0,
                    width: 96.0,
                    height: 32.0,
                },
            }],
        );
    }

    #[test]
    fn layout_update_writes_layout_rect_component() {
        let mut app = App::new();
        let entity = app.world_mut().spawn_empty().id();
        app.insert_resource(LayoutEntity(entity));
        app.add_systems(Update, update_layout_rect);

        app.update();

        let rect = app
            .world()
            .entity(entity)
            .get::<ViewLayoutRect>()
            .expect("layout rect should be inserted");
        assert_eq!(rect.x, 12.0);
        assert_eq!(rect.y, 8.0);
        assert_eq!(rect.width, 96.0);
        assert_eq!(rect.height, 32.0);
    }

    fn spawn_parent_with_child(mut commands: Commands) {
        let mut parent = DesiredElement::new(
            super::super::tree::ViewElementKey::new("test::Parent"),
            "Parent",
        );
        parent.children.push(DesiredElement::new(
            super::super::tree::ViewElementKey::new("test::Child"),
            "Child",
        ));
        apply_deltas(
            &mut commands,
            &[ViewDelta::Spawn {
                parent: None,
                spec: parent,
            }],
        );
    }

    #[test]
    fn spawn_delta_recursively_spawns_children() {
        let mut app = App::new();
        app.add_systems(Update, spawn_parent_with_child);

        app.update();

        let mut parent_entity = None;
        let mut child_entity = None;
        let mut query = app
            .world_mut()
            .query::<(Entity, &crate::core::view::components::ViewElement)>();
        for (entity, view_element) in query.iter(app.world()) {
            match view_element.full_name.as_str() {
                "test::Parent" => parent_entity = Some(entity),
                "test::Child" => child_entity = Some(entity),
                _ => {}
            }
        }

        let parent_entity = parent_entity.expect("parent should spawn");
        let child_entity = child_entity.expect("child should spawn");
        let child_of = app
            .world()
            .entity(child_entity)
            .get::<ChildOf>()
            .expect("child should be parented");
        assert_eq!(child_of.parent(), parent_entity);
    }

    fn spawn_text_element(mut commands: Commands) {
        let mut label = DesiredElement::new(
            super::super::tree::ViewElementKey::new("test::Label"),
            "Label",
        );
        label.texts.push(super::super::tree::DesiredText {
            content: "Hello".to_string(),
            ..Default::default()
        });
        apply_deltas(
            &mut commands,
            &[ViewDelta::Spawn {
                parent: None,
                spec: label,
            }],
        );
    }

    #[test]
    fn spawn_delta_inserts_text_for_desired_text_element() {
        let mut app = App::new();
        app.add_systems(Update, spawn_text_element);

        app.update();

        let mut query = app
            .world_mut()
            .query::<(&crate::core::view::components::ViewElement, &Text2d)>();
        let text = query
            .iter(app.world())
            .find_map(|(view_element, text)| {
                (view_element.full_name == "test::Label").then_some(text)
            })
            .expect("spawned label should carry text");
        assert_eq!(text.0, "Hello");
    }
}
