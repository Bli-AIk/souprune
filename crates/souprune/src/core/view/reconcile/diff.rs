//! # diff.rs
//!
//! # 差异算法模块
//!
//! Implements the reconciliation algorithm to compute deltas between current and desired state.
//!
//! 实现协调算法，计算当前状态与期望状态之间的差异。

use super::delta::ViewDelta;
use super::tree::{
    CurrentElement, CurrentViewTree, DesiredElement, DesiredViewTree, ViewElementKey,
};
use bevy::prelude::*;
use std::collections::HashSet;

/// Reconcile current state with desired state, producing a list of deltas.
/// This is the core diff algorithm.
///
/// 协调当前状态与期望状态，生成差异列表。
/// 这是核心差异算法。
///
/// # Algorithm
///
/// 1. For each desired element:
///    - If exists in current: compare properties, generate update deltas
///    - If not exists: generate spawn delta
///
/// 2. For each current element not in desired:
///    - Generate despawn delta
///
/// # Complexity
///
/// O(n) where n is the total number of elements, thanks to HashMap-based lookup.
pub fn reconcile(current: &CurrentViewTree, desired: &DesiredViewTree) -> Vec<ViewDelta> {
    let mut deltas = Vec::new();
    let mut visited_keys: HashSet<ViewElementKey> = HashSet::new();

    // Process desired elements
    for root in &desired.roots {
        reconcile_element(current, root, None, &mut deltas, &mut visited_keys);
    }

    // Find elements to despawn (in current but not in desired)
    for (key, current_elem) in &current.elements {
        if !visited_keys.contains(key) {
            deltas.push(ViewDelta::Despawn {
                entity: current_elem.entity,
            });
        }
    }

    deltas
}

/// Reconcile a single element and its children.
/// 协调单个元素及其子元素。
fn reconcile_element(
    current: &CurrentViewTree,
    desired: &DesiredElement,
    parent_entity: Option<Entity>,
    deltas: &mut Vec<ViewDelta>,
    visited: &mut HashSet<ViewElementKey>,
) {
    visited.insert(desired.key.clone());

    if let Some(current_elem) = current.get(&desired.key) {
        // Element exists - check for property changes
        reconcile_properties(current_elem, desired, deltas);

        // Recursively process children
        for child in &desired.children {
            reconcile_element(current, child, Some(current_elem.entity), deltas, visited);
        }
    } else {
        // Element doesn't exist - spawn it
        deltas.push(ViewDelta::Spawn {
            parent: parent_entity,
            spec: desired.clone(),
        });

        // Children will be spawned as part of the parent spawn
        // Mark them as visited to avoid duplicate spawns
        for key in desired.collect_keys() {
            visited.insert(key);
        }
    }
}

/// Compare properties between current and desired, generate update deltas.
/// 比较当前和期望之间的属性，生成更新差异。
fn reconcile_properties(
    current: &CurrentElement,
    desired: &DesiredElement,
    deltas: &mut Vec<ViewDelta>,
) {
    // Transform comparison
    // Skip for camera_anchored elements - their transform is managed by CameraAnchored system
    // 跳过 camera_anchored 元素 - 它们的 transform 由 CameraAnchored 系统管理
    if !desired.camera_anchored
        && !transforms_approximately_equal(&current.transform, &desired.transform)
    {
        deltas.push(ViewDelta::UpdateTransform {
            entity: current.entity,
            new_value: desired.transform,
        });
    }

    // Visibility comparison
    if current.visibility != desired.visibility {
        deltas.push(ViewDelta::UpdateVisibility {
            entity: current.entity,
            new_value: desired.visibility,
        });
    }

    // Sprite comparison
    if let Some(desired_sprite) = &desired.sprite {
        let needs_update = match &current.sprite {
            Some(current_sprite) => {
                // Compare sprite properties with tolerance for color
                let color_changed =
                    !colors_approximately_equal(&current_sprite.color, &desired_sprite.color);
                let flip_changed = current_sprite.flip_x != desired_sprite.flip_x
                    || current_sprite.flip_y != desired_sprite.flip_y;
                color_changed || flip_changed
            }
            None => true, // Entity should have sprite but doesn't
        };

        if needs_update {
            deltas.push(ViewDelta::UpdateSprite {
                entity: current.entity,
                new_value: desired_sprite.clone(),
            });
        }
    }

    // visible_when expression comparison
    if desired.visible_when_expr != current.visible_when_expr
        && let Some(new_expr) = &desired.visible_when_expr
    {
        deltas.push(ViewDelta::UpdateVisibleWhen {
            entity: current.entity,
            new_expression: new_expr.clone(),
        });
    }

    // Camera offset comparison
    if let Some(desired_offset) = desired.camera_offset {
        let needs_update = match current.camera_offset {
            Some(current_offset) => !vec3_approximately_equal(&current_offset, &desired_offset),
            None => true, // Entity should have CameraAnchored but doesn't
        };

        if needs_update {
            deltas.push(ViewDelta::UpdateCameraOffset {
                entity: current.entity,
                new_offset: desired_offset,
            });
        }
    }

    // Material comparison
    // Note: Material updates are primarily handled by the parameter evaluation system.
    // This comparison is for detecting shader changes or animation config changes.
    if let Some(desired_material) = &desired.material {
        let needs_update = !current.has_shader_material;
        if needs_update {
            deltas.push(ViewDelta::UpdateMaterial {
                entity: current.entity,
                new_value: desired_material.clone(),
            });
        }
    }
}

