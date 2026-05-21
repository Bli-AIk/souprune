//! Runtime placement helpers for solved View layout slots.
//!
//! 已求解 View 布局槽位的运行时放置辅助函数。

use bevy::prelude::{Transform, Vec3};

use super::ViewLayoutSlot;
use crate::core::view::spatial::valid_pixels_per_unit;

/// Runtime origin used by a spawned View layout node.
///
/// 已生成 View 布局节点使用的运行时原点。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ViewLayoutOrigin {
    Center,
    TopLeft,
}

/// Compute the local translation for a solved layout slot relative to its parent.
///
/// 计算已求解布局槽位相对父节点的本地平移。
pub(crate) fn layout_slot_local_translation(
    slot: &ViewLayoutSlot,
    parent_slot: Option<&ViewLayoutSlot>,
    parent_origin: ViewLayoutOrigin,
) -> Vec3 {
    let origin_offset = match (parent_slot, parent_origin) {
        (Some(parent), ViewLayoutOrigin::Center) => {
            Vec3::new(-parent.width * 0.5, parent.height * 0.5, 0.0)
        }
        _ => Vec3::ZERO,
    };
    origin_offset + Vec3::new(slot.x, -slot.y, 0.0)
}

/// Compute the local translation for a solved layout slot on a 3D View plane.
///
/// 计算 3D View 平面上已求解布局槽位的本地平移。
pub(crate) fn layout_slot_plane_local_translation(
    slot: &ViewLayoutSlot,
    parent_slot: Option<&ViewLayoutSlot>,
    parent_origin: ViewLayoutOrigin,
    pixels_per_unit: f32,
) -> Vec3 {
    layout_slot_local_translation(slot, parent_slot, parent_origin)
        / valid_pixels_per_unit(pixels_per_unit)
}

/// Combine a solved layout slot with an authored local transform.
///
/// 将已求解布局槽位与作者填写的本地变换组合。
pub(crate) fn combine_layout_transform(
    slot: Option<&ViewLayoutSlot>,
    parent_slot: Option<&ViewLayoutSlot>,
    parent_origin: ViewLayoutOrigin,
    transform: Transform,
) -> Transform {
    let Some(slot) = slot else {
        return transform;
    };
    combine_transforms(
        Transform::from_translation(layout_slot_local_translation(
            slot,
            parent_slot,
            parent_origin,
        )),
        transform,
    )
}

/// Combine a solved 3D plane layout slot with an authored local transform.
///
/// 将已求解的 3D 平面布局槽位与作者填写的本地变换组合。
pub(crate) fn combine_spatial_layout_transform(
    slot: Option<&ViewLayoutSlot>,
    parent_slot: Option<&ViewLayoutSlot>,
    parent_origin: ViewLayoutOrigin,
    pixels_per_unit: f32,
    transform: Transform,
) -> Transform {
    let Some(slot) = slot else {
        return transform;
    };
    combine_transforms(
        Transform::from_translation(layout_slot_plane_local_translation(
            slot,
            parent_slot,
            parent_origin,
            pixels_per_unit,
        )),
        transform,
    )
}

fn combine_transforms(parent: Transform, child: Transform) -> Transform {
    Transform {
        translation: parent.translation + child.translation,
        rotation: parent.rotation * child.rotation,
        scale: parent.scale * child.scale,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn slot(path: &str, x: f32, y: f32, width: f32, height: f32) -> ViewLayoutSlot {
        ViewLayoutSlot {
            path: path.to_string(),
            name: path.to_string(),
            x,
            y,
            width,
            height,
        }
    }

    #[test]
    fn child_layout_translation_is_relative_to_parent_slot() {
        let parent = slot("Root", 0.0, 0.0, 360.0, 220.0);
        let child = slot("Root/Child", 24.0, 24.0, 312.0, 44.0);

        let translation =
            layout_slot_local_translation(&child, Some(&parent), ViewLayoutOrigin::TopLeft);

        assert_eq!(translation, Vec3::new(24.0, -24.0, 0.0));
    }

    #[test]
    fn child_slot_is_already_parent_local_even_when_parent_has_nonzero_slot() {
        let parent = slot("Root", 120.0, 80.0, 360.0, 220.0);
        let child = slot("Root/Child", 24.0, 24.0, 312.0, 44.0);

        let translation =
            layout_slot_local_translation(&child, Some(&parent), ViewLayoutOrigin::TopLeft);

        assert_eq!(translation, Vec3::new(24.0, -24.0, 0.0));
    }

    #[test]
    fn child_under_centered_parent_offsets_from_parent_top_left() {
        let parent = slot("Root", 120.0, 80.0, 360.0, 220.0);
        let child = slot("Root/Child", 24.0, 24.0, 312.0, 44.0);

        let translation =
            layout_slot_local_translation(&child, Some(&parent), ViewLayoutOrigin::Center);

        assert_eq!(translation, Vec3::new(-156.0, 86.0, 0.0));
    }

    #[test]
    fn root_layout_translation_stays_absolute() {
        let root = slot("Root", 120.0, 80.0, 360.0, 220.0);

        let translation = layout_slot_local_translation(&root, None, ViewLayoutOrigin::Center);

        assert_eq!(translation, Vec3::new(120.0, -80.0, 0.0));
    }
}
