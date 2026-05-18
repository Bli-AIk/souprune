//! Spatial placement helpers for View roots.
//!
//! View 根的空间放置辅助逻辑。

use crate::core::view::layout::{
    SerializableTransform, ViewLayoutSlot, ViewWorld3dPlaneDef, serializable_vec3_to_static,
};
use bevy::prelude::{Component, EulerRot, Quat, Transform, Vec3};

/// Runtime marker for a View root mounted on a 3D plane.
///
/// 挂载到 3D 平面的 View 根运行时标记。
#[derive(Component, Debug, Clone)]
pub struct ViewSpatialRoot {
    /// Plane placement data used by this View root.
    ///
    /// 此 View 根使用的平面放置数据。
    pub plane: ViewWorld3dPlaneDef,
}

/// Convert a solved 2D layout slot into local coordinates on a 3D View plane.
///
/// 将求解后的二维布局槽位转换为 3D View 平面的局部坐标。
pub fn layout_slot_to_plane_translation(
    slot: &ViewLayoutSlot,
    plane: &ViewWorld3dPlaneDef,
) -> Vec3 {
    let pixels_per_unit = valid_pixels_per_unit(plane.pixels_per_unit);
    Vec3::new(slot.x / pixels_per_unit, -slot.y / pixels_per_unit, 0.0)
}

/// Build the world transform for a spatial View root from schema data.
///
/// 从 schema 数据构建空间 View 根的世界变换。
pub fn spatial_root_transform(plane: &ViewWorld3dPlaneDef) -> Transform {
    let mut transform = transform_from_serializable(&plane.transform);
    if let Some(rotation) = &plane.rotation_degrees {
        let rotation = serializable_vec3_to_static(rotation);
        transform.rotation *= Quat::from_euler(
            EulerRot::XYZ,
            rotation.x.to_radians(),
            rotation.y.to_radians(),
            rotation.z.to_radians(),
        );
    }
    transform
}

fn transform_from_serializable(transform: &SerializableTransform) -> Transform {
    let mut result = Transform::default();
    if let Some(translation) = &transform.translation {
        result.translation = serializable_vec3_to_static(translation);
    }
    if let Some(rotation) = &transform.rotation {
        result.rotation = Quat::from_rotation_z(static_float(rotation).to_radians());
    }
    if let Some(scale) = &transform.scale {
        result.scale = serializable_vec3_to_static(scale);
    }
    result
}

fn static_float(value: &crate::core::sequencer::chapter_schema::Value<f32>) -> f32 {
    match value {
        crate::core::sequencer::chapter_schema::Value::Static(value) => *value,
        crate::core::sequencer::chapter_schema::Value::Expr(_) => 0.0,
    }
}

fn valid_pixels_per_unit(value: f32) -> f32 {
    if value.is_finite() && value > 0.0 {
        value
    } else {
        1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::sequencer::chapter_schema::Value;
    use crate::core::view::layout::{
        SerializableTransform, ViewCameraTargetDef, ViewLayoutSlot, ViewWorld3dPlaneDef,
    };
    use bevy::prelude::{EulerRot, Vec3};

    fn plane() -> ViewWorld3dPlaneDef {
        ViewWorld3dPlaneDef {
            transform: SerializableTransform {
                translation: Some((Value::Static(2.0), Value::Static(3.0), Value::Static(4.0))),
                rotation: Some(Value::Static(30.0)),
                scale: Some((Value::Static(1.0), Value::Static(1.0), Value::Static(1.0))),
            },
            rotation_degrees: None,
            plane_size: (6.4, 4.8),
            pixels_per_unit: 100.0,
            camera: ViewCameraTargetDef::Main,
        }
    }

    #[test]
    fn layout_slot_maps_pixels_to_plane_local_translation() {
        let slot = ViewLayoutSlot {
            path: "Root/Child".to_string(),
            name: "Child".to_string(),
            x: 100.0,
            y: 50.0,
            width: 40.0,
            height: 20.0,
        };

        let translation = layout_slot_to_plane_translation(&slot, &plane());

        assert_eq!(translation, Vec3::new(1.0, -0.5, 0.0));
    }

    #[test]
    fn spatial_root_transform_uses_plane_transform() {
        let transform = spatial_root_transform(&plane());

        assert_eq!(transform.translation, Vec3::new(2.0, 3.0, 4.0));
    }

    #[test]
    fn spatial_root_transform_applies_plane_rotation_degrees() {
        let mut plane = plane();
        plane.transform.rotation = None;
        plane.rotation_degrees =
            Some((Value::Static(45.0), Value::Static(0.0), Value::Static(0.0)));

        let transform = spatial_root_transform(&plane);
        let (x, y, z) = transform.rotation.to_euler(EulerRot::XYZ);

        assert!((x.to_degrees() - 45.0).abs() < 0.0001);
        assert_eq!(y, 0.0);
        assert_eq!(z, 0.0);
    }
}