/// Check if two colors are approximately equal.
/// 检查两个颜色是否近似相等。
fn colors_approximately_equal(a: &Color, b: &Color) -> bool {
    const EPSILON: f32 = 0.001;

    let a_linear = a.to_linear();
    let b_linear = b.to_linear();

    (a_linear.red - b_linear.red).abs() < EPSILON
        && (a_linear.green - b_linear.green).abs() < EPSILON
        && (a_linear.blue - b_linear.blue).abs() < EPSILON
        && (a_linear.alpha - b_linear.alpha).abs() < EPSILON
}

/// Check if two Vec3 are approximately equal.
/// 检查两个 Vec3 是否近似相等。
fn vec3_approximately_equal(a: &Vec3, b: &Vec3) -> bool {
    const EPSILON: f32 = 0.001;

    (a.x - b.x).abs() < EPSILON && (a.y - b.y).abs() < EPSILON && (a.z - b.z).abs() < EPSILON
}

/// Check if two transforms are approximately equal.
/// Uses epsilon comparison for floating point values.
///
/// 检查两个变换是否近似相等。
/// 对浮点值使用 epsilon 比较。
fn transforms_approximately_equal(a: &Transform, b: &Transform) -> bool {
    const EPSILON: f32 = 0.001;

    // Translation comparison
    let translation_equal = (a.translation.x - b.translation.x).abs() < EPSILON
        && (a.translation.y - b.translation.y).abs() < EPSILON
        && (a.translation.z - b.translation.z).abs() < EPSILON;

    // Scale comparison
    let scale_equal = (a.scale.x - b.scale.x).abs() < EPSILON
        && (a.scale.y - b.scale.y).abs() < EPSILON
        && (a.scale.z - b.scale.z).abs() < EPSILON;

    // Rotation comparison (using dot product for quaternions)
    let rotation_equal = a.rotation.dot(b.rotation).abs() > 1.0 - EPSILON;

    translation_equal && scale_equal && rotation_equal
}

/// Build a CurrentViewTree from ECS queries.
/// This is the bridge from ECS world to the reconciliation abstraction.
///
/// 从 ECS 查询构建 CurrentViewTree。
/// 这是从 ECS 世界到协调抽象的桥梁。
pub fn build_current_tree(
    _root_entity: Entity,
    view_elements: &[(Entity, String, Transform, Visibility, Option<Entity>)],
) -> CurrentViewTree {
    let mut tree = CurrentViewTree::new();

    for (entity, full_name, transform, visibility, parent) in view_elements {
        // Extract repeat index from name if present (e.g., "EnemyHpBar_0" -> Some(0))
        let repeat_index = extract_repeat_index(full_name);

        let key = if let Some(idx) = repeat_index {
            ViewElementKey::with_repeat_index(full_name, idx)
        } else {
            ViewElementKey::new(full_name)
        };

        tree.insert(CurrentElement {
            entity: *entity,
            key,
            transform: *transform,
            visibility: *visibility,
            parent: *parent,
            sprite: None,
            visible_when_expr: None,
            camera_offset: None,
            has_shader_material: false,
        });
    }

    tree
}

/// Extract repeat index from element name.
/// e.g., "namespace::EnemyHpBar_2" -> Some(2)
///
/// 从元素名称提取重复索引。
fn extract_repeat_index(full_name: &str) -> Option<usize> {
    // Find the last underscore followed by a number
    if let Some(underscore_pos) = full_name.rfind('_') {
        let suffix = &full_name[underscore_pos + 1..];
        suffix.parse::<usize>().ok()
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_repeat_index() {
        assert_eq!(extract_repeat_index("EnemyHpBar_0"), Some(0));
        assert_eq!(extract_repeat_index("battle::EnemyHpBar_2"), Some(2));
        assert_eq!(extract_repeat_index("Cursor"), None);
        assert_eq!(extract_repeat_index("Some_Name_With_Underscore"), None); // "Underscore" is not a number
        assert_eq!(extract_repeat_index("Name_123"), Some(123));
    }

    #[test]
    fn test_transforms_approximately_equal() {
        let t1 = Transform::from_translation(Vec3::new(1.0, 2.0, 3.0));
        let t2 = Transform::from_translation(Vec3::new(1.0001, 2.0001, 3.0001));
        let t3 = Transform::from_translation(Vec3::new(2.0, 2.0, 3.0));

        assert!(transforms_approximately_equal(&t1, &t2));
        assert!(!transforms_approximately_equal(&t1, &t3));
    }

    #[test]
    fn test_reconcile_empty_to_desired() {
        let current = CurrentViewTree::new();
        let mut desired = DesiredViewTree::new();

        let element = DesiredElement::new(ViewElementKey::new("test::Element"), "Element");
        desired.roots.push(element);

        let deltas = reconcile(&current, &desired);

        assert_eq!(deltas.len(), 1);
        assert!(matches!(deltas[0], ViewDelta::Spawn { .. }));
    }

    #[test]
    fn test_reconcile_despawn() {
        let mut current = CurrentViewTree::new();
        // Use from_raw_u32 to create a test entity (may return None if row is 0)
        // For testing we use row 1 which should succeed
        let test_entity = Entity::from_raw_u32(1).expect("Entity row 1 should be valid");
        current.insert(CurrentElement {
            entity: test_entity,
            key: ViewElementKey::new("test::ToDelete"),
            transform: Transform::IDENTITY,
            visibility: Visibility::Inherited,
            parent: None,
            sprite: None,
            visible_when_expr: None,
            camera_offset: None,
            has_shader_material: false,
        });

        let desired = DesiredViewTree::new(); // Empty - no elements desired

        let deltas = reconcile(&current, &desired);

        assert_eq!(deltas.len(), 1);
        assert!(matches!(deltas[0], ViewDelta::Despawn { .. }));
    }
}
